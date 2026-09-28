// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod expressions;
pub mod fmt;

use super::logics::Logic;
use crate::index::Index;
use expressions::{Expression, LLExpression};
use std::collections::HashMap;
use std::iter::zip;

#[derive(Clone, Debug)]
pub struct Sequent<I: Index, L: Logic<I>> {
    pub(crate) term_arena: Vec<L::Expression>,
    pub(crate) term_ids: Vec<I>,
    pub(crate) variable_dict: Vec<String>,
}

impl<I: Index, L: Logic<I>> std::default::Default for Sequent<I, L> {
    /// Returns the empty sequent.
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Index, L: Logic<I>> Sequent<I, L> {
    /// Returns the empty sequent.
    pub const fn new() -> Self {
        Self {
            term_arena: vec![],
            term_ids: vec![],
            variable_dict: vec![],
        }
    }

    /// Check whether the internal data structure is correct
    pub fn verify_integrity(&self) -> Result<(), crate::Error> {
        let num_vars = Index::from_usize(self.variable_dict.len());
        let num_terms = Index::from_usize(self.term_arena.len());

        // check that terms only reference
        //   - other terms with lower IDs than themselves
        //   - existing variable IDS
        self.term_arena
            .iter()
            .enumerate()
            .try_for_each(|(n, e)| e.check_bounds(num_vars, Index::from_usize(n)))?;

        // check that all term indices are valid
        self.term_ids.iter().try_for_each(|n| {
            if *n >= num_terms {
                Err(crate::Error::TermIndexOutOfBounds(
                    (*n).as_usize(),
                    num_terms.as_usize(),
                ))
            } else {
                Ok(())
            }
        })?;

        Ok(())
    }

    /// Sorts the root term indices and checks that each names a term.
    fn optimize_term_ids(&mut self) -> Result<(), crate::Error> {
        self.term_ids.sort();
        self.term_ids.shrink_to_fit();
        let Some(n) = self.term_ids.last() else {
            return Ok(());
        };
        let num_terms = Index::from_usize(self.term_arena.len());
        if *n >= num_terms {
            Err(crate::Error::TermIndexOutOfBounds(
                (*n).as_usize(),
                num_terms.as_usize(),
            ))
        } else {
            Ok(())
        }
    }

    /// Drops the terms no root formula reaches and merges equal terms, keeping
    /// the arena topologically sorted. Fails if an index breaks that order or
    /// points outside the arena.
    fn optimize_term_arena(&mut self) -> Result<(), crate::Error> {
        use LLExpression::*;

        let num_terms = Index::from_usize(self.term_arena.len());

        for n in self.term_ids.iter() {
            if *n >= num_terms {
                return Err(crate::Error::TermIndexOutOfBounds(
                    (*n).as_usize(),
                    num_terms.as_usize(),
                ));
            }
        }

        let mut reachable = vec![false; num_terms.as_usize()];
        let mut visit_stack = self.term_ids.clone();

        while let Some(n) = visit_stack.pop() {
            if !reachable[n.as_usize()] {
                reachable[n.as_usize()] = true;
                match self.term_arena[n.as_usize()].into() {
                    Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => {
                        if k >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(
                                k.as_usize(),
                                n.as_usize(),
                            ));
                        }
                        if l >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(
                                l.as_usize(),
                                n.as_usize(),
                            ));
                        }
                        if !reachable[k.as_usize()] {
                            visit_stack.push(k);
                        }
                        if !reachable[l.as_usize()] {
                            visit_stack.push(l);
                        }
                    }
                    Bang(k) | Quest(k) => {
                        if k >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(
                                k.as_usize(),
                                n.as_usize(),
                            ));
                        }
                        if !reachable[k.as_usize()] {
                            visit_stack.push(k);
                        }
                    }
                    _ => {}
                }
            }
        }

        // In index order, every subterm has its final index before its parents
        // are rebuilt, so hashing the rebuilt terms merges equal terms at any
        // depth.
        let mut new_index = vec![None; num_terms.as_usize()];
        let mut term_arena = Vec::with_capacity(num_terms.as_usize());
        let mut kept = HashMap::<L::Expression, I>::new();

        for (n, (is_reachable, e)) in zip(reachable, self.term_arena.iter()).enumerate() {
            if !is_reachable {
                continue;
            }
            // The subterms of a reachable term are reachable and precede it.
            let moved = |k: I| new_index[k.as_usize()].unwrap();
            let e = L::Expression::try_from(match (*e).into() {
                Tensor(k, l) => Tensor(moved(k), moved(l)),
                Par(k, l) => Par(moved(k), moved(l)),
                With(k, l) => With(moved(k), moved(l)),
                Plus(k, l) => Plus(moved(k), moved(l)),
                Bang(k) => Bang(moved(k)),
                Quest(k) => Quest(moved(k)),
                e => e,
            })
            .unwrap();
            let fresh_index = Index::from_usize(term_arena.len());
            let index = *kept.entry(e).or_insert(fresh_index);
            if index == fresh_index {
                term_arena.push(e);
            }
            new_index[n] = Some(index);
        }

        self.term_arena = term_arena;
        self.term_ids = self
            .term_ids
            .iter()
            .map(|k| new_index[k.as_usize()].unwrap())
            .collect();

        Ok(())
    }

    /// Merges variables of the same name, numbered in order of first occurrence.
    fn optimize_variable_dict(&mut self) -> Result<(), crate::Error> {
        use LLExpression::*;
        let num_variables = Index::from_usize(self.variable_dict.len());
        let mut var_dict_ = Vec::<String>::with_capacity(num_variables);
        let mut hm = HashMap::<String, I>::with_capacity(num_variables);

        for e in self.term_arena.iter_mut() {
            let e_in_ll = (*e).into();
            match e_in_ll {
                Var(n) | DualVar(n) => {
                    let name_ref: &str = self
                        .variable_dict
                        .get(n.as_usize())
                        .ok_or(crate::Error::InvalidVariableIndex(
                            n.as_usize(),
                            num_variables.as_usize(),
                        ))?
                        .as_ref();
                    let k = match hm.get(name_ref) {
                        Some(k) => *k,
                        None => {
                            let fresh_variable_id = Index::from_usize(var_dict_.len());
                            let name = name_ref.to_string();
                            hm.insert(name.clone(), fresh_variable_id);
                            var_dict_.push(name);
                            fresh_variable_id
                        }
                    };
                    let renamed = match e_in_ll {
                        Var(_) => Var(k),
                        DualVar(_) => DualVar(k),
                        _ => unreachable!(),
                    };
                    *e = L::Expression::try_from(renamed).unwrap();
                }
                _ => {}
            }
        }
        var_dict_.shrink_to_fit();
        self.variable_dict = var_dict_;
        Ok(())
    }

    /// Merges variables of the same name and equal terms, drops the terms no
    /// root formula reaches and sorts the root formulas. Fails if the arena
    /// breaks its invariants.
    pub fn optimize(&mut self) -> Result<(), crate::Error> {
        self.optimize_variable_dict()?;
        self.optimize_term_arena()?;
        self.optimize_term_ids()?;
        self.optimize_variable_dict()?;
        Ok(())
    }

    /// Consume another sequent and add its terms to self
    pub fn add(&mut self, s: Self) {
        let offset_variables = Index::from_usize(self.variable_dict.len());
        let offset_terms = Index::from_usize(self.term_arena.len());

        self.term_arena.extend(
            s.term_arena
                .into_iter()
                .map(|e| e.offset(offset_variables, offset_terms)),
        );

        self.term_ids
            .extend(s.term_ids.into_iter().map(|n| n + offset_terms));

        self.variable_dict.extend(s.variable_dict);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logics::LL;

    /// The empty sequent survives optimisation unchanged.
    #[test]
    fn optimize_empty() {
        let mut s = Sequent::<usize, LL>::new();
        s.optimize().unwrap();
        assert!(s.term_arena.is_empty() && s.term_ids.is_empty() && s.variable_dict.is_empty());
    }

    /// Variables of one name merge, and the rest are renumbered to match, also
    /// when a repeated name comes before a new one.
    #[test]
    fn optimize_merges_variables() {
        use LLExpression::*;
        let mut s = Sequent::<usize, LL> {
            term_arena: vec![Var(0), DualVar(1), Var(2)],
            term_ids: vec![0, 1, 2],
            variable_dict: vec!["A".into(), "A".into(), "B".into()],
        };
        s.optimize().unwrap();
        assert_eq!(s.variable_dict, ["A", "B"]);
        assert_eq!(s.term_arena, [Var(0), DualVar(0), Var(1)]);
    }

    /// References to a duplicate term point to its kept copy, also after
    /// unreachable terms before it are dropped.
    #[test]
    fn optimize_redirects_duplicates() {
        use LLExpression::*;
        let mut s = Sequent::<usize, LL> {
            term_arena: vec![Zero, One, Top, Top, Tensor(1, 3)],
            term_ids: vec![2, 4],
            variable_dict: vec![],
        };
        s.optimize().unwrap();
        assert_eq!(s.term_arena, [One, Top, Tensor(0, 1)]);
        assert_eq!(s.term_ids, [1, 2]);
    }

    /// Equal terms merge at every depth, not only equal leaves.
    #[test]
    fn optimize_merges_equal_subterms() {
        use LLExpression::*;
        let mut s = Sequent::<usize, LL> {
            term_arena: vec![
                Var(0),
                Var(1),
                Tensor(0, 1),
                Bang(2),
                Var(0),
                Var(1),
                Tensor(4, 5),
                Bang(6),
            ],
            term_ids: vec![3, 7],
            variable_dict: vec!["A".into(), "B".into()],
        };
        s.optimize().unwrap();
        assert_eq!(s.term_arena, [Var(0), Var(1), Tensor(0, 1), Bang(2)]);
        assert_eq!(s.term_ids, [3, 3]);
    }
}

// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod expressions;
pub mod fmt;

use super::logics::LogicNew;
use crate::index::Index;
use expressions::{ExpressionNew, LLExpressionNew};
use std::collections::HashMap;
use std::iter::zip;

#[derive(Clone, Debug)]
pub struct SequentNew<I: Index, L: LogicNew<I>> {
    pub(crate) term_arena: Vec<L::Expression>,
    pub(crate) term_ids: Vec<I>,
    pub(crate) variable_dict: Vec<String>,
}

impl<I: Index, L: LogicNew<I>> std::default::Default for SequentNew<I, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Index, L: LogicNew<I>> SequentNew<I, L> {
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

    fn optimize_term_ids(&mut self) -> Result<(), crate::Error> {
        self.term_ids.sort();
        self.term_ids.shrink_to_fit();
        let n = self.term_ids.last().unwrap();
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

    fn optimize_term_arena(&mut self) -> Result<(), crate::Error> {
        use LLExpressionNew::*;

        // determine cleanup measures and check integrity

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

        #[derive(Copy, Clone, Debug)]
        enum TermState<I: Index> {
            Remain,
            Unreachable,
            DuplicateOf(I),
        }

        use TermState::*;

        let mut hm = HashMap::<L::Expression, I>::new();
        let mut states = Vec::<TermState<I>>::with_capacity(num_terms.as_usize());

        for (n, (is_reachable, e)) in zip(reachable.into_iter(), self.term_arena.iter()).enumerate()
        {
            if is_reachable {
                if let Some(k) = hm.get(e) {
                    states.push(DuplicateOf(*k));
                } else {
                    hm.insert(*e, Index::from_usize(n));
                    states.push(Remain);
                }
            } else {
                states.push(Unreachable);
            }
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        enum RedirectState<I: Index> {
            Remove,
            MoveTo(I),
            RemoveAndPointTo(I),
        }

        use RedirectState::*;

        let mut fresh_index = Index::from_usize(0);
        let mut redirects = Vec::<RedirectState<I>>::with_capacity(num_terms.as_usize());

        for s in states.into_iter() {
            match s {
                Unreachable => redirects.push(Remove),
                Remain => {
                    redirects.push(MoveTo(fresh_index));
                    fresh_index += Index::from_usize(1);
                }
                DuplicateOf(n) => {
                    if let MoveTo(k) = redirects[n.as_usize()] {
                        redirects.push(RemoveAndPointTo(k));
                    } else {
                        unreachable!()
                    }
                }
            }
        }

        // execute cleanup

        let where_to_find = |k: &I| {
            debug_assert!(*k < Index::from_usize(redirects.len()));
            debug_assert!(redirects[(*k).as_usize()] != Remove);

            match redirects[(*k).as_usize()] {
                MoveTo(n) => n,
                RemoveAndPointTo(n) => {
                    if let MoveTo(m) = redirects[n.as_usize()] {
                        m
                    } else {
                        unreachable!()
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        };

        let apply_cleanup_terms = |(state, e): (&RedirectState<I>, &L::Expression)| {
            if let MoveTo(_) = state {
                Some(
                    L::Expression::try_from(match (*e).into() {
                        Tensor(m, n) => Tensor(where_to_find(&m), where_to_find(&n)),
                        Par(m, n) => Par(where_to_find(&m), where_to_find(&n)),
                        With(m, n) => With(where_to_find(&m), where_to_find(&n)),
                        Plus(m, n) => Plus(where_to_find(&m), where_to_find(&n)),
                        Bang(m) => Bang(where_to_find(&m)),
                        Quest(m) => Quest(where_to_find(&m)),
                        e => e,
                    })
                    .unwrap(),
                )
            } else {
                None
            }
        };

        self.term_arena = zip(redirects.iter(), self.term_arena.iter())
            .filter_map(apply_cleanup_terms)
            .collect();

        self.term_ids = self.term_ids.iter().map(where_to_find).collect();

        Ok(())
    }

    fn optimize_variable_dict(&mut self) -> Result<(), crate::Error> {
        use LLExpressionNew::*;
        let num_variables = Index::from_usize(self.variable_dict.len());
        let mut var_dict_new = Vec::<String>::with_capacity(num_variables);
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
                    if let Some(k) = hm.get(name_ref) {
                        let new_e = match e_in_ll {
                            Var(_) => Var(*k),
                            DualVar(_) => DualVar(*k),
                            _ => unreachable!(),
                        };
                        *e = L::Expression::try_from(new_e).unwrap();
                    } else {
                        let fresh_variable_id = Index::from_usize(var_dict_new.len());
                        let name = name_ref.to_string();
                        hm.insert(name.clone(), fresh_variable_id);
                        var_dict_new.push(name);
                    }
                }
                _ => {}
            }
        }
        var_dict_new.shrink_to_fit();
        self.variable_dict = var_dict_new;
        Ok(())
    }

    /// Remove unreachable terms and collapse duplicate items
    /// This is rather inefficient, so use only if necessary
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

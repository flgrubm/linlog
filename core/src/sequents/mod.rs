// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod expressions;
pub mod fmt;

use super::logics::Logic;
use expressions::{Expression, LLExpression};
use std::collections::HashMap;
use std::iter::zip;

#[derive(Clone, Debug)]
pub struct Sequent<L: Logic> {
    pub(crate) term_arena: Vec<L::Expression>,
    pub(crate) term_ids: Vec<usize>,
    pub(crate) variable_dict: Vec<String>,
}

impl<L: Logic> std::default::Default for Sequent<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Logic> Sequent<L> {
    pub const fn new() -> Self {
        Self {
            term_arena: vec![],
            term_ids: vec![],
            variable_dict: vec![],
        }
    }

    /// Check whether the internal data structure is correct
    pub fn verify_integrity(&self) -> Result<(), crate::Error> {
        let num_vars = self.variable_dict.len();
        let num_terms = self.term_arena.len();

        // check that terms only reference
        //   - other terms with lower IDs than themselves
        //   - existint variable IDS
        self.term_arena
            .iter()
            .enumerate()
            .try_for_each(|(n, e)| e.check_bounds(num_vars, n))?;

        // check that all term indices are valid
        self.term_ids.iter().try_for_each(|n| {
            if *n >= num_terms {
                Err(crate::Error::TermIndexOutOfBounds(*n, num_terms))
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
        let num_terms = self.term_arena.len();
        if *n >= num_terms {
            Err(crate::Error::TermIndexOutOfBounds(*n, num_terms))
        } else {
            Ok(())
        }
    }

    fn optimize_term_arena(&mut self) -> Result<(), crate::Error> {
        use LLExpression::*;

        // determine cleanup measures and check integrity

        let num_terms = self.term_arena.len();

        for n in self.term_ids.iter() {
            if *n >= num_terms {
                return Err(crate::Error::TermIndexOutOfBounds(*n, num_terms));
            }
        }

        let mut reachable = vec![false; num_terms];
        let mut visit_stack = self.term_ids.clone();

        while let Some(n) = visit_stack.pop() {
            if !reachable[n] {
                reachable[n] = true;
                match self.term_arena[n].into() {
                    Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => {
                        if k >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(k, n));
                        }
                        if l >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(l, n));
                        }
                        if !reachable[k] {
                            visit_stack.push(k);
                        }
                        if !reachable[l] {
                            visit_stack.push(l);
                        }
                    }
                    Bang(k) | Quest(k) => {
                        if k >= n {
                            return Err(crate::Error::SubtermIndexNotDecreasing(k, n));
                        }
                        if !reachable[k] {
                            reachable[k] = true;
                            visit_stack.push(k);
                        }
                    }
                    _ => {}
                }
            }
        }

        #[derive(Copy, Clone, Debug)]
        enum TermState {
            Remain,
            Unreachable,
            DuplicateOf(usize),
        }

        use TermState::*;

        let mut hm = HashMap::<L::Expression, usize>::new();
        let mut states = Vec::<TermState>::with_capacity(num_terms);

        for (n, (is_reachable, e)) in zip(reachable.into_iter(), self.term_arena.iter()).enumerate()
        {
            if is_reachable {
                if let Some(k) = hm.get(e) {
                    states.push(DuplicateOf(*k));
                } else {
                    hm.insert(*e, n);
                    states.push(Remain);
                }
            } else {
                states.push(Unreachable);
            }
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        enum RedirectState {
            Remove,
            MoveTo(usize),
            RemoveAndPointTo(usize),
        }

        use RedirectState::*;

        let mut fresh_index = 0usize;
        let mut redirects = Vec::<RedirectState>::with_capacity(num_terms);

        for s in states.into_iter() {
            match s {
                Unreachable => redirects.push(Remove),
                Remain => {
                    redirects.push(MoveTo(fresh_index));
                    fresh_index += 1;
                }
                DuplicateOf(n) => {
                    if let MoveTo(k) = redirects[n] {
                        redirects.push(RemoveAndPointTo(k));
                    } else {
                        unreachable!()
                    }
                }
            }
        }

        // execute cleanup

        let where_to_find = |k: &usize| {
            debug_assert!(*k < redirects.len());
            debug_assert!(redirects[*k] != Remove);

            match redirects[*k] {
                MoveTo(n) => n,
                RemoveAndPointTo(n) => {
                    if let MoveTo(m) = redirects[n] {
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

        let apply_cleanup_terms = |(state, e): (&RedirectState, &L::Expression)| {
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
        use LLExpression::*;
        let num_variables = self.variable_dict.len();
        let mut var_dict_new = Vec::<String>::with_capacity(num_variables);
        let mut hm = HashMap::<String, usize>::with_capacity(num_variables);

        for e in self.term_arena.iter_mut() {
            let e_in_ll = (*e).into();
            match e_in_ll {
                Var(n) | DualVar(n) => {
                    let name_ref: &str = self
                        .variable_dict
                        .get(n)
                        .ok_or(crate::Error::InvalidVariableIndex(n, num_variables))?
                        .as_ref();
                    if let Some(k) = hm.get(name_ref) {
                        let new_e = match e_in_ll {
                            Var(_) => Var(*k),
                            DualVar(_) => DualVar(*k),
                            _ => unreachable!(),
                        };
                        *e = L::Expression::try_from(new_e).unwrap();
                    } else {
                        let fresh_variable_id = var_dict_new.len();
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
        self.optimize_term_arena()?;
        self.optimize_term_ids()?;
        self.optimize_variable_dict()?;
        Ok(())
    }

    /// Consume another sequent and add its terms to self
    pub fn add(&mut self, s: Sequent<L>) {
        let offset_variables = self.variable_dict.len();
        let offset_terms = self.term_arena.len();

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

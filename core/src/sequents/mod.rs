// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod logics;
pub mod parsing;
mod serialize;
pub mod terms;

use logics::{LL, Logic};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;

use terms::{Expression, Terms};

#[derive(Clone, Debug, Default)]
pub struct Sequent<L: Logic> {
    term_arena: Vec<Expression<L, usize>>,
    term_ids: Vec<usize>,
    variable_dict: Vec<String>,
}

impl Serialize for Sequent<LL> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proxy = serialize::Sequent::from(self.clone());
        proxy.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Sequent<LL> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = serialize::Sequent::deserialize(deserializer)?;
        Sequent::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

impl<L: Logic> Sequent<L> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn detect_cycles(&self) -> Result<(), crate::Error> {
        use terms::Expression::*;

        let mut check_next = self.term_ids.clone();
        let mut connections = vec![HashSet::<usize>::new(); self.term_ids.len()];

        while let Some(n) = check_next.pop() {
            let mut visit = Vec::<usize>::with_capacity(2);
            match self
                .term_arena
                .get(n)
                .ok_or(crate::Error::InvalidTermIndex(n, self.term_arena.len()))?
            {
                MultOp(_, _, k, l) | AddOp(_, _, k, l) => {
                    visit.push(*k);
                    visit.push(*l);
                }
                ExpOp(_, _, k) => visit.push(*k),
                Var(_, _) | MultConst(_, _) | AddConst(_, _) => {}
            }
            for k in visit.into_iter() {
                if connections[k].is_empty() {
                    check_next.push(k);
                }

                if k == n {
                    return Err(crate::Error::CycleDetected(k));
                } else if k < n {
                    let (left, right) = connections.split_at_mut(k + 1);
                    for l in right[n - k].iter() {
                        left[k].insert(*l);
                    }
                } else {
                    let (left, right) = connections.split_at_mut(n + 1);
                    for l in left[n].iter() {
                        right[k - n].insert(*l);
                    }
                }
                if connections[k].contains(&k) {
                    return Err(crate::Error::CycleDetected(k));
                }
            }
        }
        Ok(())
    }

    /// Check whether the internal data structure is correct
    pub fn verify_integrity(&self) -> Result<(), crate::Error> {
        let num_vars = self.variable_dict.len();
        let num_terms = self.term_arena.len();

        // check that variable and linked subterm indices are valid for all terms
        self.term_arena
            .iter()
            .try_for_each(|e| e.check_bounds(num_vars, num_terms))?;

        // check that all term indices are valid
        self.term_ids.iter().try_for_each(|n| {
            if *n >= num_terms {
                Err(crate::Error::InvalidTermIndex(*n, num_terms))
            } else {
                Ok(())
            }
        })?;

        // detect cycles
        self.detect_cycles()?;

        Ok(())
    }

    /// Remove unreachable and collapse duplicate items
    /// This is rather inefficient, so use only if necessary
    pub fn optimize(&mut self) -> Result<(), crate::Error> {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    /// The caller must know that the variable dict of the sequent matches the variable schema used in the terms
    ///
    /// # Safety
    /// No checks whether term IDs and variable IDs exist
    pub unsafe fn add_terms_matching_variable_dict_unchecked(&mut self, terms: &[Terms<L, usize>]) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    /// The caller must know that the variable dict of the sequent matches the variable schema used in the terms
    pub fn add_terms_matching_variable_dict(
        &mut self,
        terms: &[Terms<L, usize>],
    ) -> Result<(), crate::Error> {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    ///
    /// # Safety
    /// No checks whether term IDs exist
    pub unsafe fn add_terms_variable_str_unchecked(&mut self, terms: &[Terms<L, &str>]) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    pub fn add_terms_variable_str(&mut self, terms: &[Terms<L, &str>]) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    /// variable dictionary is appended to the existing one
    ///
    /// # Safety
    /// No checks whether term IDs and variable IDs exist
    pub unsafe fn add_terms_with_variable_dict_unchecked(
        &mut self,
        terms: &[Terms<L, usize>],
        dict: &[&str],
    ) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    /// variable dictionary is appended to the existing one
    pub fn add_terms_with_variable_dict(
        &mut self,
        terms: &[Terms<L, usize>],
        dict: &[&str],
    ) -> Result<(), crate::Error> {
        todo!()
    }
}

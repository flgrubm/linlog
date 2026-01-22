// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod logics;
pub mod serialize;
pub mod terms;
// pub mod parsing;

use logics::Logic;

use terms::{Expression, Terms};

pub struct Sequent<L: Logic> {
    term_arena: Vec<Expression<L, usize>>,
    term_ids: Vec<usize>,
    variable_dict: Vec<String>,
}

impl<L: Logic> Sequent<L> {
    pub fn new() -> Self {
        Sequent {
            term_arena: vec![],
            term_ids: vec![],
            variable_dict: vec![],
        }
    }

    // Check whether the internal data structure is correct
    pub fn verify_integrity(&self) -> Result<(), crate::Error> {
        todo!()
    }

    /// Remove unreachable and collapse duplicate items
    /// This is rather inefficient, so use only if necessary
    pub fn optimize(&mut self) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    /// The caller must know that the variable dict of the sequent matches the variable schema used in the terms
    /// Unsafe: no checks whether term IDs and variable IDs exist
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
    /// Unsafe: no checks whether term IDs exist
    pub unsafe fn add_terms_variable_str_unchecked(&mut self, terms: &[Terms<L, &str>]) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    pub fn add_terms_variable_str(&mut self, terms: &[Terms<L, &str>]) {
        todo!()
    }

    /// Add more terms to the sequent (without optimizations)
    /// variable dictionary is appended to the existing one
    /// Unsafe: no checks whether term IDs and variable IDs exist
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

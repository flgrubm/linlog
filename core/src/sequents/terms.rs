// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::logics::Logic;

use std::collections::HashMap;

pub type Polarity = bool;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression<L: Logic, VarType> {
    Var(bool, VarType),
    MultConst(L::Mult, Polarity),
    AddConst(L::Add, Polarity),
    MultOp(L::Mult, Polarity, usize, usize),
    AddOp(L::Add, Polarity, usize, usize),
    ExpOp(L::Exp, Polarity, usize),
}

pub struct Terms<L: Logic> {
    pub(super) term_arena: Vec<Expression<L, String>>,
    pub(super) term_ids: Vec<usize>,
}

impl<L: Logic> Expression<L, usize> {
    pub fn offset(self, offset_variables: usize, offset_terms: usize) -> Expression<L, usize> {
        use Expression::*;
        match self {
            Var(b, n) => Var(b, n + offset_variables),
            MultConst(marker, b) => MultConst(marker, b),
            AddConst(marker, b) => AddConst(marker, b),
            MultOp(marker, b, m, n) => MultOp(marker, b, m + offset_terms, n + offset_terms),
            AddOp(marker, b, m, n) => AddOp(marker, b, m + offset_terms, n + offset_terms),
            ExpOp(marker, b, n) => ExpOp(marker, b, n + offset_terms),
        }
    }

    pub fn check_bounds(&self, num_variables: usize, exp_index: usize) -> Result<(), crate::Error> {
        use Expression::*;
        match *self {
            Var(_, n) if n >= num_variables => {
                Err(crate::Error::InvalidVariableIndex(n, num_variables))
            }
            MultOp(_, _, n, _) | AddOp(_, _, n, _) if n >= exp_index => {
                Err(crate::Error::SubtermIndexNotDecreasing(n, num_variables))
            }
            MultOp(_, _, _, n) | AddOp(_, _, _, n) if n >= exp_index => {
                Err(crate::Error::SubtermIndexNotDecreasing(n, num_variables))
            }
            ExpOp(_, _, n) if n >= exp_index => {
                Err(crate::Error::SubtermIndexNotDecreasing(n, num_variables))
            }
            _ => Ok(()),
        }
    }
}

pub(super) enum VariableIDAssignmentResult<L: Logic> {
    NotNewID(Expression<L, usize>),
    NewID(Expression<L, usize>, usize, String),
}

impl<L: Logic> Expression<L, String> {
    pub(super) fn assign_variable_id(
        self,
        hm: &HashMap<String, usize>,
    ) -> VariableIDAssignmentResult<L> {
        use Expression::*;
        use VariableIDAssignmentResult::*;
        match self {
            Var(b, name) => {
                if let Some(n) = hm.get(&name) {
                    NotNewID(Var(b, *n))
                } else {
                    let fresh_id = hm.len();
                    NewID(Var(b, fresh_id), fresh_id, name)
                }
            }
            MultConst(marker, b) => NotNewID(MultConst(marker, b)),
            AddConst(marker, b) => NotNewID(AddConst(marker, b)),
            MultOp(marker, b, m, n) => NotNewID(MultOp(marker, b, m, n)),
            AddOp(marker, b, m, n) => NotNewID(AddOp(marker, b, m, n)),
            ExpOp(marker, b, m) => NotNewID(ExpOp(marker, b, m)),
        }
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;
    #[test]
    fn test_check_bounds() {
        assert!(
            match Expression::<super::super::logics::LL, usize>::MultOp((), true, 10, 0)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL, usize>::MultOp((), true, 0, 10)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL, usize>::MultOp((), true, 5, 0)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL, usize>::MultOp((), true, 0, 5)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );

        assert!(
            match Expression::<super::super::logics::LL, usize>::AddOp((), true, 0, 10)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL, usize>::AddOp((), true, 10, 0)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL, usize>::AddOp((), true, 0, 5)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL, usize>::AddOp((), true, 5, 0)
                .check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
    }
}

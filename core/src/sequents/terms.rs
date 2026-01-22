// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::logics::Logic;

pub type Polarity = bool;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Expression<L: Logic, VarType> {
    Var(bool, VarType),
    MultConst(L::Mult, Polarity),
    AddConst(L::Add, Polarity),
    MultOp(L::Mult, Polarity, usize, usize),
    AddOp(L::Add, Polarity, usize, usize),
    ExpOp(L::Exp, Polarity, usize),
}

pub struct Terms<L: Logic, VarType> {
    term_arena: Vec<Expression<L, VarType>>,
    term_ids: Vec<usize>,
}

impl<L: Logic> Expression<L, usize> {
    pub fn check_bounds(&self, num_variables: usize, num_terms: usize) -> Result<(), crate::Error> {
        use Expression::*;
        match *self {
            Var(_, n) if n >= num_variables => {
                Err(crate::Error::InvalidVariableIndex(n, num_variables))
            }
            MultOp(_, _, n, _) | AddOp(_, _, n, _) if n >= num_terms => {
                Err(crate::Error::InvalidTermIndex(n, num_variables))
            }
            MultOp(_, _, _, n) | AddOp(_, _, _, n) if n >= num_terms => {
                Err(crate::Error::InvalidTermIndex(n, num_variables))
            }
            ExpOp(_, _, n) if n >= num_terms => {
                Err(crate::Error::InvalidTermIndex(n, num_variables))
            }
            _ => Ok(()),
        }
    }
}

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

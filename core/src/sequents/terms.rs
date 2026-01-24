// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::logics::Logic;

pub type Polarity = bool;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression<L: Logic> {
    Var(bool, usize),
    MultConst(L::Mult, Polarity),
    AddConst(L::Add, Polarity),
    MultOp(L::Mult, Polarity, usize, usize),
    AddOp(L::Add, Polarity, usize, usize),
    ExpOp(L::Exp, Polarity, usize),
}

impl<L: Logic> Expression<L> {
    pub fn invert(self) -> Self {
        use Expression::*;
        match self {
            Var(b, n) => Var(!b, n),
            MultConst(marker, b) => MultConst(marker, !b),
            AddConst(marker, b) => AddConst(marker, !b),
            MultOp(marker, b, n, m) => MultOp(marker, !b, n, m),
            AddOp(marker, b, n, m) => AddOp(marker, !b, n, m),
            ExpOp(marker, b, n) => ExpOp(marker, !b, n),
        }
    }
}

impl<L: Logic> Expression<L> {
    pub fn offset(self, offset_variables: usize, offset_terms: usize) -> Expression<L> {
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
    NotNewID(Expression<L>),
    NewID(Expression<L>, usize, String),
}

#[allow(unused_imports)]
mod test {
    use super::*;
    #[test]
    fn test_check_bounds() {
        assert!(
            match Expression::<super::super::logics::LL>::MultOp((), true, 10, 0).check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL>::MultOp((), true, 0, 10).check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL>::MultOp((), true, 5, 0).check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL>::MultOp((), true, 0, 5).check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );

        assert!(
            match Expression::<super::super::logics::LL>::AddOp((), true, 0, 10).check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL>::AddOp((), true, 10, 0).check_bounds(5, 5)
            {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL>::AddOp((), true, 0, 5).check_bounds(5, 5) {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match Expression::<super::super::logics::LL>::AddOp((), true, 5, 0).check_bounds(5, 5) {
                Ok(_) => false,
                Err(_) => true,
            }
        );
    }
}

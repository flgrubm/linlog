// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;
use crate::IndexT;
use subenum::subenum;

#[subenum(MLLExpression)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LLExpression {
    #[subenum(MLLExpression)]
    Var(IndexT),
    #[subenum(MLLExpression)]
    DualVar(IndexT),
    #[subenum(MLLExpression)]
    One,
    #[subenum(MLLExpression)]
    Bot,
    Top,
    Zero,
    #[subenum(MLLExpression)]
    Tensor(IndexT, IndexT),
    #[subenum(MLLExpression)]
    Par(IndexT, IndexT),
    With(IndexT, IndexT),
    Plus(IndexT, IndexT),
    Bang(IndexT),
    Quest(IndexT),
}

#[inline]
const fn dualize_ll(e: LLExpression) -> LLExpression {
    use LLExpression::*;
    match e {
        Var(n) => DualVar(n),
        DualVar(n) => Var(n),
        One => Bot,
        Bot => One,
        Top => Zero,
        Zero => Top,
        Tensor(m, n) => Par(m, n),
        Par(m, n) => Tensor(m, n),
        With(m, n) => Plus(m, n),
        Plus(m, n) => With(m, n),
        Bang(n) => Quest(n),
        Quest(n) => Bang(n),
    }
}

#[inline]
const fn offset_ll(e: LLExpression, offset_variable: IndexT, offset_terms: IndexT) -> LLExpression {
    use LLExpression::*;
    match e {
        Var(n) => Var(n + offset_variable),
        DualVar(n) => DualVar(n + offset_variable),
        One => One,
        Bot => Bot,
        Top => Top,
        Zero => Zero,
        Tensor(m, n) => Tensor(m + offset_terms, n + offset_terms),
        Par(m, n) => Par(m + offset_terms, n + offset_terms),
        With(m, n) => With(m + offset_terms, n + offset_terms),
        Plus(m, n) => Plus(m + offset_terms, n + offset_terms),
        Bang(n) => Bang(n + offset_terms),
        Quest(n) => Quest(n + offset_terms),
    }
}

#[inline]
const fn check_bounds_ll(
    e: LLExpression,
    variable_bound: IndexT,
    index_bound: IndexT,
) -> Result<(), Error> {
    use LLExpression::*;
    match e {
        Var(n) | DualVar(n) if n >= variable_bound => {
            Err(Error::InvalidVariableIndex(n, variable_bound))
        }
        Tensor(m, _) | Par(m, _) | With(m, _) | Plus(m, _) | Bang(m) | Quest(m)
            if m >= index_bound =>
        {
            Err(Error::SubtermIndexNotDecreasing(m, index_bound))
        }
        Tensor(_, n) | Par(_, n) | With(_, n) | Plus(_, n) if n >= index_bound => {
            Err(Error::SubtermIndexNotDecreasing(n, index_bound))
        }
        _ => Ok(()),
    }
}

pub trait Expression:
    Into<LLExpression>
    + TryFrom<LLExpression, Error: std::error::Error>
    + Clone
    + Copy
    + std::fmt::Debug
    + PartialEq
    + Eq
    + std::hash::Hash
{
    fn dualize(&self) -> Self {
        Self::try_from(dualize_ll((*self).into())).unwrap()
    }

    fn offset(&self, offset_variable: IndexT, offset_terms: IndexT) -> Self {
        Self::try_from(offset_ll((*self).into(), offset_variable, offset_terms)).unwrap()
    }

    fn check_bounds(&self, variable_bound: IndexT, index_bound: IndexT) -> Result<(), Error> {
        check_bounds_ll((*self).into(), variable_bound, index_bound)
    }
}

impl Expression for LLExpression {}
impl Expression for MLLExpression {}

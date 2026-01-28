// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;
use crate::index::Index;
use subenum::subenum;

#[subenum(MLLExpression)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LLExpression<I: Index> {
    #[subenum(MLLExpression)]
    Var(I),
    #[subenum(MLLExpression)]
    DualVar(I),
    #[subenum(MLLExpression)]
    One,
    #[subenum(MLLExpression)]
    Bot,
    Top,
    Zero,
    #[subenum(MLLExpression)]
    Tensor(I, I),
    #[subenum(MLLExpression)]
    Par(I, I),
    With(I, I),
    Plus(I, I),
    Bang(I),
    Quest(I),
}

#[inline(always)]
const fn dualize_ll<I: Index>(e: LLExpression<I>) -> LLExpression<I> {
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

#[inline(always)]
fn offset_ll<I: Index>(e: LLExpression<I>, offset_variable: I, offset_terms: I) -> LLExpression<I> {
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

#[inline(always)]
fn check_bounds_ll<I: Index>(
    e: LLExpression<I>,
    variable_bound: I,
    index_bound: I,
) -> Result<(), Error> {
    use LLExpression::*;
    match e {
        Var(n) | DualVar(n) if n >= variable_bound => Err(Error::InvalidVariableIndex(
            n.as_usize(),
            variable_bound.as_usize(),
        )),
        Tensor(m, _) | Par(m, _) | With(m, _) | Plus(m, _) | Bang(m) | Quest(m)
            if m >= index_bound =>
        {
            Err(Error::SubtermIndexNotDecreasing(
                m.as_usize(),
                index_bound.as_usize(),
            ))
        }
        Tensor(_, n) | Par(_, n) | With(_, n) | Plus(_, n) if n >= index_bound => Err(
            Error::SubtermIndexNotDecreasing(n.as_usize(), index_bound.as_usize()),
        ),
        _ => Ok(()),
    }
}

pub trait Expression<I: Index>:
    Into<LLExpression<I>>
    + TryFrom<LLExpression<I>, Error: std::error::Error>
    + Clone
    + Copy
    + std::fmt::Debug
    + PartialEq
    + Eq
    + std::hash::Hash
{
    fn dualize(self) -> Self {
        Self::try_from(dualize_ll(self.into())).unwrap()
    }

    fn offset(self, offset_variable: I, offset_terms: I) -> Self {
        Self::try_from(offset_ll(self.into(), offset_variable, offset_terms)).unwrap()
    }

    fn check_bounds(self, variable_bound: I, index_bound: I) -> Result<(), Error> {
        check_bounds_ll(self.into(), variable_bound, index_bound)
    }
}

impl<I: Index> Expression<I> for LLExpression<I> {}
impl<I: Index> Expression<I> for MLLExpression<I> {}

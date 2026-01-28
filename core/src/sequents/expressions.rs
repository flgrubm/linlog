// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;
use crate::index::Index;
use subenum::subenum;

#[subenum(MLLExpressionNew)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LLExpressionNew<I: Index> {
    #[subenum(MLLExpressionNew)]
    Var(I),
    #[subenum(MLLExpressionNew)]
    DualVar(I),
    #[subenum(MLLExpressionNew)]
    One,
    #[subenum(MLLExpressionNew)]
    Bot,
    Top,
    Zero,
    #[subenum(MLLExpressionNew)]
    Tensor(I, I),
    #[subenum(MLLExpressionNew)]
    Par(I, I),
    With(I, I),
    Plus(I, I),
    Bang(I),
    Quest(I),
}

#[inline]
const fn dualize_llnew<I: Index>(e: LLExpressionNew<I>) -> LLExpressionNew<I> {
    use LLExpressionNew::*;
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
fn offset_llnew<I: Index>(
    e: LLExpressionNew<I>,
    offset_variable: I,
    offset_terms: I,
) -> LLExpressionNew<I> {
    use LLExpressionNew::*;
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
fn check_bounds_llnew<I: Index>(
    e: LLExpressionNew<I>,
    variable_bound: I,
    index_bound: I,
) -> Result<(), Error> {
    use LLExpressionNew::*;
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

pub trait ExpressionNew<I: Index>:
    Into<LLExpressionNew<I>>
    + TryFrom<LLExpressionNew<I>, Error: std::error::Error>
    + Clone
    + Copy
    + std::fmt::Debug
    + PartialEq
    + Eq
    + std::hash::Hash
{
    fn dualize(&self) -> Self {
        Self::try_from(dualize_llnew((*self).into())).unwrap()
    }

    fn offset(&self, offset_variable: I, offset_terms: I) -> Self {
        Self::try_from(offset_llnew((*self).into(), offset_variable, offset_terms)).unwrap()
    }

    fn check_bounds(&self, variable_bound: I, index_bound: I) -> Result<(), Error> {
        check_bounds_llnew((*self).into(), variable_bound, index_bound)
    }
}

impl<I: Index> ExpressionNew<I> for LLExpressionNew<I> {}
impl<I: Index> ExpressionNew<I> for MLLExpressionNew<I> {}

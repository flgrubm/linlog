// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;
use crate::index::Index;
use subenum::subenum;

// subenum copies a `///` on `LLExpression` to every subenum, so each gets its
// own doc here instead.
#[subenum(
    LLExpression(
        doc = "A node of a classical linear logic formula, which refers to its \
               subterms by arena index."
    ),
    MLLExpression(
        doc = "A node of a multiplicative linear logic formula, which refers to its \
               subterms by arena index."
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LLExpression<I: Index> {
    /// A variable, by its index in the variable dictionary.
    #[subenum(MLLExpression)]
    Var(I),
    /// The negation of a variable, by its index in the variable dictionary.
    #[subenum(MLLExpression)]
    DualVar(I),
    /// `1`, the unit of `⊗`.
    #[subenum(MLLExpression)]
    One,
    /// `⊥`, the unit of `⅋`.
    #[subenum(MLLExpression)]
    Bot,
    /// `⊤`, the unit of `&`.
    Top,
    /// `0`, the unit of `⊕`.
    Zero,
    /// `A ⊗ B`
    #[subenum(MLLExpression)]
    Tensor(I, I),
    /// `A ⅋ B`
    #[subenum(MLLExpression)]
    Par(I, I),
    /// `A & B`
    With(I, I),
    /// `A ⊕ B`
    Plus(I, I),
    /// `!A`
    Bang(I),
    /// `?A`
    Quest(I),
}

/// Returns `e` with its top node dualised (`⊗` to `⅋`, `A` to `~A`, …) and
/// the same subterm indices.
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

/// Shifts the variable and subterm indices in `e` by the given offsets.
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

/// Checks that `e` refers only to variables below `variable_bound` and to
/// subterms below `index_bound`.
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

/// An expression node of any logic fragment. Every operation is implemented
/// once, on `LLExpression`, and reached through conversion.
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
    /// Returns the expression with its top node dualised (`⊗` to `⅋`, `A` to
    /// `~A`, …) and the same subterm indices.
    fn dualize(self) -> Self {
        Self::try_from(dualize_ll(self.into())).unwrap()
    }

    /// Shifts the variable and subterm indices by the given offsets.
    fn offset(self, offset_variable: I, offset_terms: I) -> Self {
        Self::try_from(offset_ll(self.into(), offset_variable, offset_terms)).unwrap()
    }

    /// Checks that the expression refers only to variables below
    /// `variable_bound` and to subterms below `index_bound`.
    fn check_bounds(self, variable_bound: I, index_bound: I) -> Result<(), Error> {
        check_bounds_ll(self.into(), variable_bound, index_bound)
    }
}

impl<I: Index> Expression<I> for LLExpression<I> {}
impl<I: Index> Expression<I> for MLLExpression<I> {}

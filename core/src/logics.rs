// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::expressions::{Expression, LLExpression, MLLExpression};
use crate::index::Index;

/// A logic fragment, which fixes the expressions its sequents may contain.
pub trait Logic<I: Index>: std::fmt::Debug {
    /// The expression type of this fragment's sequents.
    type Expression: Expression<I>;
}

/// Classical linear logic, with every connective.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LL;

impl<I: Index> Logic<I> for LL {
    type Expression = LLExpression<I>;
}

/// Classical multiplicative linear logic: variables, `1`, `⊥`, `⊗` and `⅋`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MLL;

impl<I: Index> Logic<I> for MLL {
    type Expression = MLLExpression<I>;
}

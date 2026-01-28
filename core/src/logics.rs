// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::expressions::{Expression, LLExpression, MLLExpression};
use crate::index::Index;

pub trait Logic<I: Index>: std::fmt::Debug {
    type Expression: Expression<I>;
}

// classical linear logic

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LL;

impl<I: Index> Logic<I> for LL {
    type Expression = LLExpression<I>;
}

// classical multiplicative linear logic

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MLL;

impl<I: Index> Logic<I> for MLL {
    type Expression = MLLExpression<I>;
}

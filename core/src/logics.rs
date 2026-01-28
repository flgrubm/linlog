// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::expressions::{ExpressionNew, LLExpressionNew, MLLExpressionNew};
use crate::index::Index;

pub trait LogicNew<I: Index>: std::fmt::Debug {
    type Expression: ExpressionNew<I>;
}

// classical linear logic

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LLNew;

impl<I: Index> LogicNew<I> for LLNew {
    type Expression = LLExpressionNew<I>;
}

// classical multiplicative linear logic

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MLLNew;

impl<I: Index> LogicNew<I> for MLLNew {
    type Expression = MLLExpressionNew<I>;
}

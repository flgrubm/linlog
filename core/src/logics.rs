// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::expressions::{Expression, LLExpression, MLLExpression};

pub trait Logic: std::fmt::Debug {
    type Expression: Expression;
}

// classical linear logic

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LL;

impl Logic for LL {
    type Expression = LLExpression;
}

// classical multiplicative linear logic

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MLL;

impl Logic for MLL {
    type Expression = MLLExpression;
}

#[test]
fn test_from() {
    assert_eq!(
        LLExpression::One,
        <LL as Logic>::Expression::from(LLExpression::One)
    );
}

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

// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::expressions::{Expression, LLExpression, MLLExpression};

pub trait Logic {
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

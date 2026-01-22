// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::logics::Logic;

pub type Polarity = bool;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Expression<L: Logic, VarType> {
    Var(bool, VarType),
    MultConst(L::Mult, Polarity),
    AddConst(L::Add, Polarity),
    MultOp(L::Mult, Polarity, usize, usize),
    AddOp(L::Add, Polarity, usize, usize),
    ExpOp(L::Exp, Polarity, usize),
}

pub struct Terms<L: Logic, VarType> {
    term_arena: Vec<Expression<L, VarType>>,
    term_ids: Vec<usize>,
}

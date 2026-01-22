// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod logics;
// pub mod parsing;

use logics::Logic;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Term<L: Logic> {
    Var(bool, usize),
    MultConst(L::MultConst),
    AddConst(L::AddConst),
    MultOp(L::MultOp),
    AddOp(L::AddOp),
    ExpOp(L::ExpOp),
}

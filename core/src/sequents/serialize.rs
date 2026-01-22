// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::Sequent as Seq;
use super::logics::LL;
use super::terms::Expression as Expr;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "E")]
pub(super) enum Expression {
    #[serde(rename = "V")]
    Var(usize),
    #[serde(rename = "D")]
    DualVar(usize),
    #[serde(rename = "1")]
    One,
    #[serde(rename = "⊥")]
    Bot,
    #[serde(rename = "⊤")]
    Top,
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "!")]
    Bang(usize),
    #[serde(rename = "?")]
    Quest(usize),
    #[serde(rename = "⊗")]
    Tensor(usize, usize),
    #[serde(rename = "⅋")]
    Par(usize, usize),
    #[serde(rename = "&")]
    With(usize, usize),
    #[serde(rename = "⊕")]
    Plus(usize, usize),
}

impl From<Expr<LL, usize>> for Expression {
    fn from(e: Expr<LL, usize>) -> Expression {
        use Expr::*;
        use Expression::*;
        match e {
            Expr::Var(true, n) => Expression::Var(n),
            Expr::Var(false, n) => DualVar(n),
            MultConst(_, true) => One,
            MultConst(_, false) => Bot,
            AddConst(_, true) => Top,
            AddConst(_, false) => Zero,
            MultOp(_, true, s, t) => Tensor(s, t),
            MultOp(_, false, s, t) => Par(s, t),
            AddOp(_, true, s, t) => With(s, t),
            AddOp(_, false, s, t) => Plus(s, t),
            ExpOp(_, true, s) => Bang(s),
            ExpOp(_, false, s) => Quest(s),
        }
    }
}

impl From<Expression> for Expr<LL, usize> {
    fn from(e: Expression) -> Expr<LL, usize> {
        use Expr::*;
        use Expression::*;
        match e {
            Expression::Var(n) => Expr::Var(true, n),
            DualVar(n) => Expr::Var(false, n),
            One => MultConst((), true),
            Bot => MultConst((), false),
            Top => AddConst((), true),
            Zero => AddConst((), false),
            Tensor(s, t) => MultOp((), true, s, t),
            Par(s, t) => MultOp((), false, s, t),
            With(s, t) => AddOp((), true, s, t),
            Plus(s, t) => AddOp((), false, s, t),
            Bang(s) => ExpOp((), true, s),
            Quest(s) => ExpOp((), false, s),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Sequent {
    terms: Vec<Expression>,
    ids: Vec<usize>,
    var_dict: Vec<String>,
}

impl From<Seq<LL>> for Sequent {
    fn from(s: Seq<LL>) -> Sequent {
        Sequent {
            terms: s.term_arena.into_iter().map(Expression::from).collect(),
            ids: s.term_ids,
            var_dict: s.variable_dict,
        }
    }
}

impl TryFrom<Sequent> for Seq<LL> {
    type Error = crate::Error;

    fn try_from(s: Sequent) -> Result<Seq<LL>, Self::Error> {
        let s = Seq::<LL> {
            term_arena: s.terms.into_iter().map(Expr::from).collect(),
            term_ids: s.ids,
            variable_dict: s.var_dict,
        };
        s.verify_integrity()?;
        Ok(s)
    }
}

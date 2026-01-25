// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::logics::LL;
use crate::sequents::Sequent as Seq;
use crate::sequents::expressions::LLExpression;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "E")]
enum Expression {
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
    #[serde(rename = "⊗")]
    Tensor(usize, usize),
    #[serde(rename = "⅋")]
    Par(usize, usize),
    #[serde(rename = "&")]
    With(usize, usize),
    #[serde(rename = "⊕")]
    Plus(usize, usize),
    #[serde(rename = "!")]
    Bang(usize),
    #[serde(rename = "?")]
    Quest(usize),
}

impl From<LLExpression> for Expression {
    fn from(e: LLExpression) -> Self {
        use Expression as E;
        use LLExpression::*;
        match e {
            Var(n) => E::Var(n),
            DualVar(n) => E::DualVar(n),
            One => E::One,
            Bot => E::Bot,
            Top => E::Top,
            Zero => E::Zero,
            Tensor(m, n) => E::Tensor(m, n),
            Par(m, n) => E::Par(m, n),
            With(m, n) => E::With(m, n),
            Plus(m, n) => E::Plus(m, n),
            Bang(m) => E::Bang(m),
            Quest(m) => E::Quest(m),
        }
    }
}

impl From<Expression> for LLExpression {
    fn from(e: Expression) -> Self {
        use Expression::*;
        use LLExpression as E;
        match e {
            Var(n) => E::Var(n),
            DualVar(n) => E::DualVar(n),
            One => E::One,
            Bot => E::Bot,
            Top => E::Top,
            Zero => E::Zero,
            Tensor(m, n) => E::Tensor(m, n),
            Par(m, n) => E::Par(m, n),
            With(m, n) => E::With(m, n),
            Plus(m, n) => E::Plus(m, n),
            Bang(m) => E::Bang(m),
            Quest(m) => E::Quest(m),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Sequent {
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
            term_arena: s.terms.into_iter().map(LLExpression::from).collect(),
            term_ids: s.ids,
            variable_dict: s.var_dict,
        };
        s.verify_integrity()?;
        Ok(s)
    }
}

impl serde::Serialize for Seq<LL> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proxy = Sequent::from(self.clone());
        proxy.serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for Seq<LL> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = Sequent::deserialize(deserializer)?;
        Seq::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

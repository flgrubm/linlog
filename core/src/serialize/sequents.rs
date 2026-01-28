// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::logics::LLNew;
use crate::sequents::SequentNew as SeqNew;
use crate::sequents::expressions::LLExpressionNew;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Sequent {
    terms: Vec<Expression>,
    ids: Vec<usize>,
    var_dict: Vec<String>,
}

impl From<LLExpressionNew<usize>> for Expression {
    fn from(e: LLExpressionNew<usize>) -> Self {
        use Expression as E;
        use LLExpressionNew::*;
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

impl From<Expression> for LLExpressionNew<usize> {
    fn from(e: Expression) -> Self {
        use Expression::*;
        use LLExpressionNew as E;
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

impl From<SeqNew<usize, LLNew>> for Sequent {
    fn from(s: SeqNew<usize, LLNew>) -> Sequent {
        Sequent {
            terms: s.term_arena.into_iter().map(Expression::from).collect(),
            ids: s.term_ids,
            var_dict: s.variable_dict,
        }
    }
}

impl TryFrom<Sequent> for SeqNew<usize, LLNew> {
    type Error = crate::Error;

    fn try_from(s: Sequent) -> Result<SeqNew<usize, LLNew>, Self::Error> {
        let s = SeqNew::<usize, LLNew> {
            term_arena: s.terms.into_iter().map(LLExpressionNew::from).collect(),
            term_ids: s.ids,
            variable_dict: s.var_dict,
        };
        s.verify_integrity()?;
        Ok(s)
    }
}

impl serde::Serialize for SeqNew<usize, LLNew> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proxy = Sequent::from(self.clone());
        proxy.serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for SeqNew<usize, LLNew> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = Sequent::deserialize(deserializer)?;
        SeqNew::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

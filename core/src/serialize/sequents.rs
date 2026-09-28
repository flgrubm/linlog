// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::logics::LL;
use crate::sequents::Sequent as Seq;
use crate::sequents::expressions::LLExpression;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of an arena expression. Its tags are part of the
/// interchange format, so renaming one breaks it.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "E")]
enum Expression {
    /// A variable, by dictionary index.
    #[serde(rename = "V")]
    Var(usize),
    /// A negated variable, by dictionary index.
    #[serde(rename = "D")]
    DualVar(usize),
    /// `1`
    #[serde(rename = "1")]
    One,
    /// `⊥`
    #[serde(rename = "⊥")]
    Bot,
    /// `⊤`
    #[serde(rename = "⊤")]
    Top,
    /// `0`
    #[serde(rename = "0")]
    Zero,
    /// `A ⊗ B`
    #[serde(rename = "⊗")]
    Tensor(usize, usize),
    /// `A ⅋ B`
    #[serde(rename = "⅋")]
    Par(usize, usize),
    /// `A & B`
    #[serde(rename = "&")]
    With(usize, usize),
    /// `A ⊕ B`
    #[serde(rename = "⊕")]
    Plus(usize, usize),
    /// `!A`
    #[serde(rename = "!")]
    Bang(usize),
    /// `?A`
    #[serde(rename = "?")]
    Quest(usize),
}

/// The serialized form of a sequent.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Sequent {
    /// The arena.
    terms: Vec<Expression>,
    /// The root formulas, as arena indices.
    ids: Vec<usize>,
    /// The variable names.
    var_dict: Vec<String>,
}

impl From<LLExpression<usize>> for Expression {
    /// Converts an arena expression into its serialized form.
    fn from(e: LLExpression<usize>) -> Self {
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

impl From<Expression> for LLExpression<usize> {
    /// Converts a serialized expression back into an arena expression.
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

impl From<Seq<usize, LL>> for Sequent {
    /// Converts a sequent into its serialized form.
    fn from(s: Seq<usize, LL>) -> Sequent {
        Sequent {
            terms: s.term_arena.into_iter().map(Expression::from).collect(),
            ids: s.term_ids,
            var_dict: s.variable_dict,
        }
    }
}

impl TryFrom<Sequent> for Seq<usize, LL> {
    type Error = crate::Error;

    /// Converts a deserialized sequent back, failing if its arena breaks the
    /// invariants.
    fn try_from(s: Sequent) -> Result<Seq<usize, LL>, Self::Error> {
        let s = Seq::<usize, LL> {
            term_arena: s.terms.into_iter().map(LLExpression::from).collect(),
            term_ids: s.ids,
            variable_dict: s.var_dict,
        };
        s.verify_integrity()?;
        Ok(s)
    }
}

impl serde::Serialize for Seq<usize, LL> {
    /// Serializes the sequent as its arena, root term indices and variable names.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proxy = Sequent::from(self.clone());
        proxy.serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for Seq<usize, LL> {
    /// Deserializes a sequent and checks that its arena keeps the invariants.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = Sequent::deserialize(deserializer)?;
        Seq::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

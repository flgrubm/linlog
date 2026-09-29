// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::sequents::{Atom, Sequent as Seq, Term, TermId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of an arena term. Its tags are part of the interchange
/// format, so renaming one breaks it.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "E")]
enum Expression {
    /// A variable, by dictionary index.
    #[serde(rename = "V")]
    Var(u32),
    /// A negated variable, by dictionary index.
    #[serde(rename = "D")]
    DualVar(u32),
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
    Tensor(u32, u32),
    /// `A ⅋ B`
    #[serde(rename = "⅋")]
    Par(u32, u32),
    /// `A & B`
    #[serde(rename = "&")]
    With(u32, u32),
    /// `A ⊕ B`
    #[serde(rename = "⊕")]
    Plus(u32, u32),
    /// `!A`
    #[serde(rename = "!")]
    Bang(u32),
    /// `?A`
    #[serde(rename = "?")]
    Quest(u32),
}

/// The serialized form of a sequent.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Sequent {
    /// The arena.
    terms: Vec<Expression>,
    /// The root formulas, as arena indices.
    ids: Vec<u32>,
    /// The atom names.
    var_dict: Vec<String>,
}

impl From<Term> for Expression {
    /// Converts an arena term into its serialized form.
    fn from(e: Term) -> Self {
        use Expression as E;
        use Term::*;
        match e {
            Var(a) => E::Var(a.get()),
            DualVar(a) => E::DualVar(a.get()),
            One => E::One,
            Bot => E::Bot,
            Top => E::Top,
            Zero => E::Zero,
            Tensor(k, l) => E::Tensor(k.get(), l.get()),
            Par(k, l) => E::Par(k.get(), l.get()),
            With(k, l) => E::With(k.get(), l.get()),
            Plus(k, l) => E::Plus(k.get(), l.get()),
            Bang(k) => E::Bang(k.get()),
            Quest(k) => E::Quest(k.get()),
        }
    }
}

impl From<Expression> for Term {
    /// Converts a serialized expression back into an arena term.
    fn from(e: Expression) -> Self {
        use Expression::*;
        use Term as E;
        let t = TermId::new;
        match e {
            Var(a) => E::Var(Atom::new(a)),
            DualVar(a) => E::DualVar(Atom::new(a)),
            One => E::One,
            Bot => E::Bot,
            Top => E::Top,
            Zero => E::Zero,
            Tensor(k, l) => E::Tensor(t(k), t(l)),
            Par(k, l) => E::Par(t(k), t(l)),
            With(k, l) => E::With(t(k), t(l)),
            Plus(k, l) => E::Plus(t(k), t(l)),
            Bang(k) => E::Bang(t(k)),
            Quest(k) => E::Quest(t(k)),
        }
    }
}

impl From<&Seq> for Sequent {
    /// Converts a sequent into its serialized form.
    fn from(s: &Seq) -> Sequent {
        Sequent {
            terms: s.terms.iter().copied().map(Expression::from).collect(),
            ids: s.roots.iter().map(|k| k.get()).collect(),
            var_dict: s.atoms.clone(),
        }
    }
}

impl TryFrom<Sequent> for Seq {
    type Error = crate::Error;

    /// Converts a deserialized sequent back, failing if its arena breaks the
    /// invariants.
    fn try_from(s: Sequent) -> Result<Seq, Self::Error> {
        let s = Seq {
            terms: s.terms.into_iter().map(Term::from).collect(),
            roots: s.ids.into_iter().map(TermId::new).collect(),
            atoms: s.var_dict,
        };
        s.verify_integrity()?;
        Ok(s)
    }
}

impl serde::Serialize for Seq {
    /// Serializes the sequent as its arena, root term indices and atom names.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Sequent::from(self).serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for Seq {
    /// Deserializes a sequent and checks that its arena keeps the invariants.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = Sequent::deserialize(deserializer)?;
        Seq::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

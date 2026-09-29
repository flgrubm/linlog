// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::occurrences::{Forest, OccId};
use crate::proofs::{Node, NodeId, Proof as Prf, Rule, Side};
use crate::sequents::Sequent;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of a proof node: the rule's tag with its occurrence
/// ids and premise node indices. The tags are part of the interchange
/// format, so renaming one breaks it.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "N")]
enum Step {
    /// The axiom on two literals.
    #[serde(rename = "ax")]
    Ax(u32, u32),
    /// `⊗` on an occurrence, from the left and the right premise.
    #[serde(rename = "⊗")]
    Tensor(u32, u32, u32),
    /// `⅋` on an occurrence, from a premise.
    #[serde(rename = "⅋")]
    Par(u32, u32),
    /// `1` on an occurrence.
    #[serde(rename = "1")]
    One(u32),
    /// `⊥` on an occurrence, from a premise.
    #[serde(rename = "⊥")]
    Bot(u32, u32),
    /// `&` on an occurrence, from the left and the right premise.
    #[serde(rename = "&")]
    With(u32, u32, u32),
    /// `⊕₁` on an occurrence, from a premise.
    #[serde(rename = "⊕₁")]
    PlusLeft(u32, u32),
    /// `⊕₂` on an occurrence, from a premise.
    #[serde(rename = "⊕₂")]
    PlusRight(u32, u32),
    /// `⊤` on an occurrence.
    #[serde(rename = "⊤")]
    Top(u32),
    /// Promotion on an occurrence, from a premise.
    #[serde(rename = "!")]
    Bang(u32, u32),
    /// The `?` step on an occurrence, from a premise.
    #[serde(rename = "?")]
    Quest(u32, u32),
    /// A copy of an occurrence, from a premise.
    #[serde(rename = "copy")]
    Copy(u32, u32),
    /// Weakening of an occurrence, from a premise.
    #[serde(rename = "wk")]
    Weaken(u32, u32),
    /// Mix, from two premises.
    #[serde(rename = "mix")]
    Mix(u32, u32),
}

/// The serialized form of a proof: its sequent and its nodes, premises
/// before conclusions, the root last.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Proof {
    /// The sequent proved.
    sequent: Sequent,
    /// The nodes.
    proof: Vec<Step>,
}

impl From<Node> for Step {
    /// Converts a node into its serialized form.
    fn from(n: Node) -> Self {
        use Node::*;
        use Step as S;
        let (o, n_) = (|o: OccId| o.get(), |p: NodeId| p.get());
        match n {
            Ax(a, b) => S::Ax(o(a), o(b)),
            Tensor(x, l, r) => S::Tensor(o(x), n_(l), n_(r)),
            Par(x, p) => S::Par(o(x), n_(p)),
            One(x) => S::One(o(x)),
            Bot(x, p) => S::Bot(o(x), n_(p)),
            With(x, l, r) => S::With(o(x), n_(l), n_(r)),
            Plus(x, Side::Left, p) => S::PlusLeft(o(x), n_(p)),
            Plus(x, Side::Right, p) => S::PlusRight(o(x), n_(p)),
            Top(x) => S::Top(o(x)),
            Bang(x, p) => S::Bang(o(x), n_(p)),
            Quest(x, p) => S::Quest(o(x), n_(p)),
            Copy(x, p) => S::Copy(o(x), n_(p)),
            Weaken(x, p) => S::Weaken(o(x), n_(p)),
            Mix(l, r) => S::Mix(n_(l), n_(r)),
        }
    }
}

impl From<Step> for Node {
    /// Converts a serialized step back into a node.
    fn from(s: Step) -> Self {
        use Node as N;
        use Step::*;
        let (o, n) = (OccId::new, NodeId::new);
        match s {
            Ax(a, b) => N::Ax(o(a), o(b)),
            Tensor(x, l, r) => N::Tensor(o(x), n(l), n(r)),
            Par(x, p) => N::Par(o(x), n(p)),
            One(x) => N::One(o(x)),
            Bot(x, p) => N::Bot(o(x), n(p)),
            With(x, l, r) => N::With(o(x), n(l), n(r)),
            PlusLeft(x, p) => N::Plus(o(x), Side::Left, n(p)),
            PlusRight(x, p) => N::Plus(o(x), Side::Right, n(p)),
            Top(x) => N::Top(o(x)),
            Bang(x, p) => N::Bang(o(x), n(p)),
            Quest(x, p) => N::Quest(o(x), n(p)),
            Copy(x, p) => N::Copy(o(x), n(p)),
            Weaken(x, p) => N::Weaken(o(x), n(p)),
            Mix(l, r) => N::Mix(n(l), n(r)),
        }
    }
}

impl From<&Prf> for Proof {
    /// Converts a proof into its serialized form.
    fn from(p: &Prf) -> Proof {
        Proof {
            sequent: p.sequent().clone(),
            proof: p.nodes().iter().copied().map(Step::from).collect(),
        }
    }
}

impl TryFrom<Proof> for Prf {
    type Error = crate::Error;

    /// Rebuilds the forest of the sequent and the proof over it, failing if
    /// a node refers outside the forest or the arena, or the arena is empty.
    fn try_from(p: Proof) -> Result<Prf, Self::Error> {
        let forest = Forest::try_from(p.sequent)?;
        let nodes: Vec<Node> = p.proof.into_iter().map(Node::from).collect();
        let root = NodeId::new(nodes.len().saturating_sub(1) as u32);
        Prf::new(forest, nodes, root)
    }
}

impl serde::Serialize for Prf {
    /// Serializes the proof as its sequent and its nodes.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Proof::from(self).serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for Prf {
    /// Deserializes a proof and checks that its nodes stay within the
    /// sequent's forest and refer only to earlier nodes; whether it proves
    /// the sequent is for [`Prf::check`].
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = Proof::deserialize(deserializer)?;
        Prf::try_from(proxy).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for Rule {
    /// Serializes the rule as its name.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'a> serde::Deserialize<'a> for Rule {
    /// Deserializes a rule from its name, as `Rule::from_str` reads it.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let name = <std::borrow::Cow<'a, str>>::deserialize(deserializer)?;
        name.parse().map_err(serde::de::Error::custom)
    }
}

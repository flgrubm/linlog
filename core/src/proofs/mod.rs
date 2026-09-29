// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proofs: terms over occurrence ids, the checker that validates a term
//! against its sequent independently of any engine, and the derivation view
//! for humans and exporters, as `plan/README.md` lays out.
//!
//! A [`Proof`] is an arena of [`Node`]s, one per rule instance, over the
//! [`Forest`] of the sequent it proves. A node names the rule, the
//! occurrence the rule acts on and the nodes that prove its premises. The
//! rules are those of the dyadic sequent calculus every engine searches in:
//! the standard rules, with the `?` formulas handled through an unrestricted
//! zone `Θ` (a `?` step moves a formula into it, a copy uses it without
//! consuming it), plus weakening for affine mode and Mix. A node does not
//! record the sequent it proves: the checker derives that from the premises,
//! and the derivation view shows it.

/// The checker.
pub mod check;
/// The derivation view.
pub mod derivation;
/// Text rendering of derivations.
mod fmt;
/// Multisets of occurrence ids.
mod multiset;

pub use check::{CheckError, Dyadic, Problem};
pub use derivation::{Derivation, InfId, Inference, Rule};

use crate::Error;
use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId};
use crate::sequents::Sequent;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The index of a node in a proof's arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// Wraps a raw arena index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw arena index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the arena.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Which subformula of a binary connective a rule picks.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// The left subformula.
    Left,
    /// The right subformula.
    Right,
}

/// One rule instance of a proof: the rule, the occurrence it acts on, and
/// the nodes that prove its premises, which precede it in the arena. The
/// rules are the dyadic calculus's, so `⊢ Θ ; Γ` below is a sequent with the
/// unrestricted zone `Θ` and the linear zone `Γ`; `Θ` is empty until a
/// [`Quest`](Self::Quest) fills it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    /// The axiom `⊢ Θ ; a, ~a` on two literal occurrences of one atom with
    /// opposite signs, in either order.
    Ax(OccId, OccId),
    /// `⊢ Θ ; Γ, Δ, A ⊗ B` from `⊢ Θ ; Γ, A` and `⊢ Θ ; Δ, B`, on the
    /// occurrence of `A ⊗ B`; the premises prove its left and right
    /// subformula, in that order.
    Tensor(OccId, NodeId, NodeId),
    /// `⊢ Θ ; Γ, A ⅋ B` from `⊢ Θ ; Γ, A, B`.
    Par(OccId, NodeId),
    /// `⊢ Θ ; 1`.
    One(OccId),
    /// `⊢ Θ ; Γ, ⊥` from `⊢ Θ ; Γ`.
    Bot(OccId, NodeId),
    /// `⊢ Θ ; Γ, A & B` from `⊢ Θ ; Γ, A` and `⊢ Θ ; Γ, B`, the premises in
    /// that order.
    With(OccId, NodeId, NodeId),
    /// `⊢ Θ ; Γ, A ⊕ B` from `⊢ Θ ; Γ, A` or from `⊢ Θ ; Γ, B`, as the side
    /// says.
    Plus(OccId, Side, NodeId),
    /// `⊢ Θ ; Γ, ⊤` for any `Γ`.
    Top(OccId),
    /// Promotion: `⊢ Θ ; !A` from `⊢ Θ ; A`, with an empty linear zone.
    Bang(OccId, NodeId),
    /// `⊢ Θ ; Γ, ?A` from `⊢ Θ, A ; Γ`: the subformula moves into the
    /// unrestricted zone.
    Quest(OccId, NodeId),
    /// A copy: `⊢ Θ, A ; Γ` from `⊢ Θ, A ; Γ, A`, on the occurrence of `A`,
    /// which stays in `Θ`. The only rule that can repeat an occurrence in a
    /// branch, which is why the linear zone is a multiset.
    Copy(OccId, NodeId),
    /// Weakening: `⊢ Θ ; Γ, A` from `⊢ Θ ; Γ`, in affine mode, or in any
    /// mode when `A` is a `?` formula (the standard `?w`; the dyadic form of
    /// it is a [`Quest`](Self::Quest) whose formula goes unused).
    Weaken(OccId, NodeId),
    /// Mix, when the mode allows it: `⊢ Θ ; Γ, Δ` from `⊢ Θ ; Γ` and
    /// `⊢ Θ ; Δ`.
    Mix(NodeId, NodeId),
}

/// A node is a tag, an occurrence and two node indices at most.
const _: () = assert!(size_of::<Node>() == 16);

impl Node {
    /// Returns the occurrence the rule acts on: `None` for Mix, and the
    /// first literal of an axiom.
    pub const fn principal(self) -> Option<OccId> {
        use Node::*;
        match self {
            Ax(o, _)
            | Tensor(o, ..)
            | Par(o, _)
            | One(o)
            | Bot(o, _)
            | With(o, ..)
            | Plus(o, ..)
            | Top(o)
            | Bang(o, _)
            | Quest(o, _)
            | Copy(o, _)
            | Weaken(o, _) => Some(o),
            Mix(..) => None,
        }
    }

    /// Returns the occurrences the node names: one, or two for an axiom,
    /// or none for Mix.
    pub fn occurrences(self) -> impl Iterator<Item = OccId> {
        let second = match self {
            Node::Ax(_, b) => Some(b),
            _ => None,
        };
        self.principal().into_iter().chain(second)
    }

    /// Returns the nodes that prove the premises, in the rule's order: none
    /// for an axiom, `1` or `⊤`, two for `⊗`, `&` and Mix, one otherwise.
    pub fn premises(self) -> impl Iterator<Item = NodeId> {
        use Node::*;
        let (first, second) = match self {
            Ax(..) | One(_) | Top(_) => (None, None),
            Tensor(_, l, r) | With(_, l, r) | Mix(l, r) => (Some(l), Some(r)),
            Par(_, p)
            | Bot(_, p)
            | Plus(_, _, p)
            | Bang(_, p)
            | Quest(_, p)
            | Copy(_, p)
            | Weaken(_, p) => (Some(p), None),
        };
        first.into_iter().chain(second)
    }

    /// Returns the rule's name: `ax`, `⊗`, `⅋`, `1`, `⊥`, `&`, `⊕₁`, `⊕₂`,
    /// `⊤`, `!`, `?`, `copy`, `wk` or `mix`.
    pub const fn name(self) -> &'static str {
        use Node::*;
        match self {
            Ax(..) => "ax",
            Tensor(..) => "⊗",
            Par(..) => "⅋",
            One(_) => "1",
            Bot(..) => "⊥",
            With(..) => "&",
            Plus(_, Side::Left, _) => "⊕₁",
            Plus(_, Side::Right, _) => "⊕₂",
            Top(_) => "⊤",
            Bang(..) => "!",
            Quest(..) => "?",
            Copy(..) => "copy",
            Weaken(..) => "wk",
            Mix(..) => "mix",
        }
    }
}

impl Display for Node {
    /// Writes the rule's name, then `on` the occurrences and `from` the
    /// premise nodes, as in `⊗ on 1 from 0, 2`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())?;
        let mut occurrences = self.occurrences().map(|o| o.get());
        if let Some(o) = occurrences.next() {
            write!(f, " on {o}")?;
            for o in occurrences {
                write!(f, ", {o}")?;
            }
        }
        let mut premises = self.premises().map(|p| p.get());
        if let Some(p) = premises.next() {
            write!(f, " from {p}")?;
            for p in premises {
                write!(f, ", {p}")?;
            }
        }
        Ok(())
    }
}

/// A proof of a sequent: an arena of rule instances over the sequent's
/// occurrence forest, which the proof owns. Every node refers only to nodes
/// before it, the last node is the root, and every node is reachable from
/// the root; a subproof that two nodes share is stored once.
///
/// A proof is built bottom-up, premises before conclusions, and is what a
/// search engine returns. Whether it proves its sequent is a separate
/// question, answered by [`check`](Self::check).
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, Mode, Node, NodeId, OccId, Proof, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the occurrences 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let (o, n) = (OccId::new, NodeId::new);
/// let nodes = vec![
///     Node::Ax(o(0), o(2)),
///     Node::Ax(o(3), o(4)),
///     Node::Tensor(o(1), n(0), n(1)),
/// ];
/// let proof = Proof::new(forest, nodes, n(2))?;
/// proof.check(Mode::CLASSICAL)?;
/// assert_eq!(
///     proof.derivation()?.to_string(),
///     "─────── ax   ─────── ax\n\
///      ⊢ ~A, A      ⊢ ~B, B\n\
///      ──────────────────── ⊗\n\
///     \x20 ⊢ ~A, A ⊗ ~B, B"
/// );
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Proof {
    /// The forest of the sequent the proof is of.
    forest: Forest,
    /// The rule instances, premises before conclusions, the root last.
    nodes: Box<[Node]>,
}

impl Proof {
    /// Builds a proof from the nodes of an arena and its root, keeping the
    /// nodes the root reaches in their order. Fails if a node the root
    /// reaches names an occurrence outside the forest or a premise that does
    /// not precede it, or if the root lies outside the arena.
    pub fn new(forest: Forest, nodes: Vec<Node>, root: NodeId) -> Result<Self, Error> {
        if root.index() >= nodes.len() {
            return Err(Error::NodeIndexOutOfBounds(root.index(), nodes.len()));
        }
        // A premise precedes its conclusion, so one pass from the root down
        // visits every reachable node after the node that reaches it.
        let mut reachable = vec![false; root.index() + 1];
        reachable[root.index()] = true;
        for i in (0..=root.index()).rev() {
            if !reachable[i] {
                continue;
            }
            for o in nodes[i].occurrences() {
                if o.index() >= forest.len() {
                    return Err(Error::OccurrenceIndexOutOfBounds(o.index(), forest.len()));
                }
            }
            for p in nodes[i].premises() {
                if p.index() >= i {
                    return Err(Error::PremiseIndexNotDecreasing(p.index(), i));
                }
                reachable[p.index()] = true;
            }
        }
        let mut new_index = vec![NodeId::new(u32::MAX); root.index() + 1];
        let mut kept = Vec::with_capacity(reachable.iter().filter(|&&r| r).count());
        for (i, node) in nodes.into_iter().enumerate().take(root.index() + 1) {
            if !reachable[i] {
                continue;
            }
            new_index[i] = NodeId::new(kept.len() as u32);
            kept.push(node.map_premises(|p| new_index[p.index()]));
        }
        Ok(Self {
            forest,
            nodes: kept.into_boxed_slice(),
        })
    }

    /// Returns the forest of the sequent the proof is of.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the sequent the proof is of.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns the arena: every node, premises before conclusions, the root
    /// last.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Returns the node at `id`, which must belong to this proof.
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.index()]
    }

    /// Returns every node id in arena order.
    pub fn ids(&self) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator {
        (0..self.nodes.len() as u32).map(NodeId::new)
    }

    /// Returns the root: the node that concludes the sequent, the last one.
    pub fn root(&self) -> NodeId {
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Checks that the proof proves its sequent under the rules the mode
    /// allows; see [`check::check`].
    pub fn check(&self, mode: Mode) -> Result<(), CheckError> {
        check::check(self, mode)
    }

    /// Unfolds the proof into the derivation of the standard sequent
    /// calculus it stands for, or reports why it is not a proof; see
    /// [`Derivation`].
    pub fn derivation(&self) -> Result<Derivation<'_>, CheckError> {
        Derivation::new(self)
    }
}

impl Node {
    /// Returns the node with every premise index replaced by what `f` maps
    /// it to.
    fn map_premises(self, mut f: impl FnMut(NodeId) -> NodeId) -> Self {
        use Node::*;
        match self {
            Tensor(o, l, r) => Tensor(o, f(l), f(r)),
            Par(o, p) => Par(o, f(p)),
            Bot(o, p) => Bot(o, f(p)),
            With(o, l, r) => With(o, f(l), f(r)),
            Plus(o, s, p) => Plus(o, s, f(p)),
            Bang(o, p) => Bang(o, f(p)),
            Quest(o, p) => Quest(o, f(p)),
            Copy(o, p) => Copy(o, f(p)),
            Weaken(o, p) => Weaken(o, f(p)),
            Mix(l, r) => Mix(f(l), f(r)),
            leaf @ (Ax(..) | One(_) | Top(_)) => leaf,
        }
    }
}

// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The derivation view: a proof term unfolded into the tree of explicit
//! sequents and rule names of the standard one-sided sequent calculus, with
//! the term's dyadic bookkeeping expanded into dereliction, contraction and
//! weakening. Renderers and exporters read this view, never the term.
//!
//! The translation keeps the derivation small: the standard sequent of a
//! subproof is `⊢ ?Θ, Γ` for the unrestricted zone `Θ` it needs (see
//! [`check`](crate::proofs::check)), not for the zone in force. So a copy is a
//! dereliction, and a contraction as well when the copied formula is used
//! again above; the `?` step is nothing when its formula is used above and
//! a weakening otherwise; a `⊗` or Mix contracts the `?` formulas both
//! premises use, below the rule; and a `&` weakens, above each premise, the
//! `?` formulas only the other premise uses. A `⊤` absorbs whatever context
//! reaches it, so nothing is weakened above it.
//!
//! An intuitionistic derivation is the same tree over the same term, read
//! two-sided: every sequent has one goal, the inferences carry the
//! intuitionistic rule names (`⊸L` for a `⊗` on a hypothesis, `⊗L` for a
//! `⅋` on one, `!L` for a dereliction, and so on), and what a `⊤` absorbs
//! is distributed so that each premise keeps exactly one goal.

use super::check::{self, CheckError, Derived};
use super::multiset::Multiset;
use super::{Node, NodeId, Proof, Side};
use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId, Position, Reading};
use crate::sequents::Kind;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The index of an inference in a derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InfId(u32);

impl InfId {
    /// Wraps a raw index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the inferences.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A rule of the standard sequent calculus, as a derivation names it: the
/// one-sided rules of classical linear logic, and the two-sided rules of
/// intuitionistic linear logic that an intuitionistic derivation shows
/// instead, each the classical rule on the hypothesis or the goal it acts
/// on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    /// `ax`
    Ax,
    /// `⊗`
    Tensor,
    /// `⅋`
    Par,
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `&`
    With,
    /// `⊕₁`, the left introduction of `⊕`.
    PlusLeft,
    /// `⊕₂`, the right introduction of `⊕`.
    PlusRight,
    /// `⊤`
    Top,
    /// `!`, promotion.
    Promotion,
    /// `?d`, dereliction.
    Dereliction,
    /// `?c`, contraction.
    Contraction,
    /// `?w`, weakening of a `?` formula.
    Weakening,
    /// `mix`
    Mix,
    /// `wk`, weakening of a formula that is not a `?`, in affine mode.
    AffineWeakening,
    /// `⊸L`: a `⊗` on a hypothesis `A ⊸ B`.
    ImpLeft,
    /// `⊸R`: a `⅋` on the goal `A ⊸ B`.
    ImpRight,
    /// `⊗L`: a `⅋` on a hypothesis `A ⊗ B`.
    TensorLeft,
    /// `⊗R`: a `⊗` on the goal.
    TensorRight,
    /// `&L₁`: a `⊕₁` on a hypothesis `A & B`.
    WithLeft1,
    /// `&L₂`: a `⊕₂` on a hypothesis `A & B`.
    WithLeft2,
    /// `&R`: a `&` on the goal.
    WithRight,
    /// `⊕L`: a `&` on a hypothesis `A ⊕ B`.
    PlusLeftRule,
    /// `⊕R₁`: a `⊕₁` on the goal.
    PlusRight1,
    /// `⊕R₂`: a `⊕₂` on the goal.
    PlusRight2,
    /// `1L`: a `⊥` on a hypothesis `1`.
    OneLeft,
    /// `1R`: the goal `1`.
    OneRight,
    /// `0L`: a `⊤` on a hypothesis `0`.
    ZeroLeft,
    /// `⊤R`: the goal `⊤`.
    TopRight,
    /// `!L`: a dereliction of a hypothesis `!A`.
    BangLeft,
    /// `!R`: a promotion of the goal `!A`.
    BangRight,
    /// `!c`: a contraction of a hypothesis `!A`.
    BangContraction,
    /// `!w`: a weakening of a hypothesis `!A`.
    BangWeakening,
}

impl Rule {
    /// Returns the rule's usual spelling.
    pub const fn name(self) -> &'static str {
        use Rule::*;
        match self {
            Ax => "ax",
            Tensor => "⊗",
            Par => "⅋",
            One => "1",
            Bot => "⊥",
            With => "&",
            PlusLeft => "⊕₁",
            PlusRight => "⊕₂",
            Top => "⊤",
            Promotion => "!",
            Dereliction => "?d",
            Contraction => "?c",
            Weakening => "?w",
            Mix => "mix",
            AffineWeakening => "wk",
            ImpLeft => "⊸L",
            ImpRight => "⊸R",
            TensorLeft => "⊗L",
            TensorRight => "⊗R",
            WithLeft1 => "&L₁",
            WithLeft2 => "&L₂",
            WithRight => "&R",
            PlusLeftRule => "⊕L",
            PlusRight1 => "⊕R₁",
            PlusRight2 => "⊕R₂",
            OneLeft => "1L",
            OneRight => "1R",
            ZeroLeft => "0L",
            TopRight => "⊤R",
            BangLeft => "!L",
            BangRight => "!R",
            BangContraction => "!c",
            BangWeakening => "!w",
        }
    }

    /// Returns the intuitionistic name of a classical rule applied to a
    /// formula in `position`: `⊗` on a hypothesis is `⊸L`, on the goal
    /// `⊗R`, and so on. The axiom, Mix and affine weakening keep their
    /// names.
    pub const fn intuitionistic(self, position: Position) -> Self {
        use Position::{Input, Output};
        use Rule::*;
        match (self, position) {
            (Tensor, Input) => ImpLeft,
            (Tensor, Output) => TensorRight,
            (Par, Input) => TensorLeft,
            (Par, Output) => ImpRight,
            (With, Input) => PlusLeftRule,
            (With, Output) => WithRight,
            (PlusLeft, Input) => WithLeft1,
            (PlusLeft, Output) => PlusRight1,
            (PlusRight, Input) => WithLeft2,
            (PlusRight, Output) => PlusRight2,
            (Bot, _) => OneLeft,
            (One, _) => OneRight,
            (Top, Input) => ZeroLeft,
            (Top, Output) => TopRight,
            (Dereliction, _) => BangLeft,
            (Promotion, _) => BangRight,
            (Contraction, _) => BangContraction,
            (Weakening, _) => BangWeakening,
            (rule, _) => rule,
        }
    }
}

impl Display for Rule {
    /// Writes the rule's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// One inference of a derivation: the sequent it concludes, the rule, and
/// the inferences of its premises.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inference {
    /// The sequent concluded, as occurrence ids in ascending order, an
    /// occurrence repeated as often as the sequent holds it.
    pub sequent: Vec<OccId>,
    /// The rule applied.
    pub rule: Rule,
    /// The position in `sequent` of the formula the rule introduces or
    /// removes: `None` for an axiom, whose sequent is its two literals, and
    /// for Mix.
    pub principal: Option<usize>,
    /// The premises, in the rule's order.
    pub premises: Vec<InfId>,
}

/// A derivation in the standard sequent calculus: the tree of inferences a
/// proof term stands for, over the proof's forest, one-sided for classical
/// linear logic or two-sided for intuitionistic linear logic. Premises
/// precede their conclusion and the root is the last inference.
///
/// [`Display`] draws the tree, see [`Proof`] for an example.
#[derive(Clone, Debug)]
pub struct Derivation<'a> {
    /// The forest the sequents' occurrences index.
    forest: &'a Forest,
    /// The intuitionistic reading, for a two-sided derivation.
    reading: Option<Reading<'a>>,
    /// The inferences, premises before conclusions, the root last.
    inferences: Vec<Inference>,
}

impl<'a> Derivation<'a> {
    /// Unfolds a proof into the one-sided derivation of classical linear
    /// logic. The proof must be correct, and the unfolding fails as
    /// [`check`](Proof::check) would if it is not. Mode is not a question
    /// here: a derivation shows every rule the proof uses.
    pub fn new(proof: &'a Proof) -> Result<Self, CheckError> {
        Self::build(proof, Mode::CLASSICAL.affine().with_mix(), None)
    }

    /// Unfolds a proof into the two-sided derivation of intuitionistic
    /// linear logic, `Γ ⊢ A` at every inference with the intuitionistic
    /// rule names. The proof must pass the checker in intuitionistic mode
    /// (affine or not), and the unfolding fails as it would otherwise.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "parse", doc = "```")]
    #[cfg_attr(not(feature = "parse"), doc = "```ignore")]
    /// use linlog::{Mode, Options, Sequent, Verdict, prove};
    ///
    /// let sequent: Sequent = "A, A -o B |- B".parse()?;
    /// let outcome = prove(&sequent, Mode::INTUITIONISTIC, &Options::default())?;
    /// let Verdict::Proved(proof) = &outcome.verdict else {
    ///     panic!("provable");
    /// };
    /// assert_eq!(
    ///     proof.two_sided_derivation()?.to_string(),
    ///     "───── ax   ───── ax\n\
    ///      A ⊢ A      B ⊢ B\n\
    ///      ──────────────── ⊸L\n\
    ///     \x20 A, A ⊸ B ⊢ B"
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn two_sided(proof: &'a Proof) -> Result<Self, CheckError> {
        let reading = match Reading::new(proof.forest()) {
            Ok(reading) => reading,
            Err(e) => {
                return Err(CheckError {
                    node: proof.root(),
                    rule: proof.node(proof.root()),
                    premises: vec![],
                    problem: check::Problem::Shape(e),
                });
            }
        };
        Self::build(proof, Mode::INTUITIONISTIC.affine(), Some(reading))
    }

    /// Checks the proof in `mode` and unfolds it, two-sided when a reading
    /// is given.
    fn build(
        proof: &'a Proof,
        mode: Mode,
        reading: Option<Reading<'a>>,
    ) -> Result<Self, CheckError> {
        let derived = check::derive(proof, mode, reading.as_ref())?;
        check::conclude(proof, mode, &derived)?;
        let inferences = {
            let mut build = Build {
                proof,
                derived: &derived,
                reading: reading.as_ref(),
                inferences: Vec::with_capacity(derived.len()),
            };
            let roots = Multiset::of(proof.forest().roots().iter().copied());
            build.build(proof.root(), roots);
            build.inferences
        };
        Ok(Self {
            forest: proof.forest(),
            reading,
            inferences,
        })
    }

    /// Returns the forest the sequents' occurrences index.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the intuitionistic reading of a two-sided derivation, or
    /// `None` for a one-sided one.
    pub fn reading(&self) -> Option<&Reading<'a>> {
        self.reading.as_ref()
    }

    /// Returns every inference, premises before conclusions, the root last.
    pub fn inferences(&self) -> &[Inference] {
        &self.inferences
    }

    /// Returns the inference at `id`, which must belong to this derivation.
    pub fn inference(&self, id: InfId) -> &Inference {
        &self.inferences[id.index()]
    }

    /// Returns the root: the inference that concludes the sequent.
    pub fn root(&self) -> InfId {
        InfId::new(self.inferences.len() as u32 - 1)
    }
}

/// The translation in progress.
struct Build<'a> {
    /// The proof being unfolded.
    proof: &'a Proof,
    /// What the checker derived for each node.
    derived: &'a [Derived],
    /// The intuitionistic reading, for a two-sided derivation.
    reading: Option<&'a Reading<'a>>,
    /// The inferences made so far.
    inferences: Vec<Inference>,
}

impl Build<'_> {
    /// The forest.
    fn forest(&self) -> &Forest {
        self.proof.forest()
    }

    /// The `?` formula an unrestricted-zone occurrence stands for.
    fn quest(&self, a: OccId) -> OccId {
        self.forest().parent(a).unwrap()
    }

    /// The standard sequent a subproof derives: `?Θ` and `Γ` as the checker
    /// found them.
    fn standard(&self, id: NodeId) -> Multiset {
        let d = &self.derived[id.index()];
        Multiset::of(
            d.theta
                .iter()
                .map(|a| self.quest(a))
                .chain(d.gamma.as_slice().iter().copied()),
        )
    }

    /// Whether a `⊤` in the subproof absorbs any context.
    fn absorbs(&self, id: NodeId) -> bool {
        self.derived[id.index()].any
    }

    /// Adds an inference and returns its id; in a two-sided derivation the
    /// rule gets its intuitionistic name.
    fn infer(
        &mut self,
        sequent: Multiset,
        rule: Rule,
        principal: Option<OccId>,
        premises: Vec<InfId>,
    ) -> InfId {
        let rule = match (self.reading, principal) {
            (Some(reading), Some(o)) => rule.intuitionistic(reading.position(o)),
            _ => rule,
        };
        let principal = principal.map(|o| sequent.position(o).unwrap());
        self.inferences.push(Inference {
            sequent: sequent.into_vec(),
            rule,
            principal,
            premises,
        });
        InfId::new(self.inferences.len() as u32 - 1)
    }

    /// Unfolds the subproof at `id`, whose conclusion is `actual`: the
    /// standard sequent it derives plus whatever a `⊤` in it absorbs.
    fn build(&mut self, id: NodeId, actual: Multiset) -> InfId {
        use Node::*;
        let f = self.forest();
        let node = self.proof.node(id);
        let (left, right) = (
            |o: OccId| f.left(o).unwrap(),
            |o: OccId| f.right(o).unwrap(),
        );
        // The premise's conclusion when the rule adds or removes formulas.
        let above = |actual: &Multiset, removed: &[OccId], added: &[OccId]| {
            let mut up = actual.clone();
            for &o in removed {
                let present = up.remove(o);
                debug_assert!(present);
            }
            for &o in added {
                up.insert(o);
            }
            up
        };
        match node {
            Ax(..) => self.infer(actual, Rule::Ax, None, vec![]),
            One(o) => self.infer(actual, Rule::One, Some(o), vec![]),
            Top(o) => self.infer(actual, Rule::Top, Some(o), vec![]),
            Bot(o, p) => {
                let up = above(&actual, &[o], &[]);
                let premise = self.build(p, up);
                self.infer(actual, Rule::Bot, Some(o), vec![premise])
            }
            Par(o, p) => {
                let up = above(&actual, &[o], &[left(o), right(o)]);
                let premise = self.build(p, up);
                self.infer(actual, Rule::Par, Some(o), vec![premise])
            }
            Plus(o, side, p) => {
                let (chosen, rule) = match side {
                    Side::Left => (left(o), Rule::PlusLeft),
                    Side::Right => (right(o), Rule::PlusRight),
                };
                let up = above(&actual, &[o], &[chosen]);
                let premise = self.build(p, up);
                self.infer(actual, rule, Some(o), vec![premise])
            }
            Bang(o, p) => {
                let up = above(&actual, &[o], &[left(o)]);
                let premise = self.build(p, up);
                self.infer(actual, Rule::Promotion, Some(o), vec![premise])
            }
            Weaken(o, p) => {
                let rule = if f.kind(o) == Kind::Quest {
                    Rule::Weakening
                } else {
                    Rule::AffineWeakening
                };
                let up = above(&actual, &[o], &[]);
                let premise = self.build(p, up);
                self.infer(actual, rule, Some(o), vec![premise])
            }
            Quest(o, p) => {
                if self.derived[p.index()].theta.contains(left(o)) {
                    // Used above: `?A` in Γ becomes `?A` in Θ, which the
                    // standard sequent does not distinguish.
                    self.build(p, actual)
                } else {
                    let up = above(&actual, &[o], &[]);
                    let premise = self.build(p, up);
                    self.infer(actual, Rule::Weakening, Some(o), vec![premise])
                }
            }
            Copy(a, p) => {
                let q = self.quest(a);
                if self.derived[p.index()].theta.contains(a) {
                    // Used again above: derelict the copy, then contract it
                    // with the `?A` that stays.
                    let up = above(&actual, &[], &[a]);
                    let premise = self.build(p, up);
                    let derelicted = above(&actual, &[], &[q]);
                    let d = self.infer(derelicted, Rule::Dereliction, Some(q), vec![premise]);
                    self.infer(actual, Rule::Contraction, Some(q), vec![d])
                } else {
                    let up = above(&actual, &[q], &[a]);
                    let premise = self.build(p, up);
                    self.infer(actual, Rule::Dereliction, Some(q), vec![premise])
                }
            }
            Tensor(o, l, r) => self.split(id, actual, Some((o, left(o), right(o))), l, r),
            Mix(l, r) => self.split(id, actual, None, l, r),
            With(o, l, r) => {
                let up_l = above(&actual, &[o], &[left(o)]);
                let up_r = above(&actual, &[o], &[right(o)]);
                let pl = self.padded(l, up_l);
                let pr = self.padded(r, up_r);
                self.infer(actual, Rule::With, Some(o), vec![pl, pr])
            }
        }
    }

    /// Unfolds a `⊗` on `o` with subformulas `a` and `b`, or a Mix, at `id`
    /// with premises `l` and `r`: the context is split as the premises
    /// derived it, an absorbing premise taking what the `⊤` absorbs, and
    /// the `?` formulas both premises use are contracted below the rule.
    fn split(
        &mut self,
        id: NodeId,
        actual: Multiset,
        tensor: Option<(OccId, OccId, OccId)>,
        l: NodeId,
        r: NodeId,
    ) -> InfId {
        let extra = actual.difference(&self.standard(id));
        let (mut up_l, mut up_r) = (self.standard(l), self.standard(r));
        if let Some((_, a, b)) = tensor {
            up_l.ensure(a);
            up_r.ensure(b);
        }
        if let Some(reading) = self.reading
            && self.absorbs(l)
            && self.absorbs(r)
        {
            // Two-sided: the goal among the absorbed formulas goes to the
            // premise that has none, the hypotheses to the left one.
            let left_has_goal = reading.outputs(up_l.as_slice().iter().copied()) > 0;
            for &o in extra.as_slice() {
                if reading.position(o) == Position::Output && left_has_goal {
                    up_r.insert(o);
                } else {
                    up_l.insert(o);
                }
            }
        } else if self.absorbs(l) {
            up_l = up_l.sum(&extra);
        } else {
            debug_assert!(extra.is_empty() || self.absorbs(r));
            up_r = up_r.sum(&extra);
        }
        let pl = self.build(l, up_l.clone());
        let pr = self.build(r, up_r.clone());
        let (rule, principal) = match tensor {
            Some((o, a, b)) => {
                up_l.remove(a);
                up_r.remove(b);
                (Rule::Tensor, Some(o))
            }
            None => (Rule::Mix, None),
        };
        let mut sequent = up_l.sum(&up_r);
        if let Some(o) = principal {
            sequent.insert(o);
        }
        let mut inference = self.infer(sequent.clone(), rule, principal, vec![pl, pr]);
        let shared = &self.derived[l.index()].theta & &self.derived[r.index()].theta;
        for a in shared.iter() {
            let q = self.quest(a);
            sequent.remove(q);
            inference = self.infer(sequent.clone(), Rule::Contraction, Some(q), vec![inference]);
        }
        debug_assert_eq!(sequent, actual);
        inference
    }

    /// Unfolds the subproof at `id` under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless a
    /// `⊤` in it absorbs them.
    fn padded(&mut self, id: NodeId, actual: Multiset) -> InfId {
        if self.absorbs(id) {
            return self.build(id, actual);
        }
        let mut sequent = self.standard(id);
        let unused = actual.difference(&sequent);
        let mut inference = self.build(id, sequent.clone());
        for &q in unused.as_slice() {
            sequent.insert(q);
            inference = self.infer(sequent.clone(), Rule::Weakening, Some(q), vec![inference]);
        }
        debug_assert_eq!(sequent, actual);
        inference
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;

    /// Wraps a raw occurrence id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Wraps a raw node id.
    const fn n(id: u32) -> NodeId {
        NodeId::new(id)
    }

    /// Builds a proof of the sequent `input` parses to, with the last node
    /// as the root; the occurrence ids are the preorder numbering.
    fn proof(input: &str, nodes: Vec<Node>) -> Proof {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
    }

    /// Renders the derivation of a proof.
    fn render(input: &str, nodes: Vec<Node>) -> String {
        proof(input, nodes).derivation().unwrap().to_string()
    }

    /// Multiplicative rules and their units draw as a tree with the rule
    /// on the bar and the conclusion centred under it.
    #[test]
    fn mll_with_units() {
        use Node::*;
        // ⊢ (1 ⊗ A) ⅋ ~A, ⊥: 0 ⅋, 1 ⊗, 2 1, 3 A, 4 ~A, 5 ⊥
        assert_eq!(
            render(
                "|- (1 * A) par ~A, bot",
                vec![
                    One(o(2)),
                    Ax(o(3), o(4)),
                    Tensor(o(1), n(0), n(1)),
                    Par(o(0), n(2)),
                    Bot(o(5), n(3)),
                ]
            ),
            [
                "─── 1   ─────── ax",
                "⊢ 1     ⊢ A, ~A",
                "─────────────── ⊗",
                "  ⊢ 1 ⊗ A, ~A",
                " ────────────── ⅋",
                " ⊢ (1 ⊗ A) ⅋ ~A",
                "───────────────── ⊥",
                "⊢ (1 ⊗ A) ⅋ ~A, ⊥",
            ]
            .join("\n")
        );
    }

    /// Additive rules: `&` copies the context, `⊕` names its side, and a `⊤`
    /// shows the context it absorbs, also through a `⊗` split.
    #[test]
    fn mall() {
        use Node::*;
        // ⊢ A & B, ~A ⊕ ~B: 0 &, 1 A, 2 B, 3 ⊕, 4 ~A, 5 ~B
        assert_eq!(
            render(
                "|- A & B, ~A + ~B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(3), Side::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(3), Side::Right, n(2)),
                    With(o(0), n(1), n(3)),
                ]
            ),
            [
                "  ─────── ax        ─────── ax",
                "  ⊢ A, ~A           ⊢ B, ~B",
                "──────────── ⊕₁   ──────────── ⊕₂",
                "⊢ A, ~A ⊕ ~B      ⊢ B, ~A ⊕ ~B",
                "────────────────────────────── &",
                "       ⊢ A & B, ~A ⊕ ~B",
            ]
            .join("\n")
        );
        // ⊢ ⊤ ⊗ A, ~A, B: 0 ⊗, 1 ⊤, 2 A, 3 ~A, 4 B
        assert_eq!(
            render(
                "|- top * A, ~A, B",
                vec![Top(o(1)), Ax(o(2), o(3)), Tensor(o(0), n(0), n(1))]
            ),
            [
                "────── ⊤   ─────── ax",
                "⊢ ⊤, B     ⊢ A, ~A",
                "────────────────── ⊗",
                "  ⊢ ⊤ ⊗ A, ~A, B",
            ]
            .join("\n")
        );
    }

    /// Exponentials: a copy is a dereliction, a second copy of the same
    /// formula adds a contraction below the `⊗` that joins them, an unused
    /// `?` formula is weakened, a `&` premise that does not use one weakens
    /// it above, and promotion keeps the `?` context.
    #[test]
    fn mell() {
        use Node::*;
        // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A
        assert_eq!(
            render(
                "!A |- A * A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Ax(o(1), o(4)),
                    Copy(o(1), n(2)),
                    Tensor(o(2), n(1), n(3)),
                    Quest(o(0), n(4)),
                ]
            ),
            [
                "─────── ax    ─────── ax",
                "⊢ ~A, A       ⊢ ~A, A",
                "──────── ?d   ──────── ?d",
                "⊢ ?~A, A      ⊢ ?~A, A",
                "────────────────────── ⊗",
                "  ⊢ ?~A, ?~A, A ⊗ A",
                "  ───────────────── ?c",
                "    ⊢ ?~A, A ⊗ A",
            ]
            .join("\n")
        );
        // ⊢ ?~A, A & 1: 0 ?, 1 ~A, 2 &, 3 A, 4 1
        assert_eq!(
            render(
                "!A |- A & 1",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    One(o(4)),
                    With(o(2), n(1), n(2)),
                    Quest(o(0), n(3)),
                ]
            ),
            [
                "─────── ax      ─── 1",
                "⊢ ~A, A         ⊢ 1",
                "──────── ?d   ──────── ?w",
                "⊢ ?~A, A      ⊢ ?~A, 1",
                "────────────────────── &",
                "     ⊢ ?~A, A & 1",
            ]
            .join("\n")
        );
        // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B
        assert_eq!(
            render(
                "!A, !(A -o B) |- !B",
                vec![
                    Ax(o(1), o(4)),
                    Copy(o(1), n(0)),
                    Ax(o(5), o(7)),
                    Tensor(o(3), n(1), n(2)),
                    Copy(o(3), n(3)),
                    Bang(o(6), n(4)),
                    Quest(o(2), n(5)),
                    Quest(o(0), n(6)),
                ]
            ),
            [
                "─────── ax",
                "⊢ ~A, A",
                "──────── ?d   ─────── ax",
                "⊢ ?~A, A      ⊢ ~B, B",
                "───────────────────── ⊗",
                "  ⊢ ?~A, A ⊗ ~B, B",
                " ─────────────────── ?d",
                " ⊢ ?~A, ?(A ⊗ ~B), B",
                " ──────────────────── !",
                " ⊢ ?~A, ?(A ⊗ ~B), !B",
            ]
            .join("\n")
        );
    }

    /// Affine weakening and Mix have their own rule names.
    #[test]
    fn affine_and_mix() {
        use Node::*;
        // ⊢ ~A, ~B, A
        assert_eq!(
            render("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(1), n(0))]),
            ["  ─────── ax", "  ⊢ ~A, A", "─────────── wk", "⊢ ~A, ~B, A",].join("\n")
        );
        // ⊢ 1, ?A: weakening a ? formula is ?w.
        assert_eq!(
            render("|- 1, ?A", vec![One(o(0)), Weaken(o(1), n(0))]),
            ["  ─── 1", "  ⊢ 1", "─────── ?w", "⊢ 1, ?A"].join("\n")
        );
        // ⊢ ~A ⅋ ~B, A ⅋ B: 0 ⅋, 1 ~A, 2 ~B, 3 ⅋, 4 A, 5 B
        assert_eq!(
            render(
                "A * B |- A par B",
                vec![
                    Ax(o(1), o(4)),
                    Ax(o(2), o(5)),
                    Mix(n(0), n(1)),
                    Par(o(0), n(2)),
                    Par(o(3), n(3)),
                ]
            ),
            [
                "─────── ax   ─────── ax",
                "⊢ ~A, A      ⊢ ~B, B",
                "──────────────────── mix",
                "   ⊢ ~A, ~B, A, B",
                "   ─────────────── ⅋",
                "   ⊢ ~A ⅋ ~B, A, B",
                "   ──────────────── ⅋",
                "   ⊢ ~A ⅋ ~B, A ⅋ B",
            ]
            .join("\n")
        );
    }

    /// A two-sided derivation shows `Γ ⊢ A` at every inference with the
    /// intuitionistic rule names, gives the `0` premise of a `⊸L` the goal
    /// its `⊤` absorbs, and refuses a proof that is not intuitionistic.
    #[test]
    fn two_sided() {
        use Node::*;
        let render = |input: &str, nodes| {
            proof(input, nodes)
                .two_sided_derivation()
                .unwrap()
                .to_string()
        };
        // A, A ⊸ B ⊢ B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
        assert_eq!(
            render(
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))]
            ),
            [
                "───── ax   ───── ax",
                "A ⊢ A      B ⊢ B",
                "──────────────── ⊸L",
                "  A, A ⊸ B ⊢ B",
            ]
            .join("\n")
        );
        // A & B ⊢ (A ⊕ 1) & B: 0 ⊕, 1 ~A, 2 ~B, 3 &, 4 ⊕, 5 A, 6 1, 7 B
        assert_eq!(
            render(
                "A & B |- (A + 1) & B",
                vec![
                    Ax(o(1), o(5)),
                    Plus(o(4), Side::Left, n(0)),
                    Plus(o(0), Side::Left, n(1)),
                    Ax(o(2), o(7)),
                    Plus(o(0), Side::Right, n(3)),
                    With(o(3), n(2), n(4)),
                ]
            ),
            [
                "    ───── ax",
                "    A ⊢ A",
                "  ───────── ⊕R₁       ───── ax",
                "  A ⊢ A ⊕ 1           B ⊢ B",
                "───────────── &L₁   ───────── &L₂",
                "A & B ⊢ A ⊕ 1       A & B ⊢ B",
                "───────────────────────────── &R",
                "     A & B ⊢ (A ⊕ 1) & B",
            ]
            .join("\n")
        );
        // A ⊸ 0, A ⊢ B ⊗ ⊤: 0 ⊗, 1 A, 2 ⊤, 3 ~A, 4 ⊗, 5 B, 6 ⊤: the goal
        // `B ⊗ ⊤` is split first, then the `0` absorbs `B`.
        assert_eq!(
            render(
                "A -o 0, A |- B * top",
                vec![
                    Ax(o(1), o(3)),
                    Top(o(2)),
                    Tensor(o(0), n(0), n(1)),
                    Top(o(6)),
                    Tensor(o(4), n(2), n(3)),
                ]
            ),
            [
                "───── ax   ───── 0L",
                "A ⊢ A      0 ⊢ B",
                "──────────────── ⊸L   ─── ⊤R",
                "  A ⊸ 0, A ⊢ B        ⊢ ⊤",
                "  ─────────────────────── ⊗R",
                "     A ⊸ 0, A ⊢ B ⊗ ⊤",
            ]
            .join("\n")
        );
        // !A, !(A ⊸ B) ⊢ !B & 1: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 &, 7 !,
        // 8 B, 9 1
        assert_eq!(
            render(
                "!A, !(A -o B) |- !B & 1",
                vec![
                    Ax(o(1), o(4)),
                    Copy(o(1), n(0)),
                    Ax(o(5), o(8)),
                    Tensor(o(3), n(1), n(2)),
                    Copy(o(3), n(3)),
                    Bang(o(7), n(4)),
                    One(o(9)),
                    With(o(6), n(5), n(6)),
                    Quest(o(2), n(7)),
                    Quest(o(0), n(8)),
                ]
            ),
            [
                "───── ax",
                "A ⊢ A",
                "────── !L   ───── ax",
                "!A ⊢ A      B ⊢ B",
                "───────────────── ⊸L          ─── 1R",
                "  !A, A ⊸ B ⊢ B               ⊢ 1",
                " ──────────────── !L         ────── !w",
                " !A, !(A ⊸ B) ⊢ B            !A ⊢ 1",
                " ───────────────── !R   ──────────────── !w",
                " !A, !(A ⊸ B) ⊢ !B      !A, !(A ⊸ B) ⊢ 1",
                " ─────────────────────────────────────── &R",
                "          !A, !(A ⊸ B) ⊢ !B & 1",
            ]
            .join("\n")
        );
        // Affine weakening keeps its name. A classical proof that is not
        // intuitionistic has no two-sided derivation, and a classical
        // sequent none at all.
        let p = proof("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(1), n(0))]);
        assert_eq!(
            p.two_sided_derivation().unwrap().to_string(),
            [" ───── ax", " A ⊢ A", "──────── wk", "A, B ⊢ A"].join("\n")
        );
        let p = proof("|- A par B", vec![Ax(o(1), o(2))]);
        assert!(p.two_sided_derivation().is_err());
        let p = proof("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(2), n(0))]);
        assert!(p.two_sided_derivation().is_err());
    }

    /// The inferences carry the sequents as ids with repeats, the rule, the
    /// principal formula's position and the premises, premises first.
    #[test]
    fn inferences() {
        use Node::*;
        // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A
        let p = proof(
            "!A |- A * A",
            vec![
                Ax(o(1), o(3)),
                Copy(o(1), n(0)),
                Ax(o(1), o(4)),
                Copy(o(1), n(2)),
                Tensor(o(2), n(1), n(3)),
                Quest(o(0), n(4)),
            ],
        );
        let d = p.derivation().unwrap();
        let i = InfId::new;
        let inference = |sequent: &[u32], rule, principal, premises: &[InfId]| Inference {
            sequent: sequent.iter().map(|&x| o(x)).collect(),
            rule,
            principal,
            premises: premises.to_vec(),
        };
        assert_eq!(
            d.inferences(),
            [
                inference(&[1, 3], Rule::Ax, None, &[]),
                inference(&[0, 3], Rule::Dereliction, Some(0), &[i(0)]),
                inference(&[1, 4], Rule::Ax, None, &[]),
                inference(&[0, 4], Rule::Dereliction, Some(0), &[i(2)]),
                inference(&[0, 0, 2], Rule::Tensor, Some(2), &[i(1), i(3)]),
                inference(&[0, 2], Rule::Contraction, Some(0), &[i(4)]),
            ]
        );
        assert_eq!(d.root(), i(5));
        assert_eq!(d.inference(i(5)).rule.to_string(), "?c");
        // An invalid proof has no derivation.
        let p = proof("!A |- A * A", vec![Ax(o(1), o(3)), Copy(o(1), n(0))]);
        assert!(p.derivation().is_err());
    }
}

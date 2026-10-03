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

use super::check::{self, CheckError, Facts, Observer, State};
use super::multiset::Multiset;
use super::size::{self, Size};
use super::{Node, NodeId, Proof, Side};
use crate::fragment::Mode;
use crate::hash::HashMap;
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
    /// An open goal of a proof in progress: a leaf without a rule, which
    /// only the derivation of an interactive state contains.
    Open,
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
            Open => "open",
        }
    }

    /// Returns the classical rule an intuitionistic rule name stands for on
    /// the one-sided sequent (`⊸L` and `⊗R` are `⊗`, `!L` is `?d`, and so
    /// on), and every other rule unchanged.
    pub const fn classical(self) -> Self {
        use Rule::*;
        match self {
            ImpLeft | TensorRight => Tensor,
            TensorLeft | ImpRight => Par,
            PlusLeftRule | WithRight => With,
            WithLeft1 | PlusRight1 => PlusLeft,
            WithLeft2 | PlusRight2 => PlusRight,
            OneLeft => Bot,
            OneRight => One,
            ZeroLeft | TopRight => Top,
            BangLeft => Dereliction,
            BangRight => Promotion,
            BangContraction => Contraction,
            BangWeakening => Weakening,
            rule => rule,
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

/// The text is not the name of a rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownRule(pub String);

impl Display for UnknownRule {
    /// Writes which text was not a rule name.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?} is not the name of a rule", self.0)
    }
}

impl std::error::Error for UnknownRule {}

impl std::str::FromStr for Rule {
    type Err = UnknownRule;

    /// Parses a rule from its [`name`](Self::name) or an ASCII spelling of
    /// it: `*` for `⊗`, `par` or `|` for `⅋`, `+1` and `+2` for `⊕₁` and
    /// `⊕₂`, `bot` and `top` for `⊥` and `⊤`, `-oL` for `⊸L`, `*L` for
    /// `⊗L`, `&L1` for `&L₁`, `+L` for `⊕L`, `+R1` for `⊕R₁`, `topR` for
    /// `⊤R`, and so on.
    fn from_str(text: &str) -> Result<Self, UnknownRule> {
        use Rule::*;
        Ok(match text {
            "ax" => Ax,
            "⊗" | "*" => Tensor,
            "⅋" | "par" | "|" => Par,
            "1" => One,
            "⊥" | "bot" => Bot,
            "&" => With,
            "⊕₁" | "+1" => PlusLeft,
            "⊕₂" | "+2" => PlusRight,
            "⊤" | "top" => Top,
            "!" => Promotion,
            "?d" => Dereliction,
            "?c" => Contraction,
            "?w" => Weakening,
            "mix" => Mix,
            "wk" => AffineWeakening,
            "⊸L" | "-oL" => ImpLeft,
            "⊸R" | "-oR" => ImpRight,
            "⊗L" | "*L" => TensorLeft,
            "⊗R" | "*R" => TensorRight,
            "&L₁" | "&L1" => WithLeft1,
            "&L₂" | "&L2" => WithLeft2,
            "&R" => WithRight,
            "⊕L" | "+L" => PlusLeftRule,
            "⊕R₁" | "+R1" => PlusRight1,
            "⊕R₂" | "+R2" => PlusRight2,
            "1L" => OneLeft,
            "1R" => OneRight,
            "0L" => ZeroLeft,
            "⊤R" | "topR" => TopRight,
            "!L" => BangLeft,
            "!R" => BangRight,
            "!c" => BangContraction,
            "!w" => BangWeakening,
            "open" => Open,
            _ => return Err(UnknownRule(text.to_owned())),
        })
    }
}

/// How a derivation is shown: the one value that every path which builds
/// a derivation takes, whatever it then draws or writes. It holds the
/// bound that keeps a call from building what the machine cannot hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
pub struct ViewOptions {
    /// The most bytes a derivation may be estimated to take
    /// ([`Size::bytes`]) and still be built, or `None` for no bound. A
    /// proof stores a shared subproof once and its derivation repeats it,
    /// and every inference carries its whole sequent, so a derivation can
    /// be larger than its proof by any factor.
    pub limit: Option<u64>,
}

impl ViewOptions {
    /// The bound of the default options, 64 MiB: about what an editor
    /// still opens and a typesetter still takes.
    pub const DEFAULT_LIMIT: u64 = 64 << 20;

    /// The options that build a derivation of any size.
    pub const UNBOUNDED: Self = Self { limit: None };

    /// Returns the options with the bound set, or lifted with `None`.
    pub const fn limit(self, limit: Option<u64>) -> Self {
        Self { limit }
    }
}

impl Default for ViewOptions {
    /// A bound of [`DEFAULT_LIMIT`](Self::DEFAULT_LIMIT).
    fn default() -> Self {
        Self {
            limit: Some(Self::DEFAULT_LIMIT),
        }
    }
}

/// Why a proof has no derivation to show.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewError {
    /// The proof does not pass the checker.
    Invalid(CheckError),
    /// The derivation is estimated to take more than the options allow;
    /// nothing was built. The proof is not in doubt: it passed the checker
    /// on the way to its size.
    TooLarge {
        /// The derivation's size.
        size: Size,
        /// The bound in force, in bytes.
        limit: u64,
    },
    /// The derivation has more inferences than a derivation holds
    /// ([`Derivation::MOST`]), whatever the options allow; nothing was
    /// built.
    TooMany {
        /// The derivation's size.
        size: Size,
    },
    /// The caller's stop condition fired while the derivation was built.
    Stopped,
}

impl From<CheckError> for ViewError {
    /// Wraps the checker's complaint.
    fn from(error: CheckError) -> Self {
        Self::Invalid(error)
    }
}

impl Display for ViewError {
    /// Writes the reason, with the size of a derivation left out.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Invalid(error) => write!(f, "{error}"),
            Self::TooLarge { size, limit } => write!(
                f,
                "the derivation is not built: its {} inferences with {} characters of sequents \
                 are estimated at {} bytes, over the bound of {limit}",
                size.inferences,
                size.characters,
                size.bytes()
            ),
            Self::TooMany { size } => write!(
                f,
                "the derivation is not built: it has {} inferences, and a derivation holds {} \
                 at most",
                size.inferences,
                Derivation::MOST
            ),
            Self::Stopped => f.write_str("the derivation is not built: stopped"),
        }
    }
}

impl std::error::Error for ViewError {}

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
    /// logic, unless it is larger than `view` allows or `stop` fires on
    /// the way, once per inference. The proof must be correct, and the
    /// unfolding fails as [`check`](Proof::check) would if it is not. Mode
    /// is not a question here: a derivation shows every rule the proof
    /// uses.
    pub fn new(
        proof: &'a Proof,
        view: &ViewOptions,
        stop: impl FnMut() -> bool,
    ) -> Result<Self, ViewError> {
        Self::build(proof, Self::ONE_SIDED, None, view, stop)
    }

    /// The most inferences a derivation holds: as many as an [`InfId`]
    /// counts.
    pub const MOST: u64 = u32::MAX as u64;

    /// The rules of a one-sided derivation: every rule a proof can use.
    pub(crate) const ONE_SIDED: Mode = Mode::CLASSICAL.affine().with_mix();

    /// The rules of a two-sided derivation, weakening among them so that
    /// it shows where used.
    pub(crate) const TWO_SIDED: Mode = Mode::INTUITIONISTIC.affine();

    /// Unfolds a proof into the two-sided derivation of intuitionistic
    /// linear logic, `Γ ⊢ A` at every inference with the intuitionistic
    /// rule names, unless it is larger than `view` allows or `stop` fires
    /// on the way. The proof must pass the checker in intuitionistic mode
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
    pub fn two_sided(
        proof: &'a Proof,
        view: &ViewOptions,
        stop: impl FnMut() -> bool,
    ) -> Result<Self, ViewError> {
        let reading = check::reading(proof, Self::TWO_SIDED)?;
        Self::build(proof, Self::TWO_SIDED, reading, view, stop)
    }

    /// Checks the proof in `mode` and unfolds it, two-sided when a reading
    /// is given.
    fn build(
        proof: &'a Proof,
        mode: Mode,
        reading: Option<Reading<'a>>,
        view: &ViewOptions,
        stop: impl FnMut() -> bool,
    ) -> Result<Self, ViewError> {
        let roots = proof.forest().roots();
        let inferences = unfold(proof, roots, mode, reading.as_ref(), view, stop)?;
        Ok(Self {
            forest: proof.forest(),
            reading,
            inferences,
        })
    }

    /// Wraps inferences that already have the derivation's shape: premises
    /// before conclusions, the root last, two-sided under a reading.
    pub(crate) fn from_parts(
        forest: &'a Forest,
        reading: Option<Reading<'a>>,
        inferences: Vec<Inference>,
    ) -> Self {
        debug_assert!(!inferences.is_empty());
        Self {
            forest,
            reading,
            inferences,
        }
    }

    /// Unfolds a proof whose root concludes `goal` rather than the roots,
    /// as the search from a goal returns it, into the inferences of its
    /// derivation, premises before conclusions and the root last, two-sided
    /// in intuitionistic mode, unless it is larger than `view` allows or
    /// `stop` fires on the way. Fails as the checker would on a node that
    /// misapplies its rule and on a root that does not conclude the goal.
    pub(crate) fn of_goal(
        proof: &'a Proof,
        goal: &[OccId],
        mode: Mode,
        view: &ViewOptions,
        stop: impl FnMut() -> bool,
    ) -> Result<Vec<Inference>, ViewError> {
        let reading = check::reading(proof, mode)?;
        unfold(proof, goal, mode.affine(), reading.as_ref(), view, stop)
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
        // A derivation has at least one inference and [`MOST`](Self::MOST)
        // at most.
        InfId::new(self.inferences.len() as u32 - 1)
    }
}

/// Unfolds a proof that concludes `goal` into the inferences of its
/// derivation, premises before conclusions and the root last: one pass of
/// the checker for the size, which must be within the bound and of no
/// more inferences than an id counts, a second for what the translation
/// reads, then the translation.
fn unfold(
    proof: &Proof,
    goal: &[OccId],
    mode: Mode,
    reading: Option<&Reading>,
    view: &ViewOptions,
    mut stop: impl FnMut() -> bool,
) -> Result<Vec<Inference>, ViewError> {
    let size = size::measure(proof, goal, mode, reading)?;
    if let Some(limit) = view.limit
        && size.bytes() > limit
    {
        return Err(ViewError::TooLarge { size, limit });
    }
    if size.inferences > Derivation::MOST {
        return Err(ViewError::TooMany { size });
    }
    let mut record = Record::new(proof);
    check::examine(proof, goal, mode, reading, &mut record)?;
    let mut build = Build {
        proof,
        record: &record,
        reading,
        // As many as the size says, which fit a `usize` since they fit a
        // `u32`.
        inferences: Vec::with_capacity(size.inferences as usize),
        tasks: Vec::new(),
        done: Vec::new(),
        stop: &mut stop,
    };
    build.run(proof.root(), Multiset::of(goal.iter().copied()))?;
    Ok(build.inferences)
}

/// What the translation reads off the checker's pass: for every node a
/// flag or two, and the sequent of the nodes where a context is split or
/// padded, which the derivation shows anyway.
struct Record<'a> {
    /// The forest.
    forest: &'a Forest,
    /// Whether a `⊤` in a node's subproof absorbs any context.
    absorbs: Vec<bool>,
    /// For a `?` step, whether a copy above uses its formula; for a copy,
    /// whether another copy above uses the same occurrence.
    used: Vec<bool>,
    /// The nodes whose sequent is kept: every `⊗` and Mix, and the
    /// premises of `⊗`, Mix and `&`.
    kept: Vec<bool>,
    /// The standard sequents of the kept nodes: `?Θ` and `Γ` as the
    /// checker found them.
    standard: HashMap<NodeId, Multiset>,
    /// For a `⊗` or Mix, the unrestricted occurrences both premises need,
    /// ascending, where there are any.
    shared: HashMap<NodeId, Vec<OccId>>,
}

impl<'a> Record<'a> {
    /// The record of a proof the checker has yet to pass over.
    fn new(proof: &'a Proof) -> Self {
        let len = proof.nodes().len();
        let mut kept = vec![false; len];
        for id in proof.ids() {
            let node = proof.node(id);
            if matches!(node, Node::Tensor(..) | Node::Mix(..)) {
                kept[id.index()] = true;
            }
            if matches!(node, Node::Tensor(..) | Node::Mix(..) | Node::With(..)) {
                for p in node.premises() {
                    kept[p.index()] = true;
                }
            }
        }
        Self {
            forest: proof.forest(),
            absorbs: vec![false; len],
            used: vec![false; len],
            kept,
            standard: HashMap::default(),
            shared: HashMap::default(),
        }
    }
}

impl Observer for Record<'_> {
    fn derived(&mut self, id: NodeId, state: &State, facts: &Facts<'_>) {
        self.absorbs[id.index()] = state.absorbs();
        self.used[id.index()] = facts.used;
        if self.kept[id.index()] {
            let quests = state.unrestricted().map(|a| self.forest.parent(a).unwrap());
            let linear = state
                .linear()
                .flat_map(|(o, n)| std::iter::repeat_n(o, n as usize));
            self.standard.insert(id, Multiset::of(quests.chain(linear)));
        }
        if !facts.shared.is_empty() {
            let mut shared = facts.shared.to_vec();
            shared.sort_unstable();
            self.shared.insert(id, shared);
        }
    }
}

/// A step the translation has yet to take. The steps wait on a stack of
/// the translation's own, so a derivation of any height is built on a call
/// stack of any size.
enum Task {
    /// Unfold the subproof at a node under a conclusion: the standard
    /// sequent it derives plus whatever a `⊤` in it absorbs.
    Unfold(NodeId, Multiset),
    /// Unfold the subproof at a node under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless
    /// a `⊤` in it absorbs them.
    Pad(NodeId, Multiset),
    /// Conclude a sequent by a rule from the subtrees finished last, as
    /// many as the rule has premises.
    Infer {
        /// The conclusion.
        sequent: Multiset,
        /// The rule.
        rule: Rule,
        /// The formula it introduces.
        principal: Option<OccId>,
        /// How many premises it has.
        premises: usize,
    },
    /// Weaken `?` formulas one by one below the subtree finished last.
    Weaken {
        /// The conclusion of the subtree.
        sequent: Multiset,
        /// The formulas to add, in ascending order.
        unused: Multiset,
    },
    /// Contract, below the subtree finished last, the `?` formulas that
    /// both premises of the `⊗` or Mix at a node use.
    Contract {
        /// The conclusion of the subtree, which holds each of them twice.
        sequent: Multiset,
        /// The node.
        id: NodeId,
    },
}

/// The translation in progress.
struct Build<'a> {
    /// The proof being unfolded.
    proof: &'a Proof,
    /// What the checker derived for the nodes.
    record: &'a Record<'a>,
    /// The intuitionistic reading, for a two-sided derivation.
    reading: Option<&'a Reading<'a>>,
    /// The inferences made so far.
    inferences: Vec<Inference>,
    /// The steps yet to take, the next one last.
    tasks: Vec<Task>,
    /// The roots of the subtrees that are finished and wait for the
    /// inference below them, the latest last.
    done: Vec<InfId>,
    /// The caller's stop condition, polled once per node.
    stop: &'a mut dyn FnMut() -> bool,
}

impl<'a> Build<'a> {
    /// The forest.
    fn forest(&self) -> &'a Forest {
        self.proof.forest()
    }

    /// The `?` formula an unrestricted-zone occurrence stands for.
    fn quest(&self, a: OccId) -> OccId {
        self.forest().parent(a).unwrap()
    }

    /// The standard sequent a subproof derives: `?Θ` and `Γ` as the checker
    /// found them.
    fn standard(&self, id: NodeId) -> Multiset {
        self.record.standard[&id].clone()
    }

    /// Whether a `⊤` in the subproof absorbs any context.
    fn absorbs(&self, id: NodeId) -> bool {
        self.record.absorbs[id.index()]
    }

    /// The unrestricted occurrences both premises of the `⊗` or Mix at
    /// `id` need, ascending.
    fn shared(&self, id: NodeId) -> &'a [OccId] {
        self.record.shared.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Unfolds the subproof at `root`, whose conclusion is `conclusion`,
    /// into the inferences, unless the caller's condition stops it.
    fn run(&mut self, root: NodeId, conclusion: Multiset) -> Result<(), ViewError> {
        self.tasks.push(Task::Unfold(root, conclusion));
        while let Some(task) = self.tasks.pop() {
            match task {
                Task::Unfold(id, actual) => {
                    if (self.stop)() {
                        return Err(ViewError::Stopped);
                    }
                    self.unfold(id, actual);
                }
                Task::Pad(id, actual) => self.pad(id, actual),
                Task::Infer {
                    sequent,
                    rule,
                    principal,
                    premises,
                } => {
                    let premises = self.done.split_off(self.done.len() - premises);
                    let inference = self.infer(sequent, rule, principal, premises);
                    self.done.push(inference);
                }
                Task::Weaken {
                    mut sequent,
                    unused,
                } => {
                    for &q in unused.as_slice() {
                        sequent.insert(q);
                        self.below(sequent.clone(), Rule::Weakening, q);
                    }
                }
                Task::Contract { mut sequent, id } => {
                    for &a in self.shared(id) {
                        let q = self.quest(a);
                        sequent.remove(q);
                        self.below(sequent.clone(), Rule::Contraction, q);
                    }
                }
            }
        }
        Ok(())
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
        // The size of the derivation was counted before it was begun, and
        // one of more inferences than an id counts was refused.
        let id = u32::try_from(self.inferences.len())
            .ok()
            .filter(|&id| u64::from(id) < Derivation::MOST)
            .expect("a derivation has no more inferences than its size says");
        self.inferences.push(Inference {
            sequent: sequent.into_vec(),
            rule,
            principal,
            premises,
        });
        InfId::new(id)
    }

    /// Adds an inference without premises: a subtree of its own.
    fn leaf(&mut self, sequent: Multiset, rule: Rule, principal: Option<OccId>) {
        let inference = self.infer(sequent, rule, principal, vec![]);
        self.done.push(inference);
    }

    /// Adds an inference on `principal` below the subtree finished last.
    fn below(&mut self, sequent: Multiset, rule: Rule, principal: OccId) {
        let premise = self.done.pop().expect("a subtree is finished");
        let inference = self.infer(sequent, rule, Some(principal), vec![premise]);
        self.done.push(inference);
    }

    /// Plans an inference that concludes `sequent` on `principal` from the
    /// subproof at `p` under the conclusion `up`.
    fn from(&mut self, sequent: Multiset, rule: Rule, principal: OccId, p: NodeId, up: Multiset) {
        self.tasks.push(Task::Infer {
            sequent,
            rule,
            principal: Some(principal),
            premises: 1,
        });
        self.tasks.push(Task::Unfold(p, up));
    }

    /// Unfolds the subproof at `id`, whose conclusion is `actual`: the
    /// standard sequent it derives plus whatever a `⊤` in it absorbs. A
    /// leaf is concluded at once; any other rule leaves its premises and
    /// then its own inference as steps to take.
    fn unfold(&mut self, id: NodeId, actual: Multiset) {
        use Node::*;
        let f = self.forest();
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
        match self.proof.node(id) {
            Ax(..) => self.leaf(actual, Rule::Ax, None),
            One(o) => self.leaf(actual, Rule::One, Some(o)),
            Top(o) => self.leaf(actual, Rule::Top, Some(o)),
            Bot(o, p) => {
                let up = above(&actual, &[o], &[]);
                self.from(actual, Rule::Bot, o, p, up);
            }
            Par(o, p) => {
                let up = above(&actual, &[o], &[left(o), right(o)]);
                self.from(actual, Rule::Par, o, p, up);
            }
            Plus(o, side, p) => {
                let (chosen, rule) = match side {
                    Side::Left => (left(o), Rule::PlusLeft),
                    Side::Right => (right(o), Rule::PlusRight),
                };
                let up = above(&actual, &[o], &[chosen]);
                self.from(actual, rule, o, p, up);
            }
            Bang(o, p) => {
                let up = above(&actual, &[o], &[left(o)]);
                self.from(actual, Rule::Promotion, o, p, up);
            }
            Weaken(o, p) => {
                let rule = if f.kind(o) == Kind::Quest {
                    Rule::Weakening
                } else {
                    Rule::AffineWeakening
                };
                let up = above(&actual, &[o], &[]);
                self.from(actual, rule, o, p, up);
            }
            Quest(o, p) => {
                if self.record.used[id.index()] {
                    // Used above: `?A` in Γ becomes `?A` in Θ, which the
                    // standard sequent does not distinguish.
                    self.tasks.push(Task::Unfold(p, actual));
                } else {
                    let up = above(&actual, &[o], &[]);
                    self.from(actual, Rule::Weakening, o, p, up);
                }
            }
            Copy(a, p) => {
                let q = self.quest(a);
                if self.record.used[id.index()] {
                    // Used again above: derelict the copy, then contract it
                    // with the `?A` that stays.
                    let up = above(&actual, &[], &[a]);
                    let derelicted = above(&actual, &[], &[q]);
                    self.tasks.push(Task::Infer {
                        sequent: actual,
                        rule: Rule::Contraction,
                        principal: Some(q),
                        premises: 1,
                    });
                    self.from(derelicted, Rule::Dereliction, q, p, up);
                } else {
                    let up = above(&actual, &[q], &[a]);
                    self.from(actual, Rule::Dereliction, q, p, up);
                }
            }
            Tensor(o, l, r) => self.split(id, actual, Some((o, left(o), right(o))), l, r),
            Mix(l, r) => self.split(id, actual, None, l, r),
            With(o, l, r) => {
                let up_l = above(&actual, &[o], &[left(o)]);
                let up_r = above(&actual, &[o], &[right(o)]);
                self.tasks.push(Task::Infer {
                    sequent: actual,
                    rule: Rule::With,
                    principal: Some(o),
                    premises: 2,
                });
                self.tasks.push(Task::Pad(r, up_r));
                self.tasks.push(Task::Pad(l, up_l));
            }
        }
    }

    /// Plans a `⊗` on `o` with subformulas `a` and `b`, or a Mix, at `id`
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
    ) {
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
        // The rule's conclusion: both contexts, and the `⊗` for its
        // subformulas.
        let (rule, principal, sequent) = match tensor {
            Some((o, a, b)) => {
                let (mut rest_l, mut rest_r) = (up_l.clone(), up_r.clone());
                rest_l.remove(a);
                rest_r.remove(b);
                let mut sequent = rest_l.sum(&rest_r);
                sequent.insert(o);
                (Rule::Tensor, Some(o), sequent)
            }
            None => (Rule::Mix, None, up_l.sum(&up_r)),
        };
        let shared = self.shared(id);
        debug_assert_eq!(
            {
                let mut contracted = sequent.clone();
                for &a in shared {
                    contracted.remove(self.quest(a));
                }
                contracted
            },
            actual
        );
        if !shared.is_empty() {
            self.tasks.push(Task::Contract {
                sequent: sequent.clone(),
                id,
            });
        }
        self.tasks.push(Task::Infer {
            sequent,
            rule,
            principal,
            premises: 2,
        });
        self.tasks.push(Task::Unfold(r, up_r));
        self.tasks.push(Task::Unfold(l, up_l));
    }

    /// Plans the subproof at `id` under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless
    /// a `⊤` in it absorbs them.
    fn pad(&mut self, id: NodeId, actual: Multiset) {
        if self.absorbs(id) {
            self.tasks.push(Task::Unfold(id, actual));
            return;
        }
        let sequent = self.standard(id);
        debug_assert!(sequent.is_subset(&actual));
        let unused = actual.difference(&sequent);
        if !unused.is_empty() {
            self.tasks.push(Task::Weaken {
                sequent: sequent.clone(),
                unused,
            });
        }
        self.tasks.push(Task::Unfold(id, sequent));
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

    /// A derivation of any height is built on a small stack: 120 000
    /// inferences, one above the other, on a thread with 256 KiB.
    #[test]
    fn any_height_on_a_small_stack() {
        use crate::sequents::{Term, TermId};
        use Node::*;
        const ROUNDS: u32 = 40_000;
        // ⊢ ?⊥, 1: 0 ?, 1 ⊥, 2 1. Every round puts the ⊥ under the ? into
        // the linear zone and copies it out again: a ⊥ rule, a
        // dereliction and, but for the topmost, a contraction.
        let sequent = Sequent {
            terms: vec![Term::Bot, Term::Quest(TermId::new(0)), Term::One],
            roots: vec![TermId::new(1), TermId::new(2)],
            atoms: vec![],
        };
        let mut nodes = vec![One(o(2))];
        for _ in 0..ROUNDS {
            nodes.push(Bot(o(1), n(nodes.len() as u32 - 1)));
            nodes.push(Copy(o(1), n(nodes.len() as u32 - 1)));
        }
        nodes.push(Quest(o(0), n(nodes.len() as u32 - 1)));
        let root = n(nodes.len() as u32 - 1);
        let build = move || {
            let p = Proof::new(Forest::new(&sequent).unwrap(), nodes, root).unwrap();
            let d = p.derivation().unwrap();
            let inferences = d.inferences();
            assert_eq!(inferences.len(), 3 * ROUNDS as usize);
            // One branch: every inference is over the one before it.
            for (i, inference) in inferences.iter().enumerate().skip(1) {
                assert_eq!(inference.premises, [InfId::new(i as u32 - 1)]);
            }
            let size = p.derivation_size(false).unwrap();
            assert_eq!(size.height, u64::from(3 * ROUNDS));
            assert_eq!(inferences[0].rule, Rule::One);
            assert_eq!(d.inference(d.root()).rule, Rule::Contraction);
        };
        std::thread::Builder::new()
            .stack_size(256 << 10)
            .spawn(build)
            .unwrap()
            .join()
            .unwrap();
    }

    /// A proof of `⊢ 1, ⊥ & ⊥, …` with `levels` formulas `⊥ & ⊥`, whose
    /// two premises at every level are one subproof: 3 · `levels` + 1
    /// nodes that unfold into more than 2^`levels` inferences.
    fn tower(levels: u32) -> Proof {
        use crate::sequents::{Term, TermId};
        use Node::*;
        // Occurrences: 0 is 1, then &, ⊥, ⊥ for every level.
        let both = Term::With(TermId::new(1), TermId::new(1));
        let mut roots = vec![TermId::new(0)];
        roots.extend((0..levels).map(|_| TermId::new(2)));
        let sequent = Sequent {
            terms: vec![Term::One, Term::Bot, both],
            roots,
            atoms: vec![],
        };
        let mut nodes = vec![One(o(0))];
        for level in 0..levels {
            let (with, below) = (1 + 3 * level, n(nodes.len() as u32 - 1));
            nodes.push(Bot(o(with + 1), below));
            nodes.push(Bot(o(with + 2), below));
            let last = nodes.len() as u32;
            nodes.push(With(o(with), n(last - 2), n(last - 1)));
        }
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&sequent).unwrap(), nodes, root).unwrap()
    }

    /// A derivation of more inferences than an id counts is not built
    /// even with no bound on its size, and a size that no `u64` holds
    /// saturates.
    #[test]
    fn refuses_more_inferences_than_it_can_index() {
        // More than 2⁷⁰ inferences.
        let p = tower(70);
        let size = p.derivation_size(false).unwrap();
        assert_eq!((size.inferences, size.characters), (u64::MAX, u64::MAX));
        assert_eq!((size.bytes(), size.height), (u64::MAX, 141));
        let too_many = p
            .derivation_with(&ViewOptions::UNBOUNDED, || false)
            .unwrap_err();
        assert_eq!(too_many, ViewError::TooMany { size });
        assert_eq!(
            too_many.to_string(),
            "the derivation is not built: it has 18446744073709551615 inferences, and a \
             derivation holds 4294967295 at most"
        );
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

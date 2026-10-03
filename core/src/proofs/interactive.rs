// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Step-by-step proving: a derivation of the standard sequent calculus
//! built top-down, with open goals as leaves, over the occurrence forest of
//! a sequent. A client names a goal, a formula in it and a rule; the rule
//! is validated against the formula, the mode and, in intuitionistic mode,
//! the one-succedent condition, and the goals it leaves are returned. The
//! search can close any goal, and a finished derivation translates back
//! into a proof term that the checker validates, so this layer is trusted
//! no more than an engine.
//!
//! The inferences are those of the derivation view, with the same sequent
//! representation (occurrence ids in ascending order, with repeats) and
//! the same rule names; an open goal is an inference with [`Rule::Open`]
//! and no premises. The arena is top-down: the root is inference 0 and a
//! step appends the goals it opens at the end, so a premise has a larger
//! index than its conclusion, the reverse of a [`Derivation`], which
//! [`Interactive::derivation`] renumbers.

use super::derivation::{Derivation, InfId, Inference, Rule, ViewOptions};
use super::multiset::Multiset;
use super::{Node, NodeId, Proof, Side};
use crate::Error;
use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId, Position, Reading};
use crate::search::{self, Options, Outcome, Verdict, focus};
use crate::sequents::{Kind, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Why a rule does not apply to a goal as asked. Positions are those in the
/// goal's sequent, as the client named them.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refusal {
    /// No open goal has this id.
    NoGoal(InfId),
    /// The goal has no formula at the position; it has `len` formulas.
    NoFormula {
        /// The position asked for.
        position: usize,
        /// How many formulas the goal has.
        len: usize,
    },
    /// The rule does not act on the formula at the position: its connective
    /// is another, or in intuitionistic mode the rule is for the other
    /// side of `⊢`.
    Rule {
        /// The rule asked for.
        rule: Rule,
        /// The position of the formula.
        position: usize,
    },
    /// The rule is not available in the mode: weakening of a formula that
    /// is not a `?` formula needs affine mode, Mix needs Mix.
    Mode {
        /// The rule asked for.
        rule: Rule,
        /// The mode of the proof.
        mode: Mode,
    },
    /// The rule needs the formula alone in the goal (`1`), or alone with its
    /// dual (`ax`); in affine mode the other formulas can be weakened first.
    NotAlone {
        /// The rule asked for.
        rule: Rule,
        /// The position of the formula.
        position: usize,
    },
    /// The axiom needs the goal to be the literal and its dual, and the
    /// other formula is not the dual.
    NoDual {
        /// The position of the literal.
        position: usize,
    },
    /// Promotion needs every other formula to be a `?` formula, and the one
    /// at the position is not.
    NotQuest {
        /// The position of the offending formula.
        position: usize,
    },
    /// A position of the split is out of range, repeated, or the formula the
    /// rule acts on.
    Split {
        /// The offending position.
        position: usize,
    },
    /// A split was given to a rule that takes none.
    NoSplit {
        /// The rule asked for.
        rule: Rule,
    },
    /// In intuitionistic mode, a premise would have this many formulas on
    /// the right of `⊢` instead of one.
    Succedents(usize),
    /// In intuitionistic mode, the formula on the right of `⊢` cannot be
    /// weakened.
    Output {
        /// The position of the formula.
        position: usize,
    },
}

impl Display for Refusal {
    /// Writes the refusal as a phrase, naming rules and positions.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Refusal::NoGoal(id) => write!(f, "there is no open goal {}", id.get()),
            Refusal::NoFormula { position, len } => {
                write!(
                    f,
                    "the goal has no formula {position}: it has {len} formulas"
                )
            }
            Refusal::Rule { rule, position } => {
                write!(f, "the rule {rule} does not act on formula {position}")
            }
            Refusal::Mode { rule, mode } => {
                write!(f, "the rule {rule} is not available in {mode} mode")
            }
            Refusal::NotAlone { rule, position } => write!(
                f,
                "the rule {rule} needs formula {position} without other formulas in the goal"
            ),
            Refusal::NoDual { position } => write!(
                f,
                "the axiom needs formula {position} together with its dual literal and nothing else"
            ),
            Refusal::NotQuest { position } => write!(
                f,
                "promotion needs every other formula to be a ? formula, and formula {position} is not"
            ),
            Refusal::Split { position } => write!(
                f,
                "position {position} of the split is out of range, repeated, or the formula the rule acts on"
            ),
            Refusal::NoSplit { rule } => write!(f, "the rule {rule} takes no split"),
            Refusal::Succedents(n) => write!(
                f,
                "a premise would have {n} formulas on the right of ⊢ instead of one"
            ),
            Refusal::Output { position } => write!(
                f,
                "formula {position} is the one on the right of ⊢ and cannot be weakened"
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// A proof in progress: the forest of a sequent, the mode, the inferences
/// made so far and the goals still open. It starts from a sequent as one
/// open goal; rules are applied to a formula of a goal by position, the
/// search closes goals, [`undo`](Self::undo) retracts the last step (or
/// the client keeps a clone), and once no goal is open
/// [`proof`](Self::proof) gives the checked term.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Interactive, Mode, Rule, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B: the goal's formulas are 0: ~A, 1: A ⊗ ~B, 2: B.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let mut state = Interactive::new(&sequent, Mode::CLASSICAL)?;
/// let root = state.goals().next().unwrap();
/// assert_eq!(state.rules(root, 1)?, vec![Rule::Tensor]);
/// // ⊗ on formula 1, with formula 0 going to the left premise.
/// let goals = state.apply(root, 1, Rule::Tensor, &[0])?;
/// assert_eq!(goals.len(), 2);
/// state.apply(goals[0], 0, Rule::Ax, &[])?;
/// state.apply(goals[1], 0, Rule::Ax, &[])?;
/// let proof = state.proof()?;
/// assert_eq!(proof.nodes().len(), 3);
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Interactive {
    /// The forest of the sequent being proved.
    forest: Forest,
    /// The mode.
    mode: Mode,
    /// The inferences, top-down: the root first, then the goals each step
    /// opened, in order; an open goal has [`Rule::Open`].
    inferences: Vec<Inference>,
    /// The goals closed so far, in order, for undo.
    history: Vec<InfId>,
}

impl Interactive {
    /// Starts a proof of the sequent in the mode, with the whole sequent as
    /// the one open goal. Fails in intuitionistic mode for a sequent with no
    /// intuitionistic reading or with Mix.
    pub fn new(sequent: &Sequent, mode: Mode) -> Result<Self, Error> {
        let forest = Forest::new(sequent)?;
        Self::from_forest(forest, mode)
    }

    /// Starts a proof over a forest built already.
    fn from_forest(forest: Forest, mode: Mode) -> Result<Self, Error> {
        if mode.intuitionistic {
            if mode.mix {
                return Err(Error::IntuitionisticMix);
            }
            Reading::new(&forest).map_err(Error::NotIntuitionistic)?;
        }
        let root = Inference {
            sequent: forest.roots().to_vec(),
            rule: Rule::Open,
            principal: None,
            premises: vec![],
        };
        Ok(Self {
            forest,
            mode,
            inferences: vec![root],
            history: vec![],
        })
    }

    /// Rebuilds a state from its parts, as deserialization does, checking
    /// that they are consistent: inference 0 concludes the sequent, every
    /// other inference is the premise of exactly one earlier inference,
    /// every closed inference's premises are what its rule on its sequent
    /// yields, and the history names distinct closed inferences whose
    /// premises were, in order, the last inferences added.
    pub(crate) fn from_parts(
        forest: Forest,
        mode: Mode,
        inferences: Vec<Inference>,
        history: Vec<InfId>,
    ) -> Result<Self, Error> {
        let mut state = Self::from_forest(forest, mode)?;
        let Some(root) = inferences.first() else {
            return Err(Error::InconsistentState("there is no inference"));
        };
        if root.sequent != state.forest.roots() {
            return Err(Error::InconsistentState(
                "inference 0 does not conclude the sequent",
            ));
        }
        let n = inferences.len();
        let mut parent = vec![None; n];
        for (i, inference) in inferences.iter().enumerate() {
            for &o in &inference.sequent {
                if o.index() >= state.forest.len() {
                    return Err(Error::OccurrenceIndexOutOfBounds(
                        o.index(),
                        state.forest.len(),
                    ));
                }
            }
            if !inference.sequent.is_sorted() {
                return Err(Error::InconsistentState(
                    "a sequent is not in ascending order",
                ));
            }
            for &p in &inference.premises {
                if p.index() <= i || p.index() >= n || parent[p.index()].is_some() {
                    return Err(Error::InconsistentState(
                        "a premise must be a later inference and the premise of one inference only",
                    ));
                }
                parent[p.index()] = Some(i);
            }
        }
        if parent.iter().skip(1).any(Option::is_none) {
            return Err(Error::InconsistentState(
                "an inference other than the root is nobody's premise",
            ));
        }
        state.inferences = inferences;
        let reading = state.reading();
        for id in 0..n {
            let inference = &state.inferences[id];
            if inference.rule == Rule::Open {
                if inference.principal.is_some() || !inference.premises.is_empty() {
                    return Err(Error::InconsistentState(
                        "an open goal has no principal formula and no premise",
                    ));
                }
                continue;
            }
            state.replay(reading.as_ref(), InfId::new(id as u32))?;
        }
        // Seen from the last step back, the inferences a step added are
        // those of its subtree that exist at that point, and they are the
        // suffix of the arena then.
        let mut len = n;
        let mut seen = vec![false; n];
        for &goal in history.iter().rev() {
            let closed = goal.index() < n
                && state.inferences[goal.index()].rule != Rule::Open
                && !std::mem::replace(&mut seen[goal.index()], true);
            if !closed {
                return Err(Error::InconsistentState(
                    "the history names an inference that is not a closed one, or names one twice",
                ));
            }
            let mut added: Vec<usize> = state
                .subtree(goal)
                .into_iter()
                .map(InfId::index)
                .filter(|&id| id < len)
                .collect();
            added.sort_unstable();
            let start = len - added.len();
            if added
                .iter()
                .zip(start..len)
                .any(|(&id, expected)| id != expected)
            {
                return Err(Error::InconsistentState(
                    "the history does not match the order of the inferences",
                ));
            }
            len = start;
        }
        state.history = history;
        Ok(state)
    }

    /// Checks that a closed inference names a principal formula exactly
    /// when its rule has one and that its premises are what the rule
    /// yields on its sequent, recovering the split of a `⊗` or Mix from
    /// the left premise.
    fn replay(&self, reading: Option<&Reading>, id: InfId) -> Result<(), Error> {
        let inference = &self.inferences[id.index()];
        let sequent = &inference.sequent;
        let rule = inference.rule;
        let classical = rule.classical();
        let has_principal = !matches!(classical, Rule::Ax | Rule::Mix);
        if inference.principal.is_some() != has_principal {
            return Err(Error::InconsistentState(
                "a rule other than the axiom and Mix names its principal formula, those two none",
            ));
        }
        let principal = inference.principal;
        if principal.is_some_and(|p| p >= sequent.len()) {
            return Err(Error::InconsistentState(
                "a principal position is out of range",
            ));
        }
        // The formulas the left premise took from the context, as positions
        // in the sequent.
        let mut left = Vec::new();
        if matches!(classical, Rule::Tensor | Rule::Mix) {
            let Some(&first) = inference.premises.first() else {
                return Err(Error::InconsistentState("a split has no premise"));
            };
            let mut context = Multiset::of(self.inferences[first.index()].sequent.iter().copied());
            if let Some(p) = principal {
                context.remove(self.forest.left(sequent[p]).unwrap_or(sequent[p]));
            }
            let mut used = vec![false; sequent.len()];
            if let Some(p) = principal {
                used[p] = true;
            }
            for &o in context.as_slice() {
                let position = sequent
                    .iter()
                    .enumerate()
                    .position(|(i, &x)| x == o && !used[i])
                    .ok_or(Error::InconsistentState(
                        "a premise holds a formula its conclusion lacks",
                    ))?;
                used[position] = true;
                left.push(position);
            }
        }
        let position = match principal {
            Some(p) => p,
            // Mix: the first formula of the left premise stands for the
            // position; the axiom has position 0, its sequent being the
            // two literals.
            None if classical == Rule::Mix => {
                if left.is_empty() {
                    return Err(Error::InconsistentState("a Mix has an empty left premise"));
                }
                left.remove(0)
            }
            None => 0,
        };
        let expected = self.expand(reading, sequent, position, rule, &left)?;
        let same = expected.len() == inference.premises.len()
            && expected
                .iter()
                .zip(&inference.premises)
                .all(|(e, p)| e.as_slice() == self.inferences[p.index()].sequent);
        if same {
            Ok(())
        } else {
            Err(Error::InconsistentState(
                "an inference's premises are not what its rule yields",
            ))
        }
    }

    /// Returns the forest of the sequent being proved.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the sequent being proved.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns the mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the intuitionistic reading of the sequent in intuitionistic
    /// mode, which says on which side of `⊢` a client shows each formula,
    /// and `None` in classical mode.
    pub fn reading(&self) -> Option<Reading<'_>> {
        self.mode
            .intuitionistic
            .then(|| Reading::new(&self.forest).expect("checked when the proof was started"))
    }

    /// Returns every inference, the root first and then in the order the
    /// steps opened them; an open goal is an inference with [`Rule::Open`].
    pub fn inferences(&self) -> &[Inference] {
        &self.inferences
    }

    /// Returns the open goals, in the order they were opened.
    pub fn goals(&self) -> impl Iterator<Item = InfId> + '_ {
        self.inferences
            .iter()
            .enumerate()
            .filter(|(_, inference)| inference.rule == Rule::Open)
            .map(|(i, _)| InfId::new(i as u32))
    }

    /// Returns the sequent of an open goal, or `None` if the id is not one.
    pub fn goal(&self, id: InfId) -> Option<&[OccId]> {
        self.inferences
            .get(id.index())
            .filter(|inference| inference.rule == Rule::Open)
            .map(|inference| inference.sequent.as_slice())
    }

    /// Returns whether no goal is open.
    pub fn is_complete(&self) -> bool {
        self.goals().next().is_none()
    }

    /// Returns how many steps were taken: rules applied and goals closed by
    /// the search, less those undone.
    pub fn steps(&self) -> usize {
        self.history.len()
    }

    /// Returns the goals the steps closed, in order.
    pub(crate) fn history(&self) -> &[InfId] {
        &self.history
    }

    /// Returns the sequent of an open goal, or the refusal.
    fn open(&self, id: InfId) -> Result<&[OccId], Refusal> {
        self.goal(id).ok_or(Refusal::NoGoal(id))
    }

    /// Returns the rules that can act on the formula at `position` of the
    /// open goal in this mode, by the formula's connective and the mode
    /// alone: the axiom on a literal, `⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ !` on their
    /// connectives, `?d ?c ?w` on a `?` formula, `wk` on any other formula
    /// in affine mode, and Mix on any formula when the mode has it; in
    /// intuitionistic mode by their two-sided names. Nothing acts on `0`.
    /// Whether the goal's context lets a rule apply (the axiom's dual, the
    /// context of `1` and `!`, a valid split, one succedent) is what
    /// [`apply`](Self::apply) decides.
    pub fn rules(&self, goal: InfId, position: usize) -> Result<Vec<Rule>, Refusal> {
        let sequent = self.open(goal)?;
        let o = formula_at(sequent, position)?;
        use Rule::*;
        let mut rules = match self.forest.kind(o) {
            Kind::Var | Kind::DualVar => vec![Ax],
            Kind::Tensor => vec![Tensor],
            Kind::Par => vec![Par],
            Kind::One => vec![One],
            Kind::Bot => vec![Bot],
            Kind::With => vec![With],
            Kind::Plus => vec![PlusLeft, PlusRight],
            Kind::Top => vec![Top],
            Kind::Zero => vec![],
            Kind::Bang => vec![Promotion],
            Kind::Quest => vec![Dereliction, Contraction, Weakening],
        };
        if self.mode.affine && self.forest.kind(o) != Kind::Quest {
            rules.push(AffineWeakening);
        }
        if self.mode.mix {
            rules.push(Mix);
        }
        if let Some(reading) = self.reading() {
            let position = reading.position(o);
            for rule in &mut rules {
                *rule = rule.intuitionistic(position);
            }
        }
        Ok(rules)
    }

    /// Applies a rule to the formula at `position` of the open goal and
    /// returns the goals it opens, in the rule's order of premises; a
    /// closed leaf opens none. `left` is the split a `⊗` or Mix needs: the
    /// positions of the other formulas that go to the left premise (the
    /// rest go right; for Mix the formula at `position` goes left too), and
    /// must be empty for every other rule. In intuitionistic mode the rule
    /// may be given by its classical or its two-sided name and is recorded
    /// by the latter. Fails with what the rule needed, and changes nothing
    /// then.
    pub fn apply(
        &mut self,
        goal: InfId,
        position: usize,
        rule: Rule,
        left: &[usize],
    ) -> Result<Vec<InfId>, Refusal> {
        let sequent = self.open(goal)?.to_vec();
        let reading = self.reading();
        let (rule, premises) = {
            let o = formula_at(&sequent, position)?;
            let rule = match &reading {
                Some(reading) => {
                    let named = rule.classical().intuitionistic(reading.position(o));
                    if rule != rule.classical() && rule != named {
                        return Err(Refusal::Rule { rule, position });
                    }
                    named
                }
                None => rule,
            };
            (
                rule,
                self.expand(reading.as_ref(), &sequent, position, rule, left)?,
            )
        };
        let principal = (rule != Rule::Ax && rule != Rule::Mix).then_some(position);
        let base = self.inferences.len() as u32;
        let ids: Vec<InfId> = (0..premises.len() as u32)
            .map(|i| InfId::new(base + i))
            .collect();
        for premise in premises {
            self.inferences.push(Inference {
                sequent: premise.into_vec(),
                rule: Rule::Open,
                principal: None,
                premises: vec![],
            });
        }
        let inference = &mut self.inferences[goal.index()];
        inference.rule = rule;
        inference.principal = principal;
        inference.premises = ids.clone();
        self.history.push(goal);
        Ok(ids)
    }

    /// Computes the premises of a rule on a sequent, in the rule's order, or
    /// refuses: the rule must act on the formula's connective, be available
    /// in the mode, find the context it needs, take a valid split, and in
    /// intuitionistic mode leave one formula on the right of `⊢` in every
    /// premise and never weaken that formula.
    fn expand(
        &self,
        reading: Option<&Reading>,
        sequent: &[OccId],
        position: usize,
        rule: Rule,
        left: &[usize],
    ) -> Result<Vec<Multiset>, Refusal> {
        use Rule::*;
        let f = &self.forest;
        let o = formula_at(sequent, position)?;
        let kind = f.kind(o);
        let classical = rule.classical();
        // The rule acts on the connective, and on the right side of ⊢.
        let acts = match classical {
            Ax => kind.is_literal(),
            Tensor => kind == Kind::Tensor,
            Par => kind == Kind::Par,
            One => kind == Kind::One,
            Bot => kind == Kind::Bot,
            With => kind == Kind::With,
            PlusLeft | PlusRight => kind == Kind::Plus,
            Top => kind == Kind::Top,
            Promotion => kind == Kind::Bang,
            Dereliction | Contraction | Weakening => kind == Kind::Quest,
            AffineWeakening => kind != Kind::Quest,
            Mix => true,
            Open => false,
            _ => unreachable!("classical rules only"),
        };
        let named = match reading {
            Some(reading) => classical.intuitionistic(reading.position(o)),
            None => classical,
        };
        if !acts || rule != named {
            return Err(Refusal::Rule { rule, position });
        }
        match classical {
            AffineWeakening if !self.mode.affine => {
                return Err(Refusal::Mode {
                    rule,
                    mode: self.mode,
                });
            }
            Mix if !self.mode.mix => {
                return Err(Refusal::Mode {
                    rule,
                    mode: self.mode,
                });
            }
            _ => {}
        }
        let splits = matches!(classical, Tensor | Mix);
        if !splits && !left.is_empty() {
            return Err(Refusal::NoSplit { rule });
        }
        // The context without the formula, and the part of it going left.
        let mut rest = Multiset::of(sequent.iter().copied());
        rest.remove(o);
        let mut going_left = Multiset::new();
        if splits {
            let mut used = vec![false; sequent.len()];
            used[position] = true;
            for &p in left {
                if p >= sequent.len() || used[p] {
                    return Err(Refusal::Split { position: p });
                }
                used[p] = true;
                going_left.insert(sequent[p]);
            }
        }
        let child = |side: Side| match side {
            Side::Left => f.left(o).unwrap(),
            Side::Right => f.right(o).unwrap(),
        };
        let with = |added: &[OccId]| {
            let mut premise = rest.clone();
            for &a in added {
                premise.insert(a);
            }
            premise
        };
        let premises = match classical {
            Ax => {
                let [x, y] = sequent else {
                    return Err(Refusal::NotAlone { rule, position });
                };
                let other = if position == 0 { *y } else { *x };
                if f.atom(other) != f.atom(o) || f.sign(other) == f.sign(o) {
                    return Err(Refusal::NoDual { position });
                }
                vec![]
            }
            One => {
                if sequent.len() != 1 {
                    return Err(Refusal::NotAlone { rule, position });
                }
                vec![]
            }
            Top => vec![],
            Tensor => {
                let mut right = rest.difference(&going_left);
                going_left.insert(child(Side::Left));
                right.insert(child(Side::Right));
                vec![going_left, right]
            }
            Mix => {
                let right = rest.difference(&going_left);
                going_left.insert(o);
                vec![going_left, right]
            }
            Par => vec![with(&[child(Side::Left), child(Side::Right)])],
            Bot => vec![with(&[])],
            With => vec![with(&[child(Side::Left)]), with(&[child(Side::Right)])],
            PlusLeft => vec![with(&[child(Side::Left)])],
            PlusRight => vec![with(&[child(Side::Right)])],
            Promotion => {
                if let Some(p) = sequent
                    .iter()
                    .enumerate()
                    .position(|(i, &x)| i != position && f.kind(x) != Kind::Quest)
                {
                    return Err(Refusal::NotQuest { position: p });
                }
                vec![with(&[child(Side::Left)])]
            }
            Dereliction => vec![with(&[child(Side::Left)])],
            Contraction => vec![with(&[o, o])],
            Weakening | AffineWeakening => {
                if let Some(reading) = reading
                    && reading.position(o) == Position::Output
                {
                    return Err(Refusal::Output { position });
                }
                vec![with(&[])]
            }
            _ => unreachable!("handled above"),
        };
        if let Some(reading) = reading {
            for premise in &premises {
                let outputs = reading.outputs(premise.as_slice().iter().copied());
                if outputs != 1 {
                    return Err(Refusal::Succedents(outputs));
                }
            }
        }
        Ok(premises)
    }

    /// Returns whether a split of the open goal for a `⊗` at `position`
    /// (or a Mix, when the formula there is not a `⊗`) passes the count
    /// prunes of the focused engine, a cheap test that says "this split
    /// cannot close" before a client tries it; a split that passes may
    /// still fail. `left` is as for [`apply`](Self::apply).
    pub fn split_passes(
        &self,
        goal: InfId,
        position: usize,
        left: &[usize],
    ) -> Result<bool, Refusal> {
        let sequent = self.open(goal)?;
        let o = formula_at(sequent, position)?;
        let rule = if self.forest.kind(o) == Kind::Tensor {
            Rule::Tensor
        } else {
            Rule::Mix
        };
        let reading = self.reading();
        let rule = match &reading {
            Some(reading) => rule.intuitionistic(reading.position(o)),
            None => rule,
        };
        let premises = self.expand(reading.as_ref(), sequent, position, rule, left)?;
        let [l, r] = premises.as_slice() else {
            unreachable!("a split has two premises");
        };
        let fragment = search::goal_fragment(&self.forest, sequent);
        Ok(focus::split_passes(
            &self.forest,
            fragment,
            self.mode,
            l.as_slice(),
            r.as_slice(),
        ))
    }

    /// Retracts the last step, a rule applied or a goal closed by the
    /// search, and returns the goal it reopened, or `None` when no step is
    /// left.
    pub fn undo(&mut self) -> Option<InfId> {
        let goal = self.history.pop()?;
        // The step's inferences are the suffix of the arena.
        let first = self.subtree(goal).into_iter().min();
        if let Some(first) = first {
            self.inferences.truncate(first.index());
        }
        let inference = &mut self.inferences[goal.index()];
        inference.rule = Rule::Open;
        inference.principal = None;
        inference.premises.clear();
        Some(goal)
    }

    /// Returns the inferences above `id`, excluding it, in no particular
    /// order.
    fn subtree(&self, id: InfId) -> Vec<InfId> {
        let mut ids = self.inferences[id.index()].premises.clone();
        let mut i = 0;
        while i < ids.len() {
            ids.extend(self.inferences[ids[i].index()].premises.iter().copied());
            i += 1;
        }
        ids
    }

    /// Runs the search on an open goal with the options and the stop
    /// condition, as [`prove_goal`](crate::search::prove_goal) does, and
    /// returns its outcome. When the goal is proved, the derivation of the
    /// proof found is grafted onto the goal as one step, which
    /// [`undo`](Self::undo) retracts whole; otherwise nothing changes. The
    /// outcome's proof, if any, is the proof of the goal alone.
    ///
    /// The derivation grafted is within the bound of `view`: a goal whose
    /// proof unfolds into a larger one, or whose unfolding `stop` ends, is
    /// [`Error::View`] and stays open, though the search proved it.
    pub fn close(
        &mut self,
        goal: InfId,
        options: &Options,
        view: &ViewOptions,
        mut stop: impl FnMut() -> bool,
    ) -> Result<Outcome, Error> {
        let sequent = self.open(goal)?.to_vec();
        let outcome = search::prove_goal(&self.forest, &sequent, self.mode, options, &mut stop)?;
        if let Verdict::Proved(proof) = &outcome.verdict {
            let found = Derivation::of_goal(proof, &sequent, self.mode, view, &mut stop)?;
            self.graft(goal, found);
            self.history.push(goal);
        }
        Ok(outcome)
    }

    /// Runs the search on every open goal in turn, as [`close`](Self::close)
    /// does, and returns each goal with its outcome; a goal the search does
    /// not prove stays open, and the run goes on with the next.
    pub fn close_all(
        &mut self,
        options: &Options,
        view: &ViewOptions,
        mut stop: impl FnMut() -> bool,
    ) -> Result<Vec<(InfId, Outcome)>, Error> {
        let goals: Vec<InfId> = self.goals().collect();
        let mut outcomes = Vec::with_capacity(goals.len());
        for goal in goals {
            let outcome = self.close(goal, options, view, &mut stop)?;
            outcomes.push((goal, outcome));
        }
        Ok(outcomes)
    }

    /// Replaces the open goal by the root of a derivation given premises
    /// before conclusions, appending the rest in reverse, so that every
    /// premise keeps a larger index than its conclusion.
    fn graft(&mut self, goal: InfId, mut found: Vec<Inference>) {
        let root = found.len() - 1;
        let base = self.inferences.len();
        let renumber = |id: InfId| {
            if id.index() == root {
                goal
            } else {
                InfId::new((base + root - 1 - id.index()) as u32)
            }
        };
        for inference in &mut found {
            for premise in &mut inference.premises {
                *premise = renumber(*premise);
            }
        }
        let root = found.pop().unwrap();
        self.inferences[goal.index()] = root;
        self.inferences.extend(found.into_iter().rev());
    }

    /// Returns the derivation so far, with the open goals as leaves of
    /// [`Rule::Open`], two-sided in intuitionistic mode; its inferences are
    /// renumbered premises before conclusions, so its ids are not this
    /// state's.
    pub fn derivation(&self) -> Derivation<'_> {
        let mut order = Vec::with_capacity(self.inferences.len());
        let mut new_id = vec![InfId::new(u32::MAX); self.inferences.len()];
        // Postorder without recursion: a frame is an inference and whether
        // its premises were visited.
        let mut stack = vec![(InfId::new(0), false)];
        while let Some((id, visited)) = stack.pop() {
            let inference = &self.inferences[id.index()];
            if visited {
                new_id[id.index()] = InfId::new(order.len() as u32);
                order.push(id);
            } else {
                stack.push((id, true));
                stack.extend(inference.premises.iter().rev().map(|&p| (p, false)));
            }
        }
        let inferences = order
            .iter()
            .map(|&id| {
                let inference = &self.inferences[id.index()];
                Inference {
                    sequent: inference.sequent.clone(),
                    rule: inference.rule,
                    principal: inference.principal,
                    premises: inference
                        .premises
                        .iter()
                        .map(|&p| new_id[p.index()])
                        .collect(),
                }
            })
            .collect();
        Derivation::from_parts(&self.forest, self.reading(), inferences)
    }

    /// Translates the finished derivation into a proof term and checks it,
    /// returning the proof. Fails while goals are open
    /// ([`Error::OpenGoals`]) or if the checker rejects the term
    /// ([`Error::InvalidProof`]), which it never does for a derivation built
    /// through this interface, or gives the check up for the memory it
    /// would hold ([`Error::Unchecked`]).
    pub fn proof(&self) -> Result<Proof, Error> {
        let open = self.goals().count();
        if open > 0 {
            return Err(Error::OpenGoals(open));
        }
        let mut terms = Terms {
            state: self,
            nodes: Vec::with_capacity(self.inferences.len()),
        };
        let mut root = terms.term(InfId::new(0));
        for &r in self.forest.roots() {
            if self.forest.kind(r) == Kind::Quest {
                root = terms.push(Node::Quest(r, root));
            }
        }
        let proof = Proof::new(self.forest.clone(), terms.nodes, root)?;
        proof.check(self.mode)?;
        Ok(proof)
    }
}

/// Returns the formula at a position of a sequent, or the refusal.
fn formula_at(sequent: &[OccId], position: usize) -> Result<OccId, Refusal> {
    sequent.get(position).copied().ok_or(Refusal::NoFormula {
        position,
        len: sequent.len(),
    })
}

/// The translation of a finished derivation into a proof term, bottom-up
/// over the tree: every rule is its node; a dereliction is a copy, a
/// contraction and a `?w` are nothing, since a `?` formula lives in the
/// unrestricted zone from the point where it enters the derivation, which
/// is where its `?` node goes: below the rule that introduces it as a
/// subformula, or below the root for a `?` formula of the sequent.
struct Terms<'a> {
    /// The finished derivation.
    state: &'a Interactive,
    /// The arena being built, premises before conclusions.
    nodes: Vec<Node>,
}

impl Terms<'_> {
    /// Appends a node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Builds the term of the subtree at `root`, without recursion: a
    /// derivation read from a file can be as high as its formulas are
    /// deep. The steps still to take wait on a stack and the terms of the
    /// subtrees done on another, in the order a recursion would take
    /// them, premises left to right and a rule after its premises.
    fn term(&mut self, root: InfId) -> NodeId {
        use Rule::*;
        /// What is left to do.
        enum Step {
            /// Build the term of the subtree at an inference.
            Visit(InfId),
            /// Put the `?` nodes of the `?` formulas among these, which
            /// the rule below introduced, on the term just built.
            Quests([Option<OccId>; 2]),
            /// Put an inference's own node on the terms of its premises.
            Build(InfId),
        }
        let state = self.state;
        let f = &state.forest;
        let mut steps = vec![Step::Visit(root)];
        let mut done: Vec<NodeId> = Vec::new();
        while let Some(step) = steps.pop() {
            let id = match step {
                Step::Visit(id) | Step::Build(id) => id,
                Step::Quests(introduced) => {
                    let mut node = done.pop().expect("the premise's term");
                    for x in introduced.into_iter().flatten() {
                        if f.kind(x) == Kind::Quest {
                            node = self.push(Node::Quest(x, node));
                        }
                    }
                    done.push(node);
                    continue;
                }
            };
            let inference = &state.inferences[id.index()];
            let sequent = &inference.sequent;
            let premises = &inference.premises;
            let o = || {
                sequent[inference
                    .principal
                    .expect("every rule but ax and mix has one")]
            };
            let a = || f.left(o()).expect("a connective with a subformula");
            let b = || f.right(o()).expect("a binary connective");
            let rule = inference.rule.classical();
            if matches!(step, Step::Visit(_)) {
                // What each premise's sequent gains from the rule.
                let introduced: [[Option<OccId>; 2]; 2] = match rule {
                    Ax | One | Top => {
                        let leaf = match rule {
                            Ax => Node::Ax(sequent[0], sequent[1]),
                            One => Node::One(o()),
                            _ => Node::Top(o()),
                        };
                        let leaf = self.push(leaf);
                        done.push(leaf);
                        continue;
                    }
                    // Neither is a node: the subtree's term is the
                    // premise's.
                    Contraction | Weakening => {
                        steps.push(Step::Visit(premises[0]));
                        continue;
                    }
                    Par => [[Some(a()), Some(b())], [None; 2]],
                    Bot | AffineWeakening | Mix => [[None; 2]; 2],
                    PlusLeft | Promotion | Dereliction => [[Some(a()), None], [None; 2]],
                    PlusRight => [[Some(b()), None], [None; 2]],
                    With | Tensor => [[Some(a()), None], [Some(b()), None]],
                    Open => unreachable!("no goal is open"),
                    _ => unreachable!("classical rules only"),
                };
                steps.push(Step::Build(id));
                for (&premise, introduced) in premises.iter().zip(introduced).rev() {
                    steps.push(Step::Quests(introduced));
                    steps.push(Step::Visit(premise));
                }
                continue;
            }
            let mut premise = || done.pop().expect("a premise's term");
            let node = match rule {
                Par => Node::Par(o(), premise()),
                Bot => Node::Bot(o(), premise()),
                PlusLeft => Node::Plus(o(), Side::Left, premise()),
                PlusRight => Node::Plus(o(), Side::Right, premise()),
                Promotion => Node::Bang(o(), premise()),
                Dereliction => Node::Copy(a(), premise()),
                AffineWeakening => Node::Weaken(o(), premise()),
                With | Tensor | Mix => {
                    let (r, l) = (premise(), premise());
                    match rule {
                        With => Node::With(o(), l, r),
                        Tensor => Node::Tensor(o(), l, r),
                        _ => Node::Mix(l, r),
                    }
                }
                _ => unreachable!("a rule with premises"),
            };
            let node = self.push(node);
            done.push(node);
        }
        done.pop().expect("the root's term")
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::search::Reason;

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// Starts a proof of `input` in `mode` and returns it with its goal.
    fn start(input: &str, mode: Mode) -> (Interactive, InfId) {
        let state = Interactive::new(&sequent(input), mode).unwrap();
        let goal = state.goals().next().unwrap();
        (state, goal)
    }

    /// A derivation forty thousand rules high is translated into its
    /// proof term, and the term checked, on a stack that a recursion over
    /// its height would overflow.
    #[test]
    fn a_high_derivation_needs_no_stack() {
        const DEPTH: usize = 20_000;
        let text = format!("|- {}1{}", "bot | (".repeat(DEPTH), ")".repeat(DEPTH));
        let proved = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || {
                let (mut state, mut goal) = start(&text, Mode::CLASSICAL);
                for _ in 0..DEPTH {
                    goal = state.apply(goal, 0, Rule::Par, &[]).unwrap()[0];
                    goal = state.apply(goal, 0, Rule::Bot, &[]).unwrap()[0];
                }
                assert!(state.apply(goal, 0, Rule::One, &[]).unwrap().is_empty());
                state.proof().unwrap().nodes().len()
            })
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(proved, 2 * DEPTH + 1);
    }

    /// Returns the position in the goal of the first formula printed as
    /// `text`.
    fn at(state: &Interactive, goal: InfId, text: &str) -> usize {
        let sequent = state.goal(goal).unwrap();
        sequent
            .iter()
            .position(|&o| state.forest().formula(o).to_string() == text)
            .unwrap_or_else(|| panic!("no {text} in goal {}", goal.get()))
    }

    /// Applies `rule` to the formula printed as `text` and returns the
    /// goals opened.
    fn step(
        state: &mut Interactive,
        goal: InfId,
        text: &str,
        rule: Rule,
        left: &[&str],
    ) -> Vec<InfId> {
        let position = at(state, goal, text);
        let left: Vec<usize> = left.iter().map(|t| at(state, goal, t)).collect();
        state
            .apply(goal, position, rule, &left)
            .unwrap_or_else(|e| panic!("{rule} on {text}: {e}"))
    }

    /// Every rule applied once through the interface, in a proof the
    /// checker accepts, per fragment and mode.
    #[test]
    fn every_rule_once() {
        use Rule::*;
        let classical = Mode::CLASSICAL;
        // ⅋, ⊗ with a split, ax.
        let (mut s, g) = start("|- ~a par ~b, a * b", classical);
        let [g] = step(&mut s, g, "~a ⅋ ~b", Par, &[])[..] else {
            panic!()
        };
        let [l, r] = step(&mut s, g, "a ⊗ b", Tensor, &["~a"])[..] else {
            panic!()
        };
        assert_eq!(s.goals().collect::<Vec<_>>(), [l, r]);
        step(&mut s, l, "a", Ax, &[]);
        assert_eq!(s.rules(r, 0).unwrap(), [Ax]);
        step(&mut s, r, "b", Ax, &[]);
        assert!(s.is_complete());
        let proof = s.proof().unwrap();
        assert_eq!(proof.nodes().len(), 4);
        assert_eq!(
            s.derivation().to_string(),
            proof.derivation().unwrap().to_string()
        );
        // ⊥ and 1.
        let (mut s, g) = start("|- bot, 1", classical);
        let [g] = step(&mut s, g, "⊥", Bot, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "1", One, &[]);
        s.proof().unwrap();
        // &, ⊕₁, ⊕₂ and ⊤.
        let (mut s, g) = start("|- a & b, ~a + ~b", classical);
        let [l, r] = step(&mut s, g, "a & b", With, &[])[..] else {
            panic!()
        };
        let [l] = step(&mut s, l, "~a ⊕ ~b", PlusLeft, &[])[..] else {
            panic!()
        };
        step(&mut s, l, "a", Ax, &[]);
        let [r] = step(&mut s, r, "~a ⊕ ~b", PlusRight, &[])[..] else {
            panic!()
        };
        step(&mut s, r, "~b", Ax, &[]);
        s.proof().unwrap();
        let (mut s, g) = start("|- top, 0", classical);
        assert_eq!(s.rules(g, at(&s, g, "0")).unwrap(), []);
        step(&mut s, g, "⊤", Top, &[]);
        s.proof().unwrap();
        // ?c, ?d, ! and ?w.
        let (mut s, g) = start("!a |- a * a", classical);
        let [g] = step(&mut s, g, "?~a", Contraction, &[])[..] else {
            panic!()
        };
        let [l, r] = step(&mut s, g, "a ⊗ a", Tensor, &["?~a"])[..] else {
            panic!()
        };
        for g in [l, r] {
            let [g] = step(&mut s, g, "?~a", Dereliction, &[])[..] else {
                panic!()
            };
            step(&mut s, g, "~a", Ax, &[]);
        }
        let proof = s.proof().unwrap();
        assert_eq!(
            proof.derivation().unwrap().to_string(),
            "─────── ax    ─────── ax\n\
             ⊢ ~a, a       ⊢ ~a, a\n\
             ──────── ?d   ──────── ?d\n\
             ⊢ ?~a, a      ⊢ ?~a, a\n\
             ────────────────────── ⊗\n\
            \x20 ⊢ ?~a, ?~a, a ⊗ a\n\
            \x20 ───────────────── ?c\n\
            \x20   ⊢ ?~a, a ⊗ a"
        );
        let (mut s, g) = start("!a |- !a", classical);
        let [g] = step(&mut s, g, "!a", Promotion, &[])[..] else {
            panic!()
        };
        let [g] = step(&mut s, g, "?~a", Dereliction, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "~a", Ax, &[]);
        s.proof().unwrap();
        let (mut s, g) = start("|- ?a, 1", classical);
        let [g] = step(&mut s, g, "?a", Weakening, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "1", One, &[]);
        s.proof().unwrap();
        // wk in affine mode, and Mix.
        let (mut s, g) = start("a, b |- a", classical.affine());
        assert_eq!(s.rules(g, at(&s, g, "~b")).unwrap(), [Ax, AffineWeakening]);
        let [g] = step(&mut s, g, "~b", AffineWeakening, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "~a", Ax, &[]);
        s.proof().unwrap();
        let (mut s, g) = start("|- a, ~a, b, ~b", classical.with_mix());
        assert_eq!(s.rules(g, 0).unwrap(), [Ax, Mix]);
        let [l, r] = step(&mut s, g, "a", Mix, &["~a"])[..] else {
            panic!()
        };
        step(&mut s, l, "a", Ax, &[]);
        step(&mut s, r, "b", Ax, &[]);
        assert_eq!(s.steps(), 3);
        s.proof().unwrap();
    }

    /// A rule that does not apply is refused with what it needed, and the
    /// state is unchanged.
    #[test]
    fn rejections() {
        use Rule::*;
        let (mut s, g) = start("|- ~a par ~b, a * b, 1, !c, ?d, top", Mode::CLASSICAL);
        let before = s.clone();
        let p = |s: &Interactive, text: &str| at(s, g, text);
        let cases: Vec<(&str, Rule, Vec<usize>, Refusal)> = vec![
            (
                "~a ⅋ ~b",
                Tensor,
                vec![],
                Refusal::Rule {
                    rule: Tensor,
                    position: p(&s, "~a ⅋ ~b"),
                },
            ),
            (
                "1",
                AffineWeakening,
                vec![],
                Refusal::Mode {
                    rule: AffineWeakening,
                    mode: Mode::CLASSICAL,
                },
            ),
            (
                "1",
                Mix,
                vec![],
                Refusal::Mode {
                    rule: Mix,
                    mode: Mode::CLASSICAL,
                },
            ),
            (
                "1",
                One,
                vec![],
                Refusal::NotAlone {
                    rule: One,
                    position: p(&s, "1"),
                },
            ),
            ("!c", Promotion, vec![], Refusal::NotQuest { position: 0 }),
            ("~a ⅋ ~b", Par, vec![1], Refusal::NoSplit { rule: Par }),
            ("a ⊗ b", Tensor, vec![9], Refusal::Split { position: 9 }),
            ("a ⊗ b", Tensor, vec![0, 0], Refusal::Split { position: 0 }),
            (
                "a ⊗ b",
                Tensor,
                vec![p(&s, "a ⊗ b")],
                Refusal::Split {
                    position: p(&s, "a ⊗ b"),
                },
            ),
            (
                "1",
                ImpLeft,
                vec![],
                Refusal::Rule {
                    rule: ImpLeft,
                    position: p(&s, "1"),
                },
            ),
        ];
        for (text, rule, left, refusal) in cases {
            let position = p(&s, text);
            assert_eq!(
                s.apply(g, position, rule, &left),
                Err(refusal),
                "{rule} on {text}"
            );
        }
        assert_eq!(
            s.apply(g, 99, Par, &[]),
            Err(Refusal::NoFormula {
                position: 99,
                len: 6
            })
        );
        assert_eq!(
            s.apply(InfId::new(7), 0, Par, &[]),
            Err(Refusal::NoGoal(InfId::new(7)))
        );
        assert_eq!(
            s.rules(InfId::new(7), 0),
            Err(Refusal::NoGoal(InfId::new(7)))
        );
        assert_eq!(s.inferences(), before.inferences());
        assert_eq!(s.steps(), 0);
        // The axiom needs the dual and nothing else.
        let (mut s, g) = start("|- a, ~a, b", Mode::CLASSICAL);
        assert_eq!(
            s.apply(g, 0, Ax, &[]),
            Err(Refusal::NotAlone {
                rule: Ax,
                position: 0
            })
        );
        let (mut s, g) = start("|- a, b", Mode::CLASSICAL);
        assert_eq!(s.apply(g, 0, Ax, &[]), Err(Refusal::NoDual { position: 0 }));
        assert_eq!(
            s.apply(g, 0, Ax, &[]).unwrap_err().to_string(),
            "the axiom needs formula 0 together with its dual literal and nothing else"
        );
        // Intuitionistic mode: one goal per premise, the goal never weakened,
        // and the two-sided names.
        let i = Mode::INTUITIONISTIC;
        let (mut s, g) = start("a, a -o b |- b", i);
        let imp = at(&s, g, "a ⊗ ~b");
        assert_eq!(s.rules(g, imp).unwrap(), [ImpLeft]);
        assert_eq!(
            s.apply(g, imp, TensorRight, &[]),
            Err(Refusal::Rule {
                rule: TensorRight,
                position: imp
            })
        );
        // The goal b with the antecedent a: two on the right.
        let b = at(&s, g, "b");
        assert_eq!(s.apply(g, imp, ImpLeft, &[b]), Err(Refusal::Succedents(2)));
        let (mut s, g) = start("a, b |- a", i.affine());
        let goal = at(&s, g, "a");
        assert_eq!(
            s.apply(g, goal, AffineWeakening, &[]),
            Err(Refusal::Output { position: goal })
        );
        assert!(Interactive::new(&sequent("|- a par b"), i).is_err());
        assert!(matches!(
            Interactive::new(&sequent("a |- a"), i.with_mix()),
            Err(Error::IntuitionisticMix)
        ));
    }

    /// In intuitionistic mode the rules carry their two-sided names, the
    /// classical name is accepted, and the derivation is the two-sided one.
    #[test]
    fn intuitionistic() {
        use Rule::*;
        let i = Mode::INTUITIONISTIC;
        let (mut s, g) = start("a, a -o b |- b", i);
        let imp = at(&s, g, "a ⊗ ~b");
        let a = at(&s, g, "~a");
        let [l, r] = s.apply(g, imp, Tensor, &[a]).unwrap()[..] else {
            panic!()
        };
        assert_eq!(s.inferences()[g.index()].rule, ImpLeft);
        assert_eq!(
            s.derivation().to_string(),
            "a ⊢ a   b ⊢ b\n\
             ───────────── ⊸L\n\
             a, a ⊸ b ⊢ b"
        );
        s.apply(l, 0, Ax, &[]).unwrap();
        s.apply(r, 0, Ax, &[]).unwrap();
        let proof = s.proof().unwrap();
        assert_eq!(proof.check(i), Ok(()));
        assert_eq!(
            s.derivation().to_string(),
            proof.two_sided_derivation().unwrap().to_string()
        );
        // !L, !c and !R by their two-sided names.
        let (mut s, g) = start("!a |- !(a * a)", i);
        let [g] = s.apply(g, at(&s, g, "!(a ⊗ a)"), BangRight, &[]).unwrap()[..] else {
            panic!()
        };
        let [g] = s.apply(g, at(&s, g, "?~a"), BangContraction, &[]).unwrap()[..] else {
            panic!()
        };
        assert_eq!(
            s.rules(g, at(&s, g, "?~a")).unwrap(),
            [BangLeft, BangContraction, BangWeakening]
        );
        let [l, r] = s
            .apply(g, at(&s, g, "a ⊗ a"), TensorRight, &[at(&s, g, "?~a")])
            .unwrap()[..]
        else {
            panic!()
        };
        for g in [l, r] {
            let [g] = s.apply(g, at(&s, g, "?~a"), BangLeft, &[]).unwrap()[..] else {
                panic!()
            };
            s.apply(g, 0, Ax, &[]).unwrap();
        }
        assert_eq!(s.proof().unwrap().check(i), Ok(()));
    }

    /// The search closes one goal or every goal, grafting its derivation,
    /// and the result passes the checker; an unprovable goal stays open.
    #[test]
    fn search_closes_goals() {
        use Rule::*;
        let options = Options::default();
        let (mut s, g) = start("|- (a & b) * c, ~a + ~b, ~c", Mode::CLASSICAL);
        let t = at(&s, g, "(a & b) ⊗ c");
        let [l, r] = s.apply(g, t, Tensor, &[at(&s, g, "~a ⊕ ~b")]).unwrap()[..] else {
            panic!()
        };
        let outcome = s
            .close(l, &options, &ViewOptions::default(), || false)
            .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Proved(_)));
        assert_eq!(outcome.engine, crate::search::Engine::Additive);
        assert_eq!(s.goals().collect::<Vec<_>>(), [r]);
        assert_eq!(s.steps(), 2);
        let outcomes = s
            .close_all(&options, &ViewOptions::default(), || false)
            .unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].0, r);
        assert!(s.is_complete());
        let proof = s.proof().unwrap();
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
        assert!(s.derivation().to_string().contains("⊕₂"));

        // An unprovable goal stays open, a stopped search too.
        let (mut s, g) = start("|- a * b, ~a, ~b", Mode::CLASSICAL);
        let [l, r] = s
            .apply(g, at(&s, g, "a ⊗ b"), Tensor, &[at(&s, g, "~b")])
            .unwrap()[..]
        else {
            panic!()
        };
        let outcome = s
            .close(l, &options, &ViewOptions::default(), || false)
            .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unprovable));
        let outcome = s
            .close(r, &options, &ViewOptions::default(), || true)
            .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)));
        assert_eq!(s.goals().collect::<Vec<_>>(), [l, r]);
        assert!(matches!(s.proof(), Err(Error::OpenGoals(2))));
        assert!(matches!(
            s.close(g, &options, &ViewOptions::default(), || false),
            Err(Error::Refused(Refusal::NoGoal(_)))
        ));

        // Two-sided: the grafted derivation carries the two-sided names.
        let (mut s, g) = start("a & b, !(a -o c) |- c", Mode::INTUITIONISTIC);
        let [g] = s.apply(g, at(&s, g, "~a ⊕ ~b"), WithLeft1, &[]).unwrap()[..] else {
            panic!()
        };
        let outcome = s
            .close(g, &options, &ViewOptions::default(), || false)
            .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Proved(_)));
        assert!(s.derivation().to_string().contains("!L"));
        assert_eq!(s.proof().unwrap().check(Mode::INTUITIONISTIC), Ok(()));
    }

    /// The split helper says which splits the counts refuse.
    #[test]
    fn split_helper() {
        let (s, g) = start("|- a * b, ~a, ~b", Mode::CLASSICAL);
        let t = at(&s, g, "a ⊗ b");
        assert!(s.split_passes(g, t, &[at(&s, g, "~a")]).unwrap());
        assert!(!s.split_passes(g, t, &[at(&s, g, "~b")]).unwrap());
        assert!(!s.split_passes(g, t, &[]).unwrap());
        assert_eq!(
            s.split_passes(g, t, &[t]),
            Err(Refusal::Split { position: t })
        );
        let (s, g) = start("|- a, ~a, b, ~b", Mode::CLASSICAL.with_mix());
        assert!(s.split_passes(g, 0, &[at(&s, g, "~a")]).unwrap());
        assert!(!s.split_passes(g, 0, &[at(&s, g, "b")]).unwrap());
    }

    /// Undo retracts a rule application and a grafted search alike,
    /// restoring the goals, and there is nothing to undo at the start.
    #[test]
    fn undo_restores_goals() {
        use Rule::*;
        let (mut s, g) = start("|- ~a par ~b, a * b", Mode::CLASSICAL);
        assert_eq!(s.undo(), None);
        let start_state = s.clone();
        let [g1] = s.apply(g, at(&s, g, "~a ⅋ ~b"), Par, &[]).unwrap()[..] else {
            panic!()
        };
        let after_par = s.clone();
        let [l, _] = s
            .apply(g1, at(&s, g1, "a ⊗ b"), Tensor, &[at(&s, g1, "~a")])
            .unwrap()[..]
        else {
            panic!()
        };
        s.close(l, &Options::default(), &ViewOptions::default(), || false)
            .unwrap();
        assert_eq!(s.undo(), Some(l));
        assert_eq!(s.goals().count(), 2);
        assert_eq!(s.undo(), Some(g1));
        assert_eq!(s.inferences(), after_par.inferences());
        assert_eq!(s.undo(), Some(g));
        assert_eq!(s.inferences(), start_state.inferences());
        assert_eq!(s.undo(), None);
        let outcome = s
            .close(g, &Options::default(), &ViewOptions::default(), || false)
            .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Proved(_)));
        assert!(s.is_complete());
        assert_eq!(s.undo(), Some(g));
        assert_eq!(s.inferences(), start_state.inferences());
    }

    /// The derivation of a proof in progress draws open goals as bare
    /// sequents.
    #[test]
    fn open_goals_render() {
        use Rule::*;
        let (mut s, g) = start("|- ~a par ~b, a * b", Mode::CLASSICAL);
        let [g] = s.apply(g, at(&s, g, "~a ⅋ ~b"), Par, &[]).unwrap()[..] else {
            panic!()
        };
        s.apply(g, at(&s, g, "a ⊗ b"), Tensor, &[at(&s, g, "~a")])
            .unwrap();
        assert_eq!(
            s.derivation().to_string(),
            "⊢ ~a, a   ⊢ ~b, b\n\
             ───────────────── ⊗\n\
            \x20⊢ ~a, ~b, a ⊗ b\n\
            \x20──────────────── ⅋\n\
            \x20⊢ ~a ⅋ ~b, a ⊗ b"
        );
        let d = s.derivation();
        assert_eq!(d.inference(d.root()).rule, Par);
        assert_eq!(d.inference(InfId::new(0)).rule, Open);
    }

    /// Mix on a goal that repeats the formula it is applied at keeps both
    /// copies.
    #[test]
    fn mix_keeps_repeated_formulas() {
        use Rule::*;
        let (mut s, g) = start("|- ?(a par ~a)", Mode::CLASSICAL.with_mix());
        let [g] = step(&mut s, g, "?(a ⅋ ~a)", Contraction, &[])[..] else {
            panic!()
        };
        let [g] = step(&mut s, g, "?(a ⅋ ~a)", Dereliction, &[])[..] else {
            panic!()
        };
        let [g] = step(&mut s, g, "?(a ⅋ ~a)", Dereliction, &[])[..] else {
            panic!()
        };
        let [l, r] = step(&mut s, g, "a ⅋ ~a", Mix, &[])[..] else {
            panic!()
        };
        assert_eq!(s.goal(l), s.goal(r));
        s.close_all(&Options::default(), &ViewOptions::default(), || false)
            .unwrap();
        assert!(s.is_complete());
    }
}

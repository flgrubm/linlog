// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The proof checker: it derives, node by node, the sequent a proof term
//! proves under the rules of the dyadic sequent calculus and accepts the
//! term only if the root derives the proof's sequent. It is the reference
//! for what a proof is, so it shares no code with any search engine and must
//! never: an engine's proofs are validated by something that cannot repeat
//! the engine's mistakes.
//!
//! The pass is bottom-up, one node at a time in arena order, and never
//! recurses. What a subproof proves is a `State`, a sequent kept as two
//! tables of its members; the one rule whose conclusion the premises do not
//! determine, `⊤`, is handled by a flag that says the subproof proves its
//! sequent under any further context. A node's sequent is built from its
//! premises' own: the last node to read a premise takes its sequent and
//! changes it in place, an earlier reader a copy, so what the pass holds at
//! any moment is the sequents some later node still reads.
//!
//! No arithmetic here may wrap, in any build: a term from a file is
//! hostile input, and a counter that wrapped once made a term a proof
//! that was none. So every integer says at its declaration why it stays
//! in range, or saturates into a refusal.
//!
//! In intuitionistic mode the same pass also checks the one-succedent
//! condition against the sequent's intuitionistic reading: every derived
//! linear zone holds at most one formula in output position, exactly one
//! unless a `⊤` above absorbs the rest, a `⊤` never absorbs a second
//! output, weakening never discards the goal, and Mix has no premise with
//! a goal at all. Every intuitionistic rule is a classical rule on the
//! one-sided sequent, so nothing else is intuitionistic about a proof.

use super::{Node, NodeId, Proof, Side};
use crate::fragment::Mode;
use crate::hash::{HashMap, HashSet};
use crate::occurrences::{Forest, OccId, Position, Reading, ShapeError};
use crate::sequents::Kind;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// How many copies of each occurrence a linear zone holds: a multiset
/// whose insertions and removals cost the same whatever its size.
///
/// The counters saturate and never wrap: a zone of more than
/// [`MOST`](Self::MOST) members is refused by the pass before any rule
/// reads it, so every count a rule sees is the true one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Bag {
    /// The copies of every member, never zero. A count saturates at
    /// `u32::MAX`, and then the zone has that many members at least, more
    /// than [`MOST`](Self::MOST): a count that a rule reads is exact.
    counts: HashMap<OccId, u32>,
    /// The number of members, with repeats. The sum of two zones within
    /// [`MOST`](Self::MOST) is below 2³³, so on a 64-bit target it is
    /// exact until the pass refuses it; where a `usize` has 32 bits it
    /// saturates at `u32::MAX`, which is over the bound as well.
    len: usize,
}

impl Bag {
    /// The most members a zone may hold: fewer than a counter saturates
    /// at, so that a saturated counter is always over it. It fits a
    /// `usize` of 32 bits.
    const MOST: usize = (u32::MAX - 1) as usize;

    /// Returns the multiset of the given ids.
    fn of(ids: impl IntoIterator<Item = OccId>) -> Self {
        let mut bag = Self::default();
        for o in ids {
            bag.insert(o);
        }
        bag
    }

    /// Returns whether the multiset has no member.
    fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the members with their numbers of copies, in no particular
    /// order.
    fn counts(&self) -> impl Iterator<Item = (OccId, u32)> + '_ {
        self.counts.iter().map(|(&o, &n)| (o, n))
    }

    /// Returns the members in ascending order, with repeats.
    fn sorted(&self) -> Vec<OccId> {
        let mut ids: Vec<OccId> = self
            .counts()
            .flat_map(|(o, n)| std::iter::repeat_n(o, n as usize))
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Adds one copy of `o`.
    fn insert(&mut self, o: OccId) {
        let n = self.counts.entry(o).or_insert(0);
        *n = n.saturating_add(1);
        self.len = self.len.saturating_add(1);
    }

    /// Removes one copy of `o` and returns whether there was one.
    fn remove(&mut self, o: OccId) -> bool {
        let Some(n) = self.counts.get_mut(&o) else {
            return false;
        };
        // A count in the table is one at least, and the length is no less
        // than any count.
        *n -= 1;
        if *n == 0 {
            self.counts.remove(&o);
        }
        self.len -= 1;
        true
    }

    /// Adds every copy of every member of `other`: the multiset sum, the
    /// smaller table poured into the larger.
    fn add(&mut self, mut other: Self) {
        if other.counts.len() > self.counts.len() {
            std::mem::swap(self, &mut other);
        }
        for (o, n) in other.counts {
            let own = self.counts.entry(o).or_insert(0);
            *own = own.saturating_add(n);
        }
        self.len = self.len.saturating_add(other.len);
    }

    /// Raises every member's copies to those `other` has of it: the
    /// multiset union.
    fn unite(&mut self, mut other: Self) {
        if other.counts.len() > self.counts.len() {
            std::mem::swap(self, &mut other);
        }
        for (o, n) in other.counts {
            let own = self.counts.entry(o).or_insert(0);
            if n > *own {
                self.len = self.len.saturating_add((n - *own) as usize);
                *own = n;
            }
        }
    }

    /// Returns whether no id has more copies here than in `other`.
    fn is_subset(&self, other: &Self) -> bool {
        self.len <= other.len
            && self
                .counts()
                .all(|(o, n)| other.counts.get(&o).is_some_and(|&m| n <= m))
    }
}

/// What a subproof proves, as the checker derives it from the node's
/// premises: the dyadic sequent `⊢ Θ ; Γ`, or with `any` set `⊢ Θ ; Γ, Δ` for
/// every `Δ`, because a `⊤` leaf above absorbs whatever context it is given.
///
/// `Θ` is the least unrestricted zone the subproof needs, the occurrences it
/// copies without moving them into `Θ` itself; since weakening and
/// contraction on `Θ` are implicit, any larger zone does as well. So a proof
/// is correct when its root needs an empty `Θ` and derives the sequent's
/// formulas, and the checker never has to know the actual zone.
///
/// Both zones are tables of their members, so a sequent takes memory for
/// what it holds and none for the width of the forest.
///
/// A sequent that a rule or an observer reads has passed
/// [`Pass::within`], so its linear zone has [`Bag::MOST`] members at
/// most, fewer than 2³² − 1. The sums below are exact for such a sequent;
/// while a rule builds one they saturate, which only a longer zone can
/// make them do, and that zone is refused before anything reads them.
#[derive(Clone, Debug)]
pub(crate) struct State {
    /// The unrestricted zone the subproof needs.
    theta: HashSet<OccId>,
    /// The linear zone.
    gamma: Bag,
    /// The members of the linear zone in output position, under a reading:
    /// no more than the zone has members, and saturating with its length.
    outputs: usize,
    /// The weights of the linear zone's members, added up: fewer than
    /// 2³² − 1 members of a weight below 2³² each, so below 2⁶⁴.
    linear: u64,
    /// The weights of the `?` formulas of the unrestricted zone's members,
    /// added up. It is the sum over the set as it stands at every moment,
    /// and a set of occurrences has fewer than 2³² − 1 members (a forest
    /// has no more occurrences), each of a weight below 2³²: below 2⁶⁴.
    unrestricted: u64,
    /// The weights of the linear zone's members in output position: no
    /// more than `linear`.
    goal: u64,
    /// Whether a `⊤` above absorbs any further linear context.
    any: bool,
}

impl State {
    /// Returns the sequent as ids for an error report.
    fn to_dyadic(&self) -> Dyadic {
        let mut theta: Vec<OccId> = self.theta.iter().copied().collect();
        theta.sort_unstable();
        Dyadic {
            theta,
            gamma: self.gamma.sorted(),
            any: self.any,
        }
    }

    /// Returns the unrestricted occurrences the subproof needs, in no
    /// particular order.
    pub(crate) fn unrestricted(&self) -> impl Iterator<Item = OccId> + '_ {
        self.theta.iter().copied()
    }

    /// Returns the linear zone's members with their numbers of copies, in
    /// no particular order.
    pub(crate) fn linear(&self) -> impl Iterator<Item = (OccId, u32)> + '_ {
        self.gamma.counts()
    }

    /// Returns how many unrestricted occurrences the subproof needs.
    pub(crate) fn needs(&self) -> usize {
        self.theta.len()
    }

    /// Returns whether a `⊤` above absorbs any further linear context.
    pub(crate) fn absorbs(&self) -> bool {
        self.any
    }

    /// Returns the weight of the standard sequent `⊢ ?Θ, Γ`: its formulas'
    /// weights added up, or `u64::MAX` when that is less. Each zone's sum
    /// is below 2⁶⁴ and the two together need not be.
    pub(crate) fn weight(&self) -> u64 {
        self.linear.saturating_add(self.unrestricted)
    }

    /// Returns the weight of the linear zone's members in output position.
    pub(crate) fn goal_weight(&self) -> u64 {
        self.goal
    }
}

/// A dyadic sequent as the checker derived it, by occurrence ids: the
/// unrestricted zone `Θ`, the linear zone `Γ` with repeats, and whether a
/// `⊤` above absorbs any further linear context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dyadic {
    /// The unrestricted zone, ascending.
    pub theta: Vec<OccId>,
    /// The linear zone, ascending, with repeats.
    pub gamma: Vec<OccId>,
    /// Whether the linear zone may hold anything more.
    pub any: bool,
}

impl Display for Dyadic {
    /// Writes `⊢ Θ ; Γ` with the zones as ids, omitting `Θ ;` when it is
    /// empty, and `…` after `Γ` when anything more is allowed.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None)
    }
}

impl Dyadic {
    /// Writes the sequent as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>) -> FmtResult {
        let list = |f: &mut Formatter<'_>, ids: &[OccId]| {
            for (i, &o) in ids.iter().enumerate() {
                f.write_str(if i == 0 { " " } else { ", " })?;
                occurrence(f, forest, o)?;
            }
            Ok(())
        };
        f.write_str("⊢")?;
        if !self.theta.is_empty() {
            list(f, &self.theta)?;
            f.write_str(" ;")?;
        }
        list(f, &self.gamma)?;
        if self.any {
            f.write_str(if self.gamma.is_empty() {
                " …"
            } else {
                ", …"
            })?;
        }
        Ok(())
    }
}

/// Writes an occurrence as its id, or as its formula when a forest is given.
fn occurrence(f: &mut Formatter<'_>, forest: Option<&Forest>, o: OccId) -> FmtResult {
    match forest {
        Some(forest) => write!(f, "{}", forest.formula(o)),
        None => write!(f, "{}", o.get()),
    }
}

/// Why a proof term is not a proof of its sequent: the node at fault, what
/// the checker had derived for its premises, and what the rule required.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckError {
    /// The node at fault; the root when the proof concludes the wrong sequent
    /// or the sequent has no intuitionistic reading.
    pub node: NodeId,
    /// The node's rule instance.
    pub rule: Node,
    /// The sequents derived for the node's premises, in the node's order.
    pub premises: Vec<Dyadic>,
    /// What the rule required and did not get.
    pub problem: Problem,
}

/// What a rule required and did not get.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The mode forbids the rule: weakening of a formula that is not a `?`
    /// outside affine mode, Mix without Mix or in intuitionistic mode.
    Forbidden,
    /// The mode is intuitionistic and the sequent has no intuitionistic
    /// reading, so no proof of it is checked.
    Shape(ShapeError),
    /// The mode is intuitionistic and a sequent of the rule has this many
    /// formulas on the right of `⊢` instead of one: the premise's linear
    /// zone after the rule, or the context a `⊤` would absorb, or a
    /// weakening of the goal.
    Succedents(usize),
    /// The occurrence is not of the kind the rule acts on.
    Kind(OccId),
    /// The two literals of an axiom are not an atom and its negation.
    NotDual,
    /// A premise, by its index in the node, lacks the occurrence the rule
    /// consumes from it.
    Missing {
        /// Which premise.
        premise: usize,
        /// What it lacks.
        occurrence: OccId,
    },
    /// The premise of a promotion has a linear zone besides the promoted
    /// subformula.
    NotEmpty,
    /// The premises of `&` do not share their linear zone.
    Differ,
    /// A copy of an occurrence that is not the subformula of a `?`.
    NotUnderQuest(OccId),
    /// The node's linear zone holds more formulas than the rest of the
    /// proof can consume: every later rule takes two at most, and the root
    /// concludes the sequent.
    Surplus,
    /// The root derives this sequent, which is not the proof's: its linear
    /// zone differs from the sequent's formulas, or its unrestricted zone
    /// holds a copied occurrence that no `?` rule below moved there.
    Conclusion(Dyadic),
}

impl Display for CheckError {
    /// Writes the node, its premises and the problem, as in `node 2 (⊗ on 1
    /// from 0, 1) with premises ⊢ 0, 3 and ⊢ 3, 4: premise 0 lacks
    /// occurrence 2`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None)
    }
}

impl CheckError {
    /// Returns the error for display with formulas instead of occurrence
    /// ids, read from `forest`, the forest of the proof that failed; the
    /// nodes keep their ids. For instance `node 2 (⊗ on A ⊗ ~B from 0, 1)
    /// with premises ⊢ ~A, ~B and ⊢ ~B, B: premise 0 lacks A`.
    pub fn describe<'a>(&'a self, forest: &'a Forest) -> Described<'a> {
        Described {
            error: self,
            forest,
        }
    }

    /// Writes the error as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>) -> FmtResult {
        write!(f, "node {} ({}", self.node.get(), self.rule.name())?;
        for (i, o) in self.rule.occurrences().enumerate() {
            f.write_str(if i == 0 { " on " } else { ", " })?;
            occurrence(f, forest, o)?;
        }
        for (i, p) in self.rule.premises().enumerate() {
            write!(f, "{}{}", if i == 0 { " from " } else { ", " }, p.get())?;
        }
        f.write_str(")")?;
        for (i, p) in self.premises.iter().enumerate() {
            f.write_str(if i == 0 { " with premises " } else { " and " })?;
            p.write(f, forest)?;
        }
        f.write_str(": ")?;
        use Problem::*;
        match &self.problem {
            Forbidden => f.write_str("the mode forbids the rule"),
            Shape(e) => {
                f.write_str("not an intuitionistic sequent: ")?;
                match forest {
                    Some(forest) => write!(f, "{}", e.describe(forest)),
                    None => write!(f, "{e}"),
                }
            }
            Succedents(n) => write!(
                f,
                "a sequent of the rule has {n} formulas on the right of ⊢ instead of one"
            ),
            Kind(o) => {
                if forest.is_none() {
                    f.write_str("occurrence ")?;
                }
                occurrence(f, forest, *o)?;
                f.write_str(" is not what the rule acts on")
            }
            NotDual => f.write_str("the literals are not an atom and its negation"),
            Missing {
                premise,
                occurrence: o,
            } => {
                write!(f, "premise {premise} lacks ")?;
                if forest.is_none() {
                    f.write_str("occurrence ")?;
                }
                occurrence(f, forest, *o)
            }
            NotEmpty => f.write_str("the linear zone is not empty"),
            Differ => f.write_str("the premises differ"),
            NotUnderQuest(o) => {
                if forest.is_none() {
                    f.write_str("occurrence ")?;
                }
                occurrence(f, forest, *o)?;
                f.write_str(" is not under a ?")
            }
            Surplus => f.write_str(
                "the linear zone holds more formulas than the rest of the proof can consume",
            ),
            Conclusion(d) => {
                f.write_str("the proof concludes ")?;
                d.write(f, forest)?;
                f.write_str(", not the sequent")
            }
        }
    }
}

/// A [`CheckError`] displayed with formulas, as [`CheckError::describe`]
/// returns it.
pub struct Described<'a> {
    /// The error.
    error: &'a CheckError,
    /// The forest of the proof that failed.
    forest: &'a Forest,
}

impl Display for Described<'_> {
    /// Writes the error with formulas instead of occurrence ids.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.error.write(f, Some(self.forest))
    }
}

impl std::error::Error for CheckError {}

/// Checks that a proof proves its sequent: every node applies its rule to
/// what its premises derive, the mode allows the rule, and the root derives
/// exactly the sequent's formulas with nothing left in the unrestricted
/// zone; in intuitionistic mode also that the sequent has an intuitionistic
/// reading and every sequent of the proof one formula on the right of `⊢`.
/// Returns the first node that fails, in arena order, with what it needed.
///
/// The memory taken is that of the sequents some later node still reads,
/// which for a proof without shared subproofs is proportional to the proof.
pub fn check(proof: &Proof, mode: Mode) -> Result<(), CheckError> {
    let reading = reading(proof, mode)?;
    examine(
        proof,
        proof.forest().roots(),
        mode,
        reading.as_ref(),
        &mut (),
    )
}

/// Returns the intuitionistic reading of the proof's sequent in
/// intuitionistic mode and none otherwise, or the error of a sequent that
/// has no reading.
pub(crate) fn reading(proof: &Proof, mode: Mode) -> Result<Option<Reading<'_>>, CheckError> {
    if !mode.intuitionistic {
        return Ok(None);
    }
    match Reading::new(proof.forest()) {
        Ok(reading) => Ok(Some(reading)),
        Err(e) => Err(CheckError {
            node: proof.root(),
            rule: proof.node(proof.root()),
            premises: vec![],
            problem: Problem::Shape(e),
        }),
    }
}

/// Checks that a proof concludes `goal`, a multiset of occurrences in any
/// order (the roots, for a proof of the sequent), as [`check`] does, with
/// the one-succedent condition when a reading is given, and shows every
/// node's sequent to `observer` on the way.
pub(crate) fn examine<O: Observer>(
    proof: &Proof,
    goal: &[OccId],
    mode: Mode,
    reading: Option<&Reading>,
    observer: &mut O,
) -> Result<(), CheckError> {
    let mut pass = Pass::new(proof, goal.len(), mode, reading, observer);
    let end = proof.nodes().len();
    let (node, problem) = match pass.run(end) {
        Err(failure) => failure,
        Ok(()) => match pass.conclude(goal) {
            Ok(()) => return Ok(()),
            Err(problem) => (proof.root(), problem),
        },
    };
    drop(pass);
    // The premises' sequents are gone, moved into the node that failed, so
    // the pass runs once more up to it. They are kept then, since the node
    // itself has yet to read them, and the pass does what the first did
    // up to there: it cannot fail.
    let rule = proof.node(node);
    let mut nobody = ();
    let mut pass = Pass::new(proof, goal.len(), mode, reading, &mut nobody);
    let again = pass.run(node.index());
    debug_assert!(again.is_ok());
    let premises = rule
        .premises()
        .filter_map(|p| pass.live[p.index()].as_deref().map(State::to_dyadic))
        .collect();
    Err(CheckError {
        node,
        rule,
        premises,
        problem,
    })
}

/// What a caller of [`examine`] learns about every node of a correct
/// proof, in arena order.
pub(crate) trait Observer {
    /// Returns the weight of an occurrence, which a [`State`] adds up over
    /// its sequent. A `u32`, so that the sum over a zone fits a `u64`.
    fn weight(&self, o: OccId) -> u32 {
        let _ = o;
        0
    }

    /// Takes what node `id` derived and how its rule applied.
    fn derived(&mut self, id: NodeId, state: &State, facts: &Facts<'_>);
}

impl Observer for () {
    fn derived(&mut self, _: NodeId, _: &State, _: &Facts<'_>) {}
}

/// How a node's rule applied to its premises, beyond the sequent derived.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Facts<'a> {
    /// For a `?` step, whether a copy above uses its formula; for a copy,
    /// whether another copy above uses the same occurrence.
    pub(crate) used: bool,
    /// For `⊗` and Mix, the unrestricted occurrences both premises need,
    /// in no particular order.
    pub(crate) shared: &'a [OccId],
    /// Whether a subformula the rule consumes was missing from its premise
    /// and absorbed by a `⊤` above: the left and the right one of `⅋`, `⊗`
    /// and `&`, and the first for a rule that consumes one.
    pub(crate) absent: [bool; 2],
    /// For `⊗`, `&` and Mix, how many unrestricted occurrences each
    /// premise needs: the sizes of two sets of occurrences.
    pub(crate) needs: [usize; 2],
    /// For `⊗` and Mix under a reading, whether the left premise's sequent
    /// holds a formula in output position, its subformula included.
    pub(crate) left_goal: bool,
}

/// The pass over a proof's nodes.
struct Pass<'a, O> {
    /// The proof.
    proof: &'a Proof,
    /// Its forest.
    forest: &'a Forest,
    /// How many formulas the proof is to conclude: the length of a slice,
    /// only ever added to with saturation.
    goal: usize,
    /// The rules in force.
    mode: Mode,
    /// The intuitionistic reading, in intuitionistic mode.
    reading: Option<&'a Reading<'a>>,
    /// How many later nodes still read each node's sequent. A proof of `n`
    /// nodes has `n` references to one node at most: every node after it
    /// has two premises at most, and every one of them but the root is
    /// itself the premise of a later node, or the proof would not hold
    /// it. And `n` fits a `u32`, as [`Proof::new`] sees to.
    readers: Vec<u32>,
    /// The sequents of the nodes that a later node still reads.
    live: Vec<Option<Box<State>>>,
    /// Who is shown every node.
    observer: &'a mut O,
    /// The unrestricted occurrences both premises of the current node need.
    shared: Vec<OccId>,
}

impl<'a, O: Observer> Pass<'a, O> {
    /// The pass before its first node.
    fn new(
        proof: &'a Proof,
        goal: usize,
        mode: Mode,
        reading: Option<&'a Reading<'a>>,
        observer: &'a mut O,
    ) -> Self {
        let mut readers = vec![0; proof.nodes().len()];
        for node in proof.nodes() {
            for p in node.premises() {
                readers[p.index()] += 1;
            }
        }
        let mut live = Vec::new();
        live.resize_with(readers.len(), || None);
        Self {
            proof,
            forest: proof.forest(),
            goal,
            mode,
            reading,
            readers,
            live,
            observer,
            shared: Vec::new(),
        }
    }

    /// Derives the nodes before `end` in arena order, or returns the first
    /// that misapplies its rule, uses one the mode forbids, derives more
    /// than the proof can conclude or breaks the one-succedent condition,
    /// with the problem.
    fn run(&mut self, end: usize) -> Result<(), (NodeId, Problem)> {
        for id in self.proof.ids().take(end) {
            let mut facts = Facts::default();
            let state = self
                .rule(self.proof.node(id), &mut facts)
                .and_then(|state| self.within(id, state))
                .and_then(|state| self.one_succedent(state))
                .map_err(|problem| (id, problem))?;
            facts.shared = &self.shared;
            self.observer.derived(id, &state, &facts);
            // The root has no reader and is read at the end.
            if self.readers[id.index()] > 0 || id == self.proof.root() {
                self.live[id.index()] = Some(state);
            }
        }
        Ok(())
    }

    /// Checks that the root derived `goal` and needs no unrestricted
    /// occurrence.
    fn conclude(&self, goal: &[OccId]) -> Result<(), Problem> {
        let root = self.live[self.proof.root().index()]
            .as_deref()
            .expect("the root's sequent is kept");
        let goal = Bag::of(goal.iter().copied());
        let concludes = if root.any {
            root.gamma.is_subset(&goal)
        } else {
            root.gamma == goal
        };
        if root.theta.is_empty() && concludes {
            Ok(())
        } else {
            Err(Problem::Conclusion(root.to_dyadic()))
        }
    }

    /// What premise `p` derived: the sequent itself for its last reader, a
    /// copy for the others.
    fn premise(&mut self, p: NodeId) -> Box<State> {
        let slot = &mut self.live[p.index()];
        // The count includes this reader, so it is one at least.
        self.readers[p.index()] -= 1;
        if self.readers[p.index()] == 0 {
            slot.take()
        } else {
            slot.clone()
        }
        .expect("a premise's sequent is kept until its last reader")
    }

    /// Whether `o` is in output position, under a reading.
    fn is_output(&self, o: OccId) -> bool {
        self.reading
            .is_some_and(|r| r.position(o) == Position::Output)
    }

    /// Fails unless `o` has the kind the rule acts on.
    fn expect(&self, o: OccId, kind: Kind) -> Result<(), Problem> {
        if self.forest.kind(o) == kind {
            Ok(())
        } else {
            Err(Problem::Kind(o))
        }
    }

    /// Adds one copy of `o` to the linear zone of `d`.
    fn put(&self, d: &mut State, o: OccId) {
        d.gamma.insert(o);
        let weight = u64::from(self.observer.weight(o));
        d.linear = d.linear.saturating_add(weight);
        if self.is_output(o) {
            d.outputs = d.outputs.saturating_add(1);
            d.goal = d.goal.saturating_add(weight);
        }
    }

    /// Consumes one copy of `o` from the linear zone of `d`, the derived
    /// sequent of premise `premise`, and returns whether there was none: a
    /// `⊤` above stands in for a missing one, but never for a second goal.
    fn take(&self, d: &mut State, o: OccId, premise: usize) -> Result<bool, Problem> {
        if d.gamma.remove(o) {
            // The premise's sums are exact and count `o`, which was a
            // member: nothing goes below zero.
            let weight = u64::from(self.observer.weight(o));
            d.linear -= weight;
            if self.is_output(o) {
                d.outputs -= 1;
                d.goal -= weight;
            }
            return Ok(false);
        }
        if !d.any {
            return Err(Problem::Missing {
                premise,
                occurrence: o,
            });
        }
        // The premise's sequent holds `o` besides its zone: one goal at
        // most.
        if self.is_output(o) && d.outputs > 0 {
            return Err(Problem::Succedents(2));
        }
        Ok(true)
    }

    /// The `?` formula an unrestricted occurrence stands for.
    fn quest(&self, a: OccId) -> OccId {
        self.forest
            .parent(a)
            .expect("an unrestricted occurrence is under a ?")
    }

    /// Checks that the linear zone node `id` derived is one the proof can
    /// still conclude its goal from. A rule consumes two members of a
    /// premise's zone at most and passes the others on, and the root's zone
    /// lies within the goal, so a zone with more members than the goal has
    /// formulas plus two for every later node belongs to no proof. This is
    /// what keeps a zone's counters exact: a term may double a zone at
    /// every node (a Mix of a subproof with itself), which no counter of a
    /// fixed width follows for long.
    fn within(&self, id: NodeId, d: Box<State>) -> Result<Box<State>, Problem> {
        // A node's index is below the number of nodes.
        let later = self.proof.nodes().len() - 1 - id.index();
        let room = self
            .goal
            .saturating_add(later.saturating_mul(2))
            .min(Bag::MOST);
        if d.gamma.len > room {
            return Err(Problem::Surplus);
        }
        Ok(d)
    }

    /// Checks the one-succedent condition on what a node derived: one
    /// output at most, and exactly one unless a `⊤` above supplies it.
    fn one_succedent(&self, d: Box<State>) -> Result<Box<State>, Problem> {
        if self.reading.is_some() && (d.outputs > 1 || (d.outputs == 0 && !d.any)) {
            return Err(Problem::Succedents(d.outputs));
        }
        Ok(d)
    }

    /// The left subformula of `o`, which must have one.
    fn left(&self, o: OccId) -> OccId {
        self.forest.left(o).unwrap()
    }

    /// The right subformula of `o`, which must have one.
    fn right(&self, o: OccId) -> OccId {
        self.forest.right(o).unwrap()
    }

    /// The sequent with only the given occurrences in its linear zone.
    fn just(&self, ids: impl IntoIterator<Item = OccId>, any: bool) -> Box<State> {
        let mut d = Box::new(State {
            theta: HashSet::default(),
            gamma: Bag::default(),
            outputs: 0,
            linear: 0,
            unrestricted: 0,
            goal: 0,
            any,
        });
        for o in ids {
            self.put(&mut d, o);
        }
        d
    }

    /// Unites the unrestricted zone of `r` into that of `l`, the smaller
    /// set into the larger, and notes the occurrences both hold.
    fn unite(&mut self, l: &mut State, r: &mut State) {
        if r.theta.len() > l.theta.len() {
            std::mem::swap(&mut l.theta, &mut r.theta);
            std::mem::swap(&mut l.unrestricted, &mut r.unrestricted);
        }
        for a in r.theta.drain() {
            if l.theta.insert(a) {
                l.unrestricted += u64::from(self.observer.weight(self.quest(a)));
            } else {
                self.shared.push(a);
            }
        }
    }

    /// The sequent both premises of a two-premise rule give together: the
    /// unrestricted zones united, the linear zones summed, absorbing if
    /// either is.
    fn join(&mut self, mut l: Box<State>, mut r: Box<State>) -> Box<State> {
        self.unite(&mut l, &mut r);
        l.gamma.add(std::mem::take(&mut r.gamma));
        l.outputs = l.outputs.saturating_add(r.outputs);
        l.linear = l.linear.saturating_add(r.linear);
        l.goal = l.goal.saturating_add(r.goal);
        l.any |= r.any;
        l
    }

    /// Applies a node's rule to what its premises derived, noting in
    /// `facts` how it applied.
    fn rule(&mut self, node: Node, facts: &mut Facts<'_>) -> Result<Box<State>, Problem> {
        use Node::*;
        let f = self.forest;
        self.shared.clear();
        match node {
            Ax(a, b) => {
                for o in [a, b] {
                    if !f.is_literal(o) {
                        return Err(Problem::Kind(o));
                    }
                }
                if f.atom(a) != f.atom(b) || f.sign(a) == f.sign(b) {
                    return Err(Problem::NotDual);
                }
                Ok(self.just([a, b], false))
            }
            One(o) => {
                self.expect(o, Kind::One)?;
                Ok(self.just([o], false))
            }
            Top(o) => {
                self.expect(o, Kind::Top)?;
                Ok(self.just([o], true))
            }
            Bot(o, p) => {
                self.expect(o, Kind::Bot)?;
                let mut d = self.premise(p);
                self.put(&mut d, o);
                Ok(d)
            }
            Par(o, p) => {
                self.expect(o, Kind::Par)?;
                let mut d = self.premise(p);
                facts.absent[0] = self.take(&mut d, self.left(o), 0)?;
                facts.absent[1] = self.take(&mut d, self.right(o), 0)?;
                self.put(&mut d, o);
                Ok(d)
            }
            Tensor(o, l, r) => {
                self.expect(o, Kind::Tensor)?;
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                facts.needs = [dl.theta.len(), dr.theta.len()];
                facts.left_goal = dl.outputs > 0 || self.is_output(self.left(o));
                facts.absent[0] = self.take(&mut dl, self.left(o), 0)?;
                facts.absent[1] = self.take(&mut dr, self.right(o), 1)?;
                let mut d = self.join(dl, dr);
                self.put(&mut d, o);
                Ok(d)
            }
            With(o, l, r) => {
                self.expect(o, Kind::With)?;
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                facts.needs = [dl.theta.len(), dr.theta.len()];
                facts.absent[0] = self.take(&mut dl, self.left(o), 0)?;
                facts.absent[1] = self.take(&mut dr, self.right(o), 1)?;
                // The conclusion's context is what both premises can prove
                // it under: an absorbing premise adapts to the other one.
                let mut d = match (dl.any, dr.any) {
                    (false, false) if dl.gamma == dr.gamma => self.over(dl, dr),
                    (true, false) if dl.gamma.is_subset(&dr.gamma) => self.over(dr, dl),
                    (false, true) if dr.gamma.is_subset(&dl.gamma) => self.over(dl, dr),
                    (true, true) => {
                        let gamma = std::mem::take(&mut dr.gamma);
                        dl.gamma.unite(gamma);
                        let mut d = self.over(dl, dr);
                        d.any = true;
                        self.recount(&mut d);
                        d
                    }
                    _ => return Err(Problem::Differ),
                };
                self.shared.clear();
                self.put(&mut d, o);
                Ok(d)
            }
            Plus(o, side, p) => {
                self.expect(o, Kind::Plus)?;
                let mut d = self.premise(p);
                let chosen = match side {
                    Side::Left => self.left(o),
                    Side::Right => self.right(o),
                };
                facts.absent[0] = self.take(&mut d, chosen, 0)?;
                self.put(&mut d, o);
                Ok(d)
            }
            Bang(o, p) => {
                self.expect(o, Kind::Bang)?;
                let mut d = self.premise(p);
                facts.absent[0] = self.take(&mut d, self.left(o), 0)?;
                if !d.gamma.is_empty() {
                    return Err(Problem::NotEmpty);
                }
                // Promotion fixes the linear zone: a ⊤ above cannot absorb
                // past it.
                d.any = false;
                self.put(&mut d, o);
                Ok(d)
            }
            Quest(o, p) => {
                self.expect(o, Kind::Quest)?;
                let mut d = self.premise(p);
                if d.theta.remove(&self.left(o)) {
                    // What the member added when it came in.
                    facts.used = true;
                    d.unrestricted -= u64::from(self.observer.weight(o));
                }
                self.put(&mut d, o);
                Ok(d)
            }
            Copy(a, p) => {
                if f.parent(a).map(|q| f.kind(q)) != Some(Kind::Quest) {
                    return Err(Problem::NotUnderQuest(a));
                }
                let mut d = self.premise(p);
                facts.absent[0] = self.take(&mut d, a, 0)?;
                if d.theta.insert(a) {
                    d.unrestricted += u64::from(self.observer.weight(self.quest(a)));
                } else {
                    facts.used = true;
                }
                Ok(d)
            }
            Weaken(o, p) => {
                // Weakening a `?` formula is a rule of every mode; the goal
                // is never weakened.
                if !self.mode.affine && f.kind(o) != Kind::Quest {
                    return Err(Problem::Forbidden);
                }
                if self.is_output(o) {
                    return Err(Problem::Succedents(0));
                }
                let mut d = self.premise(p);
                self.put(&mut d, o);
                Ok(d)
            }
            Mix(l, r) => {
                // Mix has no intuitionistic form: a premise would lack the
                // goal.
                if !self.mode.mix || self.mode.intuitionistic {
                    return Err(Problem::Forbidden);
                }
                let (dl, dr) = (self.premise(l), self.premise(r));
                facts.needs = [dl.theta.len(), dr.theta.len()];
                facts.left_goal = dl.outputs > 0;
                Ok(self.join(dl, dr))
            }
        }
    }

    /// The conclusion of a `&` whose linear zone is that of `keep`: its
    /// sequent with the unrestricted zone of `other` united into it.
    fn over(&mut self, mut keep: Box<State>, mut other: Box<State>) -> Box<State> {
        self.unite(&mut keep, &mut other);
        keep
    }

    /// Counts the outputs and the weights of the linear zone of `d` anew,
    /// after it was replaced.
    fn recount(&self, d: &mut State) {
        let (mut outputs, mut linear, mut goal) = (0usize, 0u64, 0u64);
        for (o, n) in d.gamma.counts() {
            // Two factors below 2³².
            let weight = u64::from(self.observer.weight(o)) * u64::from(n);
            linear = linear.saturating_add(weight);
            if self.is_output(o) {
                outputs = outputs.saturating_add(n as usize);
                goal = goal.saturating_add(weight);
            }
        }
        (d.outputs, d.linear, d.goal) = (outputs, linear, goal);
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::proofs::oracle;
    use crate::search::generate::Rng;
    use crate::{Error, Sequent};

    /// Wraps a raw occurrence id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Wraps a raw node id.
    const fn n(id: u32) -> NodeId {
        NodeId::new(id)
    }

    /// Builds a proof of the sequent `input` parses to, with the last node
    /// as the root. Occurrence ids are the preorder numbering, which each
    /// test lists in a comment.
    fn proof(input: &str, nodes: Vec<Node>) -> Proof {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
    }

    /// A proof with one node changed at random, or none when the change
    /// does not leave a proof term: another occurrence, another premise,
    /// another rule on the same operands, the premises swapped, or a
    /// weakening in the node's place.
    fn mutant(rng: &mut Rng, proof: &Proof) -> Option<Proof> {
        use Node::*;
        let mut nodes = proof.nodes().to_vec();
        let i = rng.below(nodes.len());
        let occurrences = proof.forest().len();
        let occ = |rng: &mut Rng| o(rng.below(occurrences) as u32);
        let earlier = |rng: &mut Rng| n(rng.below(i.max(1)) as u32);
        let unary = |k: usize, o: OccId, p: NodeId| match k % 8 {
            0 => Bot(o, p),
            1 => Par(o, p),
            2 => Bang(o, p),
            3 => Quest(o, p),
            4 => Copy(o, p),
            5 => Weaken(o, p),
            6 => Plus(o, Side::Left, p),
            _ => Plus(o, Side::Right, p),
        };
        let choice = rng.below(5);
        nodes[i] = match (nodes[i], choice) {
            (Ax(a, _), 0) => Ax(a, occ(rng)),
            (Ax(a, b), 1) => Ax(b, a),
            (Ax(a, _), 2) => One(a),
            (Ax(_, b), 3) => Top(b),
            (One(a) | Top(a), 0 | 1) => Ax(a, occ(rng)),
            (One(a), _) => Top(a),
            (Top(a), _) => One(a),
            (Tensor(_, l, r) | With(_, l, r), 0) => Tensor(occ(rng), l, r),
            (Tensor(a, _, r), 1) => Tensor(a, earlier(rng), r),
            (With(a, l, _), 1) => With(a, l, earlier(rng)),
            (Tensor(a, l, r), 2) => With(a, l, r),
            (With(a, l, r), 2) => Tensor(a, l, r),
            (Tensor(a, l, r), 3) => Tensor(a, r, l),
            (With(a, l, r), 3) => With(a, r, l),
            (Tensor(_, l, r) | With(_, l, r), _) => Mix(l, r),
            (Mix(l, r), 0 | 1) => Tensor(occ(rng), l, r),
            (Mix(l, r), 2) => With(occ(rng), l, r),
            (Mix(l, r), 3) => Mix(r, l),
            (Mix(l, _), _) => Mix(l, earlier(rng)),
            (node, 0) => unary(rng.below(8), occ(rng), node.premises().next()?),
            (node, 1) => unary(rng.below(8), node.occurrences().next()?, earlier(rng)),
            (node, 2) => unary(
                rng.below(8),
                node.occurrences().next()?,
                node.premises().next()?,
            ),
            (node, 3) => Weaken(occ(rng), node.premises().next()?),
            (Ax(a, _), _) => Ax(a, a),
            (node, _) => {
                let p = node.premises().next()?;
                Mix(p, earlier(rng))
            }
        };
        Proof::new(proof.forest().clone(), nodes, proof.root()).ok()
    }

    /// The checker agrees with its first implementation, which keeps every
    /// node's sequent, on the verdict and on the error: on the proofs the
    /// engines find of generated sequents, classical and intuitionistic,
    /// and of the smallest instance of every family, in the mode they were
    /// found in and in the others, and on mutants of them.
    #[test]
    fn agrees_with_the_first_implementation() {
        let classical = Mode::CLASSICAL;
        let proofs = oracle::proofs();
        assert!(proofs.len() > 1000, "{} proofs", proofs.len());

        let modes = [
            classical,
            classical.affine().with_mix(),
            Mode::INTUITIONISTIC,
            Mode::INTUITIONISTIC.affine(),
        ];
        let mut rng = Rng::new(7);
        let (mut mutants, mut rejected) = (0, 0);
        for (proof, mode) in &proofs {
            assert_eq!(check(proof, *mode), Ok(()), "{}", proof.sequent());
            for mode in modes {
                assert_eq!(
                    check(proof, mode),
                    oracle::check(proof, mode),
                    "{} in {mode} mode",
                    proof.sequent()
                );
            }
            for _ in 0..8 {
                let Some(mutant) = mutant(&mut rng, proof) else {
                    continue;
                };
                mutants += 1;
                for mode in [*mode, classical.affine().with_mix()] {
                    let result = check(&mutant, mode);
                    rejected += u32::from(result.is_err());
                    assert_eq!(
                        result,
                        oracle::check(&mutant, mode),
                        "{:?} of {} in {mode} mode",
                        mutant.nodes(),
                        mutant.sequent()
                    );
                }
            }
        }
        assert!(mutants > 5000, "{mutants} mutants");
        assert!(
            rejected > mutants,
            "{rejected} of {mutants} mutants rejected"
        );
    }

    /// Every rule applied correctly is accepted: the multiplicatives and
    /// their units, the additives with `⊤` absorbing its context, the
    /// exponentials in dyadic form, weakening in affine mode and Mix.
    #[test]
    fn accepts_every_rule() {
        use Node::*;
        let classical = Mode::CLASSICAL;
        for (input, nodes, mode) in [
            // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))],
                classical,
            ),
            // ⊢ A ⅋ ~A: 0 ⅋, 1 A, 2 ~A
            (
                "|- A par ~A",
                vec![Ax(o(2), o(1)), Par(o(0), n(0))],
                classical,
            ),
            // ⊢ 1, ⊥
            ("|- 1, bot", vec![One(o(0)), Bot(o(1), n(0))], classical),
            // ⊢ A & B, ~A ⊕ ~B: 0 &, 1 A, 2 B, 3 ⊕, 4 ~A, 5 ~B
            (
                "|- A & B, ~A + ~B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(3), Side::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(3), Side::Right, n(2)),
                    With(o(0), n(1), n(3)),
                ],
                classical,
            ),
            // ⊢ ⊤, A: ⊤ absorbs A.
            ("|- top, A", vec![Top(o(0))], classical),
            // ⊢ A & ⊤, ~A: 0 &, 1 A, 2 ⊤, 3 ~A: the ⊤ side adapts to the
            // other.
            (
                "|- A & top, ~A",
                vec![Ax(o(1), o(3)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ⊤ ⊗ A, ~A, B: 0 ⊗, 1 ⊤, 2 A, 3 ~A, 4 B: the ⊤ side of the
            // split takes B.
            (
                "|- top * A, ~A, B",
                vec![Top(o(1)), Ax(o(2), o(3)), Tensor(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ?~A, A: 0 ?, 1 ~A, 2 A: a copy, then the ? step.
            (
                "!A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0)), Quest(o(0), n(1))],
                classical,
            ),
            // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A: two copies of one
            // occurrence.
            (
                "!A |- A * A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Ax(o(1), o(4)),
                    Copy(o(1), n(2)),
                    Tensor(o(2), n(1), n(3)),
                    Quest(o(0), n(4)),
                ],
                classical,
            ),
            // ⊢ ?~A, 1: 0 ?, 1 ~A, 2 1: an unused ? formula.
            ("!A |- 1", vec![One(o(2)), Quest(o(0), n(0))], classical),
            // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B:
            // promotion under two ? formulas.
            (
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
                ],
                classical,
            ),
            // ⊢ !(A ⅋ ~A): 0 !, 1 ⅋, 2 A, 3 ~A: promotion of a closed
            // formula.
            (
                "|- !(A par ~A)",
                vec![Ax(o(2), o(3)), Par(o(1), n(0)), Bang(o(0), n(1))],
                classical,
            ),
            // ⊢ ~A, ~B, A: weakening of ~B.
            (
                "A, B |- A",
                vec![Ax(o(0), o(2)), Weaken(o(1), n(0))],
                classical.affine(),
            ),
            // ⊢ 1, ?A: weakening of a ? formula needs no affine mode.
            ("|- 1, ?A", vec![One(o(0)), Weaken(o(1), n(0))], classical),
            // ⊢ ?⊤, !0: 0 ?, 1 ⊤, 2 !, 3 0: the ⊤ stands in for the 0 the
            // promotion consumes, and the zone is empty after it.
            (
                "|- ?top, !0",
                vec![
                    Top(o(1)),
                    Copy(o(1), n(0)),
                    Bang(o(2), n(1)),
                    Quest(o(0), n(2)),
                ],
                classical,
            ),
            // ⊢ ⊤ & ⊤, A: both sides absorb.
            (
                "|- top & top, A",
                vec![Top(o(1)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ⊤, ?A: 0 ⊤, 1 ?, 2 A: a copy the ⊤ stands in for.
            (
                "|- top, ?A",
                vec![Top(o(0)), Copy(o(2), n(0)), Quest(o(1), n(1))],
                classical,
            ),
            // ⊢ ?(?~A), A ⊗ A: 0 ?, 1 ?, 2 ~A, 3 ⊗, 4 A, 5 A: the inner ?
            // is copied twice and its ? step taken twice on one path.
            (
                "|- ?(?~A), A * A",
                vec![
                    Ax(o(2), o(4)),
                    Copy(o(2), n(0)),
                    Ax(o(2), o(5)),
                    Copy(o(2), n(2)),
                    Tensor(o(3), n(1), n(3)),
                    Quest(o(1), n(4)),
                    Copy(o(1), n(5)),
                    Quest(o(1), n(6)),
                    Copy(o(1), n(7)),
                    Quest(o(0), n(8)),
                ],
                classical,
            ),
            // ⊢ ?A, ?~A: 0 ?, 1 A, 2 ?, 3 ~A: one subproof shared by both
            // sides of a Mix.
            (
                "|- ?A, ?~A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Copy(o(3), n(1)),
                    Mix(n(2), n(2)),
                    Quest(o(2), n(3)),
                    Quest(o(0), n(4)),
                ],
                classical.with_mix(),
            ),
            // ⊢ ~A ⅋ ~B, A ⅋ B: 0 ⅋, 1 ~A, 2 ~B, 3 ⅋, 4 A, 5 B: needs Mix.
            (
                "A * B |- A par B",
                vec![
                    Ax(o(1), o(4)),
                    Ax(o(2), o(5)),
                    Mix(n(0), n(1)),
                    Par(o(0), n(2)),
                    Par(o(3), n(3)),
                ],
                classical.with_mix(),
            ),
        ] {
            let p = proof(input, nodes);
            assert_eq!(p.check(mode), Ok(()), "{input:?}");
        }
    }

    /// Every rule misapplied is rejected at the node that misapplies it,
    /// with the problem named; a rule the mode forbids is rejected too, and
    /// so is a root that concludes something else.
    #[test]
    fn rejects_every_misuse() {
        use Node::*;
        use Problem::*;
        let classical = Mode::CLASSICAL;
        for (input, nodes, mode, node, problem) in [
            // Axiom on two copies of one literal.
            ("|- A, A", vec![Ax(o(0), o(1))], classical, 0, NotDual),
            // Axiom on a connective: 0 ⊗, 1 A, 2 B, 3 ~A.
            (
                "|- A * B, ~A",
                vec![Ax(o(0), o(3))],
                classical,
                0,
                Kind(o(0)),
            ),
            // ⊗ with its premises swapped: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B.
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(1), n(0))],
                classical,
                2,
                Missing {
                    premise: 0,
                    occurrence: o(2),
                },
            ),
            // ⅋ on a ⊗ occurrence.
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Par(o(1), n(0))],
                classical,
                1,
                Kind(o(1)),
            ),
            // ⅋ whose premise lacks a subformula: 0 A, 1 ⅋, 2 ~A, 3 B.
            (
                "|- A, ~A par B",
                vec![Ax(o(0), o(2)), Par(o(1), n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    occurrence: o(3),
                },
            ),
            // 1 on ⊥.
            ("|- bot", vec![One(o(0))], classical, 0, Kind(o(0))),
            // ⊥ on 1.
            (
                "|- 1, bot",
                vec![One(o(0)), Bot(o(0), n(0))],
                classical,
                1,
                Kind(o(0)),
            ),
            // & whose premises consume different contexts: 0 &, 1 A, 2 B,
            // 3 ~A, 4 ~B.
            (
                "|- A & B, ~A, ~B",
                vec![Ax(o(1), o(3)), Ax(o(2), o(4)), With(o(0), n(0), n(1))],
                classical,
                2,
                Differ,
            ),
            // & whose ⊤ side needs more than the other side has: the root
            // then concludes ⊢ A & ⊤, ~A without ~B.
            (
                "|- A & top, ~A, ~B",
                vec![Ax(o(1), o(3)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
                2,
                Conclusion(Dyadic {
                    theta: vec![],
                    gamma: vec![o(0), o(3)],
                    any: false,
                }),
            ),
            // ⊕ on the side the premise does not prove: 0 ⊕, 1 A, 2 B, 3 ~A.
            (
                "|- A + B, ~A",
                vec![Ax(o(1), o(3)), Plus(o(0), Side::Right, n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    occurrence: o(2),
                },
            ),
            // ⊤ on 0.
            ("|- 0", vec![Top(o(0))], classical, 0, Kind(o(0))),
            // Promotion with a linear formula in the context: 0 ~A, 1 !, 2 A.
            (
                "A |- !A",
                vec![Ax(o(0), o(2)), Bang(o(1), n(0))],
                classical,
                1,
                NotEmpty,
            ),
            // A copy of a formula under !: 0 !, 1 ~A, 2 A.
            (
                "?A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0))],
                classical,
                1,
                NotUnderQuest(o(1)),
            ),
            // A copy of a formula the premise does not hold: 0 ?, 1 ~A, 2 1.
            (
                "!A |- 1",
                vec![One(o(2)), Copy(o(1), n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    occurrence: o(1),
                },
            ),
            // A copy without the ? step below it: the root still needs ~A
            // in the unrestricted zone.
            (
                "!A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0))],
                classical,
                1,
                Conclusion(Dyadic {
                    theta: vec![o(1)],
                    gamma: vec![o(2)],
                    any: false,
                }),
            ),
            // ? on a ! occurrence.
            (
                "|- !(A par ~A)",
                vec![Ax(o(2), o(3)), Par(o(1), n(0)), Quest(o(0), n(1))],
                classical,
                2,
                Kind(o(0)),
            ),
            // Weakening outside affine mode.
            (
                "A, B |- A",
                vec![Ax(o(0), o(2)), Weaken(o(1), n(0))],
                classical,
                1,
                Forbidden,
            ),
            // Mix without Mix.
            (
                "|- A, ~A, B, ~B",
                vec![Ax(o(0), o(1)), Ax(o(2), o(3)), Mix(n(0), n(1))],
                classical,
                2,
                Forbidden,
            ),
            // A proof of a smaller sequent.
            (
                "|- A, ~A, 1",
                vec![Ax(o(0), o(1))],
                classical,
                0,
                Conclusion(Dyadic {
                    theta: vec![],
                    gamma: vec![o(0), o(1)],
                    any: false,
                }),
            ),
            // Intuitionistic mode needs an intuitionistic sequent, whatever
            // the proof.
            (
                "|- A par B",
                vec![Ax(o(1), o(2))],
                Mode::INTUITIONISTIC,
                0,
                Shape(crate::occurrences::ShapeError::Formula(o(0))),
            ),
        ] {
            let p = proof(input, nodes);
            let e = p.check(mode).unwrap_err();
            let message = e.to_string();
            assert_eq!(
                (e.node, e.problem),
                (n(node), problem),
                "{input:?}: {message}"
            );
        }
    }

    /// A term that doubles a zone at every node is refused where the zone
    /// outgrows what the rest of the proof can consume, long before a
    /// counter is full: this one mixes 2⁶⁴ copies of `⊢ 1`, promotes `⊥`
    /// over them and mixes one more in, and `⊢ !⊥, 1` has no proof.
    #[test]
    fn refuses_a_zone_too_large_to_conclude() {
        use Node::*;
        // ⊢ !⊥, 1: 0 !, 1 ⊥, 2 1
        let mut nodes = vec![One(o(2))];
        for i in 0..63 {
            nodes.push(Mix(n(i), n(i)));
        }
        let mut all = n(0);
        for i in 1..64 {
            nodes.push(Mix(all, n(i)));
            all = n(nodes.len() as u32 - 1);
        }
        nodes.push(Mix(all, n(0)));
        nodes.push(Bot(o(1), n(nodes.len() as u32 - 1)));
        nodes.push(Bang(o(0), n(nodes.len() as u32 - 1)));
        nodes.push(Mix(n(nodes.len() as u32 - 1), n(0)));
        let p = proof("|- !bot, 1", nodes);
        let mode = Mode::CLASSICAL.with_mix();
        let e = p.check(mode).unwrap_err();
        // Node 8 holds 256 copies with 122 nodes to come.
        assert_eq!((e.node, &e.problem), (n(8), &Problem::Surplus));
        assert_eq!(Err(e.clone()), oracle::check(&p, mode));
        // The pass that adds up weights for the size refuses it there too.
        assert_eq!(p.derivation_size(false), Err(e));
    }

    /// Intuitionistic mode accepts the classical terms of intuitionistic
    /// proofs and rejects a sequent with two goals or none: a `⊸L` split
    /// that keeps the goal on the antecedent's side, a weakened goal, Mix.
    #[test]
    fn intuitionistic() {
        use Node::*;
        use Problem::*;
        let m = Mode::INTUITIONISTIC;
        for (input, nodes) in [
            // A, A ⊸ B ⊢ B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))],
            ),
            // A & B ⊢ A & B: 0 ⊕, 1 ~A, 2 ~B, 3 &, 4 A, 5 B
            (
                "A & B |- A & B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(0), Side::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(0), Side::Right, n(2)),
                    With(o(3), n(1), n(3)),
                ],
            ),
            // A ⊸ 0, A ⊢ B: 0 ⊗, 1 A, 2 ⊤, 3 ~A, 4 B: the `0` on the left
            // absorbs the goal.
            (
                "A -o 0, A |- B",
                vec![Ax(o(1), o(3)), Top(o(2)), Tensor(o(0), n(0), n(1))],
            ),
            // ⊢ ⊤, ⊤ ⊗ ⊤ is read with the last root as the goal, 0 ⊢ ⊤ ⊗ ⊤:
            // 0 ⊤, 1 ⊗, 2 ⊤, 3 ⊤; the hypothesis `0` is absorbed by one
            // `⊤R`.
            (
                "|- top, top * top",
                vec![Top(o(2)), Top(o(3)), Tensor(o(1), n(0), n(1))],
            ),
            // !A, !(A ⊸ B) ⊢ !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B
            (
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
                ],
            ),
        ] {
            let p = proof(input, nodes);
            p.check(m)
                .unwrap_or_else(|e| panic!("{input:?}: {}", e.describe(p.forest())));
        }
        // A, B ⊢ A in affine mode: weakening the hypothesis is fine, the
        // goal never.
        let p = proof("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(1), n(0))]);
        assert_eq!(p.check(m.affine()), Ok(()));
        // A, 0 ⊢ B: 0 ~A, 1 ⊤, 2 B
        let p = proof("A, 0 |- B", vec![Top(o(1)), Weaken(o(2), n(0))]);
        assert_eq!(p.check(m.affine()).unwrap_err().problem, Succedents(0));
        // ((A ⊗ ⊤) & (B ⊗ ⊤)) ⊸ 0 ⊢ (A ⊸ C) ⊕ (B ⊸ C), which classical
        // linear logic proves and intuitionistic linear logic does not:
        // 0 ⊗, 1 &, 2 ⊗, 3 A, 4 ⊤, 5 ⊗, 6 B, 7 ⊤, 8 ⊤, 9 ⊕, 10 ⅋, 11 ~A,
        // 12 C, 13 ⅋, 14 ~B, 15 C. The classical proof splits the `⊸L`
        // with the goal on the antecedent's side, where a `⊤` absorbs it.
        let p = proof(
            "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)",
            vec![
                Ax(o(3), o(11)),
                Top(o(4)),
                Tensor(o(2), n(0), n(1)),
                Par(o(10), n(2)),
                Plus(o(9), Side::Left, n(3)),
                Ax(o(6), o(14)),
                Top(o(7)),
                Tensor(o(5), n(5), n(6)),
                Par(o(13), n(7)),
                Plus(o(9), Side::Right, n(8)),
                With(o(1), n(4), n(9)),
                Top(o(8)),
                Tensor(o(0), n(10), n(11)),
            ],
        );
        assert_eq!(p.check(Mode::CLASSICAL), Ok(()));
        let e = p.check(m).unwrap_err();
        assert_eq!((e.node, e.problem.clone()), (n(3), Succedents(2)));
        assert_eq!(
            e.describe(p.forest()).to_string(),
            "node 3 (⅋ on ~A ⅋ C from 2) with premises ⊢ A ⊗ ⊤, ~A, …: \
             a sequent of the rule has 2 formulas on the right of ⊢ instead of one"
        );
        // Mix is never intuitionistic. 0 ⊗, 1 A, 2 ~B, 3 ~A, 4 B: A ⊸ B, A
        // ⊢ B with a Mix of the axioms.
        let p = proof(
            "A -o B, A |- B",
            vec![Ax(o(1), o(3)), Ax(o(2), o(4)), Mix(n(0), n(1))],
        );
        assert_eq!(p.check(m.with_mix()).unwrap_err().problem, Forbidden);
    }

    /// The error names the node, its premises and the problem.
    #[test]
    fn error_message() {
        use Node::*;
        // ⊢ ~A, A ⊗ ~B, B with the ⊗ premises swapped.
        let p = proof(
            "A, A -o B |- B",
            vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(1), n(0))],
        );
        assert_eq!(
            p.check(Mode::CLASSICAL).unwrap_err().to_string(),
            "node 2 (⊗ on 1 from 1, 0) with premises ⊢ 3, 4 and ⊢ 0, 2: \
             premise 0 lacks occurrence 2"
        );
        assert_eq!(
            p.check(Mode::CLASSICAL)
                .unwrap_err()
                .describe(p.forest())
                .to_string(),
            "node 2 (⊗ on A ⊗ ~B from 1, 0) with premises ⊢ ~B, B and ⊢ ~A, A: \
             premise 0 lacks A"
        );
        // ⊢ ?~A, A with the ? step missing: a dyadic sequent with Θ.
        let p = proof("!A |- A", vec![Ax(o(1), o(2)), Copy(o(1), n(0))]);
        assert_eq!(
            p.check(Mode::CLASSICAL).unwrap_err().to_string(),
            "node 1 (copy on 1 from 0) with premises ⊢ 1, 2: \
             the proof concludes ⊢ 1 ; 2, not the sequent"
        );
        assert_eq!(
            Dyadic {
                theta: vec![],
                gamma: vec![o(0)],
                any: true
            }
            .to_string(),
            "⊢ 0, …"
        );
    }

    /// A proof keeps only the nodes its root reaches, renumbered in order,
    /// and refuses a node that points outside the arena or the forest, or
    /// at a later node.
    #[test]
    fn construction() {
        use Node::*;
        let s: Sequent = "A, A -o B |- B".parse().unwrap();
        let f = Forest::new(&s).unwrap();
        // Node 1 is unused.
        let p = Proof::new(
            f.clone(),
            vec![
                Ax(o(0), o(2)),
                One(o(4)),
                Ax(o(3), o(4)),
                Tensor(o(1), n(0), n(2)),
            ],
            n(3),
        )
        .unwrap();
        assert_eq!(
            p.nodes(),
            [Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))]
        );
        assert_eq!(p.root(), n(2));
        assert_eq!(p.check(Mode::CLASSICAL), Ok(()));
        // The root is beyond the arena.
        assert!(matches!(
            Proof::new(f.clone(), vec![Ax(o(0), o(2))], n(1)),
            Err(Error::NodeIndexOutOfBounds(1, 1))
        ));
        // A premise does not precede its conclusion.
        assert!(matches!(
            Proof::new(f.clone(), vec![Par(o(1), n(0))], n(0)),
            Err(Error::PremiseIndexNotDecreasing(0, 0))
        ));
        // An occurrence beyond the forest.
        assert!(matches!(
            Proof::new(f, vec![Ax(o(0), o(5))], n(0)),
            Err(Error::OccurrenceIndexOutOfBounds(5, 5))
        ));
    }
}

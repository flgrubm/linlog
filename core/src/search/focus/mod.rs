// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused sequent engine: backward search over occurrence bitsets with
//! a memo of stable sequents, as the `MALL-Seq` specification states it.
//!
//! A sequent inside the search is an [`OccSet`] of the forest. The
//! *asynchronous phase* decomposes the negative formulas of a sequent
//! without choice (`⅋` opens, `⊥` drops, `⊤` closes, `&` branches into two
//! premises with the same context) until a *stable* sequent remains: a set
//! of positive formulas and negative literals. `prove` decides a stable
//! sequent by looking it up in the memo, by the immediate tests (a `0`, an
//! unbalanced atom, a dual pair), and otherwise by choosing a positive
//! formula to *focus* on. `focus` applies the positive rules along that
//! formula: `⊕` picks a side, `⊗` splits the context, `1` needs it empty, a
//! positive literal needs exactly its dual; and when the formula turns
//! negative it is *released* into the asynchronous phase. Every stable
//! sequent proved or refuted goes into the memo, so a `&` that duplicates
//! the context, or a split that revisits a part, never pays twice. With
//! Mix, a stable sequent no focus proves is split into two provable parts.
//!
//! The counts of [`counts`] prune: a stable sequent or a side of a split
//! whose intervals exclude zero for some atom is refuted without search,
//! and in the multiplicative fragments the count equation as well.

/// The count invariants the engine prunes with.
mod counts;
/// The memo of stable sequents.
mod memo;

use self::counts::{Counts, Tally};
use self::memo::{Entry, Memo};
use super::{Options, Reason, Statistics, Verdict};
use crate::fragment::{Fragment, Mode};
use crate::occurrences::{Forest, OccId, OccSet, Polarity, submasks};
use crate::proofs::{Node, NodeId, Proof, Side};
use crate::sequents::Kind;

/// The most members a context can have for its splits to be enumerated.
const MAX_SPLIT: usize = 63;

/// Runs the focused engine on the forest of a sequent of `fragment` under
/// `mode`, polling `stop` at every stable sequent, and returns the verdict
/// with the statistics of the run.
pub(crate) fn search(
    forest: &Forest,
    fragment: Fragment,
    mode: Mode,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Verdict, Statistics) {
    let mut engine = Engine::new(forest, fragment, mode, options, stop);
    let result = engine.run();
    let statistics = engine.statistics();
    let verdict = match result {
        Ok(Some(root)) => {
            let proof = Proof::new(forest.clone(), engine.nodes, root)
                .expect("the engine pushes premises before conclusions");
            debug_assert_eq!(proof.check(mode), Ok(()), "the engine's proof");
            Verdict::Proved(Box::new(proof))
        }
        Ok(None) => Verdict::Unprovable,
        Err(reason) => Verdict::Unknown(reason),
    };
    (verdict, statistics)
}

/// The rules in force beyond the core ones, switched by fragment and mode.
#[derive(Clone, Copy, Debug)]
struct Rules {
    /// The Mix rule, tried last on a stable sequent.
    mix: bool,
    /// The `MLL` count equation as a prune: only without additives and
    /// exponentials anywhere in the problem.
    equation: bool,
}

/// The result of a search step: the node proving the sequent, `None` when
/// it is unprovable, or the reason the whole search stops.
type Search = Result<Option<NodeId>, Reason>;

/// The state of one run: the problem, the memo, the proof arena, the
/// counters, and pools of scratch buffers so that no step allocates once
/// the pools are warm.
struct Engine<'a> {
    /// The problem.
    forest: &'a Forest,
    /// Its count invariants.
    counts: Counts,
    /// The rules in force.
    rules: Rules,
    /// The memo of stable sequents.
    memo: Memo,
    /// The proof arena: every node built so far, failed branches included.
    nodes: Vec<Node>,
    /// The counters.
    statistics: Statistics,
    /// The nesting of engine calls right now.
    depth: u32,
    /// The deepest nesting allowed.
    recursion_limit: u32,
    /// The caller's stop condition.
    stop: &'a mut dyn FnMut() -> bool,
    /// Spare occurrence sets of the forest's width.
    sets: Vec<OccSet>,
    /// Spare lists of occurrences.
    lists: Vec<Vec<OccId>>,
    /// Spare tallies of the forest's atoms.
    tallies: Vec<Tally>,
}

impl<'a> Engine<'a> {
    /// Prepares a run on the forest.
    fn new(
        forest: &'a Forest,
        fragment: Fragment,
        mode: Mode,
        options: &Options,
        stop: &'a mut dyn FnMut() -> bool,
    ) -> Self {
        let rules = Rules {
            mix: mode.mix,
            equation: !fragment.has_additives()
                && !fragment.has_additive_units()
                && !fragment.has_exponentials(),
        };
        Self {
            forest,
            counts: Counts::new(forest),
            rules,
            memo: Memo::new(options.memo_limit),
            nodes: Vec::new(),
            statistics: Statistics::default(),
            depth: 0,
            recursion_limit: options.recursion_limit,
            stop,
            sets: Vec::new(),
            lists: Vec::new(),
            tallies: Vec::new(),
        }
    }

    /// Returns the counters, with the memo's.
    fn statistics(&self) -> Statistics {
        Statistics {
            memo_hits: self.memo.hits(),
            memo_entries: self.memo.peak(),
            ..self.statistics
        }
    }

    /// Searches the sequent: the asynchronous phase on its root formulas.
    fn run(&mut self) -> Search {
        let mut gamma = self.take_set();
        let mut list = self.take_list();
        // A stack: the first root is decomposed first.
        list.extend(self.forest.roots().iter().rev());
        let result = self.asynchronous(&mut gamma, &mut list);
        self.give_set(gamma);
        self.give_list(list);
        result
    }

    // The phases.

    /// The asynchronous phase on `⊢ Γ ⇑ L`: decomposes the negative
    /// formulas of the list until the sequent is stable, then proves it.
    /// The list is a stack; the order does not matter for completeness.
    fn asynchronous(&mut self, gamma: &mut OccSet, list: &mut Vec<OccId>) -> Search {
        self.enter()?;
        let result = self.decompose(gamma, list);
        self.leave();
        result
    }

    /// The body of the asynchronous phase.
    fn decompose(&mut self, gamma: &mut OccSet, list: &mut Vec<OccId>) -> Search {
        // The `⅋` and `⊥` rules applied, to wrap around the proof of what
        // remains; the first applied is the lowest.
        let mut applied = self.take_list();
        let result = loop {
            let Some(o) = list.pop() else {
                break self.prove(gamma);
            };
            match self.forest.kind(o) {
                Kind::Par => {
                    list.push(self.forest.right(o).unwrap());
                    list.push(self.forest.left(o).unwrap());
                    applied.push(o);
                }
                Kind::Bot => applied.push(o),
                Kind::Top => break Ok(Some(self.push(Node::Top(o)))),
                Kind::With => break self.with(gamma, list, o),
                Kind::Quest => unreachable!("exponentials are not dispatched to this engine"),
                _ => {
                    // A positive formula or a literal: part of the stable
                    // sequent.
                    gamma.insert(o);
                }
            }
        };
        let result = result.map(|proved| {
            proved.map(|mut node| {
                for &o in applied.iter().rev() {
                    node = self.push(match self.forest.kind(o) {
                        Kind::Par => Node::Par(o, node),
                        _ => Node::Bot(o, node),
                    });
                }
                node
            })
        });
        self.give_list(applied);
        result
    }

    /// The `&` rule: both premises with the same context, the left one on
    /// copies of the state, the right one on the state itself.
    fn with(&mut self, gamma: &mut OccSet, list: &mut Vec<OccId>, o: OccId) -> Search {
        let mut left_gamma = self.take_set();
        left_gamma.clone_from(gamma);
        let mut left_list = self.take_list();
        left_list.clone_from(list);
        left_list.push(self.forest.left(o).unwrap());
        let left = self.asynchronous(&mut left_gamma, &mut left_list);
        self.give_set(left_gamma);
        self.give_list(left_list);
        let Some(left) = left? else {
            return Ok(None);
        };
        list.push(self.forest.right(o).unwrap());
        let right = self.asynchronous(gamma, list)?;
        Ok(right.map(|right| self.push(Node::With(o, left, right))))
    }

    /// `prove(Γ)` for a stable `Γ`: the memo, the immediate tests, then a
    /// focus on each positive formula in turn, then Mix.
    fn prove(&mut self, gamma: &OccSet) -> Search {
        self.enter()?;
        let result = self.prove_stable(gamma);
        self.leave();
        result
    }

    /// The body of `prove`: the memo around the decision.
    fn prove_stable(&mut self, gamma: &OccSet) -> Search {
        self.statistics.nodes += 1;
        if (self.stop)() {
            return Err(Reason::Stopped);
        }
        if let Some(entry) = self.memo.get(gamma) {
            return Ok(match entry {
                Entry::Proved(node) => Some(node),
                Entry::Failed => None,
            });
        }
        let result = self.decide(gamma)?;
        self.memo.insert(
            gamma,
            match result {
                Some(node) => Entry::Proved(node),
                None => Entry::Failed,
            },
        );
        Ok(result)
    }

    /// Decides a stable sequent the memo does not know.
    fn decide(&mut self, gamma: &OccSet) -> Search {
        let mut members = self.take_list();
        members.extend(gamma);
        let mut tally = self.take_tally();
        let mut candidates = self.take_list();
        let result = self.decide_with(gamma, &members, &mut tally, &mut candidates);
        self.give_list(members);
        self.give_tally(tally);
        self.give_list(candidates);
        result
    }

    /// The body of `decide`, with its scratch buffers.
    fn decide_with(
        &mut self,
        gamma: &OccSet,
        members: &[OccId],
        tally: &mut Tally,
        candidates: &mut Vec<OccId>,
    ) -> Search {
        // One pass over the members: the counts, a `0`, and the positive
        // formulas worth focusing on. `1` only when it is alone, since it
        // needs an empty context; a literal never, since a focus on it
        // succeeds only in the dual-pair case tested below.
        let mut zero = false;
        for &o in members {
            tally.add(&self.counts, o);
            match self.forest.kind(o) {
                Kind::Zero => zero = true,
                Kind::Tensor | Kind::Plus => candidates.push(o),
                Kind::One if members.len() == 1 => candidates.push(o),
                Kind::One | Kind::Var | Kind::DualVar => {}
                kind => unreachable!("{kind:?} in a stable sequent"),
            }
        }
        if zero {
            return Ok(None);
        }
        if let [p, q] = *members
            && self.forest.is_literal(p)
            && self.forest.atom(p) == self.forest.atom(q)
            && self.forest.sign(p) != self.forest.sign(q)
        {
            return Ok(Some(self.push(Node::Ax(p, q))));
        }
        if !tally.balanced() || (self.rules.equation && !tally.equation(self.rules.mix)) {
            return Ok(None);
        }

        // Forced splits first, then `⊕`, then the free splits; by id within
        // a class, so that the run is deterministic.
        candidates.sort_by_key(|&o| (self.focus_class(o), o));
        let mut rest = self.take_set();
        for &f in candidates.iter() {
            rest.clone_from(gamma);
            rest.remove(f);
            if let Some(node) = self.focus(&rest, f)? {
                self.give_set(rest);
                return Ok(Some(node));
            }
        }
        self.give_set(rest);

        if self.rules.mix {
            return self.mix(gamma, members, tally);
        }
        Ok(None)
    }

    /// Orders the focus candidates: a `⊗` whose split is forced by a factor
    /// (a positive literal, `1`; or `0`, which fails at once), then `⊕`,
    /// then a `⊗` whose split must be enumerated.
    fn focus_class(&self, o: OccId) -> u8 {
        match self.forest.kind(o) {
            Kind::Tensor => {
                let forced = self
                    .forest
                    .children(o)
                    .any(|c| self.forced_side(c).is_some());
                if forced { 0 } else { 2 }
            }
            _ => 1,
        }
    }

    /// `focus(Γ, F)` with `F ∉ Γ`: the positive rules along `F`, and a
    /// release into the asynchronous phase once `F` is negative.
    fn focus(&mut self, gamma: &OccSet, f: OccId) -> Search {
        self.enter()?;
        let result = self.focus_on(gamma, f);
        self.leave();
        result
    }

    /// The body of `focus`.
    fn focus_on(&mut self, gamma: &OccSet, f: OccId) -> Search {
        match self.forest.kind(f) {
            Kind::Plus => {
                for (side, sub) in [
                    (Side::Left, self.forest.left(f).unwrap()),
                    (Side::Right, self.forest.right(f).unwrap()),
                ] {
                    if let Some(node) = self.focus(gamma, sub)? {
                        return Ok(Some(self.push(Node::Plus(f, side, node))));
                    }
                }
                Ok(None)
            }
            Kind::Tensor => self.split(gamma, f),
            Kind::One => Ok(gamma.is_empty().then(|| self.push(Node::One(f)))),
            Kind::Zero => Ok(None),
            Kind::Var | Kind::DualVar if self.forest.polarity(f) == Polarity::Positive => {
                // The initial rule: the context is exactly the dual literal.
                let dual = gamma.first().filter(|&d| {
                    gamma.len() == 1
                        && self.forest.atom(d) == self.forest.atom(f)
                        && self.forest.sign(d) != self.forest.sign(f)
                });
                Ok(dual.map(|d| self.push(Node::Ax(f, d))))
            }
            _ => {
                // Negative: release.
                let mut released = self.take_set();
                released.clone_from(gamma);
                let mut list = self.take_list();
                list.push(f);
                let result = self.asynchronous(&mut released, &mut list);
                self.give_set(released);
                self.give_list(list);
                result
            }
        }
    }

    // The context splits.

    /// What a factor of a `⊗` forces on its side of the split: nothing at
    /// all for `0`, the empty context for `1`, and the dual literal alone
    /// for a positive literal.
    fn forced_side(&self, factor: OccId) -> Option<Forced> {
        match self.forest.kind(factor) {
            Kind::Zero => Some(Forced::Nothing),
            Kind::One => Some(Forced::Empty),
            Kind::Var | Kind::DualVar if self.forest.polarity(factor) == Polarity::Positive => {
                Some(Forced::Dual)
            }
            _ => None,
        }
    }

    /// The `⊗` rule on `F = A ⊗ B` with context `Γ`: the forced split when
    /// a factor allows only one, else every submask of `Γ` for the left
    /// premise in Gray-code order, each side checked by the counts before
    /// either premise is searched.
    fn split(&mut self, gamma: &OccSet, f: OccId) -> Search {
        let (a, b) = (self.forest.left(f).unwrap(), self.forest.right(f).unwrap());
        for (x, y, x_is_left) in [(a, b, true), (b, a, false)] {
            let Some(forced) = self.forced_side(x) else {
                continue;
            };
            self.statistics.splits += 1;
            // The premise of `x`, pushed once the premise of `y` succeeds.
            let x_node = match forced {
                Forced::Nothing => return Ok(None),
                Forced::Empty => Node::One(x),
                Forced::Dual => {
                    let dual = gamma.iter().find(|&m| {
                        self.forest.atom(m) == self.forest.atom(x)
                            && self.forest.sign(m) != self.forest.sign(x)
                    });
                    let Some(dual) = dual else {
                        return Ok(None);
                    };
                    Node::Ax(x, dual)
                }
            };
            let mut rest = self.take_set();
            rest.clone_from(gamma);
            if let Node::Ax(_, dual) = x_node {
                rest.remove(dual);
            }
            let y_node = self.focus(&rest, y);
            self.give_set(rest);
            let Some(y_node) = y_node? else {
                return Ok(None);
            };
            let x_node = self.push(x_node);
            let (left, right) = if x_is_left {
                (x_node, y_node)
            } else {
                (y_node, x_node)
            };
            return Ok(Some(self.push(Node::Tensor(f, left, right))));
        }

        let mut members = self.take_list();
        members.extend(gamma);
        if members.len() > MAX_SPLIT {
            return Err(Reason::ContextTooWide(members.len()));
        }
        let mut left = self.take_set();
        let mut right = self.take_set();
        right.clone_from(gamma);
        let mut left_tally = self.take_tally();
        left_tally.add(&self.counts, a);
        let mut right_tally = self.take_tally();
        right_tally.add(&self.counts, b);
        for &m in &members {
            right_tally.add(&self.counts, m);
        }

        // The empty submask is the starting state of the enumeration.
        self.statistics.splits += 1;
        let mut result = if self.sides_pass(&left_tally, &right_tally) {
            self.premises(&left, &right, f, a, b)?
        } else {
            None
        };
        if result.is_none() {
            for flip in submasks(members.len()) {
                let m = members[flip.position as usize];
                self.statistics.splits += 1;
                if left.contains(m) {
                    left.remove(m);
                    right.insert(m);
                    left_tally.remove(&self.counts, m);
                    right_tally.add(&self.counts, m);
                } else {
                    right.remove(m);
                    left.insert(m);
                    right_tally.remove(&self.counts, m);
                    left_tally.add(&self.counts, m);
                }
                if !self.sides_pass(&left_tally, &right_tally) {
                    continue;
                }
                if let Some(node) = self.premises(&left, &right, f, a, b)? {
                    result = Some(node);
                    break;
                }
            }
        }
        self.give_list(members);
        self.give_set(left);
        self.give_set(right);
        self.give_tally(left_tally);
        self.give_tally(right_tally);
        Ok(result)
    }

    /// Whether both sides of a split pass the counts.
    fn sides_pass(&self, left: &Tally, right: &Tally) -> bool {
        left.balanced()
            && right.balanced()
            && (!self.rules.equation
                || (left.equation(self.rules.mix) && right.equation(self.rules.mix)))
    }

    /// Both premises of `F = A ⊗ B` for one split of the context, and the
    /// `⊗` node if both succeed.
    fn premises(&mut self, left: &OccSet, right: &OccSet, f: OccId, a: OccId, b: OccId) -> Search {
        let Some(l) = self.focus(left, a)? else {
            return Ok(None);
        };
        let Some(r) = self.focus(right, b)? else {
            return Ok(None);
        };
        Ok(Some(self.push(Node::Tensor(f, l, r))))
    }

    /// The Mix rule on a stable sequent no focus proves: a split into two
    /// non-empty provable parts, enumerated over the members after the
    /// first, which stays on the left, so that each unordered partition
    /// comes up once. In the multiplicative fragments only when the count
    /// equation admits a Mix.
    fn mix(&mut self, gamma: &OccSet, members: &[OccId], tally: &Tally) -> Search {
        if members.len() < 2 || (self.rules.equation && !tally.admits_mix()) {
            return Ok(None);
        }
        let rest = &members[1..];
        if rest.len() > MAX_SPLIT {
            return Err(Reason::ContextTooWide(members.len()));
        }
        let mut left = self.take_set();
        left.insert(members[0]);
        let mut right = self.take_set();
        right.clone_from(gamma);
        right.remove(members[0]);
        let mut left_tally = self.take_tally();
        left_tally.add(&self.counts, members[0]);
        let mut right_tally = self.take_tally();
        for &m in rest {
            right_tally.add(&self.counts, m);
        }
        let everything = (1u64 << rest.len()) - 1;
        // The first member alone is the starting state of the enumeration.
        self.statistics.splits += 1;
        let mut result = if self.sides_pass(&left_tally, &right_tally) {
            self.parts(&left, &right)?
        } else {
            None
        };
        for flip in submasks(rest.len()) {
            if result.is_some() {
                break;
            }
            let m = rest[flip.position as usize];
            self.statistics.splits += 1;
            if left.contains(m) {
                left.remove(m);
                right.insert(m);
                left_tally.remove(&self.counts, m);
                right_tally.add(&self.counts, m);
            } else {
                right.remove(m);
                left.insert(m);
                right_tally.remove(&self.counts, m);
                left_tally.add(&self.counts, m);
            }
            if flip.mask == everything || !self.sides_pass(&left_tally, &right_tally) {
                continue;
            }
            result = self.parts(&left, &right)?;
        }
        self.give_set(left);
        self.give_set(right);
        self.give_tally(left_tally);
        self.give_tally(right_tally);
        Ok(result)
    }

    /// Both parts of a Mix, and the Mix node if both are provable.
    fn parts(&mut self, left: &OccSet, right: &OccSet) -> Search {
        let Some(l) = self.prove(left)? else {
            return Ok(None);
        };
        let Some(r) = self.prove(right)? else {
            return Ok(None);
        };
        Ok(Some(self.push(Node::Mix(l, r))))
    }

    // Bookkeeping.

    /// Appends a node to the arena and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }

    /// Enters a nested engine call, unless the nesting is at its limit.
    fn enter(&mut self) -> Result<(), Reason> {
        if self.depth >= self.recursion_limit {
            return Err(Reason::RecursionLimit);
        }
        self.depth += 1;
        Ok(())
    }

    /// Leaves a nested engine call.
    fn leave(&mut self) {
        self.depth -= 1;
    }

    /// Takes an empty set from the pool.
    fn take_set(&mut self) -> OccSet {
        match self.sets.pop() {
            Some(mut set) => {
                set.clear();
                set
            }
            None => self.forest.empty_set(),
        }
    }

    /// Returns a set to the pool.
    fn give_set(&mut self, set: OccSet) {
        self.sets.push(set);
    }

    /// Takes an empty list from the pool.
    fn take_list(&mut self) -> Vec<OccId> {
        let mut list = self.lists.pop().unwrap_or_default();
        list.clear();
        list
    }

    /// Returns a list to the pool.
    fn give_list(&mut self, list: Vec<OccId>) {
        self.lists.push(list);
    }

    /// Takes an empty tally from the pool.
    fn take_tally(&mut self) -> Tally {
        match self.tallies.pop() {
            Some(mut tally) => {
                tally.clear();
                tally
            }
            None => self.counts.tally(),
        }
    }

    /// Returns a tally to the pool.
    fn give_tally(&mut self, tally: Tally) {
        self.tallies.push(tally);
    }
}

/// What a factor of a `⊗` forces on its side of the split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Forced {
    /// No split works: the factor is `0`.
    Nothing,
    /// The empty context: the factor is `1`.
    Empty,
    /// The dual literal alone: the factor is a positive literal.
    Dual,
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;

    /// Runs the engine on `input` under `mode` with `options`, checks the
    /// proof if there is one, and returns the verdict and the statistics.
    fn run(input: &str, mode: Mode, options: &Options) -> (Verdict, Statistics) {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let forest = Forest::new(&s).unwrap();
        let (verdict, statistics) = search(&forest, s.fragment(), mode, options, &mut || false);
        if let Verdict::Proved(proof) = &verdict {
            assert_eq!(proof.sequent(), &s);
            proof
                .check(mode)
                .unwrap_or_else(|e| panic!("{input:?}: the proof is wrong: {e}"));
        }
        (verdict, statistics)
    }

    /// Whether `input` is provable under `mode`, panicking on `Unknown`.
    fn provable(input: &str, mode: Mode) -> bool {
        match run(input, mode, &Options::default()).0 {
            Verdict::Proved(_) => true,
            Verdict::Unprovable => false,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// The classic small sequents, in `MLL` and `MALL`, get the verdicts
    /// the textbooks give them.
    #[test]
    fn classic_sequents() {
        let m = Mode::CLASSICAL;
        for (input, expected) in [
            ("|- ~a, a", true),
            ("a |- a", true),
            ("|- a, a", false),
            ("|-", false),
            ("a * b |- a * b", true),
            ("|- a * b, ~a par ~b", true),
            ("|- a * b, ~a, ~b", true),
            ("|- a par b, ~a, ~b", false),
            ("|- (a * b) par (~a * ~b)", false),
            ("a, a -o b, b -o c |- c", true),
            ("a -o b, b -o c |- a -o c", true),
            ("a -o b |- b -o a", false),
            ("|- a & b, ~a + ~b", true),
            ("|- a + b, ~a", true),
            ("|- a & b, ~a", false),
            ("(a & b) + (a & c) |- a & (b + c)", true),
            ("a & (b + c) |- (a & b) + (a & c)", false),
            ("a * (b + c) |- (a * b) + (a * c)", true),
            ("(a * b) + (a * c) |- a * (b + c)", true),
            ("a & b |- a", true),
            ("a |- a & b", false),
            ("a |- a + b", true),
            ("a + b |- a", false),
        ] {
            assert_eq!(provable(input, m), expected, "{input:?}");
        }
    }

    /// The units behave as the spec's checks say, and `⊤` closes any
    /// context while `0` closes none.
    #[test]
    fn units() {
        let m = Mode::CLASSICAL;
        for (input, expected) in [
            ("|- 1", true),
            ("|- bot, 1", true),
            ("|- 1 * 1", true),
            ("|- bot par 1", true),
            ("|- bot par bot", false),
            ("|- bot", false),
            ("|- 1, 1", false),
            ("|- 1, a, ~a", false),
            ("|- a * 1, ~a", true),
            ("|- top", true),
            ("|- top, a", true),
            ("|- top * a, ~a, b", true),
            ("|- a * top, b", false),
            ("|- a & top, ~a", true),
            ("|- 0", false),
            ("|- 0, top", true),
            ("|- 0 + a, ~a", true),
            ("|- 0 * a, ~a", false),
            ("0 |- a", true),
            ("a |- top", true),
        ] {
            assert_eq!(provable(input, m), expected, "{input:?}");
        }
    }

    /// Mix proves exactly what needs it.
    #[test]
    fn mix() {
        let mix = Mode::CLASSICAL.with_mix();
        for (input, without, with) in [
            ("|- a par b, ~a, ~b", false, true),
            ("a * b |- a par b", false, true),
            ("|- a, ~a, b, ~b", false, true),
            ("|- 1, a, ~a", false, true),
            ("|- 1, 1", false, true),
            ("|- a * b, ~a, ~b", true, true),
            ("|- a, b", false, false),
            ("|- a + b, ~a, c, ~c", false, true),
            ("|- (a & b) par c, ~a, ~b, ~c", false, false),
            ("|- a & b, ~a, ~b", false, false),
            ("|-", false, false),
        ] {
            assert_eq!(provable(input, Mode::CLASSICAL), without, "{input:?}");
            assert_eq!(provable(input, mix), with, "{input:?} with Mix");
        }
    }

    /// The memo makes no difference to the verdicts, and the counters say
    /// what the search did.
    #[test]
    fn memo_and_statistics() {
        let (verdict, statistics) = run("|- ~a, a", Mode::CLASSICAL, &Options::default());
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.nodes, 1);
        assert_eq!(statistics.memo_hits, 0);
        assert_eq!(statistics.memo_entries, 1);
        assert_eq!(statistics.splits, 0);

        // ⊢ ~a ⊕ ~b, ~a ⊕ ~b, a ⊗ b, (x ⅋ ~x) ⊗ (y ⊗ ~y) is unprovable,
        // and the two orders of choosing the `⊕` sides reach the stable
        // sequent ⊢ ~a, ~b, a ⊗ b, (x ⅋ ~x) ⊗ (y ⊗ ~y), whose refutation
        // takes a few stable sequents; the memo answers it the second time.
        let input = "|- ~a + ~b, ~a + ~b, a * b, (x par ~x) * (y * ~y)";
        let with_memo = run(input, Mode::CLASSICAL, &Options::default());
        let without = run(input, Mode::CLASSICAL, &Options::default().memo_limit(0));
        assert!(matches!(with_memo.0, Verdict::Unprovable));
        assert!(matches!(without.0, Verdict::Unprovable));
        assert!(with_memo.1.memo_hits >= 1);
        assert_eq!(without.1.memo_hits, 0);
        assert_eq!(without.1.memo_entries, 0);
        assert!(without.1.nodes > with_memo.1.nodes);
    }

    /// The same input and options give the same proof, node for node.
    #[test]
    fn deterministic() {
        let input = "|- (a * b) + (a * c), ~a par (~b & ~c), a, ~a";
        let first = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
        let second = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
        assert_eq!(
            first.0.proof().unwrap().nodes(),
            second.0.proof().unwrap().nodes()
        );
        assert_eq!(first.1, second.1);
    }

    /// The stop condition, the recursion limit and the split width each end
    /// the search with their reason.
    #[test]
    fn limits() {
        let input = "|- a * b, ~a, ~b";
        let s: Sequent = input.parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let mut calls = 0;
        let (verdict, _) = search(
            &forest,
            s.fragment(),
            Mode::CLASSICAL,
            &Options::default(),
            &mut || {
                calls += 1;
                calls >= 1
            },
        );
        assert!(matches!(verdict, Verdict::Unknown(Reason::Stopped)));

        let (verdict, _) = run(input, Mode::CLASSICAL, &Options::default().recursion_limit(2));
        assert!(matches!(verdict, Verdict::Unknown(Reason::RecursionLimit)));
        let (verdict, _) = run(input, Mode::CLASSICAL, &Options::default().recursion_limit(8));
        assert!(verdict.proof().is_some());

        // A free split over 126 formulas, in MALL so that the count
        // equation does not refute it first.
        let wide = format!(
            "|- ((a & a) par b) * (~a par ~b), {}",
            (0..63).map(|_| "~a, a").collect::<Vec<_>>().join(", ")
        );
        let (verdict, _) = run(&wide, Mode::CLASSICAL, &Options::default());
        assert!(matches!(
            verdict,
            Verdict::Unknown(Reason::ContextTooWide(126))
        ));
    }
}

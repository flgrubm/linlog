// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused sequent engine: backward search over dyadic sequents of
//! occurrence ids with a memo of stable sequents, as the `MALL-Seq` and
//! `MELL-Seq` specifications state it.
//!
//! A sequent inside the search is `⊢ Θ ; Γ`: the *unrestricted zone* `Θ`, a
//! set of the formulas that arrived under a `?` and may be copied any number
//! of times, and the *linear zone* `Γ`, a multiset of occurrences each of
//! which is used exactly once. The *asynchronous phase* decomposes the
//! negative formulas of a sequent without choice (`⅋` opens, `⊥` drops, `⊤`
//! closes, `&` branches into two premises with the same context, `?` moves
//! its subformula into `Θ`) until a *stable* sequent remains: `Γ` holds
//! positive formulas and negative literals only. `prove` decides a stable
//! sequent by looking it up in the memo, by the immediate tests (a `0`, an
//! unbalanced atom, a dual pair, a literal whose dual lies in `Θ`), and
//! otherwise by choosing a positive formula to *focus* on: one of `Γ`, which
//! it consumes, or one of `Θ`, which stays there (a *copy*, the one step
//! that can repeat, bounded per branch). `focus` applies the positive rules
//! along that formula: `⊕` picks a side, `⊗` splits `Γ`, `1` needs it empty,
//! `!` needs it empty and continues with its subformula, a positive literal
//! needs exactly its dual, in `Γ` or in `Θ`; and when the formula turns
//! negative it is *released* into the asynchronous phase. Every stable
//! sequent proved or refuted goes into the memo, so a `&` that duplicates
//! the context, or a split that revisits a part, never pays twice. With
//! Mix, a stable sequent no focus proves is split into two provable parts.
//! In affine mode a leaf weakens whatever is left over, and a sequent that
//! contains one below it on its branch is pruned.
//!
//! The copies are bounded per branch and the bound deepens iteratively:
//! `Unprovable` is answered only after a level that never hit its bound.
//! The counts of `counts.rs` prune: a stable sequent or a side of a split
//! whose intervals exclude zero for some atom is refuted without search,
//! and in the multiplicative fragments the count equation as well.

/// The linear zone as a multiset.
mod context;
/// The count invariants the engine prunes with.
mod counts;
/// The memo of stable sequents.
mod memo;

use self::context::Context;
use self::counts::{Counts, Tally};
use self::memo::{Entry, Failure, Key, Memo};
use super::{Options, Reason, Statistics, Verdict};
use crate::fragment::{Fragment, Mode};
use crate::occurrences::{Forest, OccId, OccSet, Polarity, submasks};
use crate::proofs::{Node, NodeId, Proof, Side};
use crate::sequents::Kind;

/// The most members a context can have for its splits to be enumerated.
const MAX_SPLIT: usize = 63;

/// The stack depth that stands for "no pruned sequent depends on an
/// ancestor".
const NO_DEPENDENCY: u32 = u32::MAX;

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
    let (result, nodes, statistics) =
        search_goal(forest, forest.roots(), fragment, mode, options, stop);
    let verdict = match result {
        Ok(Some(root)) => {
            let proof = Proof::new(forest.clone(), nodes, root)
                .expect("the engine pushes premises before conclusions");
            debug_assert_eq!(proof.check(mode), Ok(()), "the engine's proof");
            Verdict::Proved(Box::new(proof))
        }
        Ok(None) => Verdict::Unprovable,
        Err(reason) => Verdict::Unknown(reason),
    };
    (verdict, statistics)
}

/// Runs the focused engine from a goal: a multiset of occurrences of the
/// forest, the sequent's roots for a search of the sequent itself, or any
/// other with an empty unrestricted zone, as an interactive prover hands
/// over an open goal. Returns the node proving the goal (`None` when it is
/// unprovable, or the reason the search gave up), the arena the node lives
/// in, and the statistics.
pub(crate) fn search_goal(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Search, Vec<Node>, Statistics) {
    let mut engine = Engine::new(forest, fragment, mode, options, stop);
    let result = engine.run(goal);
    let statistics = engine.statistics();
    (result, engine.nodes, statistics)
}

/// The rules in force beyond the core ones, switched by fragment and mode.
#[derive(Clone, Copy, Debug)]
struct Rules {
    /// The Mix rule, tried last on a stable sequent.
    mix: bool,
    /// The `MLL` count equation as a prune: only without additives and
    /// exponentials anywhere in the problem, and never with weakening.
    equation: bool,
    /// The interval check as a prune: never with weakening, which can
    /// discard any imbalance.
    intervals: bool,
    /// The exponential rules: `?` into the unrestricted zone, copies from it
    /// under the bound, `!` in focus.
    exponentials: bool,
    /// Weakening: a leaf discards what is left over, and a stable sequent
    /// that contains an ancestor is pruned.
    affine: bool,
    /// The stack of the branch's stable sequents: for the loop check when
    /// copies can repeat a sequent, and for the affine prune.
    stack: bool,
}

/// The result of a search step: the node proving the sequent, `None` when
/// it is unprovable, or the reason the whole search stops.
pub(crate) type Search = Result<Option<NodeId>, Reason>;

/// The state of one run: the problem, the memo, the proof arena, the
/// counters, the bound bookkeeping, and pools of scratch buffers so that no
/// step allocates once the pools are warm.
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
    /// The most copies a branch may take at the last deepening level.
    copies: u32,
    /// Whether, since the flag was last cleared, some branch failed because
    /// its copy budget was spent: the level's answer is then not
    /// `Unprovable`.
    exhausted: bool,
    /// The shallowest stack depth of an ancestor that a pruned sequent
    /// below the current one contained (or repeated), or `NO_DEPENDENCY`:
    /// a failure that rests on such a prune is a fact about the branch,
    /// not about the sequent, and is not memoized.
    dependency: u32,
    /// The caller's stop condition.
    stop: &'a mut dyn FnMut() -> bool,
    /// The stable sequents of the current branch, the root end first; only
    /// the first `stack_len` are live, the rest are spare buffers.
    stack: Vec<Key>,
    /// How many entries of the stack are live.
    stack_len: usize,
    /// Spare occurrence sets of the forest's width.
    sets: Vec<OccSet>,
    /// Spare linear zones of the forest's width.
    contexts: Vec<Context>,
    /// Spare memo keys of the forest's width.
    keys: Vec<Key>,
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
        let exponentials = fragment.has_exponentials();
        let counts = Counts::new(forest);
        let rules = Rules {
            mix: mode.mix,
            equation: !mode.affine
                && !fragment.has_additives()
                && !fragment.has_additive_units()
                && !exponentials,
            intervals: !mode.affine && !counts.absorbs_from_copies(),
            exponentials,
            affine: mode.affine,
            stack: exponentials || mode.affine,
        };
        Self {
            forest,
            counts,
            rules,
            memo: Memo::new(options.memo_limit),
            nodes: Vec::new(),
            statistics: Statistics::default(),
            depth: 0,
            recursion_limit: options.recursion_limit,
            copies: options.copies,
            exhausted: false,
            dependency: NO_DEPENDENCY,
            stop,
            stack: Vec::new(),
            stack_len: 0,
            sets: Vec::new(),
            contexts: Vec::new(),
            keys: Vec::new(),
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

    /// Searches the goal with an empty unrestricted zone: the asynchronous
    /// phase on its formulas, once per copy bound from zero up to the
    /// configured one, until a level proves it or fails without ever
    /// spending its budget. Without exponentials there is one level.
    fn run(&mut self, goal: &[OccId]) -> Search {
        let levels = if self.rules.exponentials {
            self.copies
        } else {
            0
        };
        let theta = self.take_set();
        for budget in 0..=levels {
            self.exhausted = false;
            self.dependency = NO_DEPENDENCY;
            let mut gamma = self.take_context();
            let mut list = self.take_list();
            // A stack: the first formula is decomposed first.
            list.extend(goal.iter().rev());
            let result = self.asynchronous(&theta, &mut gamma, &mut list, budget);
            self.give_context(gamma);
            self.give_list(list);
            match result {
                Ok(None) if self.exhausted => {}
                decided => {
                    self.give_set(theta);
                    return decided;
                }
            }
        }
        self.give_set(theta);
        Err(Reason::CopyBound(levels))
    }

    // The phases.

    /// The asynchronous phase on `⊢ Θ ; Γ ⇑ L`: decomposes the negative
    /// formulas of the list until the sequent is stable, then proves it.
    /// The list is a stack; the order does not matter for completeness.
    fn asynchronous(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        budget: u32,
    ) -> Search {
        self.enter()?;
        let result = self.decompose(theta, gamma, list, budget);
        self.leave();
        result
    }

    /// The body of the asynchronous phase.
    fn decompose(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        budget: u32,
    ) -> Search {
        // The `⅋` and `⊥` rules applied, to wrap around the proof of what
        // remains; the first applied is the lowest.
        let mut applied = self.take_list();
        let result = loop {
            let Some(o) = list.pop() else {
                break self.prove(theta, gamma, budget);
            };
            match self.forest.kind(o) {
                Kind::Par => {
                    list.push(self.forest.right(o).unwrap());
                    list.push(self.forest.left(o).unwrap());
                    applied.push(o);
                }
                Kind::Bot => applied.push(o),
                Kind::Top => break Ok(Some(self.push(Node::Top(o)))),
                Kind::With => break self.with(theta, gamma, list, o, budget),
                Kind::Quest => break self.quest(theta, gamma, list, o, budget),
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
    /// copies of the state, the right one on the state itself; each
    /// premise keeps the whole copy budget.
    fn with(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        o: OccId,
        budget: u32,
    ) -> Search {
        let mut left_gamma = self.take_context();
        left_gamma.clone_from(gamma);
        let mut left_list = self.take_list();
        left_list.clone_from(list);
        left_list.push(self.forest.left(o).unwrap());
        let left = self.asynchronous(theta, &mut left_gamma, &mut left_list, budget);
        self.give_context(left_gamma);
        self.give_list(left_list);
        let Some(left) = left? else {
            return Ok(None);
        };
        list.push(self.forest.right(o).unwrap());
        let right = self.asynchronous(theta, gamma, list, budget)?;
        Ok(right.map(|right| self.push(Node::With(o, left, right))))
    }

    /// The `?` rule: the subformula joins the unrestricted zone, which is a
    /// set, so a formula already there changes nothing.
    fn quest(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        o: OccId,
        budget: u32,
    ) -> Search {
        let a = self.forest.left(o).unwrap();
        let result = if theta.contains(a) {
            self.asynchronous(theta, gamma, list, budget)
        } else {
            let mut larger = self.take_set();
            larger.clone_from(theta);
            larger.insert(a);
            let result = self.asynchronous(&larger, gamma, list, budget);
            self.give_set(larger);
            result
        };
        Ok(result?.map(|node| self.push(Node::Quest(o, node))))
    }

    /// `prove(Θ ; Γ)` for a stable `Γ`: the memo, the loop check and the
    /// affine prune, the immediate tests, then a focus on each positive
    /// formula in turn, first from `Γ`, then copied from `Θ`, then Mix.
    fn prove(&mut self, theta: &OccSet, gamma: &Context, budget: u32) -> Search {
        self.enter()?;
        let result = self.prove_stable(theta, gamma, budget);
        self.leave();
        result
    }

    /// The body of `prove`: the memo and the branch stack around the
    /// decision.
    fn prove_stable(&mut self, theta: &OccSet, gamma: &Context, budget: u32) -> Search {
        self.statistics.nodes += 1;
        if (self.stop)() {
            return Err(Reason::Stopped);
        }
        let mut key = self.take_key();
        key.assign(theta, gamma);
        // A proof or a complete failure from the memo settles it; a
        // failure cut by the budget waits for the loop check, which may
        // give the stronger answer that the branch is redundant.
        let entry = self.memo.get(&key, budget);
        match entry {
            Some(Entry::Proved(node)) => {
                self.give_key(key);
                return Ok(Some(node));
            }
            Some(Entry::Failed(Failure::Complete)) => {
                self.give_key(key);
                return Ok(None);
            }
            Some(Entry::Failed(Failure::Exhausted(_))) | None => {}
        }
        // A sequent that repeats an ancestor on its branch, or in affine
        // mode contains one, is pruned: a smallest proof of the ancestor
        // never passes through it. The failure this causes above is a fact
        // about the branch, so it is remembered as a dependency on the
        // ancestor's depth and keeps the sequents between them out of the
        // memo, until the ancestor itself is decided.
        if self.rules.stack {
            for depth in 0..self.stack_len {
                let ancestor = &self.stack[depth];
                let contains = if self.rules.affine {
                    ancestor.theta.is_subset(&key.theta) && key.gamma.includes(&ancestor.gamma)
                } else {
                    *ancestor == key
                };
                if contains {
                    self.dependency = self.dependency.min(depth as u32);
                    self.give_key(key);
                    return Ok(None);
                }
            }
        }
        if entry.is_some() {
            // Cut by the budget at this or a larger remaining budget.
            self.exhausted = true;
            self.give_key(key);
            return Ok(None);
        }
        if self.rules.stack {
            if self.stack_len < self.stack.len() {
                self.stack[self.stack_len].clone_from(&key);
            } else {
                self.stack.push(key.clone());
            }
            self.stack_len += 1;
        }
        let saved_exhausted = self.exhausted;
        let saved_dependency = self.dependency;
        self.exhausted = false;
        self.dependency = NO_DEPENDENCY;
        let result = self.decide(theta, gamma, budget);
        let own_depth = if self.rules.stack {
            self.stack_len -= 1;
            self.stack_len as u32
        } else {
            NO_DEPENDENCY
        };
        let exhausted = self.exhausted;
        // A dependency on this very sequent is settled with it: the pruned
        // descendants could only have proved what this sequent proves.
        let dependency = if self.dependency >= own_depth {
            NO_DEPENDENCY
        } else {
            self.dependency
        };
        self.exhausted = saved_exhausted || exhausted;
        self.dependency = saved_dependency.min(dependency);
        match result {
            Ok(Some(node)) => self.memo.insert(&key, Entry::Proved(node)),
            Ok(None) if dependency == NO_DEPENDENCY => {
                let failure = if exhausted {
                    Failure::Exhausted(budget)
                } else {
                    Failure::Complete
                };
                self.memo.insert(&key, Entry::Failed(failure));
            }
            _ => {}
        }
        self.give_key(key);
        result
    }

    /// Decides a stable sequent the memo does not know.
    fn decide(&mut self, theta: &OccSet, gamma: &Context, budget: u32) -> Search {
        let mut members = self.take_list();
        members.extend(gamma.iter());
        let mut tally = self.take_tally();
        let mut candidates = self.take_list();
        let mut copies = self.take_list();
        let result = self.decide_with(
            theta,
            gamma,
            &members,
            &mut tally,
            &mut candidates,
            &mut copies,
            budget,
        );
        self.give_list(members);
        self.give_tally(tally);
        self.give_list(candidates);
        self.give_list(copies);
        result
    }

    /// The body of `decide`, with its scratch buffers.
    #[allow(clippy::too_many_arguments)]
    fn decide_with(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &mut Tally,
        candidates: &mut Vec<OccId>,
        copies: &mut Vec<OccId>,
        budget: u32,
    ) -> Search {
        let affine = self.rules.affine;
        // One pass over the members: the counts, a `0`, and the positive
        // formulas worth focusing on. `1` and `!` only when alone, since
        // they need an empty context (any context, with weakening); a
        // literal never, since a focus on it succeeds only in the initial
        // cases tested below.
        let mut zero = false;
        for &o in members {
            tally.add(&self.counts, o);
            match self.forest.kind(o) {
                Kind::Zero => zero = true,
                Kind::Tensor | Kind::Plus => candidates.push(o),
                Kind::One | Kind::Bang if members.len() == 1 || affine => candidates.push(o),
                Kind::One | Kind::Bang | Kind::Var | Kind::DualVar => {}
                kind => unreachable!("{kind:?} in a stable sequent"),
            }
        }
        // A `0` has no rule, so only a `⊤` below some member can prove the
        // sequent: ⊢ 0, ⊤ ⊕ b is provable. (The spec calls a `0` in a
        // stable sequent fatal, which overlooks this.) With weakening the
        // `0` is discarded at a leaf like anything else.
        if zero && !affine && !tally.absorbs() {
            return Ok(None);
        }
        if let Some(node) = self.initial(theta, gamma, members, budget)? {
            return Ok(Some(node));
        }
        if self.rules.intervals && !tally.balanced() {
            return Ok(None);
        }
        if self.rules.equation && !tally.equation(self.rules.mix) {
            return Ok(None);
        }

        // Forced splits first, then `⊕`, then the free splits; by id within
        // a class, so that the run is deterministic.
        candidates.sort_by_key(|&o| (self.focus_class(o), o));
        let mut rest = self.take_context();
        for &f in candidates.iter() {
            rest.clone_from(gamma);
            rest.remove(f);
            if let Some(node) = self.focus(theta, &rest, f, budget)? {
                self.give_context(rest);
                return Ok(Some(node));
            }
        }
        self.give_context(rest);

        // Then the copies from `Θ`, a formula with an unconsumed copy in `Γ`
        // skipped (a second copy cannot help before the first is used),
        // those that can meet a literal of `Γ` first, then by id.
        if self.rules.exponentials {
            copies.extend(theta.iter().filter(|&a| !gamma.contains(a)));
            if budget == 0 {
                // A branch cut by the bound: the level cannot claim
                // completeness, unless nothing was there to copy.
                self.exhausted |= !copies.is_empty();
            } else {
                copies.sort_by_key(|&a| (!self.meets(a, members), a));
                for &a in copies.iter() {
                    if let Some(node) = self.focus(theta, gamma, a, budget - 1)? {
                        return Ok(Some(self.push(Node::Copy(a, node))));
                    }
                }
            }
        }

        if self.rules.mix {
            return self.mix(theta, gamma, members, tally, budget);
        }
        Ok(None)
    }

    /// The initial rules on a stable sequent: a dual pair in `Γ`, or a
    /// literal of `Γ` whose dual lies in `Θ`, which is copied and counts
    /// against the budget; exactly that in linear mode, with anything else
    /// in `Γ` weakened away in affine mode.
    fn initial(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        budget: u32,
    ) -> Search {
        let f = self.forest;
        let dual = |p: OccId, q: OccId| {
            f.is_literal(p) && f.atom(p) == f.atom(q) && f.sign(p) != f.sign(q)
        };
        let affine = self.rules.affine;
        for (i, &p) in members.iter().enumerate() {
            if !f.is_literal(p) || (!affine && members.len() > 2) {
                continue;
            }
            if let Some(&q) = members[i + 1..].iter().find(|&&q| dual(p, q))
                && (affine || members.len() == 2)
            {
                let ax = self.push(Node::Ax(p, q));
                if !affine {
                    return Ok(Some(ax));
                }
                let mut rest = self.take_context();
                rest.clone_from(gamma);
                rest.remove(p);
                rest.remove(q);
                let node = self.weakened(&rest, ax);
                self.give_context(rest);
                return Ok(Some(node));
            }
            if let Some(d) = theta.iter().find(|&d| dual(p, d))
                && (affine || members.len() == 1)
            {
                if budget == 0 {
                    self.exhausted = true;
                    return Ok(None);
                }
                let ax = self.push(Node::Ax(p, d));
                let copy = self.push(Node::Copy(d, ax));
                if !affine {
                    return Ok(Some(copy));
                }
                let mut rest = self.take_context();
                rest.clone_from(gamma);
                rest.remove(p);
                let node = self.weakened(&rest, copy);
                self.give_context(rest);
                return Ok(Some(node));
            }
        }
        Ok(None)
    }

    /// Whether a formula of `Θ` has a literal below it whose dual is a
    /// member: the copy heuristic's notion of a copy that can meet
    /// something.
    fn meets(&self, a: OccId, members: &[OccId]) -> bool {
        let f = self.forest;
        f.subtree(a).any(|l| {
            f.is_literal(l)
                && members
                    .iter()
                    .any(|&m| f.atom(m) == f.atom(l) && f.sign(m) != f.sign(l))
        })
    }

    /// Orders the focus candidates: `1` and `!`, then a `⊗` whose split is
    /// forced by a factor (a positive literal, `1`, `!`; or `0`, which
    /// fails at once), then `⊕`, then a `⊗` whose split must be enumerated.
    fn focus_class(&self, o: OccId) -> u8 {
        match self.forest.kind(o) {
            Kind::One | Kind::Bang => 0,
            Kind::Tensor => {
                let forced = self
                    .forest
                    .children(o)
                    .any(|c| self.forced_side(c).is_some());
                if forced { 1 } else { 3 }
            }
            _ => 2,
        }
    }

    /// `focus(Θ ; Γ ⇓ F)` with `F` taken out of `Γ` (or copied from `Θ`):
    /// the positive rules along `F`, and a release into the asynchronous
    /// phase once `F` is negative.
    fn focus(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Search {
        self.enter()?;
        let result = self.focus_on(theta, gamma, f, budget);
        self.leave();
        result
    }

    /// The body of `focus`.
    fn focus_on(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Search {
        match self.forest.kind(f) {
            Kind::Plus => {
                for (side, sub) in [
                    (Side::Left, self.forest.left(f).unwrap()),
                    (Side::Right, self.forest.right(f).unwrap()),
                ] {
                    if let Some(node) = self.focus(theta, gamma, sub, budget)? {
                        return Ok(Some(self.push(Node::Plus(f, side, node))));
                    }
                }
                Ok(None)
            }
            Kind::Tensor => self.split(theta, gamma, f, budget),
            Kind::One => Ok(self.leftover(gamma).then(|| {
                let one = self.push(Node::One(f));
                self.weakened(gamma, one)
            })),
            Kind::Bang => {
                // Promotion: the subformula is released into an empty
                // linear zone, and what was there is weakened below.
                if !self.leftover(gamma) {
                    return Ok(None);
                }
                let mut released = self.take_context();
                let mut list = self.take_list();
                list.push(self.forest.left(f).unwrap());
                let result = self.asynchronous(theta, &mut released, &mut list, budget);
                self.give_context(released);
                self.give_list(list);
                Ok(result?.map(|node| {
                    let bang = self.push(Node::Bang(f, node));
                    self.weakened(gamma, bang)
                }))
            }
            Kind::Zero => Ok(None),
            Kind::Var | Kind::DualVar if self.forest.polarity(f) == Polarity::Positive => {
                // The initial rules: the context is the dual literal, or
                // nothing and the dual lies in `Θ`.
                let mut members = self.take_list();
                members.push(f);
                members.extend(gamma.iter());
                let mut whole = self.take_context();
                whole.clone_from(gamma);
                whole.insert(f);
                let result = self.initial(theta, &whole, &members, budget);
                self.give_list(members);
                self.give_context(whole);
                result
            }
            _ => {
                // Negative: release.
                let mut released = self.take_context();
                released.clone_from(gamma);
                let mut list = self.take_list();
                list.push(f);
                let result = self.asynchronous(theta, &mut released, &mut list, budget);
                self.give_context(released);
                self.give_list(list);
                result
            }
        }
    }

    /// Whether a rule that needs an empty linear zone can be applied with
    /// this one: it is empty, or weakening will discard it.
    fn leftover(&self, gamma: &Context) -> bool {
        gamma.is_empty() || self.rules.affine
    }

    /// Weakens every member of `gamma` below `node`, one `Weaken` per copy;
    /// nothing when it is empty.
    fn weakened(&mut self, gamma: &Context, mut node: NodeId) -> NodeId {
        debug_assert!(
            gamma.is_empty() || self.rules.affine,
            "weakening needs affine mode"
        );
        for o in gamma.iter() {
            node = self.push(Node::Weaken(o, node));
        }
        node
    }

    // The context splits.

    /// What a factor of a `⊗` forces on its side of the split: nothing at
    /// all for `0`, the empty context for `1` and `!`, and the dual literal
    /// alone for a positive literal. With weakening nothing but `0` forces
    /// anything, since every leaf takes any context.
    fn forced_side(&self, factor: OccId) -> Option<Forced> {
        match self.forest.kind(factor) {
            Kind::Zero => Some(Forced::Nothing),
            _ if self.rules.affine => None,
            Kind::One | Kind::Bang => Some(Forced::Empty),
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
    fn split(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Search {
        let (a, b) = (self.forest.left(f).unwrap(), self.forest.right(f).unwrap());
        for (x, y, x_is_left) in [(a, b, true), (b, a, false)] {
            let Some(forced) = self.forced_side(x) else {
                continue;
            };
            self.statistics.splits += 1;
            if forced == Forced::Nothing {
                return Ok(None);
            }
            let mut side = self.take_context();
            let mut rest = self.take_context();
            rest.clone_from(gamma);
            if forced == Forced::Dual {
                // The dual in `Γ` when there is one; otherwise the side
                // stays empty and the initial rule looks in `Θ`. Taking the
                // copy from `Γ` first loses nothing: the copies are the same
                // formula, so a proof that leaves this one for elsewhere
                // and copies from `Θ` here is a proof with the roles
                // swapped.
                let dual = gamma.iter().find(|&m| {
                    self.forest.atom(m) == self.forest.atom(x)
                        && self.forest.sign(m) != self.forest.sign(x)
                });
                if let Some(dual) = dual {
                    side.insert(dual);
                    rest.remove(dual);
                }
            }
            let mut nodes: Result<Option<(NodeId, NodeId)>, Reason> = Ok(None);
            {
                match self.focus(theta, &side, x, budget) {
                    Ok(Some(x_node)) => {
                        nodes = self
                            .focus(theta, &rest, y, budget)
                            .map(|y_node| y_node.map(|y_node| (x_node, y_node)));
                    }
                    Ok(None) => {}
                    Err(reason) => nodes = Err(reason),
                }
            }
            self.give_context(side);
            self.give_context(rest);
            return Ok(nodes?.map(|(x_node, y_node)| {
                let (left, right) = if x_is_left {
                    (x_node, y_node)
                } else {
                    (y_node, x_node)
                };
                self.push(Node::Tensor(f, left, right))
            }));
        }

        let mut members = self.take_list();
        members.extend(gamma.iter());
        if members.len() > MAX_SPLIT {
            return Err(Reason::ContextTooWide(members.len()));
        }
        let mut left = self.take_context();
        let mut right = self.take_context();
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
            self.premises(theta, &left, &right, f, a, b, budget)?
        } else {
            None
        };
        if result.is_none() {
            for flip in submasks(members.len()) {
                let m = members[flip.position as usize];
                self.statistics.splits += 1;
                if flip.mask >> flip.position & 1 == 1 {
                    right.remove(m);
                    left.insert(m);
                    right_tally.remove(&self.counts, m);
                    left_tally.add(&self.counts, m);
                } else {
                    left.remove(m);
                    right.insert(m);
                    left_tally.remove(&self.counts, m);
                    right_tally.add(&self.counts, m);
                }
                if !self.sides_pass(&left_tally, &right_tally) {
                    continue;
                }
                if let Some(node) = self.premises(theta, &left, &right, f, a, b, budget)? {
                    result = Some(node);
                    break;
                }
            }
        }
        self.give_list(members);
        self.give_context(left);
        self.give_context(right);
        self.give_tally(left_tally);
        self.give_tally(right_tally);
        Ok(result)
    }

    /// Whether both sides of a split pass the counts.
    fn sides_pass(&self, left: &Tally, right: &Tally) -> bool {
        (!self.rules.intervals || (left.balanced() && right.balanced()))
            && (!self.rules.equation
                || (left.equation(self.rules.mix) && right.equation(self.rules.mix)))
    }

    /// Both premises of `F = A ⊗ B` for one split of the context, and the
    /// `⊗` node if both succeed.
    #[allow(clippy::too_many_arguments)]
    fn premises(
        &mut self,
        theta: &OccSet,
        left: &Context,
        right: &Context,
        f: OccId,
        a: OccId,
        b: OccId,
        budget: u32,
    ) -> Search {
        let Some(l) = self.focus(theta, left, a, budget)? else {
            return Ok(None);
        };
        let Some(r) = self.focus(theta, right, b, budget)? else {
            return Ok(None);
        };
        Ok(Some(self.push(Node::Tensor(f, l, r))))
    }

    /// The Mix rule on a stable sequent no focus proves: a split into two
    /// non-empty provable parts, enumerated over the members after the
    /// first, which stays on the left, so that each unordered partition
    /// comes up once. In the multiplicative fragments only when the count
    /// equation admits a Mix.
    fn mix(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &Tally,
        budget: u32,
    ) -> Search {
        if members.len() < 2 || (self.rules.equation && !tally.admits_mix()) {
            return Ok(None);
        }
        let rest = &members[1..];
        if rest.len() > MAX_SPLIT {
            return Err(Reason::ContextTooWide(members.len()));
        }
        let mut left = self.take_context();
        left.insert(members[0]);
        let mut right = self.take_context();
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
            self.parts(theta, &left, &right, budget)?
        } else {
            None
        };
        for flip in submasks(rest.len()) {
            if result.is_some() {
                break;
            }
            let m = rest[flip.position as usize];
            self.statistics.splits += 1;
            if flip.mask >> flip.position & 1 == 1 {
                right.remove(m);
                left.insert(m);
                right_tally.remove(&self.counts, m);
                left_tally.add(&self.counts, m);
            } else {
                left.remove(m);
                right.insert(m);
                left_tally.remove(&self.counts, m);
                right_tally.add(&self.counts, m);
            }
            if flip.mask == everything || !self.sides_pass(&left_tally, &right_tally) {
                continue;
            }
            result = self.parts(theta, &left, &right, budget)?;
        }
        self.give_context(left);
        self.give_context(right);
        self.give_tally(left_tally);
        self.give_tally(right_tally);
        Ok(result)
    }

    /// Both parts of a Mix, and the Mix node if both are provable.
    fn parts(&mut self, theta: &OccSet, left: &Context, right: &Context, budget: u32) -> Search {
        let Some(l) = self.prove(theta, left, budget)? else {
            return Ok(None);
        };
        let Some(r) = self.prove(theta, right, budget)? else {
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

    /// Takes an empty linear zone from the pool.
    fn take_context(&mut self) -> Context {
        match self.contexts.pop() {
            Some(mut context) => {
                context.clear();
                context
            }
            None => Context::empty(self.forest.len()),
        }
    }

    /// Returns a linear zone to the pool.
    fn give_context(&mut self, context: Context) {
        self.contexts.push(context);
    }

    /// Takes a memo key from the pool, with any contents.
    fn take_key(&mut self) -> Key {
        self.keys.pop().unwrap_or_else(|| Key {
            theta: self.forest.empty_set(),
            gamma: Context::empty(self.forest.len()),
        })
    }

    /// Returns a memo key to the pool.
    fn give_key(&mut self, key: Key) {
        self.keys.push(key);
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
    /// The empty context: the factor is `1` or `!`.
    Empty,
    /// The dual literal alone: the factor is a positive literal.
    Dual,
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;
    use crate::search::generate::{self, Rng, Rules};

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
            ("|- 0, top + b", true),
            ("|- 0 * a, ~a, top + b", true),
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

        let (verdict, _) = run(
            input,
            Mode::CLASSICAL,
            &Options::default().recursion_limit(2),
        );
        assert!(matches!(verdict, Verdict::Unknown(Reason::RecursionLimit)));
        let (verdict, _) = run(
            input,
            Mode::CLASSICAL,
            &Options::default().recursion_limit(8),
        );
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

    /// The classic MELL sequents: dereliction, weakening, contraction with
    /// and without promotion, and what is unprovable, all within the
    /// default copy bound.
    #[test]
    fn classic_exponentials() {
        let m = Mode::CLASSICAL;
        for (input, expected) in [
            ("!a |- a", true),
            ("!a |- 1", true),
            ("!a |- a * a", true),
            ("!a |- !a * !a", true),
            ("|- !(a -o a)", true),
            ("!a, !(a -o b) |- !b", true),
            ("!a, !a |- a", true),
            ("!a |- !!a", true),
            ("!!a |- !a", true),
            ("!a * !b |- !(a * b)", true),
            ("!a, !(a -o b), !(b -o c) |- !c", true),
            ("|- ?a, ?~a", true),
            ("!a, !~a |- 1", true),
            ("?a |- ?a par ?a", true),
            ("|- !1", true),
            ("a |- !a", false),
            ("?a |- a", false),
            ("!a |- b", false),
            ("!a, !(a -o b) |- c", false),
            ("|- ?a", false),
            ("|- ?a, ?b, ~a", true),
            ("|- ?a, b, ~a", false),
        ] {
            assert_eq!(provable(input, m), expected, "{input:?}");
        }
        // Unprovable, but every copy grows the context, so no level of the
        // bound finishes: the honest answer is unknown.
        for input in [
            "!(a * b) |- !a * !b",
            "!(a + b) |- !a + !b",
            "!(a * b) |- a",
        ] {
            assert!(
                matches!(
                    run(input, m, &Options::default()).0,
                    Verdict::Unknown(Reason::CopyBound(3))
                ),
                "{input:?}"
            );
        }
    }

    /// Full LL: the additive and the dyadic rules compose, with and
    /// without Mix.
    #[test]
    fn full_ll() {
        for (input, expected) in [
            ("!(a & b) |- !a * !b", true),
            ("!a * !b |- !(a & b)", true),
            ("?a par ?b |- ?(a + b)", true),
            ("?(a + b) |- ?a par ?b", true),
            ("!(a & b) |- !a & !b", true),
            ("!(a & b) |- !(a * b)", true),
            ("!a & !b |- !(a & b)", false),
            ("!a, !(a -o b & c) |- b * c", true),
            ("!(a -o top) |- b", false),
            ("!(a -o top), a |- b", false),
            ("!(a -o top), a |- top", true),
            ("!(0 -o a) |- a", false),
            ("!a, !(a -o 0) |- b", true),
        ] {
            assert_eq!(provable(input, Mode::CLASSICAL), expected, "{input:?}");
        }
        // The opponent chooses the side of the `&`: unprovable, and the
        // context grows with every copy, so undecided within the bound.
        assert!(matches!(
            run(
                "!a, !(a -o b + c) |- b * c",
                Mode::CLASSICAL,
                &Options::default()
            )
            .0,
            Verdict::Unknown(Reason::CopyBound(3))
        ));
        let mix = Mode::CLASSICAL.with_mix();
        for (input, without, with) in [
            ("!a |- a, 1", false, true),
            ("!a, !b |- a * b, a", false, true),
            ("!a |- a, b", false, false),
        ] {
            assert_eq!(provable(input, Mode::CLASSICAL), without, "{input:?}");
            assert_eq!(provable(input, mix), with, "{input:?} with Mix");
        }
    }

    /// Affine mode proves what needs weakening and nothing more, in every
    /// fragment, and decides what linear mode cannot.
    #[test]
    fn affine() {
        let affine = Mode::CLASSICAL.affine();
        for (input, linear, weakened) in [
            ("a |- 1", false, true),
            ("a, b |- a", false, true),
            ("a * b |- a", false, true),
            ("|- a, ~a, b", false, true),
            ("a & b |- 1", false, true),
            ("a |- 0 + 1", false, true),
            ("|- 0, a, ~a", false, true),
            ("!a, b |- a", false, true),
            ("a |- !1", false, true),
            ("a |- !b", false, false),
            ("|- a, b", false, false),
            ("|- 0", false, false),
            ("a * b |- a * b", true, true),
            ("a |- a * a", false, false),
            ("!a |- a * a", true, true),
            ("a & b |- a + b", true, true),
            ("(a * b) par c |- a, b, c", false, true),
            ("a -o b |- b", false, false),
        ] {
            assert_eq!(provable(input, Mode::CLASSICAL), linear, "{input:?}");
            assert_eq!(provable(input, affine), weakened, "{input:?} affinely");
        }
        // The context grows with every copy, so linear mode gives up at the
        // bound; affine mode prunes the grown context against its ancestor
        // and decides.
        let growing = "!(a -o a * a), a |- ?b";
        assert!(matches!(
            run(growing, Mode::CLASSICAL, &Options::default()).0,
            Verdict::Unknown(Reason::CopyBound(3))
        ));
        assert!(!provable(growing, affine));
        // Weakening goes below a promotion, never above it.
        let (verdict, _) = run("b |- !(a -o a)", affine, &Options::default());
        let proof = verdict.proof().unwrap();
        let Node::Weaken(_, below) = proof.node(proof.root()) else {
            panic!("the leftover is weakened at the root");
        };
        assert!(matches!(proof.node(below), Node::Bang(..)));
    }

    /// The copy bound: a level that hit its bound never answers
    /// `Unprovable`, a level that did not answers it, a failure recorded at
    /// a smaller remaining budget is not reused at a larger one, and the
    /// bound is per branch.
    #[test]
    fn copy_bound() {
        let m = Mode::CLASSICAL;
        let with = |copies| Options::default().copies(copies);
        // ⊢ ?~a, a needs one copy: bound 0 is hit, bound 1 proves.
        assert!(matches!(
            run("!a |- a", m, &with(0)).0,
            Verdict::Unknown(Reason::CopyBound(0))
        ));
        assert!(run("!a |- a", m, &with(1)).0.proof().is_some());
        // ⊢ ?~a, a ⊗ a: the stable sequent ⊢ ~a ; a fails at level 0 for
        // lack of budget and must be searched again at level 1.
        assert!(run("!a |- a * a", m, &with(1)).0.proof().is_some());
        // The clause and one `~a` are copied on every branch to an `a`:
        // bound 2, however many branches there are; and the two branches
        // of ⊢ ?~a, (a ⊗ a) ⊗ a take one copy each.
        assert!(matches!(
            run("!a, !(a -o a -o a -o b) |- b", m, &with(1)).0,
            Verdict::Unknown(Reason::CopyBound(1))
        ));
        assert!(
            run("!a, !(a -o a -o a -o b) |- b", m, &with(2))
                .0
                .proof()
                .is_some()
        );
        assert!(run("!a |- (a * a) * a", m, &with(1)).0.proof().is_some());
        // Unprovable, decided at a level that never hit the bound: after
        // one copy of ~a nothing is left to copy. (⊢ ?~a, b is refuted by
        // the balance of b before any copy.)
        assert!(matches!(run("!a |- b", m, &with(0)).0, Verdict::Unprovable));
        assert!(matches!(
            run("!a |- ?b", m, &with(1)).0,
            Verdict::Unknown(Reason::CopyBound(1))
        ));
        assert!(matches!(
            run("!a |- ?b", m, &with(2)).0,
            Verdict::Unprovable
        ));
        // The loop check decides a sequent whose copies repeat a stable
        // sequent, without a bound to hit.
        assert!(matches!(
            run("!(a -o a), a |- b", m, &with(1)).0,
            Verdict::Unprovable
        ));
        // A growing context is never decided within a bound.
        for copies in [0, 2, 5] {
            assert!(matches!(
                run("!(a -o a * a), a |- ?b", m, &with(copies)).0,
                Verdict::Unknown(Reason::CopyBound(c)) if c == copies
            ));
        }
        assert_eq!(
            Reason::CopyBound(3).to_string(),
            "the copy bound of 3 was reached"
        );
    }

    /// The memo makes no difference to a MELL verdict, with and without
    /// the copy bound binding, and the entries survive across levels.
    #[test]
    fn memo_across_levels() {
        for input in [
            "!a, !(a -o b), !(b -o c) |- c * c * c",
            "!(a -o a * a), a |- b",
            "!a, !(a -o b) |- c",
            "!(a & b) |- !a * !b",
        ] {
            for copies in [1, 3] {
                let options = Options::default().copies(copies);
                let (with, stats) = run(input, Mode::CLASSICAL, &options);
                let (without, _) = run(input, Mode::CLASSICAL, &options.clone().memo_limit(0));
                assert_eq!(
                    std::mem::discriminant(&with),
                    std::mem::discriminant(&without),
                    "{input:?} at {copies} copies: {with:?} with the memo, {without:?} without"
                );
                if copies == 3 && input.contains("c * c * c") {
                    assert!(with.proof().is_some());
                    assert!(stats.memo_hits > 0, "{input:?}: the memo was used");
                }
            }
        }
    }

    /// The dyadic proofs the engine finds unfold into the standard
    /// derivations, with dereliction, contraction, weakening and promotion
    /// where they belong.
    #[test]
    fn exponential_derivations() {
        let render = |input: &str, mode: Mode| {
            let (verdict, _) = run(input, mode, &Options::default());
            verdict.proof().unwrap().derivation().unwrap().to_string()
        };
        assert_eq!(
            render("!A |- A", Mode::CLASSICAL),
            ["─────── ax", "⊢ ~A, A", "──────── ?d", "⊢ ?~A, A"].join("\n")
        );
        assert_eq!(
            render("!A |- 1", Mode::CLASSICAL),
            ["  ─── 1", "  ⊢ 1", "──────── ?w", "⊢ ?~A, 1"].join("\n")
        );
        assert_eq!(
            render("!A |- A * A", Mode::CLASSICAL),
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
        assert_eq!(
            render("!A, !(A -o B) |- !B", Mode::CLASSICAL),
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
        assert_eq!(
            render("A, B |- A", Mode::CLASSICAL.affine()),
            ["  ─────── ax", "  ⊢ ~A, A", "─────────── wk", "⊢ ~A, ~B, A",].join("\n")
        );
    }

    /// The mode a generated sequent is proved in.
    fn mode_for(rules: Rules) -> Mode {
        if rules.mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        }
    }

    /// The verdict of `input` under `mode` with `options`, as a three-way
    /// value: provable, unprovable, or undecided within the copy bound.
    fn decided(input: &str, mode: Mode, options: &Options) -> Option<bool> {
        match run(input, mode, options).0 {
            Verdict::Proved(_) => Some(true),
            Verdict::Unprovable => Some(false),
            Verdict::Unknown(Reason::CopyBound(_)) => None,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// Proves `samples` generated sequents of up to `budget` rules per rule
    /// set, each within the copies its proof took, and decides one mutant
    /// of each with and without the memo. Returns how many sequents and
    /// mutants were decided, how many mutants were provable, how many were
    /// undecided within the copy bound, and the most stable sequents one
    /// search visited.
    fn generated(samples: u64, budget: usize) -> (u64, u64, u64, u64, u64) {
        let (mut sequents, mut mutants, mut provable_mutants, mut undecided, mut most_nodes) =
            (0, 0, 0, 0, 0);
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mode = mode_for(rules);
            let mut rng = Rng::new(i as u64);
            for _ in 0..samples {
                let budget = 2 + rng.below(budget - 1);
                let generate::Provable {
                    mut formulas,
                    copies,
                } = generate::provable(&mut rng, rules, 3, budget);
                let text = generate::sequent(&formulas);
                let options = Options::default().copies(copies);
                let (verdict, statistics) = run(&text, mode, &options);
                assert!(
                    verdict.proof().is_some(),
                    "{text:?} is provable in {mode} mode within {copies} copies, but the engine \
                     says {verdict:?}"
                );
                sequents += 1;
                most_nodes = most_nodes.max(statistics.nodes);
                if generate::mutate(&mut rng, &mut formulas, 3) {
                    let text = generate::sequent(&formulas);
                    let with_memo = decided(&text, mode, &Options::default());
                    let without = decided(&text, mode, &Options::default().memo_limit(0));
                    // The memo never contradicts the memo-free search; it
                    // may decide where the other is undecided within the
                    // bound and the other way round, since an entry cut by
                    // the budget is a fact about the sequent alone while
                    // the loop check is a fact about the branch.
                    assert!(
                        with_memo.is_none() || without.is_none() || with_memo == without,
                        "{text:?} in {mode} mode: {with_memo:?} with the memo, {without:?} without"
                    );
                    mutants += 1;
                    provable_mutants += u64::from(with_memo == Some(true));
                    undecided += u64::from(with_memo.is_none());
                }
            }
        }
        (sequents, mutants, provable_mutants, undecided, most_nodes)
    }

    /// Every generated provable sequent is proved with a checked proof
    /// within the copies its proof took, in every fragment with and
    /// without Mix, and its mutant is decided the same way with and
    /// without the memo.
    #[test]
    fn generated_sequents() {
        let (sequents, mutants, _, undecided, _) = generated(40, 10);
        assert_eq!(sequents, 640);
        assert!(mutants > 400, "{mutants} mutants");
        assert!(
            undecided < mutants / 4,
            "{undecided} of {mutants} mutants undecided"
        );
    }

    /// The same on a larger sample of larger proofs; run it in release
    /// mode and read the numbers it prints.
    #[test]
    #[ignore = "a larger sample; run with --release -- --ignored --nocapture"]
    fn generated_large_sample() {
        let (sequents, mutants, provable_mutants, undecided, most_nodes) = generated(500, 24);
        println!(
            "{sequents} generated sequents proved, {mutants} mutants decided consistently \
             ({provable_mutants} of them provable, {undecided} undecided within the copy \
             bound), at most {most_nodes} stable sequents per search"
        );
    }

    /// Encodes a 3-Partition instance as a linear Horn program in the style
    /// of Kanovich's encodings: bin `j` offers `size` units `bj` and three
    /// slots `tj`; item `i` is a `&` over the bins of a clause taking its
    /// units and a slot from that bin and producing `di`; the goal is the
    /// tensor of every `di`. Every resource must be used exactly once, so
    /// the sequent is provable if and only if the items split into triples
    /// of sum `size`, one per bin.
    fn three_partition(items: &[u32], bins: usize, size: u32) -> String {
        let mut hypotheses = Vec::new();
        for j in 1..=bins {
            hypotheses.extend((0..size).map(|_| format!("b{j}")));
            hypotheses.extend((0..3).map(|_| format!("t{j}")));
        }
        for (i, &a) in items.iter().enumerate() {
            let clauses: Vec<String> = (1..=bins)
                .map(|j| {
                    let units = vec![format!("b{j}"); a as usize].join(" * ");
                    format!("({units} * t{j} -o d{i})")
                })
                .collect();
            hypotheses.push(clauses.join(" & "));
        }
        let goal: Vec<String> = (0..items.len()).map(|i| format!("d{i}")).collect();
        format!("{} |- {}", hypotheses.join(", "), goal.join(" * "))
    }

    /// A 3-Partition instance with a solution is proved: the first bin
    /// choices work out, so the search is short.
    #[test]
    fn three_partition_solved() {
        let yes = three_partition(&[1, 2, 3, 1, 2, 3], 2, 6);
        assert!(provable(&yes, Mode::CLASSICAL), "{yes}");
    }

    /// A 3-Partition instance without a solution is refuted. The atom bias
    /// makes the clause bodies' literals negative, so every clause's `⊗`
    /// split is enumerated rather than forced, and the refutation walks
    /// billions of splits: about a minute in release mode.
    #[test]
    #[ignore = "about a minute in release mode; run with --release -- --ignored"]
    fn three_partition_refuted() {
        let no = three_partition(&[1, 1, 1, 3, 1, 1], 2, 4);
        assert!(!provable(&no, Mode::CLASSICAL), "{no}");
    }
}

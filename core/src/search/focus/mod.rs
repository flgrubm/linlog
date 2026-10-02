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
//! In affine mode a leaf weakens whatever is left over.
//!
//! The copies are bounded per branch and the bound deepens iteratively:
//! `Unprovable` is answered only after a level that never hit its bound.
//!
//! The same engine searches intuitionistic linear logic two-sided, given
//! the sequent's intuitionistic reading: every rule of the two-sided
//! focused calculus is a rule of this one on the one-sided sequent, and
//! the one-succedent condition holds on every branch by itself except at
//! the split of a hypothesis `A ⊸ B` (a `⊗` in input position), where the
//! goal must stay on the consequent's side; that is the one place the
//! reading is consulted.
//! The counts of `counts.rs` prune: a stable sequent or a side of a split
//! whose intervals exclude zero for some atom is refuted without search,
//! and in the multiplicative fragments the count equation as well.

/// Interchangeable occurrences.
mod classes;
/// The linear zone as a multiset.
mod context;
/// The count invariants the engine prunes with.
mod counts;
/// The memo of stable sequents.
mod memo;
#[cfg(feature = "parallel")]
pub(crate) mod parallel;

use self::classes::Classes;
use self::context::Context;
use self::counts::{Counts, Split, Tally};
use self::memo::{Entry, Failure, Key, Memo, Table};
use super::{Options, Reason, Statistics, Stop, Verdict};
use crate::fragment::{Fragment, Mode};
use crate::occurrences::{Bias, Forest, OccId, OccSet, Position, Reading};
use crate::proofs::{Node, NodeId, Proof, Side};
use crate::sequents::Kind;
use std::hash::BuildHasher as _;
use std::sync::Mutex;

/// The stack depth that stands for "no pruned sequent depends on an
/// ancestor".
const NO_DEPENDENCY: u32 = u32::MAX;

/// How many steps of its searches for the splits of a `⊗` or a Mix the
/// engine takes between two polls of the stop condition.
const SPLITS_PER_POLL: u64 = 4096;

/// Runs the focused engine on the forest of a sequent of `fragment` under
/// `mode`, two-sided when the sequent's intuitionistic reading is given,
/// polling `stop` at every stable sequent, and returns the verdict with the
/// statistics of the run.
pub(crate) fn search(
    forest: &Forest,
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Verdict, Statistics) {
    let (result, nodes, statistics) = search_goal(
        forest,
        forest.roots(),
        fragment,
        mode,
        reading,
        options,
        stop,
    );
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
/// over an open goal (with exactly one occurrence in output position when
/// a reading is given). Returns the node proving the goal (`None` when it
/// is unprovable, or the reason the search gave up), the arena the node
/// lives in, and the statistics.
#[allow(clippy::too_many_arguments)]
pub(crate) fn search_goal(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Search, Vec<Node>, Statistics) {
    let counts = Counts::new(forest, bias(options, mode));
    let classes = Classes::new(forest, reading);
    let rules = Rules::new(fragment, mode, &counts);
    let mut engine = Engine::new(
        forest,
        rules,
        reading,
        (&counts, &classes),
        options,
        Stop::Closure(stop),
        Table::Own(Memo::new(options.memo_limit)),
        Arena::new(Kept::Own(Vec::new())),
    );
    let result = engine
        .run(goal)
        .map(|root| root.map(|root| engine.nodes.keep(0, root)));
    let statistics = engine.statistics();
    let nodes = match engine.nodes.kept {
        Kept::Own(nodes) => nodes,
        Kept::Shared(_) => unreachable!("a sequential search owns its arena"),
    };
    (result, nodes, statistics)
}

/// The bias a search runs under: the one the options name, where `Auto`
/// under weakening is the rarer literal, since nothing forces a split
/// there and the factors have nothing to say.
pub(crate) fn bias(options: &Options, mode: Mode) -> Bias {
    if options.bias == Bias::Auto && mode.affine {
        Bias::Rarer
    } else {
        options.bias
    }
}

/// Whether a split of a goal into the two premises of a `⊗`, each given
/// with its subformula of the `⊗`, or of a Mix passes the count prunes the
/// engine applies to every split: the interval check per atom and, in the
/// multiplicative fragments, the count equation. A split that fails cannot
/// close; one that passes may still fail. `fragment` is the goal's.
pub(crate) fn split_passes(
    forest: &Forest,
    fragment: Fragment,
    mode: Mode,
    left: &[OccId],
    right: &[OccId],
) -> bool {
    let counts = Counts::new(forest, Bias::Auto);
    let rules = Rules::new(fragment, mode, &counts);
    let mut split = counts.split();
    for (members, side) in [(left, Side::Left), (right, Side::Right)] {
        for &m in members {
            split.place(&counts, m, side);
        }
    }
    split.feasible(rules.intervals, rules.equation, rules.mix)
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
    /// Weakening: a leaf discards what is left over.
    affine: bool,
    /// The stack of the branch's stable sequents, for the loop check when
    /// copies can repeat a sequent.
    stack: bool,
}

impl Rules {
    /// The rules for a fragment and a mode: the count equation only in the
    /// multiplicative fragments without weakening, the interval check
    /// unless weakening or a `⊤` under an exponential defeats it.
    fn new(fragment: Fragment, mode: Mode, counts: &Counts) -> Self {
        let exponentials = fragment.has_exponentials();
        Self {
            mix: mode.mix,
            equation: !mode.affine
                && !fragment.has_additives()
                && !fragment.has_additive_units()
                && !exponentials,
            intervals: !mode.affine && !counts.absorbs_from_copies(),
            exponentials,
            affine: mode.affine,
            stack: exponentials,
        }
    }
}

/// The result of a search step: the node proving the sequent, `None` when
/// it is unprovable, or the reason the whole search stops.
pub(crate) type Search = Result<Option<NodeId>, Reason>;

/// Where the nodes of a run's proofs are kept: the engine's own arena in
/// a sequential search, the one arena of a parallel search behind its
/// lock.
pub(crate) enum Kept<'a> {
    /// The engine's own arena.
    Own(Vec<Node>),
    /// The arena of a parallel search.
    Shared(&'a Mutex<Vec<Node>>),
}

/// The flag in the id of a node that is still pending.
const PENDING: u32 = 1 << 31;

/// The proof arena of a run, in two parts. A node is first *pending*: it
/// belongs to the branch being searched, lives in the engine's own
/// buffer, a stack, and vanishes when the branch fails
/// ([`release`](Self::release)). Once the stable sequent it helps to prove
/// is proved, its nodes are *kept* ([`keep`](Self::keep)): moved to the
/// arena the memo's entries and the final proof refer to, where a node's
/// id is its position and a premise precedes its conclusion. So the nodes
/// of failed branches never reach the kept arena, and a refutation's
/// memory is bounded by what the memo holds, however many branches it
/// tried. A pending node's id carries the [`PENDING`] flag and is only
/// valid until its branch is released or kept; in a parallel search no
/// pending id leaves its worker.
pub(crate) struct Arena<'a> {
    /// The nodes kept.
    kept: Kept<'a>,
    /// The nodes of the branch being searched.
    pending: Vec<Node>,
}

impl<'a> Arena<'a> {
    /// An arena with nothing pending.
    pub(crate) fn new(kept: Kept<'a>) -> Self {
        Self {
            kept,
            pending: Vec::new(),
        }
    }

    /// Appends a pending node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId::new(PENDING | self.pending.len() as u32);
        self.pending.push(node);
        id
    }

    /// Returns the point of the pending nodes to come back to.
    fn mark(&self) -> usize {
        self.pending.len()
    }

    /// Drops the nodes pending since the mark: their branch failed.
    fn release(&mut self, mark: usize) {
        self.pending.truncate(mark);
    }

    /// Keeps the nodes pending since the mark, which must hold everything
    /// pending that `node` rests on, and returns the id `node` has from now
    /// on; a node that was kept already keeps its id.
    fn keep(&mut self, mark: usize, node: NodeId) -> NodeId {
        match &mut self.kept {
            Kept::Own(kept) => Self::append(kept, &mut self.pending, mark, node),
            Kept::Shared(kept) => {
                let mut kept = kept.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                Self::append(&mut kept, &mut self.pending, mark, node)
            }
        }
    }

    /// Moves the nodes pending since the mark to the end of the kept ones,
    /// their premises renamed, and returns the new id of `node`.
    fn append(kept: &mut Vec<Node>, pending: &mut Vec<Node>, mark: usize, node: NodeId) -> NodeId {
        let base = kept.len();
        assert!(
            base + (pending.len() - mark) <= PENDING as usize,
            "the proof arena is full"
        );
        let renamed = |id: NodeId| {
            if id.get() & PENDING == 0 {
                return id;
            }
            let index = (id.get() & !PENDING) as usize;
            debug_assert!(index >= mark, "a pending node below the mark");
            NodeId::new((base + index - mark) as u32)
        };
        kept.extend(pending.drain(mark..).map(|n| n.map_premises(renamed)));
        renamed(node)
    }
}

/// The state of one run: the problem, the memo, the proof arena, the
/// counters, the bound bookkeeping, and pools of scratch buffers so that no
/// step allocates once the pools are warm.
struct Engine<'a> {
    /// The problem.
    forest: &'a Forest,
    /// Its intuitionistic reading, for a two-sided search.
    reading: Option<&'a Reading<'a>>,
    /// Its count invariants.
    counts: &'a Counts,
    /// Its classes of interchangeable occurrences.
    classes: &'a Classes,
    /// The rules in force.
    rules: Rules,
    /// The memo of stable sequents.
    memo: Table<'a>,
    /// The proof arena.
    nodes: Arena<'a>,
    /// Whether the memo takes entries: a proof it refers to is kept.
    memoizes: bool,
    /// The counters.
    statistics: Statistics,
    /// The steps of split searches since the stop condition was last
    /// polled there.
    steps: u64,
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
    /// The stop condition.
    stop: Stop<'a>,
    /// The runtime, when the search runs on several threads.
    #[cfg(feature = "parallel")]
    runtime: Option<&'a super::parallel::Runtime>,
    /// How many choices among alternatives on this branch ran on several
    /// threads: the levels of cube-and-conquer above the current sequent.
    or_depth: u32,
    /// The seed of the order in which this engine tries the alternatives
    /// of a choice, zero for the order by id.
    seed: u64,
    /// Whether the workers of a parallel search order their choices by
    /// seeds of their own.
    portfolio: bool,
    /// The stable sequents of the current branch, the root end first; only
    /// the first `stack_len` are live, the rest are spare buffers.
    stack: Vec<Key>,
    /// The hash of every entry of the stack, live or spare: the loop check
    /// compares hashes before sequents.
    hashes: Vec<u64>,
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
    /// Spare counts of splits, boxed so that a split search holds a
    /// pointer on the stack and not the counts: a level of recursion
    /// through a searched split took 1.5 KiB with them in the frame.
    #[allow(clippy::vec_box)]
    splits: Vec<Box<Split>>,
    /// Spare trails of split searches.
    trails: Vec<Vec<Side>>,
    /// Spare lists of the links of a chain of forced splits.
    links: Vec<Vec<(OccId, NodeId, bool)>>,
}

impl<'a> Engine<'a> {
    /// Prepares a run on the forest with the rules given, its counts and
    /// classes, the stop condition, the memo and the arena to use.
    #[allow(clippy::too_many_arguments)]
    fn new(
        forest: &'a Forest,
        rules: Rules,
        reading: Option<&'a Reading<'a>>,
        (counts, classes): (&'a Counts, &'a Classes),
        options: &Options,
        stop: Stop<'a>,
        memo: Table<'a>,
        nodes: Arena<'a>,
    ) -> Self {
        Self {
            forest,
            reading,
            counts,
            classes,
            rules,
            memo,
            nodes,
            memoizes: options.memo_limit != 0,
            statistics: Statistics::default(),
            steps: 0,
            depth: 0,
            recursion_limit: options.recursion_limit,
            copies: options.copies,
            exhausted: false,
            dependency: NO_DEPENDENCY,
            stop,
            #[cfg(feature = "parallel")]
            runtime: None,
            or_depth: 0,
            seed: 0,
            portfolio: options.portfolio,
            stack: Vec::new(),
            hashes: Vec::new(),
            stack_len: 0,
            sets: Vec::new(),
            contexts: Vec::new(),
            keys: Vec::new(),
            lists: Vec::new(),
            tallies: Vec::new(),
            splits: Vec::new(),
            trails: Vec::new(),
            links: Vec::new(),
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

    /// The body of the asynchronous phase. The `?` rules are applied in the
    /// loop like `⅋` and `⊥`, on a copy of the unrestricted zone that
    /// grows, so that a sequent of thousands of `?` formulas costs one
    /// level of recursion and not one each.
    fn decompose(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        budget: u32,
    ) -> Search {
        // The `⅋`, `⊥` and `?` rules applied, to wrap around the proof of
        // what remains; the first applied is the lowest.
        let mut applied = self.take_list();
        // The unrestricted zone once a `?` added a formula to it.
        let mut grown: Option<OccSet> = None;
        let result = loop {
            let Some(o) = list.pop() else {
                break self.prove(grown.as_ref().unwrap_or(theta), gamma, budget);
            };
            match self.forest.kind(o) {
                Kind::Par => {
                    list.push(self.forest.right(o).unwrap());
                    list.push(self.forest.left(o).unwrap());
                    applied.push(o);
                }
                Kind::Bot => applied.push(o),
                Kind::Top => break Ok(Some(self.push(Node::Top(o)))),
                Kind::With => {
                    break self.with(grown.as_ref().unwrap_or(theta), gamma, list, o, budget);
                }
                Kind::Quest => {
                    // The subformula joins the unrestricted zone, which is
                    // a set: a formula already there changes nothing.
                    let a = self.forest.left(o).unwrap();
                    if !grown.as_ref().unwrap_or(theta).contains(a) {
                        if grown.is_none() {
                            let mut larger = self.take_set();
                            larger.clone_from(theta);
                            grown = Some(larger);
                        }
                        grown.as_mut().unwrap().insert(a);
                    }
                    applied.push(o);
                }
                _ => {
                    // A positive formula or a literal: part of the stable
                    // sequent.
                    gamma.insert(o);
                }
            }
        };
        if let Some(larger) = grown {
            self.give_set(larger);
        }
        let result = result.map(|proved| {
            proved.map(|mut node| {
                for &o in applied.iter().rev() {
                    node = self.push(match self.forest.kind(o) {
                        Kind::Par => Node::Par(o, node),
                        Kind::Quest => Node::Quest(o, node),
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
        #[cfg(feature = "parallel")]
        if self.cubes() {
            return self.with_parallel(theta, gamma, list, o, budget);
        }
        let mut left_gamma = self.take_context();
        left_gamma.clone_from(gamma);
        let mut left_list = self.take_list();
        left_list.clone_from(list);
        left_list.push(self.forest.left(o).unwrap());
        let mark = self.nodes.mark();
        let left = self.asynchronous(theta, &mut left_gamma, &mut left_list, budget);
        self.give_context(left_gamma);
        self.give_list(left_list);
        let Some(left) = left? else {
            return Ok(None);
        };
        list.push(self.forest.right(o).unwrap());
        let Some(right) = self.asynchronous(theta, gamma, list, budget)? else {
            self.nodes.release(mark);
            return Ok(None);
        };
        Ok(Some(self.push(Node::With(o, left, right))))
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
        if self.stop.fired() {
            return Err(Reason::Stopped);
        }
        let mut key = self.take_key();
        key.assign(theta, gamma);
        // Complete failures are recorded up to interchangeable members:
        // under the canonical key, where it differs from the sequent's own.
        let mut canonical = self.take_key();
        let renamed =
            !self.classes.distinct() && canonical.gamma.canonical_from(gamma, self.classes);
        // A proof or a complete failure from the memo settles it; a
        // failure cut by the budget waits for the loop check, which may
        // give the stronger answer that the branch is redundant.
        let entry = if renamed && {
            canonical.theta.clone_from(theta);
            self.memo.refuted(&canonical)
        } {
            Some(Entry::Failed(Failure::Complete))
        } else {
            self.memo.get(&key, budget)
        };
        match entry {
            Some(Entry::Proved(node)) => {
                self.give_key(key);
                self.give_key(canonical);
                return Ok(Some(node));
            }
            Some(Entry::Failed(Failure::Complete)) => {
                self.give_key(key);
                self.give_key(canonical);
                return Ok(None);
            }
            Some(Entry::Failed(Failure::Exhausted(_))) | None => {}
        }
        // A sequent that repeats an ancestor on its branch is pruned: a
        // smallest proof of the ancestor never passes through it. The
        // failure this causes above is a fact about the branch, so it is
        // remembered as a dependency on the ancestor's depth and keeps the
        // sequents between them out of the memo, until the ancestor itself
        // is decided. (A sequent that merely contains an ancestor is not
        // redundant, with or without weakening: a proof of the larger
        // sequent proves nothing about the smaller one, and ⊢ ?(a ⅋ ~a) is
        // proved only through ⊢ a ⅋ ~a ; a, ~a.)
        let hash = if self.rules.stack {
            crate::hash::BuildHasher::default().hash_one(&key)
        } else {
            0
        };
        if self.rules.stack {
            for depth in 0..self.stack_len {
                if self.hashes[depth] == hash && self.stack[depth] == key {
                    self.dependency = self.dependency.min(depth as u32);
                    self.give_key(key);
                    self.give_key(canonical);
                    return Ok(None);
                }
            }
        }
        if entry.is_some() {
            // Cut by the budget at this or a larger remaining budget.
            self.exhausted = true;
            self.give_key(key);
            self.give_key(canonical);
            return Ok(None);
        }
        if self.rules.stack {
            if self.stack_len < self.stack.len() {
                self.stack[self.stack_len].clone_from(&key);
                self.hashes[self.stack_len] = hash;
            } else {
                self.stack.push(key.clone());
                self.hashes.push(hash);
            }
            self.stack_len += 1;
        }
        let saved_exhausted = self.exhausted;
        let saved_dependency = self.dependency;
        self.exhausted = false;
        self.dependency = NO_DEPENDENCY;
        let mark = self.nodes.mark();
        let mut result = self.decide(theta, gamma, budget);
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
            Ok(Some(node)) if self.memoizes => {
                // The entry outlives the branch, so the proof is kept.
                let node = self.nodes.keep(mark, node);
                self.memo.insert(&key, Entry::Proved(node));
                result = Ok(Some(node));
            }
            Ok(None) => {
                self.nodes.release(mark);
                if dependency == NO_DEPENDENCY {
                    // A complete failure answers for every relative; one
                    // cut by the budget stays the sequent's own.
                    if exhausted {
                        self.memo
                            .insert(&key, Entry::Failed(Failure::Exhausted(budget)));
                    } else {
                        let key = if renamed { &canonical } else { &key };
                        self.memo.insert(key, Entry::Failed(Failure::Complete));
                    }
                }
            }
            _ => {}
        }
        self.give_key(key);
        self.give_key(canonical);
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
            tally.add(self.counts, o);
            match self.forest.kind(o) {
                Kind::Zero => zero = true,
                Kind::Tensor | Kind::Plus => candidates.push(o),
                Kind::One | Kind::Bang if members.len() == 1 || affine => candidates.push(o),
                Kind::One | Kind::Bang | Kind::Var | Kind::DualVar => {}
                kind => unreachable!("{kind:?} in a stable sequent"),
            }
        }
        // A `0` has no rule, so only a `⊤` below some member of `Γ`, or
        // below a member of `Θ` that a copy can bring in, can prove the
        // sequent: ⊢ 0, ⊤ ⊕ b is provable. (The spec calls a `0` in a
        // stable sequent fatal, which overlooks this.) With weakening the
        // `0` is discarded at a leaf like anything else.
        if zero && !affine && !tally.absorbs() && !theta.iter().any(|a| self.counts.absorbs(a)) {
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

        // Of interchangeable candidates one is enough: the focus on either
        // leaves the same sequent up to a renaming.
        self.one_of_each(candidates);
        // Forced splits first, then `⊕`, then the free splits; by id within
        // a class, so that the run is deterministic.
        candidates.sort_by_key(|&o| (self.focus_class(o), self.rank(o)));
        // After them the copies from `Θ`, a formula with an unconsumed copy
        // in `Γ` skipped (a second copy cannot help before the first is
        // used), those that can meet a literal of `Γ` first, then by id.
        if self.rules.exponentials {
            copies.extend(theta.iter().filter(|&a| !gamma.contains(a)));
            self.one_of_each(copies);
            if budget == 0 {
                // A branch cut by the bound: the level cannot claim
                // completeness, unless nothing was there to copy.
                self.exhausted |= !copies.is_empty();
                copies.clear();
            } else {
                // By rank, those that meet a member first: the heuristic is
                // asked once per formula, not once per comparison.
                copies.sort_unstable_by_key(|&a| self.rank(a));
                let mut others = self.take_list();
                let mut met = 0;
                for i in 0..copies.len() {
                    let a = copies[i];
                    if self.meets(a, members) {
                        copies[met] = a;
                        met += 1;
                    } else {
                        others.push(a);
                    }
                }
                copies.truncate(met);
                copies.extend_from_slice(&others);
                self.give_list(others);
            }
        }
        #[cfg(feature = "parallel")]
        if let Some(result) = self.choices_parallel(theta, gamma, candidates, copies, budget) {
            if let Some(node) = result? {
                return Ok(Some(node));
            }
            return self.last_resort(theta, gamma, members, tally, budget);
        }
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
        for &a in copies.iter() {
            if let Some(node) = self.focus(theta, gamma, a, budget - 1)? {
                return Ok(Some(self.push(Node::Copy(a, node))));
            }
        }
        self.last_resort(theta, gamma, members, tally, budget)
    }

    /// Keeps, of the occurrences that are interchangeable, the one with
    /// the lowest id.
    fn one_of_each(&self, occurrences: &mut Vec<OccId>) {
        occurrences.sort_unstable_by_key(|&o| (self.classes.of(o), o));
        occurrences.dedup_by_key(|o| self.classes.of(*o));
    }

    /// What is tried on a stable sequent after every focus failed: Mix,
    /// when the rules have it.
    fn last_resort(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &Tally,
        budget: u32,
    ) -> Search {
        if self.rules.mix {
            return self.mix(theta, gamma, members, tally, budget);
        }
        Ok(None)
    }

    /// The position of an occurrence in the order this engine tries the
    /// alternatives of a choice: its id, or a mix of the id with the
    /// engine's seed when it has one, which orders the alternatives
    /// differently on every worker of a portfolio.
    fn rank(&self, o: OccId) -> u64 {
        if self.seed == 0 {
            return u64::from(o.get());
        }
        // The finalizer of splitmix64 over the id and the seed.
        let mut x = u64::from(o.get()) ^ self.seed;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
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
            if (affine || members.len() == 2)
                && let Some(&q) = members[i + 1..].iter().find(|&&q| dual(p, q))
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
            if (affine || members.len() == 1)
                && let Some(d) = self.dual_in(p, |d| theta.contains(d))
            {
                if budget == 0 {
                    // Another pair may still close the sequent without a
                    // copy.
                    self.exhausted = true;
                    continue;
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

    /// The first occurrence of the literal dual to `literal` that is `within`
    /// a zone, found through the forest's list of that literal's
    /// occurrences rather than through the zone's members.
    fn dual_in(&self, literal: OccId, within: impl Fn(OccId) -> bool) -> Option<OccId> {
        let f = self.forest;
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        f.literals(atom, !sign).iter().copied().find(|&d| within(d))
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
    /// fails at once), then `⊕`, then a `⊗` whose split must be searched.
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
                #[cfg(feature = "parallel")]
                if self.cubes() {
                    return self.plus_parallel(theta, gamma, f, budget);
                }
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
            Kind::Var | Kind::DualVar if self.counts.positive(self.forest, f) => {
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
        debug_assert!(
            self.reading.is_none_or(|r| r.outputs(gamma.iter()) == 0),
            "a leaf is the goal, so only hypotheses are left over"
        );
        for o in gamma.iter() {
            node = self.push(Node::Weaken(o, node));
        }
        node
    }

    // The context splits.

    /// What a factor of a `⊗` forces on its side of the split: nothing at
    /// all for `0`, the empty context for `1` and `!`, the dual literal
    /// alone for a positive literal, and one dual per literal for a tensor
    /// of positive literals whose duals lie in the linear zone only. With
    /// weakening nothing but `0` forces anything, since every leaf takes
    /// any context.
    fn forced_side(&self, factor: OccId) -> Option<Forced> {
        match self.forest.kind(factor) {
            Kind::Zero => Some(Forced::Nothing),
            _ if self.rules.affine => None,
            Kind::One | Kind::Bang => Some(Forced::Empty),
            Kind::Var | Kind::DualVar if self.counts.positive(self.forest, factor) => {
                Some(Forced::Dual)
            }
            Kind::Tensor if self.counts.literal_tensor(factor) => Some(Forced::Duals),
            _ => None,
        }
    }

    /// The factor of `F = A ⊗ B` that forces its split, with what it
    /// forces, the other factor, and whether the forcing one is the left.
    /// When both force, one that is closed in place (a literal, a unit)
    /// before a tensor of literals, which takes a focus of its own: a
    /// tensor of a thousand literals, nested to the left as it is read,
    /// must not pay a level of recursion per link; otherwise the left one.
    fn forced_factor(&self, f: OccId) -> Option<(Forced, OccId, OccId, bool)> {
        let (a, b) = (self.forest.left(f).unwrap(), self.forest.right(f).unwrap());
        [(a, b, true), (b, a, false)]
            .into_iter()
            .filter_map(|(x, y, left)| Some((self.forced_side(x)?, x, y, left)))
            .min_by_key(|&(forced, ..)| forced == Forced::Duals)
    }

    /// The `⊗` rule on `F = A ⊗ B` with context `Γ`: the forced split when
    /// a factor allows only one, else a search over the members of `Γ` for
    /// the splits whose two sides pass the counts.
    fn split(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Search {
        if self.forced_factor(f).is_none() {
            return self.free_split(theta, gamma, f, budget);
        }
        let mark = self.nodes.mark();
        let mut rest = self.take_context();
        rest.clone_from(gamma);
        let mut links = self.take_links();
        let result = self.forced_splits(theta, &mut rest, f, &mut links, budget);
        self.give_context(rest);
        let result = match result {
            Ok(Some(mut node)) => {
                for &(f, x_node, x_is_left) in links.iter().rev() {
                    let (left, right) = if x_is_left {
                        (x_node, node)
                    } else {
                        (node, x_node)
                    };
                    node = self.push(Node::Tensor(f, left, right));
                }
                Ok(Some(node))
            }
            Ok(None) => {
                self.nodes.release(mark);
                Ok(None)
            }
            Err(reason) => Err(reason),
        };
        self.give_links(links);
        result
    }

    /// The forced splits of `F` and, as long as the factor they leave is a
    /// `⊗` with a forced split again, of that factor, in a loop: a tensor
    /// of a thousand literals costs one level of recursion and one copy of
    /// the context, not a thousand. `rest` is the context, from which
    /// every forcing factor takes its own; `links` gets every `⊗` with the
    /// proof of its forcing factor and whether that is the left one.
    /// Returns the proof of the factor left at the end with what remains
    /// of the context.
    fn forced_splits(
        &mut self,
        theta: &OccSet,
        rest: &mut Context,
        mut f: OccId,
        links: &mut Vec<(OccId, NodeId, bool)>,
        budget: u32,
    ) -> Search {
        loop {
            let Some((forced, x, y, x_is_left)) = self.forced_factor(f) else {
                return self.focus(theta, rest, f, budget);
            };
            self.statistics.splits += 1;
            let x_node = match forced {
                Forced::Nothing => return Ok(None),
                Forced::Empty => {
                    let empty = self.take_context();
                    let node = self.focus(theta, &empty, x, budget);
                    self.give_context(empty);
                    node?
                }
                // The dual in `Γ` when there is one, and the axiom on the
                // two; otherwise the side stays empty and the initial rule
                // looks in `Θ`, at the price of a copy. Taking the copy
                // from `Γ` first loses nothing: the copies are the same
                // formula, so a proof that leaves this one for elsewhere
                // and copies from `Θ` here is a proof with the roles
                // swapped.
                Forced::Dual => {
                    if let Some(dual) = self.dual_in(x, |m| rest.contains(m)) {
                        rest.remove(dual);
                        Some(self.push(Node::Ax(x, dual)))
                    } else if let Some(d) = self.dual_in(x, |d| theta.contains(d)) {
                        if budget == 0 {
                            self.exhausted = true;
                            None
                        } else {
                            let ax = self.push(Node::Ax(x, d));
                            Some(self.push(Node::Copy(d, ax)))
                        }
                    } else {
                        None
                    }
                }
                // One dual per literal, each the first left in `Γ`: every
                // literal is proved by exactly its dual, and no dual lies in
                // `Θ`, so no other context proves the factor.
                Forced::Duals => self.literal_tensor(x, rest),
            };
            let Some(x_node) = x_node else {
                return Ok(None);
            };
            links.push((f, x_node, x_is_left));
            if self.forest.kind(y) != Kind::Tensor {
                return self.focus(theta, rest, y, budget);
            }
            f = y;
        }
    }

    /// The proof of a tensor of positive literals from one dual per
    /// literal, each the first left in `rest`, which loses them; `None`
    /// when a dual is missing. Built in place, the axioms and the `⊗`
    /// nodes from the last occurrence back, so a tensor of any depth costs
    /// no recursion.
    fn literal_tensor(&mut self, x: OccId, rest: &mut Context) -> Option<NodeId> {
        let mut duals = self.take_list();
        for leaf in self.forest.subtree(x) {
            if !self.forest.is_literal(leaf) {
                continue;
            }
            let Some(dual) = self.dual_in(leaf, |m| rest.contains(m)) else {
                self.give_list(duals);
                return None;
            };
            rest.remove(dual);
            duals.push(dual);
        }
        // An occurrence's subtree follows it, so in reverse a `⊗` comes
        // after both its subformulas, the left one's proof on top.
        let mut built = self.take_links();
        for o in self.forest.subtree(x).rev() {
            let node = if self.forest.is_literal(o) {
                let dual = duals.pop().expect("a dual per literal");
                self.push(Node::Ax(o, dual))
            } else {
                self.statistics.splits += 1;
                let (_, left, _) = built.pop().expect("the left subformula's proof");
                let (_, right, _) = built.pop().expect("the right subformula's proof");
                self.push(Node::Tensor(o, left, right))
            };
            built.push((o, node, true));
        }
        let (_, node, _) = built.pop().expect("the tensor's proof");
        self.give_list(duals);
        self.give_links(built);
        Some(node)
    }

    /// The `⊗` rule on a formula no factor of which forces its split: a
    /// search over the members of `Γ` for the splits whose two sides pass
    /// the counts.
    fn free_split(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Search {
        let (a, b) = (self.forest.left(f).unwrap(), self.forest.right(f).unwrap());
        let mut members = self.take_list();
        members.extend(gamma.iter());
        let mut left = self.take_context();
        let mut right = self.take_context();
        right.clone_from(gamma);
        let mut split = self.take_split();
        split.place(self.counts, a, Side::Left);
        split.place(self.counts, b, Side::Right);
        let mut placed = self.take_list();
        placed.extend([a, b]);
        // Two-sided, on a hypothesis `A ⊸ B`: the goal stays with `B`, so it
        // is fixed on the consequent's side and left out of the search.
        if let Some(reading) = self.reading
            && let Some((_, consequent)) = reading.implication(f)
            && let Some(at) = members
                .iter()
                .position(|&m| reading.position(m) == Position::Output)
        {
            let goal = members.remove(at);
            placed.push(goal);
            if consequent == a {
                right.remove(goal);
                left.insert(goal);
                split.place(self.counts, goal, Side::Left);
            } else {
                split.place(self.counts, goal, Side::Right);
            }
        }
        self.open(&mut members, &mut split, &placed);
        self.give_list(placed);
        let join = Join::Tensor(f, a, b);

        #[cfg(feature = "parallel")]
        let result = if self.cubes() && members.len() >= 2 {
            self.split_parallel(theta, &members, (&left, &right), &split, join, budget)
        } else {
            self.search_splits(
                theta,
                &members,
                (0, 0),
                (&mut left, &mut right),
                &mut split,
                join,
                budget,
            )
        };
        #[cfg(not(feature = "parallel"))]
        let result = self.search_splits(
            theta,
            &members,
            (0, 0),
            (&mut left, &mut right),
            &mut split,
            join,
            budget,
        );
        self.give_list(members);
        self.give_context(left);
        self.give_context(right);
        self.give_split(split);
        result
    }

    /// Puts the members a split search assigns in the order it decides
    /// them in and opens them in the split's counts: those that bear on
    /// more atoms first, then by the first atom of their rows, so that the
    /// members that bear on an atom are decided one after the other and
    /// its counts are settled early (those without a row last);
    /// interchangeable members next to each other, the lowest id last.
    /// `placed` are the members that have their side already.
    fn open(&self, members: &mut [OccId], split: &mut Split, placed: &[OccId]) {
        members.sort_unstable_by_key(|&m| {
            (
                std::cmp::Reverse(self.counts.row_len(m)),
                self.counts.first_atom(m),
                self.classes.of(m),
                std::cmp::Reverse(m),
            )
        });
        for &m in members.iter() {
            split.open(self.counts, m);
        }
        // Without the equation, a split can only fail the counts through a
        // member whose own interval of some atom excludes zero: with none,
        // every sum of intervals contains zero and every split passes.
        let tight = |o: &OccId| self.counts.tight(*o);
        split.set_inert(
            !self.rules.equation
                && (!self.rules.intervals || !placed.iter().chain(members.iter()).any(tight)),
        );
    }

    /// Searches the splits of a context into two sides that pass the
    /// counts: the premises of a `⊗` or the parts of a Mix, as `join`
    /// says. The members from `start` on are assigned one by one, in their
    /// order, each to the right first and then to the left, and a partial
    /// assignment is given up as soon as the counts show that no way of
    /// assigning the rest lets both sides pass. Of interchangeable members
    /// the left side takes those with the lowest ids, so that only their
    /// number varies: any other choice of as many gives the same two
    /// sequents up to a renaming. So every such split that passes the
    /// counts is reached, each once, and none that fails them. `left` and
    /// `right` are the sides and `split` their counts, with the members
    /// before `start` assigned as the bits of `prefix` say (one for the
    /// left) and the others on the right and open. Returns the node of the
    /// first split whose two sides are proved.
    #[allow(clippy::too_many_arguments)]
    fn search_splits(
        &mut self,
        theta: &OccSet,
        members: &[OccId],
        (start, prefix): (usize, u64),
        (left, right): (&mut Context, &mut Context),
        split: &mut Split,
        join: Join,
        budget: u32,
    ) -> Search {
        let rules = self.rules;
        let mut trail = self.take_trail();
        trail.extend((0..members.len()).map(|i| {
            if i < start && prefix >> i & 1 == 1 {
                Side::Left
            } else {
                Side::Right
            }
        }));
        // The members before `next` are assigned, as the trail says.
        let mut next = start;
        let found = 'search: loop {
            self.statistics.splits += 1;
            self.poll_splits()?;
            if split.feasible(rules.intervals, rules.equation, rules.mix) {
                if next < members.len() {
                    let m = members[next];
                    // On the left at once when the member before it is
                    // interchangeable and went left: the lowest ids do.
                    let side = if next > 0
                        && trail[next - 1] == Side::Left
                        && self.classes.same(members[next - 1], m)
                    {
                        right.remove(m);
                        left.insert(m);
                        Side::Left
                    } else {
                        Side::Right
                    };
                    split.assign(self.counts, m, side);
                    trail[next] = side;
                    next += 1;
                    continue;
                }
                let joined = match join {
                    Join::Tensor(f, a, b) => self.premises(theta, left, right, f, a, b, budget)?,
                    // A Mix needs two parts.
                    Join::Mix if right.is_empty() => None,
                    Join::Mix => self.parts(theta, left, right, budget)?,
                };
                if joined.is_some() {
                    break joined;
                }
            }
            // Back to the last member assigned to the right, which goes to
            // the left; those after it are open again.
            loop {
                if next == start {
                    break 'search None;
                }
                next -= 1;
                let m = members[next];
                if trail[next] == Side::Right {
                    split.flip(self.counts, m, Side::Left);
                    right.remove(m);
                    left.insert(m);
                    trail[next] = Side::Left;
                    next += 1;
                    continue 'search;
                }
                split.unassign(self.counts, m, Side::Left);
                left.remove(m);
                right.insert(m);
            }
        };
        self.give_trail(trail);
        Ok(found)
    }

    /// Polls the stop condition once every [`SPLITS_PER_POLL`] steps of
    /// the split searches: a search whose splits fail in focus visits no
    /// stable sequent, where the condition is polled otherwise, and may
    /// run for minutes.
    fn poll_splits(&mut self) -> Result<(), Reason> {
        self.steps += 1;
        if self.steps < SPLITS_PER_POLL {
            return Ok(());
        }
        self.steps = 0;
        if self.stop.fired() {
            Err(Reason::Stopped)
        } else {
            Ok(())
        }
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
        let mark = self.nodes.mark();
        let Some(l) = self.focus(theta, left, a, budget)? else {
            return Ok(None);
        };
        let Some(r) = self.focus(theta, right, b, budget)? else {
            self.nodes.release(mark);
            return Ok(None);
        };
        Ok(Some(self.push(Node::Tensor(f, l, r))))
    }

    /// The Mix rule on a stable sequent no focus proves: a split into two
    /// non-empty provable parts, searched over the members after the
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
        let mut left = self.take_context();
        left.insert(members[0]);
        let mut right = self.take_context();
        right.clone_from(gamma);
        right.remove(members[0]);
        let mut split = self.take_split();
        split.place(self.counts, members[0], Side::Left);
        let mut rest = self.take_list();
        rest.extend_from_slice(&members[1..]);
        self.open(&mut rest, &mut split, &members[..1]);
        let result = self.search_splits(
            theta,
            &rest,
            (0, 0),
            (&mut left, &mut right),
            &mut split,
            Join::Mix,
            budget,
        );
        self.give_list(rest);
        self.give_context(left);
        self.give_context(right);
        self.give_split(split);
        result
    }

    /// Both parts of a Mix, and the Mix node if both are provable.
    fn parts(&mut self, theta: &OccSet, left: &Context, right: &Context, budget: u32) -> Search {
        let mark = self.nodes.mark();
        let Some(l) = self.prove(theta, left, budget)? else {
            return Ok(None);
        };
        let Some(r) = self.prove(theta, right, budget)? else {
            self.nodes.release(mark);
            return Ok(None);
        };
        Ok(Some(self.push(Node::Mix(l, r))))
    }

    // Bookkeeping.

    /// Appends a node to the pending ones and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node)
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

    /// Takes the counts of a split with no member from the pool.
    fn take_split(&mut self) -> Box<Split> {
        match self.splits.pop() {
            Some(mut split) => {
                split.clear();
                split
            }
            None => Box::new(self.counts.split()),
        }
    }

    /// Returns the counts of a split to the pool.
    fn give_split(&mut self, split: Box<Split>) {
        self.splits.push(split);
    }

    /// Takes an empty trail from the pool.
    fn take_trail(&mut self) -> Vec<Side> {
        let mut trail = self.trails.pop().unwrap_or_default();
        trail.clear();
        trail
    }

    /// Returns a trail to the pool.
    fn give_trail(&mut self, trail: Vec<Side>) {
        self.trails.push(trail);
    }

    /// Takes an empty list of links from the pool.
    fn take_links(&mut self) -> Vec<(OccId, NodeId, bool)> {
        let mut links = self.links.pop().unwrap_or_default();
        links.clear();
        links
    }

    /// Returns a list of links to the pool.
    fn give_links(&mut self, links: Vec<(OccId, NodeId, bool)>) {
        self.links.push(links);
    }
}

/// What joins the two sides of a split of a context.
#[derive(Clone, Copy, Debug)]
enum Join {
    /// The `⊗` rule on this formula, with its left and right subformula:
    /// the sides are its premises' contexts.
    Tensor(OccId, OccId, OccId),
    /// The Mix rule: the sides are its parts.
    Mix,
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
    /// One dual literal per literal of the factor, a tensor of positive
    /// literals.
    Duals,
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
        let reading = mode.intuitionistic.then(|| {
            Reading::new(&forest).unwrap_or_else(|e| panic!("{input:?}: {}", e.describe(&forest)))
        });
        let (verdict, statistics) = search(
            &forest,
            s.fragment(),
            mode,
            reading.as_ref(),
            options,
            &mut || false,
        );
        if let Verdict::Proved(proof) = &verdict {
            assert_eq!(proof.sequent(), &s);
            proof
                .check(mode)
                .unwrap_or_else(|e| panic!("{input:?}: the proof is wrong: {e}"));
        }
        (verdict, statistics)
    }

    /// Forty literals over atoms of their own, as the rest of a sequent.
    pub(crate) fn wide_context() -> String {
        let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
        literals.join(", ")
    }

    /// Whether `input` is provable under `mode`, panicking on `Unknown`.
    fn provable(input: &str, mode: Mode) -> bool {
        match run(input, mode, &Options::default()).0 {
            Verdict::Proved(_) => true,
            Verdict::Unprovable => false,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// A released branch leaves no node, a kept one moves to the kept
    /// arena with its premises renamed, and a node kept before keeps its
    /// id.
    #[test]
    fn arena() {
        let o = OccId::new;
        let mut arena = Arena::new(Kept::Own(Vec::new()));
        let ax = arena.push(Node::Ax(o(0), o(1)));
        let mark = arena.mark();
        let one = arena.push(Node::One(o(2)));
        arena.push(Node::Tensor(o(3), ax, one));
        arena.release(mark);
        let top = arena.push(Node::Top(o(4)));
        let top = arena.keep(mark, top);
        assert_eq!(top, NodeId::new(0));
        let tensor = arena.push(Node::Tensor(o(5), ax, top));
        let root = arena.keep(0, tensor);
        assert_eq!(root, NodeId::new(2));
        let Kept::Own(kept) = arena.kept else {
            unreachable!()
        };
        assert_eq!(
            kept,
            [
                Node::Top(o(4)),
                Node::Ax(o(0), o(1)),
                Node::Tensor(o(5), NodeId::new(1), NodeId::new(0))
            ]
        );
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

    /// The stop condition and the recursion limit each end the search with
    /// their reason, and no context is too wide to split.
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
            None,
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

        // A `?` costs no level of recursion: three thousand of them stay
        // under the default limit.
        let hypotheses: Vec<String> = (0..3000).map(|i| format!("?a{i}")).collect();
        let many = format!("|- 1, {}", hypotheses.join(", "));
        let (verdict, _) = run(&many, Mode::CLASSICAL, &Options::default());
        assert!(verdict.proof().is_some());

        // Nor does a link of a tensor of literals, nested to the left as
        // it is read: every split is forced by the literal on the right.
        // On a thread with the stack a front end gives the search, since
        // the parser and the checker recurse to the formula's depth.
        let literals = format!(
            "|- {}, {}",
            vec!["a"; 2500].join(" * "),
            vec!["~a"; 2500].join(", ")
        );
        // The same for a tensor of tensors of literals, which is proved
        // in place from the duals of all its literals.
        let pairs = format!(
            "|- {}, {}",
            vec!["(a * b)"; 2500].join(" * "),
            vec!["~a, ~b"; 2500].join(", ")
        );
        let proved = std::thread::Builder::new()
            .stack_size(Options::default().stack_size())
            .spawn(move || {
                [literals, pairs].iter().all(|chain| {
                    let (verdict, _) = run(chain, Mode::CLASSICAL, &Options::default());
                    verdict.proof().is_some()
                })
            })
            .unwrap()
            .join()
            .unwrap();
        assert!(proved);

        // A free split over 126 formulas, in MALL so that the count
        // equation does not refute it first: no width is too much, and the
        // counts of `b` refute every split at once.
        let wide = format!(
            "|- ((a & a) par b) * (~a par ~b), {}",
            (0..63).map(|_| "~a, a").collect::<Vec<_>>().join(", ")
        );
        let (verdict, _) = run(&wide, Mode::CLASSICAL, &Options::default());
        assert!(matches!(verdict, Verdict::Unprovable));
    }

    /// A sequent whose search comes back to it with other occurrences of
    /// the same formulas is refuted, not left at the copy bound by its own
    /// failure of the level before: a failure cut by the budget answers
    /// for its own sequent alone.
    #[test]
    fn repeats_up_to_equal_members() {
        for input in [
            "|- ~b, (1 * a), ?(b * ((a par ~b) * ~a))",
            "a, a, b, b, !(((b * a) * a) -o b), !(((b * a) * a) -o b) |- ((b * b) * b)",
        ] {
            assert!(!provable(input, Mode::CLASSICAL), "{input:?}");
        }
        // Here the relative is reached on another branch.
        let sibling = "!(!!(b -o c) -o !c), b |- (c * ((a -o a) * c))";
        assert!(!provable(sibling, Mode::INTUITIONISTIC));
    }

    /// The bias never changes what is provable, only what a copy bound
    /// allows: the counter with eight tokens is proved within three copies
    /// a branch chaining backward, and needs seven chaining forward, where
    /// it visits a fraction of the stable sequents.
    #[test]
    fn bias_option() {
        let (sequent, copies) = crate::families::counter(8, false);
        let forest = Forest::new(&sequent).unwrap();
        let run = |options: &Options| {
            let fragment = sequent.fragment();
            search(
                &forest,
                fragment,
                Mode::CLASSICAL,
                None,
                options,
                &mut || false,
            )
        };
        let backward = Options::default().copies(copies);
        let (verdict, slow) = run(&backward);
        assert!(verdict.proof().is_some());
        let forward = backward.bias(Bias::Factors);
        let (verdict, _) = run(&forward);
        assert!(matches!(verdict, Verdict::Unknown(Reason::CopyBound(_))));
        let (verdict, fast) = run(&forward.copies(7));
        let proof = verdict.proof().expect("seven steps on one branch");
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
        assert!(fast.nodes * 10 < slow.nodes, "{fast:?} against {slow:?}");
    }

    /// A split search whose splits all fail in focus visits no stable
    /// sequent and still stops when asked: with weakening no count cuts
    /// the 2⁴² splits of this context, and each fails at once, since no
    /// member is the `~p` its left side wants.
    #[test]
    fn stops_inside_a_split_search() {
        let input = format!(
            "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
            wide_context()
        );
        let s: Sequent = input.parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let mut polls = 0;
        let (verdict, statistics) = search(
            &forest,
            s.fragment(),
            Mode::CLASSICAL.affine(),
            None,
            &Options::default(),
            &mut || {
                polls += 1;
                polls > 3
            },
        );
        assert!(matches!(verdict, Verdict::Unknown(Reason::Stopped)));
        assert_eq!(statistics.nodes, 1);
        assert!(statistics.splits <= 4 * SPLITS_PER_POLL, "{statistics:?}");
    }

    /// Intuitionistic mode: the textbook sequents of ILL, the pitfalls of
    /// the spec (`0` on the left proves anything, `⊤` on the left is inert,
    /// promotion needs an empty linear context) and the sequent classical
    /// linear logic proves but intuitionistic linear logic does not.
    #[test]
    fn intuitionistic() {
        let i = Mode::INTUITIONISTIC;
        for (input, expected) in [
            ("a |- a", true),
            ("|- a -o a", true),
            ("a, a -o b |- b", true),
            ("a -o b, b -o c |- a -o c", true),
            ("a -o b |- b -o a", false),
            ("a * b |- b * a", true),
            ("a * b |- a", false),
            ("a |- a * a", false),
            ("(a * b) -o c |- a -o b -o c", true),
            ("a -o b -o c |- (a * b) -o c", true),
            ("a & b |- a", true),
            ("a & b |- a * b", false),
            ("a * b |- a & b", false),
            ("a |- a + b", true),
            ("a + b |- a", false),
            ("a + b |- b + a", true),
            ("a & (b + c) |- (a & b) + (a & c)", false),
            ("a * (b + c) |- (a * b) + (a * c)", true),
            ("(a -o b) -o a |- a", false),
            ("|- ((a -o b) -o a) -o a", false),
            ("(a -o 0) -o 0 |- a", false),
            ("|- 1", true),
            ("1 |- 1", true),
            ("a |- 1", false),
            ("1, a |- a", true),
            ("|- top", true),
            ("a |- top", true),
            ("a |- 0", false),
            // `0` on the left proves anything, `⊤` on the left is inert.
            ("0 |- a", true),
            ("0, b |- a", true),
            ("a -o 0, a |- b", true),
            ("a -o 0 |- a -o b", true),
            ("top |- a", false),
            ("top |- top", true),
            ("a, top |- a", false),
            ("top, 0 |- a", true),
            ("a -o top, a |- b", false),
            // Ambiguous roots read with the last as the goal: ⊤ ⊢ ⊤.
            ("|- 0, top", true),
            // Promotion needs an empty linear context.
            ("!a |- !a", true),
            ("a |- !a", false),
            ("!a, b |- !a", false),
            ("!a, !b |- !a", true),
            ("!a |- a * a", true),
            ("!a, !(a -o b) |- !b", true),
            ("!(a & b) |- !a * !b", true),
            ("!a * !b |- !(a & b)", true),
            ("!a, !(a -o b & c) |- b * c", true),
            ("!(a -o top), a |- b", false),
            ("!a, !(a -o 0) |- b", true),
        ] {
            assert_eq!(provable(input, i), expected, "{input:?}");
        }
        // Classical linear logic is not conservative over ILL with `0`: the
        // classical proof splits the `⊸L` with the goal on the antecedent's
        // side, which the two-sided search never does.
        let schellinx = "((a * top) & (b * top)) -o 0 |- (a -o c) + (b -o c)";
        assert!(provable(schellinx, Mode::CLASSICAL));
        assert!(!provable(schellinx, i));
        // Affine mode weakens hypotheses at the leaves, never the goal, and
        // lets promotion discard the linear context.
        for (input, expected) in [
            ("a, b |- a", true),
            ("a |- b", false),
            ("a |- 1", true),
            ("!a, b |- !a", true),
            ("a, b |- a * b", true),
            ("top |- a", false),
            ("a, top |- a", true),
            ("a & b |- a", true),
            ("!(a -o a * a), a |- a * a * a", true),
        ] {
            assert_eq!(provable(input, i.affine()), expected, "{input:?}");
        }
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
            ("|- 0, ?top", true),
            ("|- ~b, 0 par (c * (0 + a)), ?(top + b)", true),
            ("|- 0, ?a", false),
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
        // The context grows with every copy, so both modes give up at the
        // bound: a sequent that contains an ancestor is not redundant, and
        // affine mode is bounded like linear mode.
        let growing = "!(a -o a * a), a |- ?b";
        for mode in [Mode::CLASSICAL, affine] {
            assert!(matches!(
                run(growing, mode, &Options::default()).0,
                Verdict::Unknown(Reason::CopyBound(3))
            ));
        }
        // A sequent proved only through a larger one above it.
        for input in [
            "|- ?(a par ~a)",
            "|- ?!1",
            "!(a * ~a) |-",
            "|- ?(a par ~a), b",
        ] {
            assert!(provable(input, affine), "{input:?}");
        }
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
    /// set with or without exponentials, each within the derelictions of
    /// its proof, and decides one mutant of each with and without the memo.
    /// Returns how many sequents and mutants were decided, how many mutants
    /// were provable, how many were undecided within the copy bound, and
    /// the most stable sequents one search visited.
    fn generated(samples: u64, budget: usize, exponentials: bool) -> (u64, u64, u64, u64, u64) {
        let (mut sequents, mut mutants, mut provable_mutants, mut undecided, mut most_nodes) =
            (0, 0, 0, 0, 0);
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            if rules.exponentials != exponentials {
                continue;
            }
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
                // A linear proof is an affine proof.
                let (affine, _) = run(&text, mode.affine(), &options);
                assert!(
                    affine.proof().is_some(),
                    "{text:?} is provable in {} mode within {copies} copies, but the engine \
                     says {affine:?}",
                    mode.affine()
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
    /// within the derelictions of its proof, in every fragment with and
    /// without Mix, and its mutant is decided the same way with and
    /// without the memo.
    #[test]
    fn generated_sequents() {
        let (sequents, mutants, _, undecided, _) = generated(40, 10, false);
        assert_eq!(sequents, 320);
        assert!(mutants > 200, "{mutants} mutants");
        assert_eq!(undecided, 0);
        let (sequents, mutants, _, undecided, _) = generated(40, 10, true);
        assert_eq!(sequents, 320);
        assert!(mutants > 200, "{mutants} mutants");
        assert!(
            undecided < mutants / 4,
            "{undecided} of {mutants} mutants undecided"
        );
    }

    /// Proves `samples` generated intuitionistic sequents of up to `budget`
    /// rules per rule set, each within the derelictions of its proof, in
    /// linear and affine mode; compares the two-sided verdict on each with
    /// the classical engine's on the same one-sided sequent, which must
    /// agree without `0`; and on a mutant of each. Returns how many
    /// sequents and mutants were compared and how many mutants the classical
    /// engine proved that the two-sided one refuted, which only `0` allows.
    fn generated_ill(samples: u64, budget: usize) -> (u64, u64, u64) {
        use crate::search::generate::IllRules;
        let i = Mode::INTUITIONISTIC;
        let (mut sequents, mut mutants, mut non_conservative) = (0, 0, 0);
        for (n, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(100 + n as u64);
            for _ in 0..samples {
                let budget = 2 + rng.below(budget - 1);
                let generate::Ill {
                    mut hypotheses,
                    goal,
                    copies,
                } = generate::ill(&mut rng, rules, 3, budget);
                let text = generate::two_sided(&hypotheses, &goal);
                let options = Options::default().copies(copies);
                let (verdict, _) = run(&text, i, &options);
                assert!(
                    verdict.proof().is_some(),
                    "{text:?} is provable in ILL within {copies} copies, but the engine says \
                     {verdict:?}"
                );
                let (affine, _) = run(&text, i.affine(), &options);
                assert!(affine.proof().is_some(), "{text:?} affine: {affine:?}");
                sequents += 1;
                // The classical engine on the same one-sided sequent.
                let classical = decided(&text, Mode::CLASSICAL, &options);
                assert!(
                    classical != Some(false),
                    "{text:?} is provable in ILL, so classically too, but the engine says \
                     {classical:?}"
                );
                hypotheses.push(goal);
                if generate::mutate(&mut rng, &mut hypotheses, 3) {
                    let goal = hypotheses.pop().unwrap();
                    let text = generate::two_sided(&hypotheses, &goal);
                    let two_sided = decided(&text, i, &Options::default());
                    let classical = decided(&text, Mode::CLASSICAL, &Options::default());
                    match (two_sided, classical) {
                        (Some(true), Some(false)) => panic!("{text:?}: provable in ILL only"),
                        (Some(false), Some(true)) => {
                            assert!(rules.zero, "{text:?}: provable classically only");
                            non_conservative += 1;
                        }
                        _ => {}
                    }
                    mutants += 1;
                }
            }
        }
        (sequents, mutants, non_conservative)
    }

    /// Every generated intuitionistic sequent is proved two-sided with a
    /// checked proof within the derelictions of its proof, in every
    /// intuitionistic fragment, and the classical engine agrees on the
    /// mutants except where `0` makes classical linear logic prove more.
    #[test]
    fn generated_intuitionistic_sequents() {
        let (sequents, mutants, non_conservative) = generated_ill(40, 10);
        assert_eq!(sequents, 480);
        assert!(mutants > 300, "{mutants} mutants");
        assert!(
            non_conservative < mutants / 10,
            "{non_conservative} of {mutants}"
        );
    }

    /// The same on a larger sample of larger proofs, without exponentials:
    /// with them, a few sequents of a sample this size take minutes at the
    /// bound their derelictions give. Run it in release mode and read the
    /// numbers it prints.
    #[test]
    #[ignore = "a larger sample; run with --release -- --ignored --nocapture"]
    fn generated_large_sample() {
        let start = std::time::Instant::now();
        let (sequents, mutants, provable_mutants, _, most_nodes) = generated(500, 24, false);
        println!(
            "{sequents} generated sequents proved, {mutants} mutants decided consistently \
             ({provable_mutants} of them provable), at most {most_nodes} stable sequents per \
             search, in {:.2?}",
            start.elapsed()
        );
    }

    /// Encodes a Horn program with reusable clauses, as the ILLTP library
    /// states Petri-net reachability: every clause `body ⊸ head` (products
    /// of atoms) under a `!`, the initial marking as hypotheses, the goal
    /// marking as the conclusion.
    fn horn(clauses: &[(&str, &str)], marking: &[&str], goal: &[&str]) -> String {
        let mut hypotheses: Vec<String> = clauses
            .iter()
            .map(|(body, head)| format!("!({body} -o {head})"))
            .collect();
        hypotheses.extend(marking.iter().map(|a| (*a).to_string()));
        format!("{} |- {}", hypotheses.join(", "), goal.join(" * "))
    }

    /// The counter program: `n` tokens `a`, two `a` make a `b`, two `b` a
    /// `c`, and so on up the alphabet, with `n` a power of two.
    fn counter(n: usize) -> (Vec<(String, String)>, Vec<&'static str>, &'static str) {
        let levels = n.trailing_zeros() as usize;
        let names = ["a", "b", "c", "d", "e", "f"];
        let clauses = (0..levels)
            .map(|i| {
                (
                    format!("{} * {}", names[i], names[i]),
                    names[i + 1].to_string(),
                )
            })
            .collect();
        (clauses, vec!["a"; n], names[levels])
    }

    /// Horn problems over reusable clauses: a chain of implications takes
    /// one copy per clause on one branch, the counter program reaches its
    /// goal within the copies its firings take and not below, an
    /// unreachable marking is undecided within the bound in either mode,
    /// and affine mode reaches a goal that leaves a token over.
    #[test]
    fn horn_programs() {
        let m = Mode::CLASSICAL;
        let chain: Vec<(String, String)> = (0..6)
            .map(|i| (format!("x{i}"), format!("x{}", i + 1)))
            .collect();
        let chain: Vec<(&str, &str)> = chain.iter().map(|(b, h)| (&**b, &**h)).collect();
        let text = horn(&chain, &["x0"], &["x6"]);
        assert!(
            run(&text, m, &Options::default().copies(6))
                .0
                .proof()
                .is_some()
        );
        assert!(matches!(
            run(&text, m, &Options::default().copies(5)).0,
            Verdict::Unknown(Reason::CopyBound(5))
        ));

        let (clauses, marking, goal) = counter(4);
        let clauses: Vec<(&str, &str)> = clauses.iter().map(|(b, h)| (&**b, &**h)).collect();
        assert!(provable(&horn(&clauses, &marking, &[goal]), m));
        assert!(matches!(
            run(
                &horn(&clauses, &marking, &[goal, "a"]),
                m,
                &Options::default()
            )
            .0,
            Verdict::Unknown(Reason::CopyBound(3))
        ));
        assert!(matches!(
            run(
                &horn(&clauses, &marking, &[goal, "a"]),
                m.affine(),
                &Options::default()
            )
            .0,
            Verdict::Unknown(Reason::CopyBound(3))
        ));
        let mut five = marking.clone();
        five.push("a");
        assert!(matches!(
            run(&horn(&clauses, &five, &[goal]), m, &Options::default()).0,
            Verdict::Unknown(Reason::CopyBound(3))
        ));
        assert!(provable(&horn(&clauses, &five, &[goal]), m.affine()));
    }

    /// A 3-Partition instance with a solution is proved: the first bin
    /// choices work out, so the search is short.
    #[test]
    fn three_partition_solved() {
        let yes = crate::families::three_partition(&[1, 2, 3, 1, 2, 3], 2, 6).to_string();
        assert!(provable(&yes, Mode::CLASSICAL), "{yes}");
    }
}

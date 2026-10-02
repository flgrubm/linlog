// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused engine on several threads: cube-and-conquer over the
//! choices near the root, and-parallel `&` premises, and one memo and one
//! proof arena shared by every worker.
//!
//! A choice among alternatives (the focus on a stable sequent, the side
//! of a `⊕`, the splits of a `⊗`) within the first [`LEVELS`] such
//! choices of a branch runs its alternatives as tasks of the pool: the
//! engine that meets the choice spawns a worker for each alternative but
//! the first, a copy of its own state at that point (the branch stack,
//! the nesting depth, the budget) with pools of its own, which is the
//! per-worker scratch, and runs the first alternative on one more worker
//! on its own thread; the first alternative to succeed raises the
//! choice's flag, which every worker of the choice polls, and the others
//! stop at their next stable sequent. Below those levels every worker runs the sequential
//! engine, so the tasks are few and large. The `&` rule within the same
//! levels runs its two premises the same way, the first to fail
//! cancelling the other. The levels of the copy bound never overlap:
//! `run` deepens on the root engine, which is alone until its first
//! choice, and every task of a level has ended when the level's answer is
//! read.

use super::classes::Classes;
use super::context::Context;
use super::counts::{Counts, Split};
use super::memo::{Key, Shared, Table};
use super::{Arena, Engine, Join, Kept, NO_DEPENDENCY, Rules, Search};
use crate::fragment::{Fragment, Mode};
use crate::occurrences::{Forest, OccId, OccSet, Reading};
use crate::proofs::{Node, NodeId, Side};
use crate::search::parallel::{Flags, Runtime};
use crate::search::{Options, Reason, Statistics, Stop};
use std::hash::BuildHasher as _;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// How many choices of a branch, from the root, run their alternatives on
/// several threads.
const LEVELS: u32 = 2;

/// The most members of a context whose assignment a split fixes per
/// task when the splits run on several threads: 2⁶ tasks at most.
const MAX_FIXED: usize = 6;

/// Runs the focused engine on the runtime's pool from a goal, as
/// [`super::search_goal`] does on one thread, polling `stop` on the
/// calling thread while the pool searches. Returns the node proving the
/// goal, the shared arena and the statistics of every worker together.
#[allow(clippy::too_many_arguments)]
pub(crate) fn search_goal(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    options: &Options,
    runtime: &Runtime,
    stop: &mut dyn FnMut() -> bool,
) -> (Search, Vec<Node>, Statistics) {
    let counts = Counts::new(forest);
    let classes = Classes::new(forest, reading);
    let rules = Rules::new(fragment, mode, &counts);
    let memo = Shared::new(options.memo_limit);
    let arena = Mutex::new(Vec::new());
    let (result, statistics) = runtime.drive(stop, |flags| {
        let mut engine = Engine::new(
            forest,
            rules,
            reading,
            (&counts, &classes),
            options,
            Stop::Flags(flags),
            Table::Shared(&memo),
            Arena::new(Kept::Shared(&arena)),
        );
        engine.runtime = Some(runtime);
        let result = engine.run(goal);
        let result = engine.exported(result);
        (result, engine.statistics())
    });
    let nodes = arena
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    (result, nodes, statistics)
}

/// What a worker starts from: the state of the engine that spawns it at
/// a parallel choice, the shared parts by reference and the branch's by
/// copy.
struct Spawn<'s> {
    /// The problem.
    forest: &'s Forest,
    /// Its intuitionistic reading.
    reading: Option<&'s Reading<'s>>,
    /// Its count invariants.
    counts: &'s Counts,
    /// Its classes of interchangeable occurrences.
    classes: &'s Classes,
    /// The rules in force.
    rules: Rules,
    /// The shared memo.
    memo: &'s Shared,
    /// The shared arena.
    arena: &'s Mutex<Vec<Node>>,
    /// Whether the memo takes entries.
    memoizes: bool,
    /// The runtime.
    runtime: &'s Runtime,
    /// The spawning engine's stop flags, which the worker's chain to.
    flags: Flags<'s>,
    /// The live branch stack at the choice.
    stack: &'s [Key],
    /// The nesting of engine calls at the choice.
    depth: u32,
    /// The deepest nesting allowed.
    recursion_limit: u32,
    /// The copy bound of the last level.
    copies: u32,
    /// The workers' levels of cube-and-conquer.
    or_depth: u32,
    /// The spawning engine's seed.
    seed: u64,
    /// Whether the workers get seeds of their own.
    portfolio: bool,
}

impl<'s> Spawn<'s> {
    /// Starts a worker: a fresh engine on the spawn's state, stopped by
    /// the spawn's flags or by `cancel`, and, in a portfolio, with a seed
    /// of its own derived from its `index` among the choice's
    /// alternatives, the first alternative (index zero) keeping the
    /// spawning engine's seed.
    fn worker<'w>(&'w self, cancel: &'w AtomicBool, index: u64) -> Engine<'w> {
        let mut worker = Engine {
            forest: self.forest,
            reading: self.reading,
            counts: self.counts,
            classes: self.classes,
            rules: self.rules,
            memo: Table::Shared(self.memo),
            nodes: Arena::new(Kept::Shared(self.arena)),
            memoizes: self.memoizes,
            statistics: Statistics::default(),
            steps: 0,
            depth: self.depth,
            recursion_limit: self.recursion_limit,
            copies: self.copies,
            exhausted: false,
            dependency: NO_DEPENDENCY,
            stop: Stop::Flags(self.flags.child(cancel)),
            runtime: Some(self.runtime),
            or_depth: self.or_depth,
            seed: self.seed,
            portfolio: self.portfolio,
            stack: self.stack.to_vec(),
            hashes: self
                .stack
                .iter()
                .map(|key| crate::hash::BuildHasher::default().hash_one(key))
                .collect(),
            stack_len: self.stack.len(),
            sets: Vec::new(),
            contexts: Vec::new(),
            keys: Vec::new(),
            lists: Vec::new(),
            tallies: Vec::new(),
            splits: Vec::new(),
            trails: Vec::new(),
            links: Vec::new(),
        };
        if self.portfolio && index != 0 {
            // A seed per worker, never zero.
            worker.seed = (self.seed ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15)) | 1;
        }
        worker
    }
}

/// An alternative of a parallel choice.
#[derive(Clone, Copy)]
enum Alternative<'m> {
    /// A focus on a member of `Γ`, which leaves the context.
    Focus(OccId),
    /// A copy of a member of `Θ` in focus, at one unit less of the budget.
    Copy(OccId),
    /// A side of a `⊕` in focus, with the subformula on that side.
    Side(OccId, Side, OccId),
    /// The free splits of a `⊗` that assign the first `fixed` members as
    /// the bits of `pattern` say, one for the left premise, from the sides
    /// and counts before any member moved.
    Splits {
        /// The `⊗` rule the sides are the premises of.
        join: Join,
        /// The members of the context that the search assigns.
        members: &'m [OccId],
        /// How many of them the pattern assigns.
        fixed: usize,
        /// Their assignment.
        pattern: u64,
        /// The sides before any member moved.
        sides: (&'m Context, &'m Context),
        /// Their counts, the members open.
        split: &'m Split,
    },
}

/// What the workers of a choice among alternatives report.
#[derive(Default)]
struct Collected {
    /// The first proof found.
    proof: Option<NodeId>,
    /// The first error, a stop that a cancellation caused excepted, a
    /// stop giving way to any other reason.
    error: Option<Reason>,
    /// Whether some failed alternative was cut by the copy budget.
    exhausted: bool,
    /// The shallowest ancestor a prune below a failed alternative relied
    /// on.
    dependency: u32,
    /// The workers' counters.
    statistics: Statistics,
}

impl Collected {
    /// Nothing reported yet.
    fn new() -> Self {
        Self {
            dependency: NO_DEPENDENCY,
            ..Self::default()
        }
    }

    /// Takes a worker's result: a proof or an error settles the choice
    /// and raises its flag, a failure merges the worker's flags.
    fn take(&mut self, result: Search, worker: &Engine<'_>, cancel: &AtomicBool) {
        self.statistics.add(&worker.statistics);
        match result {
            Ok(Some(node)) => {
                self.proof.get_or_insert(node);
                cancel.store(true, Ordering::Relaxed);
            }
            Ok(None) => {
                self.exhausted |= worker.exhausted;
                self.dependency = self.dependency.min(worker.dependency);
            }
            Err(Reason::Stopped) if cancel.load(Ordering::Relaxed) => {}
            Err(reason) => {
                if self.error.is_none_or(|old| old == Reason::Stopped) {
                    self.error = Some(reason);
                }
                cancel.store(true, Ordering::Relaxed);
            }
        }
    }
}

/// A premise of a `&` searched on a worker.
struct Premise {
    /// The result.
    result: Search,
    /// Whether the search was cut by the copy budget.
    exhausted: bool,
    /// The shallowest ancestor a prune below relied on.
    dependency: u32,
    /// The worker's counters.
    statistics: Statistics,
}

impl Premise {
    /// What a worker found on a premise, with its flags and counters.
    fn of(worker: &Engine<'_>, result: Search) -> Self {
        Self {
            result,
            exhausted: worker.exhausted,
            dependency: worker.dependency,
            statistics: worker.statistics,
        }
    }
}

impl<'a> Engine<'a> {
    /// Whether the next choice of the branch runs on several threads.
    pub(super) fn cubes(&self) -> bool {
        self.runtime.is_some() && self.or_depth < LEVELS
    }

    /// The state a worker starts from, with the branch stack given and
    /// `or_depth` levels of cube-and-conquer above it.
    fn spawn<'s>(&self, stack: &'s [Key], or_depth: u32) -> Spawn<'s>
    where
        'a: 's,
    {
        let (Table::Shared(memo), Kept::Shared(arena), Stop::Flags(flags), Some(runtime)) =
            (&self.memo, &self.nodes.kept, &self.stop, self.runtime)
        else {
            unreachable!("a parallel choice is met on a worker of the pool")
        };
        Spawn {
            forest: self.forest,
            reading: self.reading,
            counts: self.counts,
            classes: self.classes,
            rules: self.rules,
            memo,
            arena,
            memoizes: self.memoizes,
            runtime,
            flags: *flags,
            stack,
            depth: self.depth,
            recursion_limit: self.recursion_limit,
            copies: self.copies,
            or_depth,
            seed: self.seed,
            portfolio: self.portfolio,
        }
    }

    /// A result as it leaves this engine for another: the proof's pending
    /// nodes kept, since a pending id means nothing outside its engine.
    fn exported(&mut self, result: Search) -> Search {
        result.map(|node| node.map(|node| self.nodes.keep(0, node)))
    }

    /// Locks what the workers report.
    fn lock<T>(shared: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Runs one alternative of a choice on this engine.
    fn alternative(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternative: Alternative<'_>,
        budget: u32,
    ) -> Search {
        match alternative {
            Alternative::Focus(f) => {
                let mut rest = self.take_context();
                rest.clone_from(gamma);
                rest.remove(f);
                let result = self.focus(theta, &rest, f, budget);
                self.give_context(rest);
                result
            }
            Alternative::Copy(a) => Ok(self
                .focus(theta, gamma, a, budget - 1)?
                .map(|node| self.push(Node::Copy(a, node)))),
            Alternative::Side(f, side, sub) => Ok(self
                .focus(theta, gamma, sub, budget)?
                .map(|node| self.push(Node::Plus(f, side, node)))),
            Alternative::Splits {
                join,
                members,
                fixed,
                pattern,
                sides,
                split,
            } => {
                let mut left = self.take_context();
                left.clone_from(sides.0);
                let mut right = self.take_context();
                right.clone_from(sides.1);
                let mut counts = self.take_split();
                counts.clone_from(split);
                for (i, &m) in members.iter().enumerate().take(fixed) {
                    if pattern >> i & 1 == 1 {
                        right.remove(m);
                        left.insert(m);
                        counts.assign(self.counts, m, Side::Left);
                    } else {
                        counts.assign(self.counts, m, Side::Right);
                    }
                }
                let result = self.search_splits(
                    theta,
                    members,
                    (fixed, pattern),
                    (&mut left, &mut right),
                    &mut counts,
                    join,
                    budget,
                );
                self.give_context(left);
                self.give_context(right);
                self.give_split(counts);
                result
            }
        }
    }

    /// Decides a choice among alternatives on the pool: a worker per
    /// alternative, the first one on this thread, and the first to
    /// succeed or to fail with an error cancels the rest. Returns the
    /// proof, or `None` when every alternative failed, their flags merged
    /// into this engine's, or the first error (a stop a cancellation
    /// caused is none). A proof found by any alternative wins over an
    /// error of another, so the pool may decide where one thread gives
    /// up.
    fn choose_parallel(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternatives: &[Alternative<'_>],
        budget: u32,
    ) -> Search {
        let cancel = AtomicBool::new(false);
        let stack = self.stack[..self.stack_len].to_vec();
        let spawn = self.spawn(&stack, self.or_depth + 1);
        let collected = Mutex::new(Collected::new());
        let (&first, rest) = alternatives
            .split_first()
            .expect("a choice has an alternative");
        spawn.runtime.pool.in_place_scope(|scope| {
            for (i, &alternative) in rest.iter().enumerate() {
                let (spawn, cancel, collected) = (&spawn, &cancel, &collected);
                scope.spawn(move |_| {
                    let mut worker = spawn.worker(cancel, i as u64 + 1);
                    let result = worker.alternative(theta, gamma, alternative, budget);
                    let result = worker.exported(result);
                    Self::lock(collected).take(result, &worker, cancel);
                });
            }
            // The first alternative on this thread, on a worker of its
            // own so that it polls the choice's flag like the others.
            let mut worker = spawn.worker(&cancel, 0);
            let result = worker.alternative(theta, gamma, first, budget);
            let result = worker.exported(result);
            Self::lock(&collected).take(result, &worker, &cancel);
        });
        let collected = collected
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.statistics.add(&collected.statistics);
        match (collected.proof, collected.error) {
            (Some(node), _) => Ok(Some(node)),
            (None, Some(reason)) => Err(reason),
            (None, None) => {
                self.exhausted |= collected.exhausted;
                self.dependency = self.dependency.min(collected.dependency);
                Ok(None)
            }
        }
    }

    /// The choice of a focus on a stable sequent on the pool: every
    /// candidate and every copy as an alternative. `None` when the choice
    /// is not one to run on the pool, or has at most one alternative,
    /// which the sequential loops handle as well.
    pub(super) fn choices_parallel(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        candidates: &[OccId],
        copies: &[OccId],
        budget: u32,
    ) -> Option<Search> {
        if !self.cubes() || candidates.len() + copies.len() < 2 {
            return None;
        }
        let alternatives: Vec<Alternative<'_>> = candidates
            .iter()
            .map(|&f| Alternative::Focus(f))
            .chain(copies.iter().map(|&a| Alternative::Copy(a)))
            .collect();
        Some(self.choose_parallel(theta, gamma, &alternatives, budget))
    }

    /// The `⊕` rule on the pool: its two sides as alternatives.
    pub(super) fn plus_parallel(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        f: OccId,
        budget: u32,
    ) -> Search {
        let alternatives = [
            Alternative::Side(f, Side::Left, self.forest.left(f).unwrap()),
            Alternative::Side(f, Side::Right, self.forest.right(f).unwrap()),
        ];
        self.choose_parallel(theta, gamma, &alternatives, budget)
    }

    /// The free splits of `F = A ⊗ B` on the pool: the assignments of the
    /// first few members of the context, as many as give the pool twice
    /// its threads in tasks (at most [`MAX_FIXED`]), as alternatives, each
    /// searching the assignments of the rest. The sides and counts given
    /// are the state before any member moved.
    pub(super) fn split_parallel(
        &mut self,
        theta: &OccSet,
        members: &[OccId],
        sides: (&Context, &Context),
        split: &Split,
        join: Join,
        budget: u32,
    ) -> Search {
        let threads = self.runtime.map_or(1, Runtime::threads);
        let bits = (2 * threads).next_power_of_two().trailing_zeros() as usize;
        let fixed = bits.clamp(1, MAX_FIXED).min(members.len());
        // Of interchangeable members the lowest ids go left, and they come
        // last among their like: a pattern that sends one left and the
        // next one right is none of the splits searched.
        let canonical = |pattern: u64| {
            (1..fixed).all(|i| {
                pattern >> (i - 1) & 1 == 0
                    || pattern >> i & 1 == 1
                    || !self.classes.same(members[i - 1], members[i])
            })
        };
        let alternatives: Vec<Alternative<'_>> = (0..1u64 << fixed)
            .filter(|&pattern| canonical(pattern))
            .map(|pattern| Alternative::Splits {
                join,
                members,
                fixed,
                pattern,
                sides,
                split,
            })
            .collect();
        self.choose_parallel(theta, sides.1, &alternatives, budget)
    }

    /// The `&` rule on the pool: the left premise on a worker on this
    /// thread, the right one on a worker of the pool, the first to fail
    /// cancelling the other. The flags of a premise count when it ran to
    /// its end and the other did not fail before it, as in the sequential
    /// rule, where the right premise runs only after the left one
    /// succeeded.
    pub(super) fn with_parallel(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        list: &[OccId],
        o: OccId,
        budget: u32,
    ) -> Search {
        let cancel = AtomicBool::new(false);
        let stack = self.stack[..self.stack_len].to_vec();
        let spawn = self.spawn(&stack, self.or_depth);
        let premise: Mutex<Option<Premise>> = Mutex::new(None);
        let (left_sub, right_sub) = (self.forest.left(o).unwrap(), self.forest.right(o).unwrap());
        // A premise on a worker: the context and the list with the
        // subformula on top, as the sequential rule sets them up.
        let search = |worker: &mut Engine<'_>, sub: OccId| {
            let mut premise_gamma = worker.take_context();
            premise_gamma.clone_from(gamma);
            let mut premise_list = worker.take_list();
            premise_list.extend_from_slice(list);
            premise_list.push(sub);
            let result = worker.asynchronous(theta, &mut premise_gamma, &mut premise_list, budget);
            let result = worker.exported(result);
            if !matches!(result, Ok(Some(_))) {
                cancel.store(true, Ordering::Relaxed);
            }
            result
        };
        let left = spawn.runtime.pool.in_place_scope(|scope| {
            let (spawn, cancel, premise, search) = (&spawn, &cancel, &premise, &search);
            scope.spawn(move |_| {
                let mut worker = spawn.worker(cancel, 1);
                let result = search(&mut worker, right_sub);
                *Self::lock(premise) = Some(Premise::of(&worker, result));
            });
            let mut worker = spawn.worker(cancel, 0);
            let result = search(&mut worker, left_sub);
            Premise::of(&worker, result)
        });
        let right = premise
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .expect("the worker reports before the scope ends");
        self.statistics.add(&left.statistics);
        self.statistics.add(&right.statistics);
        let merge = |engine: &mut Self, premise: &Premise| {
            engine.exhausted |= premise.exhausted;
            engine.dependency = engine.dependency.min(premise.dependency);
        };
        match (left.result, right.result) {
            (Ok(Some(l)), Ok(Some(r))) => {
                merge(self, &left);
                merge(self, &right);
                Ok(Some(self.push(Node::With(o, l, r))))
            }
            (Ok(None), _) => {
                merge(self, &left);
                Ok(None)
            }
            (_, Ok(None)) => {
                merge(self, &right);
                Ok(None)
            }
            (Err(reason), _) | (Ok(Some(_)), Err(reason)) => Err(reason),
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::fragment::Mode;
    use crate::search::generate::{self, IllRules, Rng, Rules};
    use crate::search::{Engine, Options, Reason, Verdict, prove, prove_until};
    use crate::sequents::Sequent;

    /// The verdict of `input` under `mode` with `options` as a three-way
    /// value: provable, unprovable, or undecided within the copy bound;
    /// every proof checked.
    fn decided(input: &str, mode: Mode, options: &Options) -> Option<bool> {
        let sequent: Sequent = input.parse().unwrap();
        let outcome = prove(&sequent, mode, options).unwrap();
        match outcome.verdict {
            Verdict::Proved(proof) => {
                assert_eq!(proof.check(mode), Ok(()), "{input:?} with {options:?}");
                Some(true)
            }
            Verdict::Unprovable => Some(false),
            Verdict::Unknown(Reason::CopyBound(_)) => None,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// The verdicts on two and four threads, with and without the
    /// portfolio, never contradict the sequential one, and a sequent is
    /// proved on the pool exactly when it is proved sequentially: the
    /// pool searches the same levels of the copy bound to their end. Only
    /// decisiveness within the bound may differ, as the memo's contents
    /// do.
    fn agree(text: &str, mode: Mode, options: &Options) {
        let sequential = decided(text, mode, options);
        for (jobs, portfolio) in [(2, false), (4, false), (4, true)] {
            let options = options.clone().jobs(jobs).portfolio(portfolio);
            let parallel = decided(text, mode, &options);
            assert!(
                sequential.is_none() || parallel.is_none() || sequential == parallel,
                "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} with {options:?}"
            );
            assert_eq!(
                sequential == Some(true),
                parallel == Some(true),
                "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} with {options:?}"
            );
        }
    }

    /// Generated classical sequents and their mutants, in every rule set,
    /// with the engine the dispatch picks and with the focused engine
    /// forced, so that unit-free MLL goes to both engines' pools.
    #[test]
    fn classical_verdicts_agree() {
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mode = if rules.mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let mut rng = Rng::new(100 + i as u64);
            for _ in 0..10 {
                let budget = 2 + rng.below(9);
                let generate::Provable {
                    mut formulas,
                    copies,
                } = generate::provable(&mut rng, rules, 3, budget);
                let texts = [
                    generate::sequent(&formulas),
                    generate::sequent(if generate::mutate(&mut rng, &mut formulas, 3) {
                        &formulas
                    } else {
                        &[]
                    }),
                ];
                for text in texts.iter().filter(|t| !t.is_empty()) {
                    for engine in [None, Some(Engine::Focus)] {
                        let options = Options::default().copies(copies).engine(engine);
                        agree(text, mode, &options);
                    }
                }
            }
        }
    }

    /// Generated intuitionistic sequents and their mutants, in every rule
    /// set, linear and affine.
    #[test]
    fn intuitionistic_verdicts_agree() {
        for (i, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(200 + i as u64);
            for _ in 0..8 {
                let budget = 2 + rng.below(9);
                let generate::Ill {
                    mut hypotheses,
                    goal,
                    copies,
                } = generate::ill(&mut rng, rules, 3, budget);
                let mut texts = vec![generate::two_sided(&hypotheses, &goal)];
                if generate::mutate(&mut rng, &mut hypotheses, 3) {
                    texts.push(generate::two_sided(&hypotheses, &goal));
                }
                for text in &texts {
                    let options = Options::default().copies(copies);
                    agree(text, Mode::INTUITIONISTIC, &options);
                    agree(text, Mode::INTUITIONISTIC.affine(), &options);
                }
            }
        }
    }

    /// Eleven tensor pairs under Mix, whose refutation takes minutes: the
    /// caller's stop condition, polled on the calling thread, stops every
    /// worker, inside a search for splits too.
    #[test]
    fn stops() {
        let sequent = crate::families::mix(11);
        let options = Options::default().jobs(4);
        let mut polls = 0;
        let outcome = prove_until(&sequent, Mode::CLASSICAL.with_mix(), &options, || {
            polls += 1;
            polls > 20
        })
        .unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)),
            "{:?}",
            outcome.verdict
        );
        assert!(outcome.statistics.nodes > 0);

        // A split search that visits no stable sequent stops as well.
        let input = format!(
            "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
            crate::search::focus::tests::wide_context()
        );
        let sequent: Sequent = input.parse().unwrap();
        let mut polls = 0;
        let outcome = prove_until(&sequent, Mode::CLASSICAL.affine(), &options, || {
            polls += 1;
            polls > 20
        })
        .unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)),
            "{:?}",
            outcome.verdict
        );
    }
}

// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof search: the front door `prove` with its options and outcome, the
//! dispatch on fragment and mode, and one submodule per engine: proof-net
//! search for unit-free MLL, the focused sequent engine for everything
//! else, one-sided in classical mode and two-sided in intuitionistic mode,
//! and the additive fast path for two additive-only formulas.

/// The additive fast path.
pub mod additive;
/// The focused sequent engine.
pub mod focus;
/// Random provable sequents for the tests.
#[cfg(test)]
pub(crate) mod generate;
/// The count of the bytes a search holds.
pub(crate) mod memory;
/// The proof-net engine.
pub mod net;
#[cfg(feature = "parallel")]
mod parallel;

use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::nets::ProofStructure;
use crate::occurrences::{Bias, Forest, OccId, Reading};
use crate::proofs::{Bytes, Proof};
use crate::sequents::{Atom, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// An engine's stop condition: the caller's closure in a sequential
/// search, the chain of stop flags of a worker in a parallel one.
pub(crate) enum Stop<'a> {
    /// The caller's condition.
    Closure(&'a mut dyn FnMut() -> bool),
    /// The caller's condition, and an amount of work after which the
    /// search stops as well: one turn of a search that takes turns with
    /// another.
    Turn(&'a mut dyn FnMut() -> bool, u64),
    /// A condition that is told, at every poll, whether a slice of work
    /// (the last field) has passed since it was last told so: one of two
    /// searches that alternate, which gives way to the other there.
    Slice(&'a mut dyn FnMut(bool) -> bool, u64, u64),
    /// A worker's flags.
    #[cfg(feature = "parallel")]
    Flags(parallel::Flags<'a>),
}

impl Stop<'_> {
    /// Polls the condition after `work` more units of work, in the unit
    /// the engine that polls counts its turns in.
    pub(crate) fn fired(&mut self, work: u64) -> bool {
        match self {
            Self::Closure(stop) => stop(),
            Self::Turn(stop, left) => {
                *left = left.saturating_sub(work);
                *left == 0 || stop()
            }
            Self::Slice(stop, left, slice) => {
                *left = left.saturating_sub(work);
                let passed = *left == 0;
                if passed {
                    *left = *slice;
                }
                stop(passed)
            }
            #[cfg(feature = "parallel")]
            Self::Flags(flags) => flags.raised(),
        }
    }
}

/// The occurrences a forest must have for a search to poll the stop
/// condition between the passes that set it up: a pass over fewer takes
/// under a millisecond, and a condition that counts its polls then sees
/// the engine's own and no others.
const SET_UP_POLL: usize = 1 << 16;

/// Polls the caller's stop condition between two passes over a forest
/// that set a search up, when the forest is large enough for a pass to
/// take time ([`SET_UP_POLL`]): on millions of occurrences the set-up
/// takes a second, and the engine's first poll comes after it.
pub(crate) fn set_up_stopped(forest: &Forest, stop: &mut dyn FnMut() -> bool) -> bool {
    forest.len() >= SET_UP_POLL && stop()
}

/// Decides a sequent under a mode with the engine its fragment calls for,
/// and returns the outcome: the verdict with a proof if there is one, the
/// fragment detected, the engine used and the statistics of the run. The
/// search runs to completion; [`prove_until`] takes a stop condition.
///
/// # Errors
///
/// A sequent outside the fragment the options assert is refused
/// ([`Error::FragmentMismatch`]); in intuitionistic mode a sequent with no
/// intuitionistic reading ([`Error::NotIntuitionistic`]) and Mix
/// ([`Error::IntuitionisticMix`]); the net engine outside unit-free MLL
/// ([`Error::NetFragment`]) and in affine mode ([`Error::NetMode`]); the
/// focus engine in intuitionistic mode and the two-sided engine in
/// classical mode ([`Error::EngineMode`]); the additive engine on anything
/// but two additive-only formulas ([`Error::NotAdditive`]); and a sequent
/// that unfolds to more subformula occurrences than
/// [`Options::occurrence_limit`] allows ([`Error::TooManyOccurrences`]). A proof that the checker rejects is
/// [`Error::Rejected`], and one whose check would hold more than
/// [`Options::memory_limit`] is [`Error::Unchecked`]: every proof returned
/// has passed the checker, unless [`Options::check`] says otherwise.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Engine, Options, Verdict, prove};
/// use linlog::{Fragment, Mode, Sequent};
///
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
/// assert_eq!(outcome.fragment, Fragment::MLL);
/// assert_eq!(outcome.engine, Engine::Net);
/// assert!(outcome.net.is_some(), "the net engine returns the net it found");
/// let Verdict::Proved(proof) = outcome.verdict else {
///     panic!("provable");
/// };
/// assert_eq!(
///     proof.derivation()?.to_string(),
///     "─────── ax   ─────── ax\n\
///      ⊢ ~A, A      ⊢ ~B, B\n\
///      ──────────────────── ⊗\n\
///     \x20 ⊢ ~A, A ⊗ ~B, B"
/// );
///
/// let sequent: Sequent = "|- A par B, ~A, ~B".parse()?;
/// let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
/// assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
/// let outcome = prove(&sequent, Mode::CLASSICAL.with_mix(), &Options::default())?;
/// assert!(matches!(outcome.verdict, Verdict::Proved(_)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove(sequent: &Sequent, mode: Mode, options: &Options) -> Result<Outcome, Error> {
    prove_until(sequent, mode, options, || false)
}

/// Decides a sequent as [`prove`] does, polling `stop` and giving up with
/// [`Reason::Stopped`] once it returns true. The condition is the
/// caller's: a deadline on a clock the caller has, a flag an interrupt
/// handler sets. This crate has no clock of its own. The engines poll at
/// every stable sequent or literal chosen and inside every loop that can
/// run long between two of them, and on a forest of tens of thousands of
/// occurrences also between the passes that set the search up; what is
/// not polled is the building of the forest before the search, the check
/// of the proof after it, and single passes over the forest, each linear
/// in it. A condition that is cheap to ask is asked often enough: one
/// that reads a clock only every so many polls is late by that many
/// polls, and on a large sequent a poll can be many milliseconds from the
/// last.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Options, Reason, Verdict, prove_until};
/// use linlog::{Mode, Sequent};
/// use std::time::{Duration, Instant};
///
/// let sequent: Sequent = "|- (a & b) + (a & c), ~a par (~b & ~c)".parse()?;
/// let deadline = Instant::now() + Duration::from_secs(10);
/// let outcome = prove_until(&sequent, Mode::CLASSICAL, &Options::default(), || {
///     Instant::now() >= deadline
/// })?;
/// assert!(!matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove_until(
    sequent: &Sequent,
    mode: Mode,
    options: &Options,
    stop: impl FnMut() -> bool,
) -> Result<Outcome, Error> {
    let forest = Forest::within(sequent, options.occurrence_limit)?;
    prove_goal(&forest, forest.roots(), mode, options, stop)
}

/// Decides a goal: a multiset of occurrences of a forest, given in any
/// order, which stands for the sequent of those subformulas. The roots are
/// the sequent itself, and [`prove_until`] is this function on them; any
/// other goal is what an interactive proof leaves open, and the engine that
/// decides it is the one its own fragment calls for, except that the net
/// engine works on the roots only. The proof of a goal other than the
/// roots is a [`Proof`] whose root concludes the goal, so
/// [`Proof::check`], which expects the sequent's roots, rejects it; it is
/// meant to be grafted onto the goal, as the interactive state does.
///
/// # Errors
///
/// Those of [`prove`], plus [`Error::OccurrenceIndexOutOfBounds`] for an
/// occurrence outside the forest, [`Error::GoalOutputs`] for an
/// intuitionistic goal without exactly one formula on the right of `⊢`,
/// and [`Error::NetGoal`] for the net engine forced on a goal other than
/// the roots.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Options, Verdict, prove_goal};
/// use linlog::{Forest, Mode, OccId, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the occurrences 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B. The goal ⊢ ~A, A is the left premise of the ⊗.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let goal = [OccId::new(0), OccId::new(2)];
/// let outcome = prove_goal(&forest, &goal, Mode::CLASSICAL, &Options::default(), || false)?;
/// assert!(matches!(outcome.verdict, Verdict::Proved(_)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove_goal(
    forest: &Forest,
    goal: &[OccId],
    mode: Mode,
    options: &Options,
    mut stop: impl FnMut() -> bool,
) -> Result<Outcome, Error> {
    if let Some(o) = goal.iter().find(|o| o.index() >= forest.len()) {
        return Err(Error::OccurrenceIndexOutOfBounds(o.index(), forest.len()));
    }
    let detected = goal_fragment(forest, goal);
    let fragment = match options.fragment {
        Some(asserted) if !asserted.contains(detected) => {
            return Err(Error::FragmentMismatch { asserted, detected });
        }
        Some(asserted) => asserted,
        None => detected,
    };
    // The dispatch: two additive-only formulas go to the additive path;
    // unit-free MLL with mostly distinct atoms to the net engine (in
    // intuitionistic mode by the embedding of IMLL into MLL); every other
    // fragment, and every fragment in affine mode, to the focused engine,
    // one-sided or two-sided by the mode. Mix has no intuitionistic form.
    if mode.intuitionistic && mode.mix {
        return Err(Error::IntuitionisticMix);
    }
    let reading = if mode.intuitionistic {
        let reading = Reading::new(forest).map_err(Error::NotIntuitionistic)?;
        let outputs = reading.outputs(goal.iter().copied());
        if outputs != 1 {
            return Err(Error::GoalOutputs(outputs));
        }
        Some(reading)
    } else {
        None
    };
    let is_roots = goal == forest.roots();
    let is_mll = Fragment::MLL.contains(fragment);
    // Two formulas of the additive fragment; by default only when some
    // additive occurs, since atoms alone are the net engine's.
    let is_additive = Fragment::ALL.contains(fragment) && goal.len() == 2;
    let engine = options.engine.unwrap_or({
        if is_additive && !fragment.is_empty() {
            Engine::Additive
        } else if is_roots && is_mll && !mode.affine && prefers_net(forest) {
            Engine::Net
        } else if mode.intuitionistic {
            Engine::TwoSided
        } else {
            Engine::Focus
        }
    });
    match engine {
        Engine::Net if !is_mll => return Err(Error::NetFragment(fragment)),
        Engine::Net if mode.affine => return Err(Error::NetMode(mode)),
        Engine::Net if !is_roots => return Err(Error::NetGoal),
        Engine::Focus if mode.intuitionistic => return Err(Error::EngineMode { engine, mode }),
        Engine::TwoSided if !mode.intuitionistic => {
            return Err(Error::EngineMode { engine, mode });
        }
        Engine::Additive if !is_additive => {
            return Err(Error::NotAdditive {
                fragment,
                roots: goal.len(),
            });
        }
        _ => {}
    }
    // The fragment, the reading and the dispatch were passes over the
    // forest: the caller's condition is asked before the engine's own.
    if set_up_stopped(forest, &mut stop) {
        return Ok(Outcome {
            verdict: Verdict::Unknown(Reason::Stopped),
            fragment,
            mode,
            engine,
            statistics: Statistics::default(),
            net: None,
        });
    }
    // What the search allocates is counted against the bound.
    let account = memory::Account::new(options.memory_limit);
    // Several threads run the focused engine and the net engine on pools
    // of their own; the additive path is sequential in every case.
    #[cfg(feature = "parallel")]
    let options = &options.clone().jobs(parallel::threads(options.jobs));
    let (mut verdict, statistics, net) = match engine {
        #[cfg(feature = "parallel")]
        Engine::Net if options.jobs > 1 => {
            let runtime = parallel::Runtime::new(options.jobs, options.stack_size())?;
            net::parallel::search(forest, mode, options, &runtime, &mut stop)
        }
        Engine::Net => net::search(forest, mode, options, &mut stop),
        Engine::Focus | Engine::TwoSided | Engine::Additive => {
            let (result, nodes, statistics) = if engine == Engine::Additive {
                additive::search_goal(forest, goal, options, &account, &mut stop)
            } else {
                #[cfg(feature = "parallel")]
                if options.jobs > 1 {
                    focus::parallel::search_goal(
                        forest,
                        goal,
                        fragment,
                        mode,
                        reading.as_ref(),
                        options,
                        &account,
                        &mut stop,
                    )?
                } else {
                    focus::search_goal(
                        forest,
                        goal,
                        fragment,
                        mode,
                        reading.as_ref(),
                        options,
                        &account,
                        &mut stop,
                    )
                }
                #[cfg(not(feature = "parallel"))]
                focus::search_goal(
                    forest,
                    goal,
                    fragment,
                    mode,
                    reading.as_ref(),
                    options,
                    &account,
                    &mut stop,
                )
            };
            let verdict = match result {
                Ok(Some(root)) => {
                    let proof = Proof::new(forest.clone(), nodes, root)
                        .expect("the engine pushes premises before conclusions");
                    Verdict::Proved(Box::new(proof))
                }
                Ok(None) => Verdict::Unprovable(Refutation::Exhausted),
                Err(reason) => Verdict::Unknown(reason),
            };
            (verdict, statistics, None)
        }
    };
    // A refutation says what the counts of the goal rule out, under the
    // same limits as the search.
    if let Verdict::Unprovable(refutation) = &mut verdict {
        let account = memory::Account::new(options.memory_limit);
        *refutation = focus::refutation(forest, goal, fragment, mode, &account, &mut stop);
    }
    // No engine is trusted with its own proof: the checker has the last
    // word on every proof of the sequent, in every build.
    if options.check
        && is_roots
        && let Verdict::Proved(proof) = &verdict
    {
        proof
            .check_within(mode, options.memory_limit)
            .map_err(|e| {
                if e.is_refusal() {
                    Error::Unchecked(e)
                } else {
                    Error::Rejected(Box::new(e))
                }
            })?;
    }
    Ok(Outcome {
        verdict,
        fragment,
        mode,
        engine,
        statistics,
        net,
    })
}

/// Returns the smallest fragment the goal's subformulas live in: the
/// sequent's own fragment for the roots, and possibly a smaller one for an
/// open goal deeper in the forest.
pub(crate) fn goal_fragment(forest: &Forest, goal: &[OccId]) -> Fragment {
    let mut fragment = Fragment::EMPTY;
    for &o in goal {
        for x in forest.subtree(o) {
            fragment |= forest.kind(x).fragment();
        }
    }
    fragment
}

/// The most occurrences of one literal, `a` or `~a`, a sequent may have for
/// the net engine to be the default on it.
const NET_MULTIPLICITY: usize = 2;

/// Whether the net engine is the better default for an MLL sequent: when
/// no literal occurs more than [`NET_MULTIPLICITY`] times. Equal literals
/// under one connective are interchangeable partners, so the linking
/// search explores every permutation of a wrong choice before a cycle
/// shows, and the focused engine, whose count prunes see the mistake at
/// once, wins by orders of magnitude on such sequents; with distinct atoms
/// the linking is nearly forced and the net engine is linear where the
/// focused engine searches context splits.
fn prefers_net(forest: &Forest) -> bool {
    use crate::occurrences::Sign;
    let atoms = forest.sequent().atom_names().len() as u32;
    (0..atoms).all(|a| {
        let atom = crate::sequents::Atom::new(a);
        forest.literals(atom, Sign::Var).len() <= NET_MULTIPLICITY
            && forest.literals(atom, Sign::DualVar).len() <= NET_MULTIPLICITY
    })
}

/// The engines, by which an outcome names the one that ran and the options
/// force one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Engine {
    /// The focused sequent engine of [`focus`], one-sided: classical mode.
    Focus,
    /// The proof-net engine of [`net`], for unit-free MLL only; in
    /// intuitionistic mode it decides IMLL through the embedding into MLL.
    Net,
    /// The focused sequent engine of [`focus`] two-sided, keeping one goal
    /// on every branch: intuitionistic mode.
    TwoSided,
    /// The fast path of [`additive`] for two additive-only formulas, in
    /// every mode.
    Additive,
}

impl Display for Engine {
    /// Writes the engine's name: `focus`, `net`, `two-sided` or `additive`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Engine::Focus => f.write_str("focus"),
            Engine::Net => f.write_str("net"),
            Engine::TwoSided => f.write_str("two-sided"),
            Engine::Additive => f.write_str("additive"),
        }
    }
}

/// The knobs of a search: how much to remember, how deep to go, and which
/// fragment and engine to use instead of the detected ones. The defaults
/// suit a sequent of a few hundred occurrences on a thread with the usual
/// stack.
///
/// # Examples
///
/// ```
/// use linlog::Fragment;
/// use linlog::search::Options;
///
/// let options = Options::default()
///     .memo_limit(1 << 16)
///     .recursion_limit(10_000)
///     .fragment(Some(Fragment::MALL));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// The most stable sequents the memo holds at once.
    memo_limit: usize,
    /// The deepest nesting of engine calls before the search gives up.
    recursion_limit: u32,
    /// The engine to use, or `None` for the one the fragment calls for.
    engine: Option<Engine>,
    /// The fragment to search in, or `None` for the detected one.
    fragment: Option<Fragment>,
    /// How many links the net engine makes between two exact acyclicity
    /// tests, or `None` for the default that depends on the size of the
    /// structure.
    test_period: Option<u32>,
    /// The most copies of `?` formulas one branch may take, or `None` for
    /// a search that deepens until it decides or is stopped.
    copies: Option<u32>,
    /// How many threads the search may use; one runs the sequential
    /// engines.
    jobs: usize,
    /// How the focused engine picks each atom's positive literal.
    bias: Bias,
    /// The most copies of `?` formulas one branch may take in the forward
    /// search of the default bias.
    forward_copies: u32,
    /// Whether a proof of the sequent passes the checker before it is
    /// returned.
    check: bool,
    /// The most bytes the search may hold at once, or `None` for no bound.
    memory_limit: Option<u64>,
    /// The most subformula occurrences a sequent may unfold to.
    occurrence_limit: u64,
}

impl Default for Options {
    /// A memo of at most [`DEFAULT_MEMO_LIMIT`](Self::DEFAULT_MEMO_LIMIT)
    /// stable sequents, a recursion limit of
    /// [`DEFAULT_RECURSION_LIMIT`](Self::DEFAULT_RECURSION_LIMIT), the
    /// engine and fragment chosen by detection, the net engine's exact test
    /// at its default cadence, a copy bound of
    /// [`DEFAULT_COPIES`](Self::DEFAULT_COPIES), one of
    /// [`DEFAULT_FORWARD_COPIES`](Self::DEFAULT_FORWARD_COPIES) for the
    /// forward search of the default bias, one thread, every proof
    /// checked ([`DEFAULT_CHECK`](Self::DEFAULT_CHECK)), a memory
    /// bound of [`DEFAULT_MEMORY_LIMIT`](Self::DEFAULT_MEMORY_LIMIT), and
    /// a sequent of at most
    /// [`DEFAULT_OCCURRENCE_LIMIT`](Self::DEFAULT_OCCURRENCE_LIMIT)
    /// occurrences.
    fn default() -> Self {
        Self {
            memo_limit: Self::DEFAULT_MEMO_LIMIT,
            recursion_limit: Self::DEFAULT_RECURSION_LIMIT,
            engine: None,
            fragment: None,
            test_period: None,
            copies: Some(Self::DEFAULT_COPIES),
            jobs: 1,
            bias: Bias::Auto,
            forward_copies: Self::DEFAULT_FORWARD_COPIES,
            check: Self::DEFAULT_CHECK,
            memory_limit: Some(Self::DEFAULT_MEMORY_LIMIT),
            occurrence_limit: Self::DEFAULT_OCCURRENCE_LIMIT,
        }
    }
}

impl Options {
    /// The memo limit of the default options: 2²⁰ stable sequents.
    pub const DEFAULT_MEMO_LIMIT: usize = 1 << 20;

    /// The recursion limit of the default options, which fits the 8 MiB
    /// stack of a main thread.
    pub const DEFAULT_RECURSION_LIMIT: u32 = 2048;

    /// The copy bound of the default options: three copies per branch, the
    /// bound llprover searches with by default. The default options keep a
    /// bound because a search without one ends only when it decides or its
    /// stop condition fires, and [`prove`] has none; a front end with a
    /// clock lifts it ([`copies`](Self::copies)).
    pub const DEFAULT_COPIES: u32 = 3;

    /// The forward search's copy bound of the default options: thirty
    /// steps of a forward chain. A deeper bound decided nothing more
    /// within seconds on Petri nets from practice, and the price of the
    /// bound is what an undecided search to that depth costs: mostly
    /// milliseconds on a small Horn program, a second and more on one
    /// whose markings grow in several places at once.
    pub const DEFAULT_FORWARD_COPIES: u32 = 30;

    /// Whether the default options check a proof before returning it: they
    /// do. The check is one pass over the proof, in memory proportional to
    /// it.
    pub const DEFAULT_CHECK: bool = true;

    /// The memory bound of the default options, in bytes: one gibibyte,
    /// which a laptop and a browser tab both have to spare.
    pub const DEFAULT_MEMORY_LIMIT: u64 = crate::proofs::DEFAULT_MEMORY_LIMIT;

    /// The most subformula occurrences of a sequent under the default
    /// options: [`Forest::DEFAULT_LIMIT`].
    pub const DEFAULT_OCCURRENCE_LIMIT: u64 = Forest::DEFAULT_LIMIT;

    /// Sets the most subformula occurrences the sequent may unfold to
    /// when [`prove`] and [`prove_until`] build its forest: a sequent
    /// beyond is refused with [`Error::TooManyOccurrences`] before
    /// anything of that size is built. A sequent read from JSON can share
    /// subterms, so a few hundred bytes unfold to any number of
    /// occurrences; a forest takes about 25 bytes for each, and it is not
    /// counted under [`memory_limit`](Self::memory_limit), since
    /// [`prove_goal`] is handed it. No limit lets through more than a
    /// forest indexes, somewhat under 2³².
    pub fn occurrence_limit(self, limit: u64) -> Self {
        Self {
            occurrence_limit: limit,
            ..self
        }
    }

    /// Sets the most bytes the search may hold at once, or `None` for no
    /// bound. Counted are the structures that grow with the search: the
    /// memo of the focused engine and of the additive path, the proof
    /// arena, the buffers every level of recursion takes, and the count
    /// invariants set up before the search; not the forest, which is the
    /// caller's, nor the proof returned. A memo that no longer fits is
    /// emptied first, as it is when it reaches
    /// [`memo_limit`](Self::memo_limit), and the proofs only it referred
    /// to are given back; the search gives up with
    /// [`Reason::MemoryLimit`] when what is left still passes the bound,
    /// or leaves the memo no room at all. The two searches that
    /// [`Bias::Auto`] runs on a sequent with exponentials have half the
    /// bound each. The check of a proof found is under the same bound.
    pub fn memory_limit(self, limit: Option<u64>) -> Self {
        Self {
            memory_limit: limit,
            ..self
        }
    }

    /// Sets whether a proof of the sequent passes the checker
    /// ([`Proof::check`], which shares no code with the engines) before
    /// the search returns it; one that does not is
    /// [`Error::Rejected`], never a verdict. Without the check the proof
    /// is the engine's word, which a caller that checks it itself, or
    /// times the search alone, may prefer. The proof of a goal other than
    /// the roots is not checked here in either case: what grafts it does.
    pub fn check(self, check: bool) -> Self {
        Self { check, ..self }
    }

    /// Sets the most copies of `?` formulas one branch of a proof may take,
    /// or `None` for no bound. The search deepens the bound from zero, one
    /// level after another; a sequent that has no proof within the bound
    /// is [`Reason::CopyBound`], unless some level finished without ever
    /// reaching its bound, which makes the sequent [`Verdict::Unprovable`].
    /// Without a bound the search goes on to the next level until it
    /// decides, its stop condition fires or a limit binds, and
    /// [`Statistics::copies`] says how far it got: the choice of a caller
    /// with a time limit, as the command is, since no bound is too small
    /// for some provable sequent and full linear logic is undecidable.
    /// Without exponentials the bound has no effect. A proof found at some
    /// level may reuse a memoized subproof found with more copies left, so
    /// the bound limits the search, not the proof returned.
    pub fn copies(self, copies: Option<u32>) -> Self {
        Self { copies, ..self }
    }

    /// Returns the copy bound as the engines count it: no bound is the
    /// largest, which no search reaches, since every level visits a stable
    /// sequent and polls at it, and four billion levels take hours at the
    /// least.
    pub(crate) fn copy_bound(&self) -> u32 {
        self.copies.unwrap_or(u32::MAX)
    }

    /// Sets the engine to use, or `None` for the one the detected fragment
    /// and the mode call for.
    pub fn engine(self, engine: Option<Engine>) -> Self {
        Self { engine, ..self }
    }

    /// Sets the fragment to search in, or `None` for the detected one. A
    /// sequent outside the fragment set here is refused; a fragment larger
    /// than the detected one switches off the prunes that only hold in the
    /// smaller one.
    pub fn fragment(self, fragment: Option<Fragment>) -> Self {
        Self { fragment, ..self }
    }

    /// Sets the most stable sequents the memo holds at once; when the memo
    /// is full it is emptied, which costs time but not correctness. Zero
    /// switches the memo off. The two searches that [`Bias::Auto`] runs
    /// on a sequent with exponentials hold a memo of this size each. This
    /// is the finer knob beside [`memory_limit`](Self::memory_limit),
    /// which bounds the memo in bytes: a table that fits the processor's
    /// cache can be faster than one that fits the memory.
    pub fn memo_limit(self, limit: usize) -> Self {
        Self {
            memo_limit: limit,
            ..self
        }
    }

    /// Sets the deepest nesting of engine calls (one per rule applied along
    /// a branch, three per occurrence at most) before the search gives up
    /// with [`Reason::RecursionLimit`]. The engine recurses on the calling
    /// thread's stack, so a caller that raises the limit runs the search on
    /// a thread with a stack to match.
    pub fn recursion_limit(self, limit: u32) -> Self {
        Self {
            recursion_limit: limit,
            ..self
        }
    }

    /// Sets how many links the net engine makes between two exact
    /// acyclicity tests, or `None` for the default: every link on a
    /// structure of at most 200 occurrences, every fourth link on a larger
    /// one. The test also runs on every complete linking, so the period
    /// trades time per link against how long a doomed branch is followed.
    /// Zero counts as one.
    pub fn test_period(self, period: Option<u32>) -> Self {
        Self {
            test_period: period,
            ..self
        }
    }

    /// The most threads the options name: more are taken as this many.
    /// A pool beyond it has found no use, and a count without a bound
    /// starts whatever it is given (ten thousand threads on `A ⊢ A` cost
    /// three minutes of processor time).
    pub const MAX_JOBS: usize = 256;

    /// Sets how many threads the search may use. One, the default, runs
    /// the sequential engines, whose proof is a function of the input.
    /// More than one, with the `parallel` feature, runs the focused engine
    /// and the net engine on that many threads of a pool of their own,
    /// which may find a different proof but never a different verdict;
    /// without the feature, or for the additive path, the search stays
    /// sequential. Zero counts as one, and more than
    /// [`MAX_JOBS`](Self::MAX_JOBS) as that many. A search starts no more
    /// threads than the machine runs at once
    /// ([`std::thread::available_parallelism`], where the platform tells):
    /// threads beyond that only take turns on the same processors.
    pub fn jobs(self, jobs: usize) -> Self {
        Self {
            jobs: jobs.clamp(1, Self::MAX_JOBS),
            ..self
        }
    }

    /// Sets how the focused engine picks the positive literal of every
    /// atom. No choice changes what is provable. [`Bias::Rarer`] mostly
    /// chains backward from the goal and [`Bias::Factors`] forward from
    /// the hypotheses; on a sequent with exponentials whose hypotheses are
    /// Horn clauses, a Petri net for one, the forward search often decides
    /// in milliseconds what the backward one does not decide at all, but
    /// its proofs take one copy per step on a single branch. The default,
    /// [`Bias::Auto`], is `Factors` without exponentials and `Rarer` under
    /// weakening; on a sequent with exponentials in linear mode it runs
    /// both searches, the backward one within [`copies`](Self::copies) and
    /// the forward one within [`forward_copies`](Self::forward_copies),
    /// and answers with the first that decides. Either of the two named
    /// explicitly is that search alone, within `copies`.
    pub fn bias(self, bias: Bias) -> Self {
        Self { bias, ..self }
    }

    /// Sets the most copies of `?` formulas one branch may take in the
    /// forward search that [`Bias::Auto`] runs beside the backward one on
    /// a Horn program: clauses `!(a ⊗ b ⊸ c ⊗ d)`, a marking and a goal
    /// of atoms, as a Petri net is. A forward chain takes a copy per
    /// step, all on one branch, so it wants a larger bound than
    /// [`copies`](Self::copies), which this is when it is the larger of
    /// the two; the forward search never runs within less than `copies`.
    /// On any other sequent, under another bias, under Mix, under
    /// weakening and without a copy bound it has no effect: unbounded, the
    /// forward search deepens as far as its share of the work takes it.
    pub fn forward_copies(self, copies: u32) -> Self {
        Self {
            forward_copies: copies,
            ..self
        }
    }

    /// Returns the stack, in bytes, a thread needs to run the search at
    /// the recursion limit: the most one level of recursion was measured
    /// to take, doubled for the derivation built from the proof, and at
    /// least a main thread's 8 MiB. The parallel search sizes its workers
    /// by it; a caller that runs the sequential search on a thread of its
    /// own sizes that thread by it.
    pub fn stack_size(&self) -> usize {
        /// The stack one level of recursion may take: twice the most the
        /// search was measured to take, on a chain of tensors whose splits
        /// are searched (3.6 KiB unoptimized, 0.9 KiB optimized).
        const PER_LEVEL: usize = if cfg!(debug_assertions) { 8192 } else { 2048 };
        /// A main thread's stack.
        const MIN: usize = 8 << 20;
        (self.recursion_limit as usize)
            .saturating_mul(PER_LEVEL)
            .max(MIN)
    }
}

/// What a search returned: the verdict, and how it was reached.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Outcome {
    /// The verdict, with the proof if there is one.
    pub verdict: Verdict,
    /// The fragment searched in: the detected one, or the one the options
    /// asserted.
    pub fragment: Fragment,
    /// The mode searched in.
    pub mode: Mode,
    /// The engine that ran.
    pub engine: Engine,
    /// What the search cost.
    pub statistics: Statistics,
    /// The proof net the proof was read off, when the net engine found
    /// one; `None` for the other engines and for any other verdict.
    pub net: Option<ProofStructure>,
}

/// What a search found: a proof, that there is none, or that it could not
/// tell.
#[derive(Clone, Debug)]
pub enum Verdict {
    /// The sequent is provable, and here is a proof, boxed because a proof
    /// carries its forest.
    Proved(Box<Proof>),
    /// The sequent is not provable: the search was exhaustive, and here is
    /// what can be said of why.
    Unprovable(Refutation),
    /// The search stopped before it could decide, for the reason given.
    Unknown(Reason),
}

impl Verdict {
    /// Returns the proof, if the sequent was proved.
    pub fn proof(&self) -> Option<&Proof> {
        match self {
            Verdict::Proved(proof) => Some(proof),
            _ => None,
        }
    }
}

/// Why a sequent is unprovable, as far as its counts tell. Every
/// refutation rests on the search that was exhaustive; where the literals
/// or the connectives of the sequent alone rule out a proof, the
/// refutation says which, as the focused engine checks them on every
/// sequent it searches.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refutation {
    /// Every way to prove the sequent was tried and failed.
    Exhausted,
    /// The literals of an atom cannot all meet their duals in axioms:
    /// whichever additive alternatives a proof takes, the positive literal
    /// occurs between `least` and `most` times more than the negative one
    /// (fewer, where these are negative), and never as often.
    Unbalanced {
        /// The atom.
        atom: Atom,
        /// Its name.
        name: String,
        /// The least excess of positive over negative literals.
        least: i32,
        /// The greatest excess.
        most: i32,
    },
    /// The count equation of the multiplicatives fails: a provable
    /// sequent of MLL with units has exactly
    /// `tensors − pars − ones + bottoms + 2` formulas, and at least that
    /// many with Mix, counted on the one-sided sequent.
    Equation {
        /// The formulas of the sequent.
        formulas: u64,
        /// Its `⊗`.
        tensors: u64,
        /// Its `⅋`.
        pars: u64,
        /// Its `1`.
        ones: u64,
        /// Its `⊥`.
        bottoms: u64,
        /// Whether Mix was allowed.
        mix: bool,
    },
}

impl Refutation {
    /// Returns how many formulas the count equation asks for, for
    /// [`Refutation::Equation`].
    pub fn needed(&self) -> Option<i128> {
        match *self {
            Refutation::Equation {
                tensors,
                pars,
                ones,
                bottoms,
                ..
            } => Some(
                i128::from(tensors) - i128::from(pars) - i128::from(ones) + i128::from(bottoms) + 2,
            ),
            _ => None,
        }
    }
}

impl Display for Refutation {
    /// Writes the refutation as a phrase, such as `the search was
    /// exhaustive`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Refutation::Exhausted => f.write_str("the search was exhaustive"),
            Refutation::Unbalanced {
                name, least, most, ..
            } => {
                let (more, fewer) = if *least > 0 {
                    (name.clone(), format!("~{name}"))
                } else {
                    (format!("~{name}"), name.clone())
                };
                let (a, b) = (least.unsigned_abs(), most.unsigned_abs());
                let (a, b) = (a.min(b), a.max(b));
                let excess = if a == b {
                    format!("{a}")
                } else {
                    format!("{a} to {b}")
                };
                write!(
                    f,
                    "{more} occurs {excess} more {} than {fewer} in the one-sided sequent{}, so \
                     they cannot all meet in axioms",
                    if a == 1 && b == 1 { "time" } else { "times" },
                    if a == b {
                        ""
                    } else {
                        ", whichever additive alternatives a proof takes"
                    }
                )
            }
            Refutation::Equation {
                formulas,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            } => {
                // A count below zero with the minus sign of the expression.
                let needed = self.needed().unwrap_or_default();
                let needed = if needed < 0 {
                    format!("−{}", needed.unsigned_abs())
                } else {
                    needed.to_string()
                };
                write!(
                    f,
                    "the count equation fails: a provable one-sided sequent of MLL has {}#⊗ − \
                     #⅋ − #1 + #⊥ + 2 formulas, here {tensors} − {pars} − {ones} + {bottoms} + 2 = \
                     {needed}, and this one has {formulas}",
                    if *mix { "at least " } else { "exactly " },
                )
            }
        }
    }
}

/// Why a search stopped without deciding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reason {
    /// The caller's stop condition fired: a time limit or an interruption.
    Stopped,
    /// The nesting of engine calls reached [`Options::recursion_limit`].
    RecursionLimit,
    /// Every level up to the bound [`Options::copies`] set, which is this
    /// value, hit its bound on some branch, so a proof with more copies of
    /// a `?` formula per branch may exist.
    CopyBound(u32),
    /// The search held [`Options::memory_limit`] bytes, which is this
    /// value, with its memo already emptied, or had no room left for a
    /// memo at all.
    MemoryLimit(u64),
    /// A structure of the search outgrew what its indices address: the
    /// proof arena at 2³¹ nodes, the count invariants at 2³² row entries.
    /// Only a search without a memory bound gets this far.
    IndexLimit,
}

impl Display for Reason {
    /// Writes the reason as a phrase, such as `the time limit was reached`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Reason::Stopped => f.write_str("the search was stopped"),
            Reason::RecursionLimit => f.write_str("the recursion limit was reached"),
            Reason::CopyBound(n) => {
                write!(f, "the copy bound of {n} was reached")
            }
            Reason::MemoryLimit(bytes) => {
                write!(f, "the memory limit of {} was reached", Bytes(*bytes))
            }
            Reason::IndexLimit => f.write_str("the search outgrew what its indices address"),
        }
    }
}

/// What a search cost. The focused engine counts stable sequents, memo use
/// and splits; the net engine counts literals chosen, links and exact
/// tests; the additive path counts pairs of subformulas; the other
/// counters stay zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Statistics {
    /// The nodes of the search: the stable sequents the focused engine
    /// visited, memo hits included, the literals the net engine chose a
    /// partner for, or the pairs of subformulas the additive path decided.
    pub nodes: u64,
    /// The visits answered from the memo.
    pub memo_hits: u64,
    /// The most stable sequents the memo held at once; of two searches
    /// that ran together, the two memos' together, and of two that took
    /// turns from their start, the most of one turn.
    pub memo_entries: usize,
    /// The context splits examined for `⊗` and Mix, most of them rejected by
    /// the counts.
    pub splits: u64,
    /// The axiom links the net engine tried: each was made, and taken back
    /// again unless it is part of the net found.
    pub links: u64,
    /// The exact acyclicity tests the net engine ran.
    pub tests: u64,
    /// The largest copy bound a level of the focused engine's deepening
    /// began under, of two searches the larger: how far a search without
    /// a bound got before it was stopped. Zero without exponentials and
    /// for the other engines.
    pub copies: u32,
}

impl Statistics {
    /// Adds another engine's counters to these, the memo's excepted: they
    /// describe a table, not a run, and a parallel search reads them off
    /// the one table its workers share.
    pub(crate) fn add(&mut self, other: &Statistics) {
        self.nodes += other.nodes;
        self.splits += other.splits;
        self.links += other.links;
        self.tests += other.tests;
        self.copies = self.copies.max(other.copies);
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// Unit-free MLL reaches the net engine unless a literal occurs more
    /// than twice, every other classical input without exponentials the
    /// focused engine, and the outcome says which fragment it was searched
    /// in.
    #[test]
    fn dispatch() {
        for (input, fragment, engine) in [
            ("|- a, ~a", Fragment::EMPTY, Engine::Net),
            ("a, a -o b |- b", Fragment::MLL, Engine::Net),
            ("a * a |- a * a", Fragment::MLL, Engine::Net),
            ("a * a * a |- a * a * a", Fragment::MLL, Engine::Focus),
            ("|- 1, bot", Fragment::MULTIPLICATIVE_UNITS, Engine::Focus),
            ("|- a & b, ~a + ~b", Fragment::ADDITIVES, Engine::Additive),
            ("|- top, 0", Fragment::ADDITIVE_UNITS, Engine::Additive),
            ("|- top, a, b", Fragment::ADDITIVE_UNITS, Engine::Focus),
            ("|- (a * top) + 1, ~a, bot", Fragment::MALL, Engine::Focus),
        ] {
            let outcome = prove(&sequent(input), Mode::CLASSICAL, &Options::default()).unwrap();
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert_eq!(outcome.engine, engine, "{input:?}");
            assert_eq!(outcome.net.is_some(), engine == Engine::Net, "{input:?}");
            assert!(outcome.verdict.proof().is_some(), "{input:?}");
        }
    }

    /// An unprovable sequent says why where its counts tell: an atom whose
    /// literals cannot pair up, also across additive alternatives, or the
    /// count equation; else that the search was exhaustive.
    #[test]
    fn refutations() {
        let refuted = |input: &str, mode: Mode| {
            let outcome = prove(&sequent(input), mode, &Options::default()).unwrap();
            let Verdict::Unprovable(refutation) = outcome.verdict else {
                panic!("{input:?}: {:?}", outcome.verdict);
            };
            refutation
        };
        let unbalanced = |name: &str, least, most| Refutation::Unbalanced {
            atom: Atom::new(0),
            name: name.to_owned(),
            least,
            most,
        };
        let classical = Mode::CLASSICAL;
        assert_eq!(refuted("|- a, a", classical), unbalanced("a", 2, 2));
        assert_eq!(
            refuted("a |- b", Mode::INTUITIONISTIC),
            unbalanced("a", -1, -1)
        );
        let hull = refuted("|- (a * a) + (a * a * a), ~a", classical);
        assert_eq!(hull, unbalanced("a", 1, 2));
        assert_eq!(
            hull.to_string(),
            "a occurs 1 to 2 more times than ~a in the one-sided sequent, whichever additive \
             alternatives a proof takes, so they cannot all meet in axioms"
        );
        let equation = refuted("|- a par b, ~a, ~b", classical);
        assert_eq!(equation.needed(), Some(1));
        assert_eq!(
            equation.to_string(),
            "the count equation fails: a provable one-sided sequent of MLL has exactly #⊗ − #⅋ − \
             #1 + #⊥ + 2 formulas, here 0 − 1 − 0 + 0 + 2 = 1, and this one has 3"
        );
        for input in ["|- a par ~a, b * ~b", "|- a & b, ~a"] {
            assert_eq!(
                refuted(input, classical),
                Refutation::Exhausted,
                "{input:?}"
            );
        }
    }

    /// `Options::engine` forces an engine: the focused engine on MLL, the
    /// net engine on MLL with Mix, and the net engine outside MLL is an
    /// error.
    #[test]
    fn engine_override() {
        let s = sequent("|- a * b, ~a par ~b");
        let focus = Options::default().engine(Some(Engine::Focus));
        let outcome = prove(&s, Mode::CLASSICAL, &focus).unwrap();
        assert_eq!(outcome.engine, Engine::Focus);
        assert!(outcome.verdict.proof().is_some());
        assert!(outcome.net.is_none());
        let net = Options::default().engine(Some(Engine::Net));
        let outcome = prove(
            &sequent("|- a, ~a, b, ~b"),
            Mode::CLASSICAL.with_mix(),
            &net,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Net);
        assert!(outcome.verdict.proof().is_some());
        for (input, options) in [
            ("|- 1", net.clone()),
            ("|- a & b, ~a", net.clone()),
            ("|- a, ~a", net.clone().fragment(Some(Fragment::MALL))),
        ] {
            let error = prove(&sequent(input), Mode::CLASSICAL, &options).unwrap_err();
            assert!(matches!(error, Error::NetFragment(_)), "{input:?}: {error}");
        }
        assert_eq!(
            prove(&sequent("|- 1"), Mode::CLASSICAL, &net)
                .unwrap_err()
                .to_string(),
            "proof nets exist for MLL without units only, not for MLL with units"
        );
    }

    /// The asserted fragment must contain the sequent's, and is what the
    /// search runs in, which also picks the engine.
    #[test]
    fn fragment_override() {
        let s = sequent("|- a * b, ~a, ~b");
        let outcome = prove(
            &s,
            Mode::CLASSICAL,
            &Options::default().fragment(Some(Fragment::MALL)),
        )
        .unwrap();
        assert_eq!(outcome.fragment, Fragment::MALL);
        assert_eq!(outcome.engine, Engine::Focus);
        assert!(outcome.verdict.proof().is_some());
        let error = prove(
            &sequent("|- a & b, ~a"),
            Mode::CLASSICAL,
            &Options::default().fragment(Some(Fragment::MLL)),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Error::FragmentMismatch {
                asserted: Fragment::MLL,
                detected: Fragment::ADDITIVES,
            }
        ));
        assert_eq!(
            error.to_string(),
            "the sequent lies in ALL, outside the asserted fragment MLL"
        );
    }

    /// Affine mode and the exponentials go to the focused engine, and the
    /// net engine is refused in affine mode.
    #[test]
    fn dispatch_by_mode() {
        for (input, mode, fragment) in [
            ("a, b |- a", Mode::CLASSICAL.affine(), Fragment::EMPTY),
            ("!a |- a", Mode::CLASSICAL, Fragment::EXPONENTIALS),
            (
                "!a |- a & a",
                Mode::CLASSICAL.with_mix(),
                Fragment::ADDITIVES | Fragment::EXPONENTIALS,
            ),
            (
                "!a, b |- a",
                Mode::CLASSICAL.affine(),
                Fragment::EXPONENTIALS,
            ),
        ] {
            let outcome = prove(&sequent(input), mode, &Options::default()).unwrap();
            assert_eq!(outcome.engine, Engine::Focus, "{input:?}");
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert!(outcome.verdict.proof().is_some(), "{input:?}");
        }
        let net = Options::default().engine(Some(Engine::Net));
        let error = prove(&sequent("a, b |- a"), Mode::CLASSICAL.affine(), &net).unwrap_err();
        assert!(matches!(error, Error::NetMode(_)));
        assert_eq!(
            error.to_string(),
            "proof nets exist in classical mode only, with or without Mix, not in classical affine mode"
        );
    }

    /// Intuitionistic mode: IMLL without units goes to the net engine by
    /// the embedding, everything else to the two-sided engine; a sequent
    /// with no intuitionistic reading, Mix, and an engine forced for the
    /// other mode are errors.
    #[test]
    fn dispatch_intuitionistic() {
        let i = Mode::INTUITIONISTIC;
        for (input, fragment, engine) in [
            ("a, a -o b |- b", Fragment::MLL, Engine::Net),
            ("a * a * a |- a * a * a", Fragment::MLL, Engine::TwoSided),
            ("1 |- 1", Fragment::MULTIPLICATIVE_UNITS, Engine::TwoSided),
            ("a & b |- a", Fragment::ADDITIVES, Engine::Additive),
            ("a & b, 0 |- a", Fragment::ALL, Engine::TwoSided),
            (
                "!a |- a * a",
                Fragment::MLL | Fragment::EXPONENTIALS,
                Engine::TwoSided,
            ),
        ] {
            let outcome = prove(&sequent(input), i, &Options::default()).unwrap();
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert_eq!(outcome.engine, engine, "{input:?}");
            let proof = outcome
                .verdict
                .proof()
                .unwrap_or_else(|| panic!("{input:?}"));
            assert_eq!(proof.check(i), Ok(()), "{input:?}");
        }
        let outcome = prove(&sequent("a, b |- a"), i.affine(), &Options::default()).unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());

        let error = prove(&sequent("|- a par b"), i, &Options::default()).unwrap_err();
        assert!(matches!(error, Error::NotIntuitionistic(_)));
        assert_eq!(
            error.to_string(),
            "not an intuitionistic sequent: subformula 0 is neither an intuitionistic formula nor \
             the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)"
        );
        let error = prove(&sequent("a |- a"), i.with_mix(), &Options::default()).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Mix has no intuitionistic form: a premise of a Mix would have no goal"
        );
        let focus = Options::default().engine(Some(Engine::Focus));
        let error = prove(&sequent("a |- a"), i, &focus).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the focus engine does not search in intuitionistic mode"
        );
        let two_sided = Options::default().engine(Some(Engine::TwoSided));
        let error = prove(&sequent("a |- a"), Mode::CLASSICAL, &two_sided).unwrap_err();
        assert!(matches!(error, Error::EngineMode { .. }));
        let outcome = prove(&sequent("a, a -o b |- b"), i, &two_sided).unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());
        let additive = Options::default().engine(Some(Engine::Additive));
        let error = prove(&sequent("a & b, c |- a"), i, &additive).unwrap_err();
        assert!(matches!(error, Error::NotAdditive { .. }));
        assert_eq!(
            error.to_string(),
            "the additive engine decides a sequent of two additive-only formulas, not 3 formulas of ALL"
        );
    }

    /// IMLL by embedding: on generated intuitionistic sequents over `⊗` and
    /// `⊸` and their mutants, where the dispatch picks the net engine on the
    /// one-sided sequent, the two-sided engine gives the same verdict, and
    /// every net-engine proof passes the intuitionistic checker (every
    /// sequent of a cut-free MLL proof of an intuitionistic sequent has one
    /// goal).
    #[test]
    fn embedding_agrees_with_the_two_sided_engine() {
        use crate::search::generate::{self, IllRules, Rng};
        let i = Mode::INTUITIONISTIC;
        let rules = IllRules {
            units: false,
            additives: false,
            zero: false,
            exponentials: false,
        };
        let mut rng = Rng::new(7);
        let (mut compared, mut provable) = (0, 0);
        for _ in 0..200 {
            let budget = 2 + rng.below(8);
            let generate::Ill {
                mut hypotheses,
                goal,
                ..
            } = generate::ill(&mut rng, rules, 3, budget);
            hypotheses.push(goal);
            for mutated in [false, true] {
                if mutated && !generate::mutate(&mut rng, &mut hypotheses, 3) {
                    continue;
                }
                let goal = hypotheses.last().unwrap();
                let text = generate::two_sided(&hypotheses[..hypotheses.len() - 1], goal);
                let s = sequent(&text);
                let by_net = prove(&s, i, &Options::default()).unwrap();
                if by_net.engine != Engine::Net {
                    // Repeated literals: the dispatch keeps the net engine
                    // off them.
                    continue;
                }
                let two_sided = Options::default().engine(Some(Engine::TwoSided));
                let by_two_sided = prove(&s, i, &two_sided).unwrap();
                assert_eq!(by_two_sided.engine, Engine::TwoSided);
                let verdict = |outcome: Outcome| match outcome.verdict {
                    Verdict::Proved(proof) => {
                        assert_eq!(proof.check(i), Ok(()), "{text:?} by {}", outcome.engine);
                        assert_eq!(outcome.net.is_some(), outcome.engine == Engine::Net);
                        true
                    }
                    Verdict::Unprovable(_) => false,
                    Verdict::Unknown(reason) => panic!("{text:?} by {}: {reason}", outcome.engine),
                };
                let (net, focus) = (verdict(by_net), verdict(by_two_sided));
                assert_eq!(net, focus, "{text:?}: net {net}, two-sided {focus}");
                assert!(mutated || net, "{text:?} is provable");
                compared += 1;
                provable += usize::from(net);
            }
        }
        assert!(
            compared > 100 && provable > 50 && provable < compared,
            "{provable} of {compared}"
        );
    }

    /// A goal below the roots is decided in its own fragment by the focused
    /// engine, two additive-only occurrences by the additive path, and the
    /// two-sided engine in intuitionistic mode; the net engine is refused
    /// off the roots, and an intuitionistic goal must have one output.
    #[test]
    fn goals() {
        use crate::occurrences::OccId;
        let o = |ids: &[u32]| ids.iter().map(|&i| OccId::new(i)).collect::<Vec<_>>();
        // 0: ~a, 1: (a ⊗ ~b) ⊗ ?c, 2: a ⊗ ~b, 3: a, 4: ~b, 5: ?c, 6: c, 7: b.
        let s = sequent("|- ~a, (a * ~b) * ?c, b");
        let forest = Forest::new(&s).unwrap();
        let options = Options::default();
        let goal = o(&[0, 3]);
        let outcome = prove_goal(&forest, &goal, Mode::CLASSICAL, &options, || false).unwrap();
        assert_eq!(outcome.engine, Engine::Focus);
        assert_eq!(outcome.fragment, Fragment::EMPTY);
        let proof = outcome.verdict.proof().unwrap();
        assert!(
            proof.check(Mode::CLASSICAL).is_err(),
            "a goal proof is not a proof of the roots"
        );
        let outcome =
            prove_goal(&forest, &o(&[0, 2, 7]), Mode::CLASSICAL, &options, || false).unwrap();
        assert!(outcome.verdict.proof().is_some());
        assert_eq!(outcome.fragment, Fragment::MLL);
        let outcome =
            prove_goal(&forest, &o(&[4, 5]), Mode::CLASSICAL, &options, || false).unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unprovable(_)),
            "{:?}",
            outcome.verdict
        );
        assert_eq!(outcome.fragment, Fragment::EXPONENTIALS);
        let net = Options::default().engine(Some(Engine::Net));
        let error = prove_goal(&forest, &goal, Mode::CLASSICAL, &net, || false).unwrap_err();
        assert!(matches!(error, Error::NetGoal), "{error}");
        let error = prove_goal(&forest, &o(&[9]), Mode::CLASSICAL, &options, || false).unwrap_err();
        assert!(
            matches!(error, Error::OccurrenceIndexOutOfBounds(9, 8)),
            "{error}"
        );

        // 0: (~a & ~b) ⅋ (a & b), 1: ~a & ~b, 2: ~a, 3: ~b, 4: a & b, 5: a,
        // 6: b: the additive path on the pair below the ⅋.
        let s = sequent("|- (~a & ~b) par (a & b)");
        let forest = Forest::new(&s).unwrap();
        let outcome =
            prove_goal(&forest, &o(&[1, 4]), Mode::CLASSICAL, &options, || false).unwrap();
        assert_eq!(outcome.engine, Engine::Additive);
        assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
        let outcome =
            prove_goal(&forest, &o(&[5, 2]), Mode::CLASSICAL, &options, || false).unwrap();
        assert_eq!(outcome.engine, Engine::Focus);
        assert!(outcome.verdict.proof().is_some());

        // 0: ~a, 1: a ⊗ ~b, 2: a, 3: ~b, 4: b, read as a, a ⊸ b ⊢ b: the
        // goal ⊢ ~a, a is a ⊢ a two-sided; ⊢ ~a alone has no output and
        // ⊢ a, b two.
        let s = sequent("a, a -o b |- b");
        let forest = Forest::new(&s).unwrap();
        let i = Mode::INTUITIONISTIC;
        let outcome = prove_goal(&forest, &o(&[0, 2]), i, &options, || false).unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());
        for (goal, outputs) in [(o(&[0]), 0), (o(&[2, 4]), 2)] {
            let error = prove_goal(&forest, &goal, i, &options, || false).unwrap_err();
            assert!(
                matches!(error, Error::GoalOutputs(n) if n == outputs),
                "{error}"
            );
        }
    }

    /// A thread count is taken as at least one and at most
    /// [`Options::MAX_JOBS`], whatever is asked for.
    #[test]
    fn jobs_are_bounded() {
        let most = Options::default().jobs(Options::MAX_JOBS);
        assert_eq!(Options::default().jobs(usize::MAX), most);
        assert_eq!(Options::default().jobs(0), Options::default().jobs(1));
        assert_ne!(Options::default().jobs(2), Options::default());
    }

    /// The stop condition ends the search with `Unknown`.
    #[test]
    fn stop() {
        let outcome = prove_until(
            &sequent("|- a * b, ~a, ~b"),
            Mode::CLASSICAL,
            &Options::default(),
            || true,
        )
        .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)));
        assert_eq!(Reason::Stopped.to_string(), "the search was stopped");
    }
}

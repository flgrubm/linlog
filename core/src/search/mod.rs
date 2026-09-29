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
/// The proof-net engine.
pub mod net;

use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::nets::ProofStructure;
use crate::occurrences::{Forest, OccId, Reading};
use crate::proofs::Proof;
use crate::sequents::Sequent;
use std::fmt::{Display, Formatter, Result as FmtResult};

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
/// with more subformula occurrences than a forest can index
/// ([`Error::TooManyOccurrences`]).
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
/// assert!(matches!(outcome.verdict, Verdict::Unprovable));
/// let outcome = prove(&sequent, Mode::CLASSICAL.with_mix(), &Options::default())?;
/// assert!(matches!(outcome.verdict, Verdict::Proved(_)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove(sequent: &Sequent, mode: Mode, options: &Options) -> Result<Outcome, Error> {
    prove_until(sequent, mode, options, || false)
}

/// Decides a sequent as [`prove`] does, polling `stop` at every stable
/// sequent and giving up with [`Reason::Stopped`] once it returns true. The
/// condition is the caller's: a deadline on a clock the caller has, a flag
/// an interrupt handler sets. This crate has no clock of its own.
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
    let forest = Forest::new(sequent)?;
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
    let (verdict, statistics, net) = match engine {
        Engine::Net => net::search(forest, mode, options, &mut stop),
        Engine::Focus | Engine::TwoSided | Engine::Additive => {
            let (result, nodes, statistics) = if engine == Engine::Additive {
                additive::search_goal(forest, goal, options, &mut stop)
            } else {
                focus::search_goal(
                    forest,
                    goal,
                    fragment,
                    mode,
                    reading.as_ref(),
                    options,
                    &mut stop,
                )
            };
            let verdict = match result {
                Ok(Some(root)) => {
                    let proof = Proof::new(forest.clone(), nodes, root)
                        .expect("the engine pushes premises before conclusions");
                    debug_assert!(
                        !is_roots || proof.check(mode).is_ok(),
                        "the engine's proof: {:?}",
                        proof.check(mode)
                    );
                    Verdict::Proved(Box::new(proof))
                }
                Ok(None) => Verdict::Unprovable,
                Err(reason) => Verdict::Unknown(reason),
            };
            (verdict, statistics, None)
        }
    };
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
fn goal_fragment(forest: &Forest, goal: &[OccId]) -> Fragment {
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
/// focused engine enumerates context splits.
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
    /// The most copies of `?` formulas one branch may take.
    copies: u32,
}

impl Default for Options {
    /// A memo of at most [`DEFAULT_MEMO_LIMIT`](Self::DEFAULT_MEMO_LIMIT)
    /// stable sequents, a recursion limit of
    /// [`DEFAULT_RECURSION_LIMIT`](Self::DEFAULT_RECURSION_LIMIT), the
    /// engine and fragment chosen by detection, the net engine's exact test
    /// at its default cadence, and a copy bound of
    /// [`DEFAULT_COPIES`](Self::DEFAULT_COPIES).
    fn default() -> Self {
        Self {
            memo_limit: Self::DEFAULT_MEMO_LIMIT,
            recursion_limit: Self::DEFAULT_RECURSION_LIMIT,
            engine: None,
            fragment: None,
            test_period: None,
            copies: Self::DEFAULT_COPIES,
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
    /// bound llprover searches with by default.
    pub const DEFAULT_COPIES: u32 = 3;

    /// Sets the most copies of `?` formulas one branch of a proof may take.
    /// The search deepens the bound from zero up to this value; a sequent
    /// that has no proof within it is [`Reason::CopyBound`], unless some
    /// level finished without ever reaching its bound, which makes the
    /// sequent [`Verdict::Unprovable`]. Without exponentials the bound has
    /// no effect. A proof found at some level may reuse a memoized subproof
    /// found with more copies left, so the bound limits the search, not the
    /// proof returned.
    pub fn copies(self, copies: u32) -> Self {
        Self { copies, ..self }
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
    /// switches the memo off.
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
    /// The sequent is not provable: the search was exhaustive.
    Unprovable,
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

/// Why a search stopped without deciding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reason {
    /// The caller's stop condition fired: a time limit or an interruption.
    Stopped,
    /// The nesting of engine calls reached [`Options::recursion_limit`].
    RecursionLimit,
    /// A `⊗` or Mix had to split a context of this many formulas, more than
    /// the split enumeration handles (63).
    ContextTooWide(usize),
    /// Every level up to [`Options::copies`], which is this value, hit its
    /// bound on some branch, so a proof with more copies of a `?` formula
    /// per branch may exist.
    CopyBound(u32),
}

impl Display for Reason {
    /// Writes the reason as a phrase, such as `the time limit was reached`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Reason::Stopped => f.write_str("the search was stopped"),
            Reason::RecursionLimit => f.write_str("the recursion limit was reached"),
            Reason::ContextTooWide(n) => {
                write!(f, "a context of {n} formulas is too wide to split")
            }
            Reason::CopyBound(n) => {
                write!(f, "the copy bound of {n} was reached")
            }
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
    /// The most stable sequents the memo held at once.
    pub memo_entries: usize,
    /// The context splits examined for `⊗` and Mix, most of them rejected by
    /// the counts.
    pub splits: u64,
    /// The axiom links the net engine tried: each was made, and taken back
    /// again unless it is part of the net found.
    pub links: u64,
    /// The exact acyclicity tests the net engine ran.
    pub tests: u64,
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
                    Verdict::Unprovable => false,
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
            matches!(outcome.verdict, Verdict::Unprovable),
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
        assert!(matches!(outcome.verdict, Verdict::Unprovable));
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

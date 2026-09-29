// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof search: the front door `prove` with its options and outcome, the
//! dispatch on fragment and mode, and one submodule per engine: proof-net
//! search for unit-free MLL, the focused sequent engine for everything else
//! up to MALL, and later the additive fast path.

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
use crate::occurrences::Forest;
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
/// No engine handles intuitionistic mode yet ([`Error::NoEngine`]); a
/// sequent outside the fragment the options assert is refused
/// ([`Error::FragmentMismatch`]); the net engine is refused outside
/// unit-free MLL ([`Error::NetFragment`]) and in affine mode
/// ([`Error::NetMode`]); and a sequent with more subformula occurrences
/// than a forest can index is refused ([`Error::TooManyOccurrences`]).
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
    mut stop: impl FnMut() -> bool,
) -> Result<Outcome, Error> {
    let detected = sequent.fragment();
    let fragment = match options.fragment {
        Some(asserted) if !asserted.contains(detected) => {
            return Err(Error::FragmentMismatch { asserted, detected });
        }
        Some(asserted) => asserted,
        None => detected,
    };
    // The dispatch: unit-free MLL with mostly distinct atoms goes to the net
    // engine in classical mode, every other classical fragment and every
    // fragment in affine mode to the focused engine; intuitionistic mode
    // has no engine yet.
    if mode.intuitionistic {
        return Err(Error::NoEngine { fragment, mode });
    }
    let is_mll = Fragment::MLL.contains(fragment);
    let forest = Forest::new(sequent)?;
    let engine = options
        .engine
        .unwrap_or(if is_mll && !mode.affine && prefers_net(&forest) {
            Engine::Net
        } else {
            Engine::Focus
        });
    if engine == Engine::Net && !is_mll {
        return Err(Error::NetFragment(fragment));
    }
    if engine == Engine::Net && mode.affine {
        return Err(Error::NetMode(mode));
    }
    let (verdict, statistics, net) = match engine {
        Engine::Focus => {
            let (verdict, statistics) = focus::search(&forest, fragment, mode, options, &mut stop);
            (verdict, statistics, None)
        }
        Engine::Net => net::search(&forest, mode, options, &mut stop),
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
    /// The focused sequent engine of [`focus`].
    Focus,
    /// The proof-net engine of [`net`], for unit-free MLL only.
    Net,
}

impl Display for Engine {
    /// Writes the engine's name: `focus` or `net`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Engine::Focus => f.write_str("focus"),
            Engine::Net => f.write_str("net"),
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
/// tests; the other counters stay zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Statistics {
    /// The nodes of the search: the stable sequents the focused engine
    /// visited, memo hits included, or the literals the net engine chose a
    /// partner for.
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
            ("|- a & b, ~a + ~b", Fragment::ADDITIVES, Engine::Focus),
            ("|- top, 0", Fragment::ADDITIVE_UNITS, Engine::Focus),
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

    /// Intuitionistic mode, which no engine handles yet, is refused, not
    /// searched; affine mode and the exponentials go to the focused engine,
    /// and the net engine is refused in affine mode.
    #[test]
    fn dispatch_by_mode() {
        let error = prove(
            &sequent("a |- a"),
            Mode::INTUITIONISTIC,
            &Options::default(),
        )
        .unwrap_err();
        assert!(matches!(error, Error::NoEngine { .. }));
        assert_eq!(
            error.to_string(),
            "no engine for MLL in intuitionistic mode yet"
        );
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

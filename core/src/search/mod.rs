// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof search: the front door `prove` with its options and outcome, the
//! dispatch on fragment and mode, and one submodule per engine (the focused
//! sequent engine, proof-net search, the additive fast path), as
//! `plan/README.md` lays out. Only the focused engine exists so far.

/// The focused sequent engine.
pub mod focus;
/// Random provable sequents for the tests.
#[cfg(test)]
pub(crate) mod generate;

use crate::Error;
use crate::fragment::{Fragment, Mode};
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
/// No engine handles intuitionistic or affine mode or exponentials yet
/// ([`Error::NoEngine`]); a sequent outside the fragment the options assert
/// is refused ([`Error::FragmentMismatch`]); and a sequent with more
/// subformula occurrences than a forest can index is refused
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
/// assert_eq!(outcome.engine, Engine::Focus);
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
    // The dispatch table of `plan/README.md` (D8): every classical row up to
    // MALL goes to the focused engine; the other rows have no engine yet.
    if mode.intuitionistic || mode.affine || fragment.has_exponentials() {
        return Err(Error::NoEngine { fragment, mode });
    }
    let engine = options.engine.unwrap_or(Engine::Focus);
    let forest = Forest::new(sequent)?;
    let (verdict, statistics) = match engine {
        Engine::Focus => focus::search(&forest, fragment, mode, options, &mut stop),
    };
    Ok(Outcome {
        verdict,
        fragment,
        engine,
        statistics,
    })
}

/// The engines, by which an outcome names the one that ran and the options
/// force one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Engine {
    /// The focused sequent engine of [`focus`].
    Focus,
}

impl Display for Engine {
    /// Writes the engine's name: `focus`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Engine::Focus => f.write_str("focus"),
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
}

impl Default for Options {
    /// A memo of at most 2²⁰ stable sequents, a recursion limit of 2048, and
    /// the engine and fragment chosen by detection.
    fn default() -> Self {
        Self {
            memo_limit: 1 << 20,
            recursion_limit: 2048,
            engine: None,
            fragment: None,
        }
    }
}

impl Options {
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
    /// The engine that ran.
    pub engine: Engine,
    /// What the search cost.
    pub statistics: Statistics,
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
        }
    }
}

/// What a search cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Statistics {
    /// The stable sequents visited, memo hits included.
    pub nodes: u64,
    /// The visits answered from the memo.
    pub memo_hits: u64,
    /// The most stable sequents the memo held at once.
    pub memo_entries: usize,
    /// The context splits examined for `⊗` and Mix, most of them rejected by
    /// the counts.
    pub splits: u64,
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// Every classical input without exponentials reaches the focused
    /// engine, and the outcome says which fragment it was searched in.
    #[test]
    fn dispatch() {
        for (input, fragment) in [
            ("|- a, ~a", Fragment::EMPTY),
            ("a, a -o b |- b", Fragment::MLL),
            ("|- 1, bot", Fragment::MULTIPLICATIVE_UNITS),
            ("|- a & b, ~a + ~b", Fragment::ADDITIVES),
            ("|- top, 0", Fragment::ADDITIVE_UNITS),
            ("|- (a * top) + 1, ~a, bot", Fragment::MALL),
        ] {
            let outcome = prove(&sequent(input), Mode::CLASSICAL, &Options::default()).unwrap();
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert_eq!(outcome.engine, Engine::Focus);
            assert!(outcome.verdict.proof().is_some(), "{input:?}");
        }
    }

    /// The asserted fragment must contain the sequent's, and is what the
    /// search runs in.
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

    /// What no engine handles yet is refused, not searched.
    #[test]
    fn no_engine_yet() {
        for (input, mode, message) in [
            (
                "a |- a",
                Mode::INTUITIONISTIC,
                "no engine for MLL in intuitionistic mode yet",
            ),
            (
                "a, b |- a",
                Mode::CLASSICAL.affine(),
                "no engine for MLL in classical affine mode yet",
            ),
            (
                "!a |- a",
                Mode::CLASSICAL,
                "no engine for MELL in classical mode yet",
            ),
            (
                "!a |- a & a",
                Mode::CLASSICAL.with_mix(),
                "no engine for LL in classical with Mix mode yet",
            ),
        ] {
            let error = prove(&sequent(input), mode, &Options::default()).unwrap_err();
            assert!(matches!(error, Error::NoEngine { .. }), "{input:?}");
            assert_eq!(error.to_string(), message, "{input:?}");
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

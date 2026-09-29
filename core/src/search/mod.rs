// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof search: the front door `prove` with its options and outcome, the
//! dispatch on fragment and mode, and one submodule per engine (the focused
//! sequent engine, proof-net search, the additive fast path), as
//! `plan/README.md` lays out. Only the focused engine exists so far.

/// The focused sequent engine.
pub mod focus;

use crate::proofs::Proof;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The knobs of a search: how much to remember and how deep to go. The
/// defaults suit a sequent of a few hundred occurrences on a thread with
/// the usual stack.
///
/// # Examples
///
/// ```
/// use linlog::search::Options;
///
/// let options = Options::default().memo_limit(1 << 16).recursion_limit(10_000);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// The most stable sequents the memo holds at once.
    memo_limit: usize,
    /// The deepest nesting of engine calls before the search gives up.
    recursion_limit: u32,
}

impl Default for Options {
    /// A memo of at most 2²⁰ stable sequents and a recursion limit of 2048.
    fn default() -> Self {
        Self {
            memo_limit: 1 << 20,
            recursion_limit: 2048,
        }
    }
}

impl Options {
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

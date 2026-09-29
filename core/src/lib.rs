// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Sequents of linear logic, stored as compact arena DAGs in negation normal
//! form, with parsing, printing and serialization; the fragment a sequent
//! lives in and the mode proof search runs in; the occurrence forest of a
//! sequent, the numbering of its subformula occurrences that proof search,
//! proof checking and proof nets are built on; and proofs as terms over
//! those occurrences, with the checker that validates them, the derivation
//! view that renders them, and their serialization; and proof search, which
//! decides a sequent with the engine its fragment calls for and returns a
//! checked proof.
//!
//! # The common path
//!
//! Parse a sequent, see which fragment it lives in, prove it, print the
//! derivation and serialize the proof:
//!
#![cfg_attr(all(feature = "parse", feature = "serialize"), doc = "```")]
#![cfg_attr(not(all(feature = "parse", feature = "serialize")), doc = "```ignore")]
//! use linlog::{Fragment, Mode, Options, Sequent, Verdict, prove};
//!
//! // Two-sided input becomes one-sided: ⊢ ~A, A ⊗ ~B, B.
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! assert_eq!(sequent.fragment(), Fragment::MLL);
//!
//! let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("the sequent is provable");
//! };
//! assert_eq!(
//!     proof.derivation()?.to_string(),
//!     "─────── ax   ─────── ax\n\
//!      ⊢ ~A, A      ⊢ ~B, B\n\
//!      ──────────────────── ⊗\n\
//!     \x20 ⊢ ~A, A ⊗ ~B, B"
//! );
//!
//! let json = serde_json::to_string(&proof)?;
//! let back: linlog::Proof = serde_json::from_str(&json)?;
//! back.check(Mode::CLASSICAL)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A [`Mode`] is three named flags, [`Options`] a builder with defaults, and
//! [`prove_until`] takes a stop condition for a time limit or an
//! interruption, since this crate has no clock:
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::{Fragment, Mode, Options, Verdict, prove_until};
//!
//! let mode = Mode { intuitionistic: false, affine: false, mix: true };
//! let options = Options::default().fragment(Some(Fragment::MALL));
//! let mut budget = 1000;
//! let outcome = prove_until(&"|- A par B, ~A, ~B".parse()?, mode, &options, || {
//!     budget -= 1;
//!     budget == 0
//! })?;
//! assert!(matches!(outcome.verdict, Verdict::Proved(_)));
//! assert_eq!(outcome.fragment, Fragment::MALL);
//! # Ok::<(), linlog::Error>(())
//! ```
//!
//! Errors are one type, [`Error`], whose messages say what is wrong with the
//! input; a proof that fails the checker gives a [`CheckError`], which
//! [`CheckError::describe`] prints with formulas.

#![allow(dead_code)]
#![allow(unused_variables)]

/// The error types of this crate.
mod errors;
/// Export of sequents and derivations to LaTeX, Typst and SVG, and of
/// proof nets to SVG.
pub mod export;
/// Problem families with known verdicts, for benchmarks and tests.
#[cfg(feature = "parse")]
pub mod families;
/// Fragments of linear logic, modes of proof search, and fragment detection.
pub mod fragment;
/// The hash tables of this crate.
mod hash;
/// Problems of the LLTP benchmark library.
#[cfg(feature = "parse")]
pub mod lltp;
/// Proof nets.
pub mod nets;
/// The occurrence forest of a sequent and sets over it.
pub mod occurrences;
/// Parsing sequents from text.
#[cfg(feature = "parse")]
mod parse;
/// Proof terms, the checker and derivations.
pub mod proofs;
/// Proof search.
pub mod search;
/// Sequents and the terms they are built from.
pub mod sequents;
/// Serde support for sequents and proofs.
#[cfg(feature = "serialize")]
mod serialize;

#[cfg(feature = "parse")]
pub use errors::ParseError;

pub use errors::Error;
pub use fragment::{Fragment, Mode};
pub use nets::{NetError, ProofStructure, Scratch};
pub use occurrences::{Forest, OccId, OccSet, Polarity, Position, Reading, ShapeError, Sign};
pub use proofs::{CheckError, Derivation, InfId, Inference, Node, NodeId, Proof, Rule, Side};
#[cfg(feature = "interactive")]
pub use proofs::{Interactive, Refusal};
pub use search::{
    Engine, Options, Outcome, Reason, Statistics, Verdict, prove, prove_goal, prove_until,
};
pub use sequents::{Atom, Formula, Kind, Sequent, Term, TermId};

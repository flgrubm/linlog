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

#![allow(dead_code)]
#![allow(unused_variables)]

/// The error types of this crate.
mod errors;
/// Export to LaTeX, Typst, SVG and Rocq.
pub mod export;
/// Fragments of linear logic, modes of proof search, and fragment detection.
pub mod fragment;
/// The hash tables of this crate.
mod hash;
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
pub use occurrences::{Forest, OccId, OccSet, Polarity, Sign};
pub use proofs::{CheckError, Derivation, InfId, Inference, Node, NodeId, Proof, Rule, Side};
pub use search::{Engine, Options, Outcome, Reason, Statistics, Verdict, prove, prove_until};
pub use sequents::{Atom, Formula, Kind, Sequent, Term, TermId};

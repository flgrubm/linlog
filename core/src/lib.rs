// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Sequents of classical linear logic, stored as compact arena DAGs in
//! negation normal form, with parsing, printing and serialization.

#![allow(dead_code)]
#![allow(unused_variables)]

/// The error types of this crate.
mod errors;
/// The hash tables of this crate.
mod hash;
/// Algorithms specific to one logic fragment, such as proof search.
pub mod linear;
/// Parsing sequents from text.
#[cfg(feature = "parse")]
mod parse;
/// Sequents and the terms they are built from.
pub mod sequents;
/// Serde support for sequents.
#[cfg(feature = "serialize")]
mod serialize;

#[cfg(feature = "parse")]
pub use errors::ParseError;

pub use errors::Error;
pub use sequents::{Atom, Formula, Kind, Sequent, Term, TermId};

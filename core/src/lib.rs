// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Sequents of classical linear logic and its fragments, stored as compact
//! arena DAGs in negation normal form, with parsing, printing and
//! serialization.

#![allow(dead_code)]
#![allow(unused_variables)]

/// The error types of this crate.
mod errors;
/// The unsigned integer types an arena can be indexed with.
pub mod index;
/// Algorithms specific to one logic fragment, such as proof search.
pub mod linear;
/// Marker types for the logic fragments and their expression types.
pub mod logics;
/// Parsing sequents from text.
#[cfg(feature = "parse")]
pub mod parse;
/// Sequents and the expressions they are built from.
pub mod sequents;
/// Serde support for sequents.
#[cfg(feature = "serialize")]
mod serialize;

#[cfg(feature = "parse")]
pub use errors::ParseError;

pub use errors::Error;

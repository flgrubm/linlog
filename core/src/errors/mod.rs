// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// The error type for text that is not a sequent.
#[cfg(feature = "parse")]
mod parse;

#[cfg(feature = "parse")]
pub use parse::ParseError;

use thiserror::Error;

/// Everything that can go wrong in this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// A term refers to a variable index (first) outside the variable
    /// dictionary (its length second).
    #[error("Variable name at index does not exist: require index < {1}, but have index == {0}")]
    InvalidVariableIndex(usize, usize),
    /// A root formula index (first) lies outside the arena (its length second).
    #[error("Term index out of bounds: tried to access index {0} while number of terms is {1}")]
    TermIndexOutOfBounds(usize, usize),
    /// A subterm index (first) is not below the index of its parent term
    /// (second).
    #[error("Term index not decreasing: term at index {0} has subterm at index {1} >= {0}")]
    SubtermIndexNotDecreasing(usize, usize),
    /// The input is not a sequent, for each of the reasons listed.
    #[cfg(feature = "parse")]
    #[error("Parsing failed with errors: {0:?}")]
    SequentParsing(Vec<ParseError>),
}

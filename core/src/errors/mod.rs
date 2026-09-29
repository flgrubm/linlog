// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// The error type for text that is not a sequent.
#[cfg(feature = "parse")]
mod parse;

#[cfg(feature = "parse")]
pub use parse::ParseError;

use crate::proofs::CheckError;
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
    #[error("Term index not decreasing: term at index {1} has subterm at index {0} >= {1}")]
    SubtermIndexNotDecreasing(usize, usize),
    /// A sequent has at least this many subformula occurrences, more than a
    /// forest can index.
    #[error("Sequent has at least {0} subformula occurrences, more than a forest can index")]
    TooManyOccurrences(u64),
    /// The input is not a sequent, for each of the reasons listed.
    #[cfg(feature = "parse")]
    #[error("Parsing failed with errors: {0:?}")]
    SequentParsing(Vec<ParseError>),
    /// A proof node index (first) lies outside the arena (its length second).
    #[error(
        "Proof node index out of bounds: tried to access node {0} while number of nodes is {1}"
    )]
    NodeIndexOutOfBounds(usize, usize),
    /// A premise's node index (first) is not below the index of the node it
    /// proves (second).
    #[error("Proof node index not decreasing: node {1} has premise {0} >= {1}")]
    PremiseIndexNotDecreasing(usize, usize),
    /// A proof node refers to an occurrence (first) outside its forest (its
    /// length second).
    #[error(
        "Occurrence index out of bounds: a proof node refers to occurrence {0} while the forest has {1}"
    )]
    OccurrenceIndexOutOfBounds(usize, usize),
    /// A proof does not prove its sequent.
    #[error("Invalid proof: {0}")]
    InvalidProof(#[from] CheckError),
}

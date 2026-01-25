// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

#[cfg(feature = "parse")]
mod parse;

#[cfg(feature = "parse")]
pub use parse::ParseError;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Variable name at index does not exist: require index < {1}, but have index == {0}")]
    InvalidVariableIndex(usize, usize),
    #[error("Term index out of bounds: tried to access index {0} while number of terms is {1}")]
    TermIndexOutOfBounds(usize, usize),
    #[error("Term index not decreasing: term at index {0} has subterm at index {1} >= {0}")]
    SubtermIndexNotDecreasing(usize, usize),
    #[cfg(feature = "parse")]
    #[error("Parsing failed with errors: {0:?}")]
    SequentParsing(Vec<ParseError>),
}

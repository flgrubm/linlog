// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use thiserror::Error;

// Chumsky parsing errors

use chumsky::error::Simple;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub span: std::ops::Range<usize>,
    pub found: Option<String>,
    pub label: Option<String>,
    pub expected: Vec<String>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at {:?}: found {}",
            self.span,
            self.found.as_deref().unwrap_or("end of input")
        )?;
        if let Some(label) = &self.label {
            write!(f, ", expected {}", label)?;
        }
        if !self.expected.is_empty() {
            write!(f, ", expected one of: {:?}", self.expected)?;
        }
        Ok(())
    }
}

impl<'a, T> From<Simple<'a, T>> for ParseError
where
    T: fmt::Display,
{
    fn from(e: Simple<'a, T>) -> Self {
        ParseError {
            span: (*e.span()).into_range(),

            found: e.found().map(|t| t.to_string()),

            label: None,
            expected: Vec::new(),
        }
    }
}

// --------------------------------------------------

#[derive(Error, Debug)]
pub enum Error {
    #[error("Preterm is not a valid term, unexpected token: {0}")]
    MalformedTerm(String),
    #[error("Conversion from LL to MELL failed, unexpected: {0}")]
    LLToMELLConversion(String),
    #[error("Conversion from LL to MLL failed, unexpected: {0}")]
    LLToMLLConversion(String),
    #[error("Conversion from MELL to MLL failed, unexpected: {0}")]
    MELLToMLLConversion(String),
    #[error("Parsing failed with errors: {0:?}")]
    SequentParsing(Vec<ParseError>),
}

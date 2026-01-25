// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

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

impl<'a, T: fmt::Display> From<Simple<'a, T>> for ParseError {
    fn from(e: Simple<'a, T>) -> Self {
        ParseError {
            span: (*e.span()).into_range(),

            found: e.found().map(|t| t.to_string()),

            label: None,
            expected: Vec::new(),
        }
    }
}

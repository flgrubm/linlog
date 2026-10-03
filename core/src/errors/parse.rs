// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use std::fmt;

/// Where and why parsing failed, owned so that it outlives the input.
#[derive(Debug)]
pub struct ParseError {
    /// The byte range of the input where parsing failed.
    pub span: std::ops::Range<usize>,
    /// The token found there, or `None` at the end of the input.
    pub found: Option<String>,
    /// What the parser was trying to parse, if known.
    pub label: Option<String>,
    /// The tokens that would have been accepted there.
    pub expected: Vec<String>,
}

impl fmt::Display for ParseError {
    /// Writes what was found where parsing failed, and what was expected if
    /// known, as in `unexpected "-" at byte 8`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.found {
            Some(token) => write!(f, "unexpected {token:?} at byte {}", self.span.start)?,
            None => f.write_str("unexpected end of input")?,
        }
        if let Some(label) = &self.label {
            write!(f, ", expected {label}")?;
        }
        if !self.expected.is_empty() {
            write!(f, ", expected one of {}", self.expected.join(", "))?;
        }
        Ok(())
    }
}

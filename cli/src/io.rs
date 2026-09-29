// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::SequentInput;
use anyhow::{Context, Result, bail};
use linlog::Sequent;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;

/// Reads all of a file, or of standard input for `None` or `-`. Refuses to
/// wait on a terminal, where the user most likely forgot the input.
pub fn read(path: Option<&Path>, what: &str) -> Result<String> {
    match path {
        Some(path) if path != Path::new("-") => {
            fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))
        }
        _ => {
            let mut stdin = io::stdin();
            if stdin.is_terminal() {
                bail!("no {what} given: pass it as an argument, with --file, or on standard input");
            }
            let mut text = String::new();
            stdin
                .read_to_string(&mut text)
                .context("cannot read standard input")?;
            Ok(text)
        }
    }
}

impl SequentInput {
    /// Reads the sequent from the argument, the file or standard input, as
    /// text or as JSON.
    pub fn sequent(&self) -> Result<Sequent> {
        let text = match &self.sequent {
            Some(text) => text.clone(),
            None => read(self.file.as_deref(), "sequent")?,
        };
        if self.json_input {
            serde_json::from_str(&text).context("not a sequent in JSON")
        } else {
            let text = text.trim();
            text.parse().map_err(|e| crate::parse_error(text, e))
        }
    }
}

/// Writes `text` and a newline to the file, or to standard output for
/// `None`.
pub fn write(path: Option<&Path>, text: &str) -> Result<()> {
    match path {
        Some(path) => fs::write(path, format!("{text}\n"))
            .with_context(|| format!("cannot write {}", path.display())),
        None => {
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{text}").context("cannot write to standard output")
        }
    }
}

// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Problems of the LLTP library (`github.com/meta-logic/lltp`), the
//! benchmark collection of linear logic provers: TPTP-style files of
//! `fof(name, role, formula).` clauses whose axioms are the hypotheses and
//! whose conjecture is the goal. The formulas are written as this crate's
//! parser reads them (`*`, `|`, `&`, `+`, `-o`, `!`, `?`, postfix `^`,
//! `1`, `0`, `bot`, `top`), with the same precedences, so a problem is
//! read by assembling `axioms |- conjecture` and parsing that.
//!
//! The file says nothing about the mode: the library keeps its
//! intuitionistic problems under `ILL/` and its classical ones under
//! `CLL/`, and the caller picks the mode by that.
//!
//! # Examples
//!
//! ```
//! use linlog::lltp::{Status, read};
//!
//! let text = "% Status (intuit.) : Theorem\n\
//!             fof(ax1, axiom, !(a -o b)).\n\
//!             fof(ax2, axiom, a).\n\
//!             fof(con, conjecture, b * 1).\n";
//! let problem = read(text)?;
//! assert_eq!(problem.status, Some(Status::Theorem));
//! assert_eq!(problem.sequent, "!(a -o b), a |- b * 1".parse()?);
//! # Ok::<(), linlog::Error>(())
//! ```

use crate::{Error, Sequent};

/// What stands for a `-` inside an atom name, as the library's Petri nets
/// write them (`P-start_1_1`): this crate's names cannot hold a `-`, which
/// starts the operator `-o`, and `‿` is a character that joins words in a
/// Unicode identifier.
pub const HYPHEN: char = '‿';

/// What stands for a `.` inside an atom name, as some of the library's
/// Petri nets write them (`merge.s00001061.input`): a `.` ends a clause in
/// the file and is no name character here, and `·` is one in a Unicode
/// identifier.
pub const DOT: char = '·';

/// A problem read from an LLTP file.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Problem {
    /// The sequent of the problem: the axioms, in the order of the file,
    /// left of `⊢`, the conjecture right of it.
    pub sequent: Sequent,
    /// The provability the file's header claims, if it has a `Status`
    /// line this reader understands.
    pub status: Option<Status>,
}

/// The provability an LLTP header claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// `Theorem`: the sequent is provable.
    Theorem,
    /// `Non-Theorem` or `CounterSatisfiable`: the sequent is not provable.
    NonTheorem,
}

/// Reads an LLTP problem. Every `fof` clause with the role `axiom` or
/// `hypothesis` is a hypothesis and every one with the role `conjecture`
/// a formula on the right of `⊢`; lines from `%` on are comments. The
/// status is the first `Status (intuit.)` or `Status (linear)` comment's,
/// else the first plain `Status` comment's: the library's translations of
/// intuitionistic problems keep the classical source's status first. A
/// `-` inside an atom name becomes [`HYPHEN`], a `.` [`DOT`].
///
/// # Errors
///
/// [`Error::Lltp`] for text that is not a sequence of `fof(name, role,
/// formula).` clauses (an annotation after the formula included) or a
/// clause with another role, and
/// [`Error::SequentParsing`] for a formula this crate's parser rejects.
pub fn read(text: &str) -> Result<Problem, Error> {
    // The status of a line for the logic of the problem, and of a plain one.
    let (mut status, mut plain) = (None, None);
    let mut code = String::with_capacity(text.len());
    for line in text.lines() {
        let (before, comment) = line.split_once('%').unwrap_or((line, ""));
        if let Some((key, word)) = comment
            .split_once("Status")
            .and_then(|(_, r)| r.rsplit_once(':'))
        {
            let word = word.trim();
            let value = if word.starts_with("Theorem") {
                Some(Status::Theorem)
            } else if word.starts_with("Non-Theorem") || word.starts_with("CounterSatisfiable") {
                Some(Status::NonTheorem)
            } else {
                None
            };
            let slot = if key.contains("(intuit") || key.contains("(linear") {
                &mut status
            } else {
                &mut plain
            };
            if slot.is_none() {
                *slot = value;
            }
        }
        let chars: Vec<char> = before.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            let name = |j: usize| {
                chars
                    .get(j)
                    .is_some_and(|&c| c.is_ascii_alphanumeric() || c == '_')
            };
            // A `-` or `.` between two characters of a name belongs to the
            // name, unless the `-` starts the operator `-o`.
            let inner = i > 0 && name(i - 1) && name(i + 1);
            code.push(match c {
                '-' if inner && !(chars[i + 1] == 'o' && !name(i + 2)) => HYPHEN,
                '.' if inner => DOT,
                c => c,
            });
        }
        code.push('\n');
    }

    let mut hypotheses = Vec::new();
    let mut conjectures = Vec::new();
    let mut rest = code.trim_start();
    while !rest.is_empty() {
        let body = rest
            .strip_prefix("fof")
            .map(str::trim_start)
            .and_then(|r| r.strip_prefix('('))
            .ok_or_else(|| Error::Lltp(format!("expected `fof(` at `{}`", excerpt(rest))))?;
        let (name, body) = body.split_once(',').ok_or_else(|| {
            Error::Lltp(format!("a clause without a role at `{}`", excerpt(rest)))
        })?;
        let name = name.trim();
        let (role, body) = body
            .split_once(',')
            .ok_or_else(|| Error::Lltp(format!("clause `{name}` has no formula")))?;
        // The formula runs to the parenthesis that closes `fof(`; a comma
        // outside its parentheses starts an annotation.
        let mut depth = 1usize;
        let end = body
            .char_indices()
            .find(|&(_, c)| {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                depth == 0 || (depth == 1 && c == ',')
            })
            .ok_or_else(|| Error::Lltp(format!("clause `{name}` is not closed")))?;
        if end.1 == ',' {
            return Err(Error::Lltp(format!(
                "clause `{name}` has an annotation, which this reader does not take"
            )));
        }
        let end = end.0;
        let formula = body[..end].trim();
        match role.trim() {
            "axiom" | "hypothesis" => hypotheses.push(formula),
            "conjecture" => conjectures.push(formula),
            other => {
                return Err(Error::Lltp(format!(
                    "clause `{name}` has the role `{other}`, not axiom, hypothesis or conjecture"
                )));
            }
        }
        rest = body[end + 1..]
            .trim_start()
            .strip_prefix('.')
            .ok_or_else(|| Error::Lltp(format!("clause `{name}` does not end with `).`")))?
            .trim_start();
    }
    if conjectures.is_empty() {
        return Err(Error::Lltp("no conjecture".to_owned()));
    }
    let sequent = format!("{} |- {}", hypotheses.join(", "), conjectures.join(", ")).parse()?;
    Ok(Problem {
        sequent,
        status: status.or(plain),
    })
}

/// Returns the start of `text`, for an error message.
fn excerpt(text: &str) -> &str {
    let end = text.char_indices().nth(24).map_or(text.len(), |(i, _)| i);
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Clauses over several lines, comments between and after them, a
    /// `hypothesis` role, uppercase atoms, names with `-` and `.`, postfix
    /// negation and every unit read into the sequent the parser gives for
    /// the same formulas; the intuitionistic status wins over the plain
    /// one before it, and malformed files are refused.
    #[test]
    fn reads_problems() {
        let text = "%---\n% Status   : Theorem\n% Status (intuit.) : Non-Theorem\n%---\n\
                    fof(ax1, axiom, !( (P-a_1) -o\n  (Q.b * Q.b) ) ).   % a transition\n\
                    fof(h, hypothesis, bot | top^).\n\
                    fof(con, conjecture, (Q.b * Q.b) + 0 & 1).\n";
        let problem = read(text).unwrap();
        assert_eq!(problem.status, Some(Status::NonTheorem));
        let expected: Sequent = "!(P‿a_1 -o Q·b * Q·b), bot | top^ |- (Q·b * Q·b) + 0 & 1"
            .parse()
            .unwrap();
        assert_eq!(problem.sequent, expected);

        for bad in [
            "fof(a, axiom, a).",
            "fof(a, axiom, a). fof(c, conjecture, a",
            "fof(c, conjecture, a) fof(d, axiom, b).",
            "fof(c, definition, a).",
            "fof(c, conjecture, a, unknown).",
            "cnf(c, conjecture, a).",
        ] {
            assert!(matches!(read(bad), Err(Error::Lltp(_))), "{bad:?}");
        }
    }
}

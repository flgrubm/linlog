// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Typst: formulas in math mode.
//!
//! The connectives are written as their Unicode characters (`⊗ ⅋ ⊕ ⊸ ⊢
//! ⊥ ⊤`), which Typst's math reads with their Unicode math class and which
//! no renaming of Typst's symbol names can break; `&` is escaped and set
//! as a binary operator, `?` set as an ordinary symbol so that no
//! punctuation space follows it, and `1` bold. A fragment is a sequent in
//! `$…$`; a standalone document sizes the page to its content.
//!
//! An atom named by one letter is written as it is, in math italic; any
//! other name is a string in `italic(…)`, with `"` and `\` escaped, since
//! Typst would read a longer name as a variable of its own. Typst is
//! Unicode throughout, so names beyond ASCII need nothing else.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::export::{Form, typst};
//! use linlog::Sequent;
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! assert_eq!(typst::sequent(&sequent, Form::Fragment), "$⊢ A^⊥, A ⊗ B^⊥, B$");
//! # Ok::<(), linlog::Error>(())
//! ```

use super::Form;
use super::notation::Notation;
use crate::occurrences::Reading;
use crate::sequents::Sequent;

/// The Typst spelling of formulas and sequents.
const NOTATION: Notation = Notation {
    tensor: "⊗",
    par: "⅋",
    with: r#"class("binary", \&)"#,
    plus: "⊕",
    lollipop: "⊸",
    bang: "!",
    quest: r#"class("normal", ?)"#,
    one: "bold(1)",
    bot: "⊥",
    top: "⊤",
    zero: "0",
    dual: "^⊥",
    turnstile: "⊢",
    align: "",
    atom,
};

/// The page setup of a standalone document: as large as its content.
const PAGE: &str = "#set page(width: auto, height: auto, margin: 5pt)\n";

/// Writes an atom's name in math mode.
fn atom(out: &mut String, name: &str) {
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next())
        && c.is_alphabetic()
    {
        out.push(c);
        return;
    }
    out.push_str("italic(\"");
    for c in name.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push_str("\")");
}

/// Returns a sequent one-sided, `$⊢ A^⊥, A$`, as a fragment or a
/// standalone document.
pub fn sequent(sequent: &Sequent, form: Form) -> String {
    let mut out = String::from("$");
    NOTATION.one_sided(&mut out, sequent);
    out.push('$');
    match form {
        Form::Fragment => out,
        Form::Standalone => format!("{PAGE}\n{out}"),
    }
}

/// Returns the sequent of an intuitionistic reading two-sided,
/// `$A, A ⊸ B ⊢ B$`, as a fragment or a standalone document.
pub fn two_sided(reading: &Reading, form: Form) -> String {
    let forest = reading.forest();
    let mut out = String::from("$");
    NOTATION.sequent(&mut out, forest, Some(reading), forest.roots(), false);
    out.push('$');
    match form {
        Form::Fragment => out,
        Form::Standalone => format!("{PAGE}\n{out}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an atom's name as the Typst export writes it.
    fn escaped(name: &str) -> String {
        let mut out = String::new();
        atom(&mut out, name);
        out
    }

    /// One letter stays as it is; every other name is an italic string
    /// with quotes and backslashes escaped.
    #[test]
    fn names_are_escaped() {
        for (name, expected) in [
            ("A", "A"),
            ("α", "α"),
            ("foo", r#"italic("foo")"#),
            ("x_1", r#"italic("x_1")"#),
            (r#"a"b\c"#, r#"italic("a\"b\\c")"#),
        ] {
            assert_eq!(escaped(name), expected, "{name:?}");
        }
    }
}

// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! LaTeX: formulas in math mode with the connectives of the `cmll` and
//! `amssymb` packages (`\otimes`, `\parr`, `\with`, `\oplus`, `\multimap`,
//! `\oc`, `\wn`, `\mathbf{1}`, `\bot`, `\top`, `0`, `A^\bot`, `\vdash`).
//!
//! A fragment is a sequent in `$…$`, for a document that loads `amssymb`
//! and `cmll`; a standalone document is of the `standalone` class, which
//! crops the page to the content, and compiles with pdfLaTeX.
//!
//! An atom named by one ASCII letter is written as it is, in math italic;
//! any other name goes into `\mathit{…}`, with `\ { } $ # % & _` escaped by
//! a backslash, `^`, `~` and `\` written as text, and a space kept. Other
//! characters are written as they are: a name beyond ASCII needs a
//! Unicode engine such as LuaLaTeX with `unicode-math`, or a declaration
//! of the character for pdfLaTeX.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::export::{Form, latex};
//! use linlog::Sequent;
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! assert_eq!(latex::sequent(&sequent, Form::Fragment), r"$\vdash A^\bot, A \otimes B^\bot, B$");
//! # Ok::<(), linlog::Error>(())
//! ```

use super::Form;
use super::notation::Notation;
use crate::occurrences::Reading;
use crate::sequents::Sequent;

/// The LaTeX spelling of formulas and sequents.
const NOTATION: Notation = Notation {
    tensor: r"\otimes",
    par: r"\parr",
    with: r"\with",
    plus: r"\oplus",
    lollipop: r"\multimap",
    bang: r"\oc ",
    quest: r"\wn ",
    one: r"\mathbf{1}",
    bot: r"\bot",
    top: r"\top",
    zero: "0",
    dual: r"^\bot",
    turnstile: r"\vdash",
    align: "&",
    atom,
};

/// The preamble of a standalone document: `amssymb` for `\multimap`,
/// `cmll` for `\parr`, `\with`, `\oc` and `\wn`.
const PREAMBLE: &str = r"\documentclass[border=5pt]{standalone}
\usepackage{amssymb}
\usepackage{cmll}
";

/// Writes an atom's name in math mode.
fn atom(out: &mut String, name: &str) {
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next())
        && c.is_ascii_alphabetic()
    {
        out.push(c);
        return;
    }
    out.push_str(r"\mathit{");
    for c in name.chars() {
        match c {
            '\\' => out.push_str(r"\mbox{\textbackslash}"),
            '{' | '}' | '$' | '#' | '%' | '&' | '_' => {
                out.push('\\');
                out.push(c);
            }
            '^' => out.push_str(r"\mbox{\textasciicircum}"),
            '~' => out.push_str(r"\mbox{\textasciitilde}"),
            ' ' => out.push_str(r"\ "),
            c => out.push(c),
        }
    }
    out.push('}');
}

/// Returns `body` as a standalone document.
fn document(body: &str) -> String {
    format!("{PREAMBLE}\\begin{{document}}\n{body}\n\\end{{document}}")
}

/// Returns a sequent one-sided, `$\vdash A^\bot, A$`, as a fragment or a
/// standalone document.
pub fn sequent(sequent: &Sequent, form: Form) -> String {
    let mut out = String::from("$");
    NOTATION.one_sided(&mut out, sequent);
    out.push('$');
    match form {
        Form::Fragment => out,
        Form::Standalone => document(&out),
    }
}

/// Returns the sequent of an intuitionistic reading two-sided,
/// `$A, A \multimap B \vdash B$`, as a fragment or a standalone document.
pub fn two_sided(reading: &Reading, form: Form) -> String {
    let forest = reading.forest();
    let mut out = String::from("$");
    NOTATION.sequent(&mut out, forest, Some(reading), forest.roots(), false);
    out.push('$');
    match form {
        Form::Fragment => out,
        Form::Standalone => document(&out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an atom's name as the LaTeX export writes it.
    fn escaped(name: &str) -> String {
        let mut out = String::new();
        atom(&mut out, name);
        out
    }

    /// One ASCII letter stays as it is; every other name is set in
    /// `\mathit` with the special characters escaped.
    #[test]
    fn names_are_escaped() {
        for (name, expected) in [
            ("A", "A"),
            ("p", "p"),
            ("foo", r"\mathit{foo}"),
            ("x_1", r"\mathit{x\_1}"),
            ("α", r"\mathit{α}"),
            (
                r"a{b}$#%&^~\ c",
                r"\mathit{a\{b\}\$\#\%\&\mbox{\textasciicircum}\mbox{\textasciitilde}\mbox{\textbackslash}\ c}",
            ),
        ] {
            assert_eq!(escaped(name), expected, "{name:?}");
        }
    }
}

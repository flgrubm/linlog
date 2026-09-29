// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! LaTeX: formulas in math mode with the connectives of the `cmll` and
//! `amssymb` packages (`\otimes`, `\parr`, `\with`, `\oplus`, `\multimap`,
//! `\oc`, `\wn`, `\mathbf{1}`, `\bot`, `\top`, `0`, `A^\bot`, `\vdash`),
//! and derivations as proof trees of the `ebproof` package, one statement
//! per inference in its postfix order.
//!
//! A fragment is a sequent in `$…$` or a `prooftree` environment, for a
//! document that loads `amssymb`, `cmll` and `ebproof`; a standalone
//! document is of the `standalone` class, which crops the page to the
//! content, and compiles with pdfLaTeX. The turnstiles of two-sided
//! sequents are aligned in the tree with ebproof's `&`; one-sided
//! sequents are centred. An open goal of a proof in progress is its
//! sequent under vertical dots, with no inference line.
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
//! use linlog::{Mode, Options, Sequent, Verdict, prove};
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! assert_eq!(latex::sequent(&sequent, Form::Fragment), r"$\vdash A^\bot, A \otimes B^\bot, B$");
//!
//! let outcome = prove(&sequent, Mode::INTUITIONISTIC, &Options::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("provable");
//! };
//! assert_eq!(
//!     latex::derivation(&proof.two_sided_derivation()?, Form::Fragment),
//!     r"\begin{prooftree}
//! \infer0[$\mathrm{ax}$]{A &\vdash A}
//! \infer0[$\mathrm{ax}$]{B &\vdash B}
//! \infer2[$\multimap\mathrm{L}$]{A, A \multimap B &\vdash B}
//! \end{prooftree}"
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::Form;
use super::notation::{Notation, Step, walk};
use crate::occurrences::Reading;
use crate::proofs::{Derivation, Rule};
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

/// Returns the label of a rule in math mode; an open goal has none.
const fn label(rule: Rule) -> &'static str {
    use Rule::*;
    match rule {
        Ax => r"\mathrm{ax}",
        Tensor => r"\otimes",
        Par => r"\parr",
        One => r"\mathbf{1}",
        Bot => r"\bot",
        With => r"\with",
        PlusLeft => r"\oplus_1",
        PlusRight => r"\oplus_2",
        Top => r"\top",
        Promotion => r"\oc",
        Dereliction => r"\wn\mathrm{d}",
        Contraction => r"\wn\mathrm{c}",
        Weakening => r"\wn\mathrm{w}",
        Mix => r"\mathrm{mix}",
        AffineWeakening => r"\mathrm{wk}",
        ImpLeft => r"\multimap\mathrm{L}",
        ImpRight => r"\multimap\mathrm{R}",
        TensorLeft => r"\otimes\mathrm{L}",
        TensorRight => r"\otimes\mathrm{R}",
        WithLeft1 => r"\with\mathrm{L}_1",
        WithLeft2 => r"\with\mathrm{L}_2",
        WithRight => r"\with\mathrm{R}",
        PlusLeftRule => r"\oplus\mathrm{L}",
        PlusRight1 => r"\oplus\mathrm{R}_1",
        PlusRight2 => r"\oplus\mathrm{R}_2",
        OneLeft => r"\mathbf{1}\mathrm{L}",
        OneRight => r"\mathbf{1}\mathrm{R}",
        ZeroLeft => r"0\mathrm{L}",
        TopRight => r"\top\mathrm{R}",
        BangLeft => r"\oc\mathrm{L}",
        BangRight => r"\oc\mathrm{R}",
        BangContraction => r"\oc\mathrm{c}",
        BangWeakening => r"\oc\mathrm{w}",
        Open => "",
    }
}

/// Returns `body` as a standalone document, with `ebproof` loaded if
/// `tree` is set.
fn document(body: &str, tree: bool) -> String {
    let ebproof = if tree { "\\usepackage{ebproof}\n" } else { "" };
    format!("{PREAMBLE}{ebproof}\\begin{{document}}\n{body}\n\\end{{document}}")
}

/// Returns a sequent one-sided, `$\vdash A^\bot, A$`, as a fragment or a
/// standalone document.
pub fn sequent(sequent: &Sequent, form: Form) -> String {
    let mut out = String::from("$");
    NOTATION.one_sided(&mut out, sequent);
    out.push('$');
    match form {
        Form::Fragment => out,
        Form::Standalone => document(&out, false),
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
        Form::Standalone => document(&out, false),
    }
}

/// Returns a derivation as an ebproof `prooftree` environment, two-sided
/// if the derivation is, as a fragment or a standalone document.
pub fn derivation(derivation: &Derivation, form: Form) -> String {
    let (forest, reading) = (derivation.forest(), derivation.reading());
    let mut out = String::from("\\begin{prooftree}\n");
    let conclusion = |out: &mut String, id| {
        out.push('{');
        let sequent = &derivation.inference(id).sequent;
        NOTATION.sequent(out, forest, reading, sequent, true);
        out.push_str("}\n");
    };
    walk(derivation, |step| {
        let Step::Exit(id, _) = step else {
            return;
        };
        let inference = derivation.inference(id);
        if inference.rule == Rule::Open {
            out.push_str("\\hypo{\\vdots}\n\\infer[no rule]1");
        } else {
            out.push_str("\\infer");
            out.push_str(&inference.premises.len().to_string());
            out.push_str("[$");
            out.push_str(label(inference.rule));
            out.push_str("$]");
        }
        conclusion(&mut out, id);
    });
    out.push_str("\\end{prooftree}");
    match form {
        Form::Fragment => out,
        Form::Standalone => document(&out, true),
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

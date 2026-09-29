// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Typst: formulas in math mode, and derivations as proof trees of the
//! `curryst` package, whose `rule(name: …, premises…, conclusion)` nests
//! the premises inside their conclusion.
//!
//! The connectives are written as their Unicode characters (`⊗ ⅋ ⊕ ⊸ ⊢
//! ⊥ ⊤`), which Typst's math reads with their Unicode math class and which
//! no renaming of Typst's symbol names can break; `&` is escaped and set
//! as a binary operator, `?` set as an ordinary symbol so that no
//! punctuation space follows it, and `1` bold. A fragment is a sequent in
//! `$…$` or a `#prooftree(…)` call, for a document that imports `rule`
//! and `prooftree` from curryst [`CURRYST`]; a standalone document imports
//! them itself and sizes the page to its content. curryst cannot align
//! turnstiles, so every sequent is centred. An open goal of a proof in
//! progress is its sequent under vertical dots, with no inference line.
//! curryst nests its layout at every level of the tree, so Typst refuses a
//! tree more than about eleven inferences high ("maximum show rule depth
//! exceeded"); the LaTeX export has no such limit.
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
//! use linlog::{Mode, Options, Sequent, Verdict, prove};
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! assert_eq!(typst::sequent(&sequent, Form::Fragment), "$⊢ A^⊥, A ⊗ B^⊥, B$");
//!
//! let outcome = prove(&sequent, Mode::INTUITIONISTIC, &Options::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("provable");
//! };
//! assert_eq!(
//!     typst::derivation(&proof.two_sided_derivation()?, Form::Fragment),
//!     r#"#prooftree(
//!   rule(
//!     name: $⊸ upright(L)$,
//!     rule(name: $"ax"$, $A ⊢ A$),
//!     rule(name: $"ax"$, $B ⊢ B$),
//!     $A, A ⊸ B ⊢ B$,
//!   ),
//! )"#
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::Form;
use super::notation::{Notation, Step, walk};
use crate::occurrences::Reading;
use crate::proofs::{Derivation, Rule};
use crate::sequents::Sequent;

/// The version of curryst a standalone document imports, the one the
/// output is written for. The flake's `export` check compiles the output
/// with this version from nixpkgs, so the two change together.
pub const CURRYST: &str = "0.6.0";

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

/// Returns the name of a rule in math mode; an open goal has none.
const fn label(rule: Rule) -> &'static str {
    use Rule::*;
    match rule {
        Ax => r#""ax""#,
        Tensor => "⊗",
        Par => "⅋",
        One => "bold(1)",
        Bot => "⊥",
        With => r"\&",
        PlusLeft => "⊕_1",
        PlusRight => "⊕_2",
        Top => "⊤",
        Promotion => "!",
        Dereliction => r#"class("normal", ?) upright(d)"#,
        Contraction => r#"class("normal", ?) upright(c)"#,
        Weakening => r#"class("normal", ?) upright(w)"#,
        Mix => r#""mix""#,
        AffineWeakening => r#""wk""#,
        ImpLeft => "⊸ upright(L)",
        ImpRight => "⊸ upright(R)",
        TensorLeft => "⊗ upright(L)",
        TensorRight => "⊗ upright(R)",
        WithLeft1 => r"\& upright(L)_1",
        WithLeft2 => r"\& upright(L)_2",
        WithRight => r"\& upright(R)",
        PlusLeftRule => "⊕ upright(L)",
        PlusRight1 => "⊕ upright(R)_1",
        PlusRight2 => "⊕ upright(R)_2",
        OneLeft => "bold(1) upright(L)",
        OneRight => "bold(1) upright(R)",
        ZeroLeft => "0 upright(L)",
        TopRight => "⊤ upright(R)",
        BangLeft => "! upright(L)",
        BangRight => "! upright(R)",
        BangContraction => "! upright(c)",
        BangWeakening => "! upright(w)",
        Open => "",
    }
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

/// Returns a derivation as a curryst `#prooftree(…)` call, two-sided if
/// the derivation is, as a fragment or a standalone document. A rule with
/// premises spans several lines, indented by its depth; a leaf is one
/// line.
pub fn derivation(derivation: &Derivation, form: Form) -> String {
    let (forest, reading) = (derivation.forest(), derivation.reading());
    let mut out = String::from("#prooftree(\n");
    let indent = |out: &mut String, depth: usize| {
        for _ in 0..depth {
            out.push_str("  ");
        }
    };
    let conclusion = |out: &mut String, id| {
        out.push('$');
        let sequent = &derivation.inference(id).sequent;
        NOTATION.sequent(out, forest, reading, sequent, false);
        out.push('$');
    };
    walk(derivation, |step| match step {
        Step::Enter(id, depth) => {
            let inference = derivation.inference(id);
            indent(&mut out, depth + 1);
            if inference.rule == Rule::Open {
                out.push_str("grid(align: center, row-gutter: 0.4em, $dots.v$, ");
                conclusion(&mut out, id);
                out.push_str("),\n");
                return;
            }
            out.push_str("rule(");
            if inference.premises.is_empty() {
                out.push_str("name: $");
                out.push_str(label(inference.rule));
                out.push_str("$, ");
                conclusion(&mut out, id);
                out.push_str("),\n");
                return;
            }
            out.push('\n');
            indent(&mut out, depth + 2);
            out.push_str("name: $");
            out.push_str(label(inference.rule));
            out.push_str("$,\n");
        }
        Step::Exit(id, depth) => {
            if derivation.inference(id).premises.is_empty() {
                return;
            }
            indent(&mut out, depth + 2);
            conclusion(&mut out, id);
            out.push_str(",\n");
            indent(&mut out, depth + 1);
            out.push_str("),\n");
        }
    });
    out.push(')');
    match form {
        Form::Fragment => out,
        Form::Standalone => {
            format!("#import \"@preview/curryst:{CURRYST}\": prooftree, rule\n{PAGE}\n{out}")
        }
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

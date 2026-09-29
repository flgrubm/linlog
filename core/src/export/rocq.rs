// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Rocq: a derivation as a proof script that the NanoYalla kernel checks,
//! version [`NANOYALLA`], the kernel of Click & coLLecT. The script is a
//! `Lemma` whose statement is the sequent as a list of the kernel's
//! formulas, `ll [dual A; tens A (dual B); B]`, over the atoms as
//! variables of type `formula`, and whose proof applies one derived rule
//! of the kernel's `macroll` module per inference, conclusion first:
//! `tens_r_ext`, `parr_r_ext`, `oc_r_ext`, `co_r_ext` and so on, the
//! axiom as `ax_expansion`. `Qed` closes it, so Rocq accepts the file only
//! if the kernel accepts the proof.
//!
//! The kernel's sequents are lists and its rules act on a formula at a
//! given position, so the script keeps the list of every goal: the lemma
//! states the conclusion in occurrence order, each rule takes the prefix
//! before its principal formula, and a `⊗` alone, whose left premise must
//! hold what precedes the `⊗` and whose right premise what follows it,
//! gets an exchange `ex_perm_r` before it when the goal is not already in
//! that order.
//!
//! The certificate is one of classical linear logic: a two-sided
//! derivation is exported as the one-sided derivation behind it
//! ([`Rule::classical`]), which proves the same sequent read one-sided.
//! Mix, the weakening of affine mode and an open goal have no rule in the
//! kernel, and a derivation with one is refused ([`Unsupported`]). A
//! fragment is the lemma alone, for a file that imports `macroll`; a
//! standalone file starts with [`Options::prelude`].
//!
//! An atom's name becomes a Rocq identifier: letters, digits, `_` and `'`
//! stay, any other character becomes its code point in hexadecimal between
//! underscores, a leading digit or `'` gets an underscore before it, and a
//! name that would clash with a keyword, a name of the kernel, the lemma's
//! name or another atom gets `'` appended until it is free.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::export::rocq::{self, Options};
//! use linlog::export::Form;
//! use linlog::{Mode, Options as Search, Sequent, Verdict, prove};
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! let outcome = prove(&sequent, Mode::CLASSICAL, &Search::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("provable");
//! };
//! assert_eq!(
//!     rocq::derivation(&proof.derivation()?, Form::Fragment, &Options::default())?,
//!     "Lemma certificate (A B : formula) : ll [dual A; tens A (dual B); B].
//! Proof.
//! apply (tens_r_ext [dual A]); cbn_sequent.
//! {
//!   ax_expansion.
//! }
//! {
//!   ax_expansion.
//! }
//! Qed."
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::Form;
use super::notation::{Step, walk};
use crate::occurrences::{Forest, OccId};
use crate::proofs::{Derivation, InfId, Rule};
use crate::sequents::{Sequent, Term, TermId};
use std::fmt::Write;
use thiserror::Error;

/// The version of NanoYalla the scripts are written for: the `nanoyalla`
/// directory of Click & coLLecT, whose `macroll` module defines the
/// derived rules the scripts apply. It builds on Rocq 9 with the standard
/// library, without Yalla.
pub const NANOYALLA: &str = "1.1.3";

/// What a user may vary in a certificate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Options {
    /// The name of the lemma.
    pub lemma: String,
    /// The lines a standalone file starts with, before the lemma: what
    /// brings the kernel's `ll`, its formulas and its derived rules into
    /// scope.
    pub prelude: String,
}

impl Default for Options {
    /// Returns the lemma `certificate` and the import of NanoYalla's
    /// `macroll`.
    fn default() -> Self {
        Self {
            lemma: "certificate".to_owned(),
            prelude: "From NanoYalla Require Import macroll.".to_owned(),
        }
    }
}

/// Why a derivation has no certificate: it uses a rule the kernel lacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Error)]
pub enum Unsupported {
    /// The derivation has an open goal: it is a proof in progress.
    #[error("the proof is not finished: an open goal has no certificate")]
    Open,
    /// The derivation uses Mix.
    #[error("NanoYalla has no Mix rule, so a proof with Mix has no certificate")]
    Mix,
    /// The derivation weakens a formula that is not a `?` formula.
    #[error(
        "NanoYalla weakens ? formulas only, so a proof with the weakening of affine mode has no certificate"
    )]
    AffineWeakening,
}

/// Rocq's keywords and the names of the kernel a script uses, which no
/// atom may take.
const RESERVED: &[&str] = &[
    "_",
    "Axiom",
    "CoFixpoint",
    "Definition",
    "Fixpoint",
    "Hypothesis",
    "IF",
    "Lemma",
    "Parameter",
    "Proof",
    "Prop",
    "Qed",
    "SProp",
    "Set",
    "Theorem",
    "Type",
    "Variable",
    "as",
    "at",
    "by",
    "cofix",
    "discriminated",
    "else",
    "end",
    "exists",
    "exists2",
    "fix",
    "for",
    "forall",
    "fun",
    "if",
    "in",
    "lazymatch",
    "let",
    "match",
    "multimatch",
    "return",
    "then",
    "using",
    "where",
    "with",
    "Atom",
    "app",
    "aplus",
    "apply",
    "awith",
    "ax_expansion",
    "ax_r_ext",
    "bidual",
    "bot",
    "bot_r_ext",
    "cbn_sequent",
    "co_r_ext",
    "cons",
    "covar",
    "de_r_ext",
    "dual",
    "ex_perm_r",
    "ex_transpL",
    "formula",
    "list",
    "ll",
    "map",
    "nat",
    "nil",
    "O",
    "oc",
    "oc_r_ext",
    "one",
    "one_r_ext",
    "parr",
    "parr_r_ext",
    "permL_of_perm",
    "plus_r1_ext",
    "plus_r2_ext",
    "S",
    "tens",
    "tens_r_ext",
    "top",
    "top_r_ext",
    "var",
    "with_r_ext",
    "wk_r_ext",
    "wn",
    "zero",
];

/// Returns an atom's name as a Rocq identifier, before the check for
/// clashes.
fn identifier(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    if name.starts_with(|c: char| c.is_ascii_digit() || c == '\'') {
        out.push('_');
    }
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '\'' {
            out.push(c);
        } else {
            write!(out, "_{:x}_", c as u32).unwrap();
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

/// Returns the identifiers of atoms with these names, in their order,
/// each free of clashes with the keywords, the kernel, `lemma` and the
/// others.
fn identifiers(atoms: &[String], lemma: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::with_capacity(atoms.len());
    for name in atoms {
        let mut id = identifier(name);
        while RESERVED.contains(&id.as_str()) || id == lemma || names.contains(&id) {
            id.push('\'');
        }
        names.push(id);
    }
    names
}

/// Writes the formula at `id` in the kernel's constructors, in brackets if
/// `argument` is set and it is an application.
fn term(out: &mut String, sequent: &Sequent, names: &[String], id: TermId, argument: bool) {
    use Term::*;
    let t = sequent.term(id);
    let atom = |out: &mut String, a| out.push_str(&names[a as usize]);
    let application = argument && !matches!(t, Var(_) | One | Bot | Top | Zero);
    if application {
        out.push('(');
    }
    match t {
        Var(a) => atom(out, a.get()),
        DualVar(a) => {
            out.push_str("dual ");
            atom(out, a.get());
        }
        One => out.push_str("one"),
        Bot => out.push_str("bot"),
        Top => out.push_str("top"),
        Zero => out.push_str("zero"),
        Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => {
            out.push_str(match t {
                Tensor(..) => "tens ",
                Par(..) => "parr ",
                With(..) => "awith ",
                _ => "aplus ",
            });
            term(out, sequent, names, k, true);
            out.push(' ');
            term(out, sequent, names, l, true);
        }
        Bang(k) | Quest(k) => {
            out.push_str(if matches!(t, Bang(_)) { "oc " } else { "wn " });
            term(out, sequent, names, k, true);
        }
    }
    if application {
        out.push(')');
    }
}

/// Writes occurrences as a list of the kernel's formulas, `[A; dual B]`.
fn list(out: &mut String, forest: &Forest, names: &[String], occurrences: &[OccId]) {
    out.push('[');
    for (i, &o) in occurrences.iter().enumerate() {
        if i > 0 {
            out.push_str("; ");
        }
        term(out, forest.sequent(), names, forest.term(o), false);
    }
    out.push(']');
}

/// The script in progress.
struct Script<'a> {
    /// The derivation being written.
    derivation: &'a Derivation<'a>,
    /// The forest its occurrences index.
    forest: &'a Forest,
    /// The identifier of every atom.
    names: Vec<String>,
    /// The text so far.
    out: String,
    /// The list of formulas Rocq shows as the goal of each inference not
    /// yet written, set when its conclusion is written.
    goals: Vec<Vec<OccId>>,
    /// Whether an inference is a premise of a binary rule, whose script
    /// goes into braces of its own.
    braced: Vec<bool>,
    /// How many braces are open.
    depth: usize,
}

impl Script<'_> {
    /// Writes a tactic line at the current indentation.
    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// Writes a rule's tactic, `apply (<rule> <arguments>); cbn_sequent.`,
    /// without the `cbn_sequent` when the rule leaves no goal.
    fn apply(&mut self, rule: &str, arguments: &[String], leaf: bool) {
        let mut text = format!("apply ({rule}");
        for a in arguments {
            text.push(' ');
            text.push_str(a);
        }
        text.push(')');
        text.push_str(if leaf { "." } else { "; cbn_sequent." });
        self.line(&text);
    }

    /// Returns occurrences as a list of the kernel's formulas.
    fn list(&self, occurrences: &[OccId]) -> String {
        let mut out = String::new();
        list(&mut out, self.forest, &self.names, occurrences);
        out
    }

    /// Writes the tactic of the inference `id`, whose goal is `goal`, and
    /// sets the goals of its premises.
    fn inference(&mut self, id: InfId, goal: Vec<OccId>) {
        let (derivation, forest) = (self.derivation, self.forest);
        let inference = derivation.inference(id);
        let rule = inference.rule.classical();
        if rule == Rule::Ax {
            self.line("ax_expansion.");
            return;
        }
        let o = inference.sequent[inference.principal.unwrap()];
        let at = goal.iter().position(|&x| x == o).unwrap();
        let (before, after) = (&goal[..at], &goal[at + 1..]);
        let prefix = self.list(before);
        let (left, right) = (forest.left(o), forest.right(o));
        // The goal of the premise when the rule replaces the principal
        // formula by `with`.
        let replaced = |with: &[OccId]| {
            let mut up = Vec::with_capacity(goal.len() + 1);
            up.extend_from_slice(before);
            up.extend_from_slice(with);
            up.extend_from_slice(after);
            up
        };
        let premise = |script: &mut Self, up: Vec<OccId>| {
            script.goals[inference.premises[0].index()] = up;
        };
        match rule {
            Rule::One => self.line("apply one_r_ext."),
            Rule::Top => self.apply("top_r_ext", &[prefix], true),
            Rule::Bot => {
                self.apply("bot_r_ext", &[prefix], false);
                premise(self, replaced(&[]));
            }
            Rule::Par => {
                self.apply("parr_r_ext", &[prefix], false);
                premise(self, replaced(&[left.unwrap(), right.unwrap()]));
            }
            Rule::PlusLeft => {
                self.apply("plus_r1_ext", &[prefix], false);
                premise(self, replaced(&[left.unwrap()]));
            }
            Rule::PlusRight => {
                self.apply("plus_r2_ext", &[prefix], false);
                premise(self, replaced(&[right.unwrap()]));
            }
            Rule::Dereliction => {
                self.apply("de_r_ext", &[prefix], false);
                premise(self, replaced(&[left.unwrap()]));
            }
            Rule::Weakening => {
                self.apply("wk_r_ext", &[prefix], false);
                premise(self, replaced(&[]));
            }
            Rule::Contraction => {
                self.apply("co_r_ext", &[prefix], false);
                premise(self, replaced(&[o, o]));
            }
            Rule::Promotion => {
                // The context is `?` formulas, which the rule takes
                // without their `?`.
                let under: Vec<OccId> = before
                    .iter()
                    .chain(after)
                    .map(|&q| forest.left(q).unwrap())
                    .collect();
                let (l1, l2) = under.split_at(before.len());
                let mut a = String::from("(");
                term(
                    &mut a,
                    forest.sequent(),
                    &self.names,
                    forest.term(left.unwrap()),
                    false,
                );
                a.push(')');
                let arguments = [self.list(l1), a, self.list(l2)];
                self.apply("oc_r_ext", &arguments, false);
                premise(self, replaced(&[left.unwrap()]));
            }
            Rule::With => {
                self.apply("with_r_ext", &[prefix], false);
                let (l, r) = (inference.premises[0], inference.premises[1]);
                self.goals[l.index()] = replaced(&[left.unwrap()]);
                self.goals[r.index()] = replaced(&[right.unwrap()]);
                self.braced[l.index()] = true;
                self.braced[r.index()] = true;
            }
            Rule::Tensor => self.tensor(id, goal),
            Rule::Ax
            | Rule::Mix
            | Rule::AffineWeakening
            | Rule::Open
            | Rule::ImpLeft
            | Rule::ImpRight
            | Rule::TensorLeft
            | Rule::TensorRight
            | Rule::WithLeft1
            | Rule::WithLeft2
            | Rule::WithRight
            | Rule::PlusLeftRule
            | Rule::PlusRight1
            | Rule::PlusRight2
            | Rule::OneLeft
            | Rule::OneRight
            | Rule::ZeroLeft
            | Rule::TopRight
            | Rule::BangLeft
            | Rule::BangRight
            | Rule::BangContraction
            | Rule::BangWeakening => unreachable!("refused or classical"),
        }
    }

    /// Writes the tactics of a `⊗`: the exchange that puts the left
    /// premise's context before the `⊗` and the right premise's after it,
    /// when the goal is not in that order already, then the rule.
    fn tensor(&mut self, id: InfId, goal: Vec<OccId>) {
        let (derivation, forest) = (self.derivation, self.forest);
        let inference = derivation.inference(id);
        let o = inference.sequent[inference.principal.unwrap()];
        let (a, b) = (forest.left(o).unwrap(), forest.right(o).unwrap());
        let (l, r) = (inference.premises[0], inference.premises[1]);
        // The left context, consumed as the goal's formulas are assigned.
        let mut left: Vec<OccId> = derivation.inference(l).sequent.clone();
        left.remove(left.iter().position(|&x| x == a).unwrap());
        let (mut before, mut after) = (Vec::new(), Vec::new());
        let mut principal = false;
        for &x in &goal {
            if x == o && !principal {
                principal = true;
            } else if let Some(i) = left.iter().position(|&y| y == x) {
                left.remove(i);
                before.push(x);
            } else {
                after.push(x);
            }
        }
        let mut target = before.clone();
        target.push(o);
        target.extend_from_slice(&after);
        if target != goal {
            // `ex_perm_r p l` proves `l` permuted so that position `i`
            // holds `l[p[i]]`, from `l`.
            let mut used = vec![false; target.len()];
            let p: Vec<String> = goal
                .iter()
                .map(|&x| {
                    let j = (0..target.len())
                        .find(|&j| !used[j] && target[j] == x)
                        .unwrap();
                    used[j] = true;
                    j.to_string()
                })
                .collect();
            let arguments = [format!("[{}]", p.join("; ")), self.list(&target)];
            self.apply("ex_perm_r", &arguments, true);
        }
        let prefix = self.list(&before);
        self.apply("tens_r_ext", &[prefix], false);
        before.push(a);
        after.insert(0, b);
        self.goals[l.index()] = before;
        self.goals[r.index()] = after;
        self.braced[l.index()] = true;
        self.braced[r.index()] = true;
    }
}

/// Returns a derivation as a Rocq lemma with its proof script for
/// NanoYalla, as a fragment or a standalone file, or why it has no
/// certificate.
pub fn derivation(
    derivation: &Derivation,
    form: Form,
    options: &Options,
) -> Result<String, Unsupported> {
    for inference in derivation.inferences() {
        match inference.rule {
            Rule::Open => return Err(Unsupported::Open),
            Rule::Mix => return Err(Unsupported::Mix),
            Rule::AffineWeakening => return Err(Unsupported::AffineWeakening),
            _ => {}
        }
    }
    let forest = derivation.forest();
    let names = identifiers(forest.sequent().atom_names(), &options.lemma);
    let count = derivation.inferences().len();
    let mut script = Script {
        derivation,
        forest,
        names,
        out: String::new(),
        goals: vec![Vec::new(); count],
        braced: vec![false; count],
        depth: 0,
    };
    let root = derivation.root();
    script.goals[root.index()] = derivation.inference(root).sequent.clone();

    write!(script.out, "Lemma {}", options.lemma).unwrap();
    if !script.names.is_empty() {
        write!(script.out, " ({} : formula)", script.names.join(" ")).unwrap();
    }
    script.out.push_str(" : ll ");
    let conclusion = script.list(&script.goals[root.index()]);
    script.out.push_str(&conclusion);
    script.out.push_str(".\nProof.\n");
    walk(derivation, |step| match step {
        Step::Enter(id, _) => {
            if script.braced[id.index()] {
                script.line("{");
                script.depth += 1;
            }
            let goal = std::mem::take(&mut script.goals[id.index()]);
            script.inference(id, goal);
        }
        Step::Exit(id, _) => {
            if script.braced[id.index()] {
                script.depth -= 1;
                script.line("}");
            }
        }
    });
    script.out.push_str("Qed.");
    match form {
        Form::Fragment => Ok(script.out),
        Form::Standalone => Ok(format!("{}\n\n{}", options.prelude, script.out)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Letters, digits, `_` and `'` stay; other characters become their
    /// code points; a clash with a keyword, the kernel, the lemma or
    /// another atom gets `'` appended.
    #[test]
    fn names_become_identifiers() {
        for (name, expected) in [
            ("A", "A"),
            ("x_1", "x_1"),
            ("p'", "p'"),
            ("1a", "_1a"),
            ("α", "_3b1_"),
            ("a b", "a_20_b"),
            ("", "_"),
        ] {
            assert_eq!(identifier(name), expected, "{name:?}");
        }
        let atoms: Vec<String> = ["tens", "with", "certificate", "α", "_3b1_", "_"]
            .map(String::from)
            .into();
        assert_eq!(
            identifiers(&atoms, "certificate"),
            ["tens'", "with'", "certificate'", "_3b1_", "_3b1_'", "_'"]
        );
    }
}

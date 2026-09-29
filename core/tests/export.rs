// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The LaTeX and Typst exports through the public API. Derivations are
//! pinned as standalone documents in `tests/snapshots/`, which the flake's
//! `export` check compiles; run with `BLESS=1` to rewrite them after an
//! intended change, and review the diff.

#![cfg(all(
    feature = "parse",
    feature = "interactive",
    feature = "latex",
    feature = "typst"
))]

use linlog::export::{Form, latex, typst};
use linlog::{Derivation, Forest, InfId, Interactive, Mode, Options, Reading, Rule, Sequent};
use linlog::{Verdict, prove};
use std::path::Path;

/// Compares `actual` with the snapshot `name`, or writes it there when the
/// environment sets `BLESS`.
fn snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&path, format!("{actual}\n")).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; run with BLESS=1", path.display()));
    assert_eq!(actual, expected.trim_end_matches('\n'), "{name}");
}

/// Pins a derivation as the snapshot `name.tex`.
fn pin(name: &str, derivation: &Derivation) {
    snapshot(
        &format!("{name}.tex"),
        &latex::derivation(derivation, Form::Standalone),
    );
}

/// Proves `input` in `mode` and pins its derivation, two-sided in
/// intuitionistic mode.
fn pin_proof(name: &str, input: &str, mode: Mode) {
    let sequent: Sequent = input.parse().unwrap();
    let outcome = prove(&sequent, mode, &Options::default()).unwrap();
    let Verdict::Proved(proof) = &outcome.verdict else {
        panic!("{input} is provable");
    };
    let derivation = if mode.intuitionistic {
        proof.two_sided_derivation()
    } else {
        proof.derivation()
    };
    pin(name, &derivation.unwrap());
}

/// Derivations of each fragment, one-sided and two-sided, as ebproof
/// trees.
#[test]
fn derivations() {
    pin_proof("mll", "A * B |- B * A", Mode::CLASSICAL);
    pin_proof("mall", "A + B, 1 |- B + A", Mode::CLASSICAL);
    pin_proof("mell", "!A, !B |- !(A * A)", Mode::CLASSICAL);
    pin_proof("ill", "1, A & B, B -o C |- C", Mode::INTUITIONISTIC);
}

/// An open goal of a proof in progress is its sequent under vertical dots,
/// with no inference line.
#[test]
fn open_goal() {
    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state.apply(InfId::new(0), 1, Rule::ImpLeft, &[0]).unwrap();
    state.apply(goals[0], 0, Rule::Ax, &[]).unwrap();
    pin("open", &state.derivation());
}

/// Sequents print as math, one-sided or two-sided, with longer names in
/// italic and escaped.
#[test]
fn sequents() {
    let sequent: Sequent = "x_1 * foo, !A |- ?B & 1, B".parse().unwrap();
    assert_eq!(
        latex::sequent(&sequent, Form::Fragment),
        r"$\vdash \mathit{x\_1}^\bot \parr \mathit{foo}^\bot, \wn A^\bot, B, \wn B \with \mathbf{1}$"
    );
    assert_eq!(
        typst::sequent(&sequent, Form::Fragment),
        r#"$⊢ italic("x_1")^⊥ ⅋ italic("foo")^⊥, class("normal", ?)A^⊥, B, class("normal", ?)B class("binary", \&) bold(1)$"#
    );

    let sequent: Sequent = "!A, A -o (B + top) |- B & 0".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let reading = Reading::new(&forest).unwrap();
    assert_eq!(
        latex::two_sided(&reading, Form::Fragment),
        r"$\oc A, A \multimap (B \oplus \top) \vdash B \with 0$"
    );
    assert_eq!(
        typst::two_sided(&reading, Form::Fragment),
        r#"$!A, A ⊸ (B ⊕ ⊤) ⊢ B class("binary", \&) 0$"#
    );
}

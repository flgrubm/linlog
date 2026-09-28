// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

#![cfg(feature = "parse")]

use linlog::logics::LL;
use linlog::sequents::Sequent;

/// Parses `input` as a sequent of classical linear logic and prints it back.
fn pretty(input: &str) -> String {
    match input.parse::<Sequent<usize, LL>>() {
        Ok(s) => s.to_string(),
        Err(e) => panic!("{input:?} does not parse: {e}"),
    }
}

/// Returns whether `input` is rejected by the parser.
fn rejected(input: &str) -> bool {
    input.parse::<Sequent<usize, LL>>().is_err()
}

/// Every constant parses in each of its spellings, dualised on the left.
#[test]
fn constants() {
    for (input, printed) in [
        ("|- 0", "⊢ 0"),
        ("|- 1", "⊢ 1"),
        ("|- bot", "⊢ ⊥"),
        ("|- ⊥", "⊢ ⊥"),
        ("|- top", "⊢ ⊤"),
        ("|- ⊤", "⊢ ⊤"),
        ("0 |-", "⊢ ⊤"),
        ("1 |-", "⊢ ⊥"),
        ("⊥ |-", "⊢ 1"),
        ("⊤ |-", "⊢ 0"),
        ("|- !(A * 1) + ?⊤", "⊢ !(A ⊗ 1) ⊕ ?⊤"),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// A sequent may have no formulas on either side, or on both.
#[test]
fn empty_sides() {
    for (input, printed) in [("|-", "⊢"), ("⊢", "⊢"), ("A |-", "⊢ ~A"), ("|- A", "⊢ A")] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// Operators bind from tightest to loosest: `^`, then `~ ! ?`, tensor, par,
/// with, plus and lollipop.
#[test]
fn precedence() {
    for (input, printed) in [
        ("|- !A^ * ?~B", "⊢ !~A ⊗ ?~B"),
        (
            "|- A -o B + C & D par E * F",
            "⊢ ~A ⅋ (B ⊕ (C & (D ⅋ (E ⊗ F))))",
        ),
        (
            "|- A * B par C & D + E -o F",
            "⊢ ((((~A ⅋ ~B) ⊗ ~C) ⊕ ~D) & ~E) ⅋ F",
        ),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// Lollipop associates to the right, every other binary operator to the left.
#[test]
fn associativity() {
    for (input, printed) in [
        ("|- A -o B -o C", "⊢ ~A ⅋ (~B ⅋ C)"),
        ("|- A * B * C", "⊢ (A ⊗ B) ⊗ C"),
        ("|- A par B par C", "⊢ (A ⅋ B) ⅋ C"),
        ("|- A & B & C", "⊢ (A & B) & C"),
        ("|- A + B + C", "⊢ (A ⊕ B) ⊕ C"),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// Every spelling of a binary operator binds and associates like the others,
/// also when spellings are mixed in one formula.
#[test]
fn operator_spellings_are_interchangeable() {
    let operators: [&[&str]; 4] = [&["*", "⊗"], &["par", "|", "⅋"], &["+", "⊕"], &["-o", "⊸"]];
    for spellings in operators {
        let expected = pretty(&format!("|- A {0} B {0} C", spellings[0]));
        for first in spellings {
            for second in spellings {
                let input = format!("|- A {first} B {second} C");
                assert_eq!(pretty(&input), expected, "{input:?}");
            }
        }
    }
}

/// A word that merely starts like a constant is a variable or an error.
#[test]
fn constant_prefixes() {
    assert_eq!(pretty("|- bottom, topx"), "⊢ bottom, topx");
    assert!(rejected("|- 0a"));
    assert!(rejected("|- 10"));
    assert!(rejected("|- ⊥A"));
}

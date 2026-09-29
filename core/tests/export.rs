// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The LaTeX and Typst exports through the public API.

#![cfg(all(feature = "parse", feature = "latex", feature = "typst"))]

use linlog::export::{Form, latex, typst};
use linlog::{Forest, Reading, Sequent};

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

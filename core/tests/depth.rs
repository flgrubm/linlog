// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Formulas nested deeper than a stack lets a recursion follow them: every
//! walk over a formula that the public API offers, on a thread whose stack
//! holds a few bytes per level. The sequents come from JSON, whose form is
//! flat.

#![cfg(feature = "serialize")]

use linlog::{Forest, Fragment, Mode, Node, NodeId, Proof, Reading, Sequent};

/// How deep the formulas are nested.
const DEPTH: usize = 100_000;

/// The stack of the thread the walks run on.
const STACK: usize = 256 * 1024;

/// Returns the JSON of the sequent over the one atom `a` whose arena starts
/// with `⊤`, `~a` and `a` and goes on with `terms`, and whose root formulas
/// are the `⊤` and the terms at `roots`.
fn json(terms: &[String], roots: &[usize]) -> String {
    let roots: Vec<String> = roots.iter().map(usize::to_string).collect();
    format!(
        r#"{{"terms":["⊤",{{"D":0}},{{"V":0}},{}],"ids":[0,{}],"var_dict":["a"]}}"#,
        terms.join(","),
        roots.join(",")
    )
}

/// `⊢ ⊤, ~a ⅋ (~a ⅋ (… ⅋ a))`, which reads `0 ⊢ a ⊸ (a ⊸ (… ⊸ a))`.
fn right_nested() -> String {
    let terms: Vec<String> = (0..DEPTH)
        .map(|i| format!(r#"{{"⅋":[1,{}]}}"#, i + 2))
        .collect();
    json(&terms, &[DEPTH + 2])
}

/// `⊢ ⊤, (((a ⊗ ~a) ⅋ a) ⊗ ~a) ⅋ …`, which reads
/// `0 ⊢ (((a ⊸ a) ⊸ a) ⊸ a) ⊸ …`.
fn left_nested() -> String {
    let terms: Vec<String> = (0..DEPTH)
        .map(|i| match i % 2 {
            0 => format!(r#"{{"⊗":[{},1]}}"#, i + 2),
            _ => format!(r#"{{"⅋":[{},2]}}"#, i + 2),
        })
        .collect();
    json(&terms, &[DEPTH + 2])
}

/// `⊢ ⊤, ?…?~a, !…!a`, which reads `0, !…!a ⊢ !…!a`.
fn tower() -> String {
    let below = |i: usize, leaf: usize, tower: usize| if i == 0 { leaf } else { tower + i - 1 };
    let mut terms: Vec<String> = (0..DEPTH)
        .map(|i| format!(r#"{{"?":{}}}"#, below(i, 1, 3)))
        .collect();
    terms.extend((0..DEPTH).map(|i| format!(r#"{{"!":{}}}"#, below(i, 2, DEPTH + 3))));
    json(&terms, &[DEPTH + 2, 2 * DEPTH + 2])
}

/// Runs every walk over the formulas of the sequent that `json` holds,
/// which lies in `fragment` and whose last formula is the goal.
fn walk(json: &str, fragment: Fragment) {
    let sequent: Sequent = serde_json::from_str(json).unwrap();
    sequent.verify_integrity().unwrap();
    assert_eq!(serde_json::to_string(&sequent).unwrap(), json);
    assert_eq!(sequent.fragment(), fragment);
    let mut optimized = sequent.clone();
    optimized.optimize().unwrap();
    assert_eq!(optimized.terms().len(), sequent.terms().len());

    // One-sided: the sequent is its formulas.
    let forest = Forest::new(&sequent).unwrap();
    let deepest = forest.ids().map(|o| forest.depth(o)).max();
    assert_eq!(deepest, Some(DEPTH as u32));
    let formulas: Vec<String> = forest
        .roots()
        .iter()
        .map(|&root| forest.formula(root).to_string())
        .collect();
    let printed = sequent.to_string();
    assert!(printed.chars().count() > DEPTH);
    assert_eq!(printed, format!("⊢ {}", formulas.join(", ")));

    // Two-sided: the hypotheses, then the goal.
    let reading = Reading::new(&forest).unwrap();
    let goal = *forest.roots().last().unwrap();
    assert_eq!(reading.goal(), goal);
    let hypotheses: Vec<String> = reading
        .hypotheses()
        .map(|h| reading.formula(h).to_string())
        .collect();
    let two_sided = reading.to_string();
    assert!(two_sided.chars().count() > DEPTH);
    assert_eq!(
        two_sided,
        format!("{} ⊢ {}", hypotheses.join(", "), reading.formula(goal))
    );

    #[cfg(feature = "latex")]
    {
        use linlog::export::{Form, latex};
        assert!(latex::sequent(&sequent, Form::Fragment).len() > DEPTH);
        assert!(latex::two_sided(&reading, Form::Fragment).len() > DEPTH);
    }
    #[cfg(feature = "typst")]
    {
        use linlog::export::{Form, typst};
        assert!(typst::sequent(&sequent, Form::Fragment).len() > DEPTH);
        assert!(typst::two_sided(&reading, Form::Fragment).len() > DEPTH);
    }
    #[cfg(feature = "svg")]
    {
        use linlog::export::svg::{self, Style};
        assert!(svg::sequent(&sequent, &Style::default()).len() > DEPTH);
        assert!(svg::two_sided(&reading, &Style::default()).len() > DEPTH);
    }

    // The `⊤` proves the sequent in one inference, which concludes all of
    // it: a derivation is what reaches the printer of the certificates.
    let top = Node::Top(forest.roots()[0]);
    let proof = Proof::new(forest.clone(), vec![top], NodeId::new(0)).unwrap();
    proof.check(Mode::CLASSICAL).unwrap();
    let derivation = proof.derivation().unwrap();
    assert!(derivation.to_string().ends_with(&printed));
    #[cfg(feature = "rocq")]
    {
        use linlog::export::Form;
        use linlog::export::rocq::{self, Options};
        let certificate = rocq::derivation(&derivation, Form::Fragment, &Options::default());
        assert!(certificate.unwrap().len() > DEPTH);
    }
}

/// A sequent whose formulas are nested 100 000 deep, to the right, to the
/// left, or as towers of `?` and of `!`, is read, printed, optimized,
/// numbered, read two-sided, exported and written back on a stack of
/// 256 KiB.
#[test]
fn deep_formulas_need_no_stack() {
    let walks = std::thread::Builder::new().stack_size(STACK).spawn(|| {
        let nested = Fragment::MULTIPLICATIVES | Fragment::ADDITIVE_UNITS;
        walk(&right_nested(), nested);
        walk(&left_nested(), nested);
        walk(&tower(), Fragment::EXPONENTIALS | Fragment::ADDITIVE_UNITS);
    });
    walks.unwrap().join().unwrap();
}

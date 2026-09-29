// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON forms of a sequent and of a proof, through the public API. The
//! formats are interchange formats, so the strings pinned here must not
//! change.

#![cfg(all(feature = "parse", feature = "serialize"))]

use linlog::{Forest, Mode, Node, NodeId, OccId, Proof, Sequent, Side};

/// Parses `input` and serializes it as compact JSON.
fn json(input: &str) -> String {
    let s: Sequent = input.parse().unwrap();
    serde_json::to_string(&s).unwrap()
}

/// The arena, the root indices and the atom names appear under their fixed
/// keys, with the short tags of the terms.
#[test]
fn json_format() {
    for (input, expected) in [
        ("|-", r#"{"terms":[],"ids":[],"var_dict":[]}"#),
        (
            "A |- A",
            r#"{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]}"#,
        ),
        (
            "|- 0, 1, bot, top",
            r#"{"terms":["0","1","⊥","⊤"],"ids":[0,1,2,3],"var_dict":[]}"#,
        ),
        (
            "A * B |- A par B",
            r#"{"terms":[{"D":0},{"D":1},{"⅋":[0,1]},{"V":0},{"V":1},{"⅋":[3,4]}],"ids":[2,5],"var_dict":["A","B"]}"#,
        ),
        (
            "!(A & B) |- ?(A + B)",
            r#"{"terms":[{"D":0},{"D":1},{"⊕":[0,1]},{"?":2},{"V":0},{"V":1},{"⊕":[4,5]},{"?":6}],"ids":[3,7],"var_dict":["A","B"]}"#,
        ),
    ] {
        assert_eq!(json(input), expected, "{input:?}");
    }
}

/// A sequent survives a trip through JSON unchanged.
#[test]
fn round_trip() {
    for input in [
        "|-",
        "A |- A",
        "A * B, C par D |- A & B, C + D",
        "!A, ?B |- ~A, B^",
        "A -o B -o C, (A -o B) -o C |- (A * B) + (0 & top)",
        "A, A -o B |- B, B * ~A",
    ] {
        let s: Sequent = input.parse().unwrap();
        let back: Sequent = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back, s, "{input:?}");
        assert_eq!(back.to_string(), s.to_string(), "{input:?}");
    }
}

/// JSON whose arena breaks an invariant is rejected on deserialization.
#[test]
fn broken_arena_is_rejected() {
    for json in [
        // A subterm index that does not precede its parent.
        r#"{"terms":[{"⊗":[1,0]},"1"],"ids":[0],"var_dict":[]}"#,
        // A root outside the arena.
        r#"{"terms":["1"],"ids":[1],"var_dict":[]}"#,
        // An atom outside the dictionary.
        r#"{"terms":[{"V":0}],"ids":[0],"var_dict":[]}"#,
    ] {
        assert!(serde_json::from_str::<Sequent>(json).is_err(), "{json}");
    }
}

/// Builds a proof of the sequent `input` parses to, with the last node as
/// the root.
fn proof(input: &str, nodes: Vec<Node>) -> Proof {
    let s: Sequent = input.parse().unwrap();
    let root = NodeId::new(nodes.len() as u32 - 1);
    Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
}

/// A proof serializes as its sequent, in the sequent's own format, and its
/// nodes, one object per node with the rule's short tag.
#[test]
fn proof_json_format() {
    use Node::*;
    let (o, n) = (OccId::new, NodeId::new);
    for (input, nodes, expected) in [
        // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
        (
            "A, A -o B |- B",
            vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))],
            r#"[{"ax":[0,2]},{"ax":[3,4]},{"⊗":[1,0,1]}]"#,
        ),
        // ⊢ A & ⊤, ~A ⊕ ⊥: 0 &, 1 A, 2 ⊤, 3 ⊕, 4 ~A, 5 ⊥
        (
            "|- A & top, ~A + bot",
            vec![
                Ax(o(1), o(4)),
                Plus(o(3), Side::Left, n(0)),
                Top(o(2)),
                With(o(0), n(1), n(2)),
            ],
            r#"[{"ax":[1,4]},{"⊕₁":[3,0]},{"⊤":2},{"&":[0,1,2]}]"#,
        ),
        // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B
        (
            "!A, !(A -o B) |- !B",
            vec![
                Ax(o(1), o(4)),
                Copy(o(1), n(0)),
                Ax(o(5), o(7)),
                Tensor(o(3), n(1), n(2)),
                Copy(o(3), n(3)),
                Bang(o(6), n(4)),
                Quest(o(2), n(5)),
                Quest(o(0), n(6)),
            ],
            r#"[{"ax":[1,4]},{"copy":[1,0]},{"ax":[5,7]},{"⊗":[3,1,2]},{"copy":[3,3]},{"!":[6,4]},{"?":[2,5]},{"?":[0,6]}]"#,
        ),
        // ⊢ ~A, ~B, A, B, 1, ⊥ with weakening and Mix: 0 ~A, 1 ~B, 2 A, 3 B,
        // 4 1, 5 ⊥
        (
            "A, B |- A, B, 1, bot",
            vec![
                Ax(o(0), o(2)),
                Weaken(o(1), n(0)),
                One(o(4)),
                Bot(o(5), n(2)),
                Mix(n(1), n(3)),
                Weaken(o(3), n(4)),
            ],
            r#"[{"ax":[0,2]},{"wk":[1,0]},{"1":4},{"⊥":[5,2]},{"mix":[1,3]},{"wk":[3,4]}]"#,
        ),
        // ⊢ A ⊕ B, ~A ⅋ ~B: 0 ⊕, 1 A, 2 B, 3 ⅋, 4 ~A, 5 ~B
        (
            "|- A + B, ~A par ~B",
            vec![
                Ax(o(2), o(5)),
                Plus(o(0), Side::Right, n(0)),
                Weaken(o(4), n(1)),
                Par(o(3), n(2)),
            ],
            r#"[{"ax":[2,5]},{"⊕₂":[0,0]},{"wk":[4,1]},{"⅋":[3,2]}]"#,
        ),
    ] {
        let p = proof(input, nodes);
        let sequent = serde_json::to_string(p.sequent()).unwrap();
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            format!(r#"{{"sequent":{sequent},"proof":{expected}}}"#),
            "{input:?}"
        );
    }
}

/// A proof survives a trip through JSON with its sequent and nodes intact,
/// and still checks.
#[test]
fn proof_round_trip() {
    use Node::*;
    let (o, n) = (OccId::new, NodeId::new);
    let p = proof(
        "!A, !(A -o B) |- !B, top",
        vec![
            Ax(o(1), o(4)),
            Copy(o(1), n(0)),
            Ax(o(5), o(7)),
            Tensor(o(3), n(1), n(2)),
            Copy(o(3), n(3)),
            Bang(o(6), n(4)),
            Quest(o(2), n(5)),
            Quest(o(0), n(6)),
            Top(o(8)),
            Mix(n(7), n(8)),
        ],
    );
    let back: Proof = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(back.sequent(), p.sequent());
    assert_eq!(back.nodes(), p.nodes());
    assert_eq!(back.check(Mode::CLASSICAL.with_mix()), Ok(()));
    assert_eq!(
        back.derivation().unwrap().to_string(),
        p.derivation().unwrap().to_string()
    );
}

/// JSON whose nodes point outside the forest or the arena, or at a later
/// node, is rejected on deserialization; whether the proof is correct is
/// left to the checker.
#[test]
fn broken_proof_is_rejected() {
    let sequent = r#"{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]}"#;
    for proof in [
        // No node at all.
        r#"[]"#,
        // An occurrence outside the forest.
        r#"[{"ax":[0,2]}]"#,
        // A premise that is the node itself.
        r#"[{"⅋":[0,0]}]"#,
        // A premise after the node.
        r#"[{"ax":[0,1]},{"⅋":[0,2]}]"#,
        // An unknown rule.
        r#"[{"cut":[0,1]}]"#,
    ] {
        let json = format!(r#"{{"sequent":{sequent},"proof":{proof}}}"#);
        assert!(serde_json::from_str::<Proof>(&json).is_err(), "{json}");
    }
    // A wrong proof deserializes and fails the checker.
    let json = format!(r#"{{"sequent":{sequent},"proof":[{{"ax":[0,0]}}]}}"#);
    let p: Proof = serde_json::from_str(&json).unwrap();
    assert!(p.check(Mode::CLASSICAL).is_err());
}

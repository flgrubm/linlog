// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON forms of a sequent, of a proof, of a proof net and of a proof
//! in progress, through the public API. The formats are interchange
//! formats, so the strings pinned here must not change.

#![cfg(all(feature = "parse", feature = "serialize"))]

use linlog::search::{Engine, Options, prove, prove_until};
use linlog::{
    Forest, Fragment, InfId, Interactive, Mode, Node, NodeId, OccId, Proof, ProofStructure, Rule,
    Sequent, Side, ViewOptions,
};

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

/// A name the atom dictionary repeats is one atom: the sequent reads as the
/// one with the name once, and is proved like it.
#[test]
fn repeated_atom_name_is_one_atom() {
    let json = r#"{"terms":[{"D":0},{"V":1}],"ids":[0,1],"var_dict":["A","A"]}"#;
    let s: Sequent = serde_json::from_str(json).unwrap();
    assert_eq!(s, "A |- A".parse().unwrap());
    let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
    assert!(matches!(outcome.verdict, linlog::Verdict::Proved(_)));
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

/// A fragment serializes as its name and reads back as the named fragment,
/// which contains it; a mode serializes as its three flags.
#[test]
fn fragment_and_mode_json_format() {
    for (fragment, name) in [
        (Fragment::EMPTY, "MLL"),
        (Fragment::MLL, "MLL"),
        (Fragment::MULTIPLICATIVE_UNITS, "MLL with units"),
        (Fragment::ADDITIVES, "ALL"),
        (Fragment::MALL, "MALL"),
        (Fragment::MELL, "MELL"),
        (Fragment::LL, "LL"),
    ] {
        let json = serde_json::to_string(&fragment).unwrap();
        assert_eq!(json, format!("{name:?}"));
        let back: Fragment = serde_json::from_str(&json).unwrap();
        assert!(back.contains(fragment) && back.name() == name, "{name}");
    }
    assert!(serde_json::from_str::<Fragment>(r#""mll""#).is_err());
    // The intuitionistic names read back as the classical fragments.
    for (name, fragment) in [("IMLL", Fragment::MLL), ("ILL", Fragment::LL)] {
        let back: Fragment = serde_json::from_str(&format!("{name:?}")).unwrap();
        assert_eq!(back, fragment);
    }

    let mode = Mode::CLASSICAL.affine();
    let json = r#"{"intuitionistic":false,"affine":true,"mix":false}"#;
    assert_eq!(serde_json::to_string(&mode).unwrap(), json);
    assert_eq!(serde_json::from_str::<Mode>(json).unwrap(), mode);
}

/// An outcome serializes as its verdict, the reason for `unknown`, the
/// fragment, mode and engine, the statistics, and for `proved` the proof's
/// own keys, so that it deserializes as the proof.
#[test]
fn outcome_json_format() {
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
    let json = serde_json::to_string(&outcome).unwrap();
    let sequent = serde_json::to_string(&s).unwrap();
    let head = r#"{"verdict":"proved","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"net","#;
    assert_eq!(
        json,
        format!(
            r#"{head}"statistics":{{"nodes":2,"memo_hits":0,"memo_entries":0,"splits":0,"links":2,"tests":2,"copies":0}},"sequent":{sequent},"proof":[{{"ax":[0,2]}},{{"ax":[3,4]}},{{"⊗":[1,0,1]}}]}}"#
        )
    );
    let proof: Proof = serde_json::from_str(&json).unwrap();
    assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
    let focus = Options::default().engine(Some(Engine::Focus));
    let json = serde_json::to_string(&prove(&s, Mode::CLASSICAL, &focus).unwrap()).unwrap();
    let head = r#"{"verdict":"proved","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"focus","#;
    assert_eq!(
        json,
        format!(
            r#"{head}"statistics":{{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":1,"links":0,"tests":0,"copies":0}},"sequent":{sequent},"proof":[{{"ax":[2,0]}},{{"ax":[3,4]}},{{"⊗":[1,0,1]}}]}}"#
        )
    );

    let s: Sequent = "|- A par B, ~A, ~B".parse().unwrap();
    let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        r#"{"verdict":"unprovable","refutation":{"equation":{"formulas":3,"needed":1,"tensors":0,"pars":1,"ones":0,"bottoms":0,"mix":false}},"fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"net","statistics":{"nodes":0,"memo_hits":0,"memo_entries":0,"splits":0,"links":0,"tests":0,"copies":0}}"#
    );
    let outcome =
        prove_until(&s, Mode::CLASSICAL.with_mix(), &Options::default(), || true).unwrap();
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        r#"{"verdict":"unknown","reason":"stopped","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":true},"engine":"net","statistics":{"nodes":1,"memo_hits":0,"memo_entries":0,"splits":0,"links":0,"tests":0,"copies":0}}"#
    );
    let s: Sequent = "!A |- A".parse().unwrap();
    let options = Options::default().copies(Some(0)).forward_copies(0);
    let outcome = prove(&s, Mode::CLASSICAL, &options).unwrap();
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        r#"{"verdict":"unknown","reason":{"copy_bound":0},"fragment":"MELL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"focus","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":0,"links":0,"tests":0,"copies":0}}"#
    );
}

/// A proof net serializes as its sequent, its Mix flag and its links as
/// pairs of occurrence ids, and reads back with its links validated.
#[test]
fn net_json_format_and_round_trip() {
    let o = OccId::new;
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let net = ProofStructure::from_links(
        Forest::new(&s).unwrap(),
        false,
        &[(o(0), o(2)), (o(3), o(4))],
    )
    .unwrap();
    let json = serde_json::to_string(&net).unwrap();
    let sequent = serde_json::to_string(&s).unwrap();
    assert_eq!(
        json,
        format!(r#"{{"sequent":{sequent},"mix":false,"links":[[0,2],[3,4]]}}"#)
    );
    let back: ProofStructure = serde_json::from_str(&json).unwrap();
    assert_eq!(back.links(), net.links());
    assert_eq!(back.to_string(), net.to_string());
    assert!(!back.mix());

    // A link that is not between dual literals, or outside the forest, is
    // rejected on deserialization; a wrong net still reads.
    for links in ["[[0,3]]", "[[1,2]]", "[[0,2],[0,2]]", "[[0,9]]"] {
        let json = format!(r#"{{"sequent":{sequent},"mix":true,"links":{links}}}"#);
        assert!(
            serde_json::from_str::<ProofStructure>(&json).is_err(),
            "{links}"
        );
    }
    let json = format!(r#"{{"sequent":{sequent},"mix":true,"links":[[0,2]]}}"#);
    let partial: ProofStructure = serde_json::from_str(&json).unwrap();
    assert!(!partial.is_complete() && partial.mix());
}

/// A proof in progress serializes as its sequent, mode, inferences (the
/// root first; an open goal is its sequent alone) and the goals its steps
/// closed, and reads back with the same goals, undo history and derivation;
/// an inference whose premises are not what its rule yields is rejected.
#[test]
fn interactive_json_format_and_round_trip() {
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&s, Mode::INTUITIONISTIC).unwrap();
    let root = InfId::new(0);
    let goals = state.apply(root, 1, Rule::ImpLeft, &[0]).unwrap();
    state.apply(goals[0], 0, Rule::Ax, &[]).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let sequent = serde_json::to_string(&s).unwrap();
    assert_eq!(
        json,
        format!(
            r#"{{"sequent":{sequent},"mode":{{"intuitionistic":true,"affine":false,"mix":false}},"inferences":[{{"sequent":[0,1,4],"rule":"⊸L","principal":1,"premises":[1,2]}},{{"sequent":[0,2],"rule":"ax"}},{{"sequent":[3,4]}}],"history":[0,1]}}"#
        )
    );
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(back.inferences(), state.inferences());
    assert_eq!(back.goals().collect::<Vec<_>>(), vec![goals[1]]);
    assert_eq!(
        back.derivation().to_string(),
        state.derivation().to_string()
    );
    let mut back = back;
    assert_eq!(back.undo(), Some(goals[0]));
    assert_eq!(back.undo(), Some(root));
    assert_eq!(back.undo(), None);

    // The rule names read back from their ASCII spellings too; a premise
    // that the rule does not yield, a rule on the wrong connective, a
    // history naming an open goal, and an occurrence outside the forest are
    // rejected.
    let ok = format!(
        r#"{{"sequent":{sequent},"mode":{{"intuitionistic":true,"affine":false,"mix":false}},"inferences":[{{"sequent":[0,1,4],"rule":"-oL","principal":1,"premises":[1,2]}},{{"sequent":[0,2]}},{{"sequent":[3,4]}}],"history":[0]}}"#
    );
    assert!(serde_json::from_str::<Interactive>(&ok).is_ok());
    for (broken, why) in [
        (
            ok.replace(r#""sequent":[0,2]}"#, r#""sequent":[0,4]}"#),
            "wrong premise",
        ),
        (
            ok.replace(r#""rule":"-oL""#, r#""rule":"&R""#),
            "wrong rule",
        ),
        (
            ok.replace(r#""history":[0]"#, r#""history":[1]"#),
            "open goal in the history",
        ),
        (
            ok.replace(r#""sequent":[3,4]}"#, r#""sequent":[3,9]}"#),
            "occurrence outside the forest",
        ),
        (
            ok.replace(r#""premises":[1,2]"#, r#""premises":[1,1]"#),
            "a premise used twice",
        ),
        (
            ok.replace(r#""history":[0]"#, r#""history":[0,0]"#),
            "a step twice in the history",
        ),
        (
            ok.replace(r#""principal":1,"#, ""),
            "a ⊗ without its principal formula",
        ),
    ] {
        assert!(
            serde_json::from_str::<Interactive>(&broken).is_err(),
            "{why}"
        );
    }

    // States with a chain of steps on one branch, a Mix, a search graft
    // and a repeated formula read back with their history.
    let mix = Mode::CLASSICAL.with_mix();
    let s: Sequent = "|- ~a par ~b, a * b, c, ~c, ?d".parse().unwrap();
    let mut state = Interactive::new(&s, mix).unwrap();
    // The position of the formula printed as `text` in an open goal.
    let at = |state: &Interactive, goal: InfId, text: &str| {
        state
            .goal(goal)
            .unwrap()
            .iter()
            .position(|&o| state.forest().formula(o).to_string() == text)
            .unwrap()
    };
    let g = state
        .apply(root, at(&state, root, "?d"), Rule::Weakening, &[])
        .unwrap()[0];
    let g = state
        .apply(g, at(&state, g, "~a ⅋ ~b"), Rule::Par, &[])
        .unwrap()[0];
    let goals = state
        .apply(g, at(&state, g, "c"), Rule::Mix, &[at(&state, g, "~c")])
        .unwrap();
    let g = goals[1];
    let goals = state
        .apply(
            g,
            at(&state, g, "a ⊗ b"),
            Rule::Tensor,
            &[at(&state, g, "~a")],
        )
        .unwrap();
    state
        .close(
            goals[0],
            &Options::default(),
            &ViewOptions::default(),
            || false,
        )
        .unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let mut back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(back.inferences(), state.inferences());
    assert_eq!(back.steps(), 5);
    let closed = back
        .close_all(&Options::default(), &ViewOptions::default(), || false)
        .unwrap();
    assert_eq!(closed.len(), 2);
    assert_eq!(back.proof().unwrap().check(mix), Ok(()));
    while back.undo().is_some() {}
    assert_eq!(back.goals().collect::<Vec<_>>(), vec![root]);
    let s: Sequent = "|- ?(a par ~a)".parse().unwrap();
    let mut state = Interactive::new(&s, mix).unwrap();
    let g = state.apply(root, 0, Rule::Contraction, &[]).unwrap()[0];
    let g = state.apply(g, 0, Rule::Dereliction, &[]).unwrap()[0];
    let g = state.apply(g, 0, Rule::Dereliction, &[]).unwrap()[0];
    state.apply(g, 1, Rule::Mix, &[]).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(back.inferences(), state.inferences());
}

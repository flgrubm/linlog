// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON form of a sequent, through the public API. The format is an
//! interchange format, so the strings pinned here must not change.

#![cfg(all(feature = "parse", feature = "serialize"))]

use linlog::Sequent;

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

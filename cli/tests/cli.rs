// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `linlog` binary end to end: verdicts, exit statuses, the JSON round
//! trip from `prove` to `check`, the `seq` commands and the help text.

use std::io::Write;
use std::process::{Command, Stdio};

/// Runs `linlog` with the arguments and `stdin` as standard input, and
/// returns the exit status, standard output and standard error.
fn linlog(args: &[&str], stdin: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

/// `prove` prints the verdict line and the derivation, and exits 0 for
/// provable, 1 for unprovable, 3 for unknown and 2 for an error.
#[test]
fn prove_verdicts_and_exit_statuses() {
    let (status, out, _) = linlog(&["prove", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "provable (MLL, classical, net engine)\n\
         ─────── ax   ─────── ax\n\
         ⊢ ~A, A      ⊢ ~B, B\n\
         ──────────────────── ⊗\n\
         \x20 ⊢ ~A, A ⊗ ~B, B\n"
    );

    for (args, status, line) in [
        (
            &["prove", "-q", "--mix", "|- A par B, ~A, ~B"][..],
            0,
            "provable (MLL, classical with Mix, net engine)",
        ),
        (
            &["prove", "|- A par B, ~A, ~B"],
            1,
            "unprovable (MLL, classical, net engine): the search was exhaustive",
        ),
        (
            &["prove", "-q", "--engine", "focus", "A * B |- A * B"],
            0,
            "provable (MLL, classical, focus engine)",
        ),
        (
            &[
                "prove",
                "--engine",
                "focus",
                "--recursion-limit",
                "1",
                "A * B |- A * B",
            ],
            3,
            "unknown (MLL, classical, focus engine): the recursion limit was reached; \
             raise it with --recursion-limit",
        ),
        (
            &["prove", "-q", "--fragment", "mall", "A |- A"],
            0,
            "provable (MALL as asserted, classical, focus engine)",
        ),
        (
            &["prove", "-q", "--copies", "0", "!A |- A"],
            3,
            "unknown (MELL, classical, focus engine): the copy bound of 0 was reached; \
             raise it with --copies",
        ),
        (
            &["prove", "-q", "-a", "A, B |- A"],
            0,
            "provable (MLL, classical affine, focus engine)",
        ),
    ] {
        assert_eq!(
            linlog(args, ""),
            (status, format!("{line}\n"), String::new())
        );
    }

    let (status, out, _) = linlog(&["prove", "-q", "--stats", "|- a * b, ~a par ~b"], "");
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "provable (MLL, classical, net engine)\nliterals chosen: 2\nlinks tried: 2\n\
             exact tests run: 2\ntime: "
        ),
        "{out}"
    );

    for (args, error) in [
        (&["prove", "A * |- A"][..], "cannot parse the sequent"),
        (
            &["prove", "-i", "|- A par B"],
            "not an intuitionistic sequent",
        ),
        (
            &["prove", "--fragment", "mll", "A & B |- A"],
            "outside the asserted fragment MLL",
        ),
        (
            &["prove", "--engine", "net", "A & B |- A"],
            "proof nets exist for MLL without units only, not for ALL",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }
}

/// The JSON output of `prove` is a proof file that `check` accepts in the
/// mode it was found in and rejects in a stricter one.
#[test]
fn check_reads_what_prove_writes() {
    let (status, json, _) = linlog(
        &["prove", "--mix", "--format", "json", "|- A par B, ~A, ~B"],
        "",
    );
    assert_eq!(status, 0);
    assert!(json.starts_with(r#"{"verdict":"proved","fragment":"MLL","#));

    let (status, out, _) = linlog(&["check", "--mix", "-q"], &json);
    assert_eq!(
        (status, out.as_str()),
        (0, "valid proof of ⊢ A ⅋ B, ~A, ~B (classical with Mix)\n")
    );
    let (status, out, _) = linlog(&["check"], &json);
    assert_eq!(status, 1);
    assert!(
        out.starts_with("invalid proof of ⊢ A ⅋ B, ~A, ~B (classical): node 2 (mix from 0, 1)"),
        "{out}"
    );
}

/// `--format net` prints the proof net of a proof after the verdict line,
/// for `prove` and for `check`, and is refused outside unit-free MLL and
/// classical mode.
#[test]
fn net_format() {
    let expected = "⊢ ~A, A ⊗ ~B, B\n\
                    ~A[0] — A[2]\n\
                    ~B[3] — B[4]\n\
                    proof net\n";
    let (status, out, _) = linlog(&["prove", "--format", "net", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        format!("provable (MLL, classical, net engine)\n{expected}")
    );
    let (status, out, _) = linlog(
        &[
            "prove",
            "--format",
            "net",
            "--engine",
            "focus",
            "A, A -o B |- B",
        ],
        "",
    );
    assert_eq!(status, 0);
    assert_eq!(
        out,
        format!("provable (MLL, classical, focus engine)\n{expected}")
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A, A -o B |- B"], "");
    let (status, out, _) = linlog(&["check", "--format", "net"], &json);
    assert_eq!(status, 0);
    assert!(out.ends_with("~B[3] — B[4]\nproof net\n"), "{out}");
    for (args, error) in [
        (
            &["prove", "--format", "net", "A & B |- A"][..],
            "proof nets exist for MLL without units only, not for ALL",
        ),
        (
            &["prove", "--format", "net", "--affine", "A |- A"],
            "proof nets exist in classical mode only",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }
}

/// `seq` prints a sequent one-sided, as JSON that it reads back, and its
/// fragment.
#[test]
fn seq_commands() {
    let (status, json, _) = linlog(&["seq", "json", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        json,
        "{\"terms\":[{\"D\":0},{\"V\":0},{\"D\":1},{\"⊗\":[1,2]},{\"V\":1}],\
         \"ids\":[0,3,4],\"var_dict\":[\"A\",\"B\"]}\n"
    );
    let printed = linlog(&["seq", "print", "--json-input"], &json);
    assert_eq!(printed, (0, "⊢ ~A, A ⊗ ~B, B\n".into(), String::new()));
    let fragment = linlog(&["seq", "fragment", "--file", "-"], "A & B |- 1");
    assert_eq!(fragment, (0, "MALL\n".into(), String::new()));
}

/// The help lists every command.
#[test]
fn help_names_every_command() {
    let (status, out, _) = linlog(&["--help"], "");
    assert_eq!(status, 0);
    for command in ["prove", "check", "seq"] {
        assert!(out.contains(&format!("\n  {command} ")), "{command}: {out}");
    }
    let (_, out, _) = linlog(&["seq", "--help"], "");
    for command in ["print", "json", "fragment"] {
        assert!(out.contains(&format!("\n  {command} ")), "{command}: {out}");
    }
}

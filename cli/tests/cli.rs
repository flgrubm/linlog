// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `linlog` binary end to end: verdicts, exit statuses, the JSON round
//! trip from `prove` to `check`, the interactive session, the `seq`
//! commands and the help text.

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

/// Returns the text with every time that follows "after " as `…`: how
/// long a search took is the machine's.
fn timeless(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("after ") {
        let (head, tail) = rest.split_at(at + "after ".len());
        out.push_str(head);
        let time = tail
            .find(|c: char| !(c.is_ascii_digit() || "._µnms".contains(c)))
            .unwrap_or(tail.len());
        if time > 0 && tail.starts_with(|c: char| c.is_ascii_digit()) {
            out.push('…');
            rest = &tail[time..];
        } else {
            rest = tail;
        }
    }
    out.push_str(rest);
    out
}

/// A parse error shows the line it is in with a caret under the character
/// that cannot go on a sequent. Of a long line it shows the part around
/// that character, cut between characters, and says which character it
/// is; in an input of several lines it says which line.
#[test]
fn parse_errors() {
    let error = |args: &[&str], stdin: &str| {
        let (status, out, err) = linlog(args, stdin);
        assert_eq!((status, out.as_str()), (2, ""), "{err}");
        err
    };
    assert_eq!(
        error(&["seq", "fragment", "|- A * )"], ""),
        "error: cannot parse the sequent\n  |- A * )\n         ^ unexpected \")\"\n"
    );

    // The 70 001st character of one line, after 17 499 characters of
    // three bytes each.
    let head = format!("|- a{}", " ⊗ a".repeat(17_499));
    assert_eq!(head.chars().count(), 70_000);
    let input = format!("{head}${}", " ⊗ a".repeat(100));
    assert_eq!(
        error(&["seq", "fragment"], &input),
        format!(
            "error: cannot parse the sequent\n  …{}${} ⊗ …\n  {}^ unexpected \"$\" at \
             character 70001\n",
            " ⊗ a".repeat(15),
            " ⊗ a".repeat(14),
            " ".repeat(61)
        )
    );

    assert_eq!(
        error(&["seq", "fragment"], "A,\n  B * |- C\n"),
        "error: cannot parse the sequent\n    B * |- C\n        ^ unexpected \"|\" at line 2, \
         character 7\n"
    );
}

/// `--timeout` ends a search that would run for hours (2⁴² splits that no
/// count cuts) with an unknown verdict, the limit in its line and exit
/// status 3, on one thread and on a pool; and it counts from the start of
/// the command, so a sequent that is not read in time is given up on with
/// a line that says so.
#[test]
fn timeout() {
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    let sequent = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        literals.join(", ")
    );
    for threads in [["--deterministic"].as_slice(), &["--jobs", "2"]] {
        let args = [&["prove", "-a", "--timeout", "200ms"], threads, &[&sequent]].concat();
        let (status, out, err) = linlog(&args, "");
        assert_eq!(
            (status, out.as_str(), err.as_str()),
            (
                3,
                "unknown (MALL, classical affine, focus engine): the time limit of 200ms was \
                 reached; --timeout DURATION gives the search longer\n",
                ""
            )
        );
    }

    // Standard input stays open and empty until the command has answered.
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(["prove", "--timeout", "100ms"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let open = child.stdin.take();
    let status = child.wait().unwrap();
    drop(open);
    let mut out = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take().unwrap(), &mut out).unwrap();
    assert_eq!(
        (status.code(), out.as_str()),
        (
            Some(3),
            "unknown: the time limit of 100ms was reached while the sequent was read\n"
        )
    );
}

/// `--jobs` beyond what the machine runs at once is taken as that many,
/// and standard error says so; the verdict and the status are the
/// search's.
/// A search that holds more than `--memory-limit` allows answers
/// "unknown" with the limit and the flag, exit status 3, and a sequent
/// that unfolds beyond `--occurrence-limit` is an error before it is
/// unfolded: 25 doublings of one atom are 427 bytes of JSON and 67 million
/// occurrences.
#[test]
fn limits_on_memory_and_occurrences() {
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    let sequent = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        literals.join(", ")
    );
    for threads in [["--deterministic"].as_slice(), &["--jobs", "2"]] {
        let args = [
            &["prove", "-a", "--memory-limit", "1KiB"],
            threads,
            &[&sequent],
        ]
        .concat();
        let (status, out, err) = linlog(&args, "");
        assert_eq!(
            (status, timeless(&out), err),
            (
                3,
                "unknown (MALL, classical affine, focus engine): the memory limit of 1 KiB was \
                 reached after …; raise it with --memory-limit SIZE\n"
                    .to_owned(),
                String::new()
            )
        );
    }

    let doublings: Vec<String> = (0..25).map(|i| format!(r#"{{"⊗":[{i},{i}]}}"#)).collect();
    let shared = format!(
        r#"{{"terms":[{{"V":0}},{}],"ids":[25],"var_dict":["A"]}}"#,
        doublings.join(",")
    );
    for command in [
        ["prove"].as_slice(),
        &["seq", "print"],
        &["seq", "fragment"],
    ] {
        let (status, out, err) = linlog(&[command, &["--json-input"]].concat(), &shared);
        assert_eq!(
            (status, out.as_str(), err.as_str()),
            (
                2,
                "",
                "error: the sequent unfolds to 67108863 subformula occurrences, more than the \
                 limit of 50000000; raise it with --occurrence-limit\n"
            )
        );
    }
    let (status, _, err) = linlog(
        &["seq", "fragment", "--occurrence-limit", "2", "A |- A * A"],
        "",
    );
    assert!(
        status == 2 && err.contains("to 4 subformula occurrences"),
        "{err}"
    );
    assert_eq!(
        linlog(
            &["seq", "fragment", "--occurrence-limit", "none", "A |- A"],
            ""
        ),
        (0, "MLL\n".to_owned(), String::new())
    );
}

#[test]
fn jobs_are_bounded() {
    let (status, out, err) = linlog(&["prove", "--quiet", "--jobs", "100000", "A |- A"], "");
    assert_eq!(
        (status, out.as_str()),
        (0, "provable (MLL, classical, net engine)\n")
    );
    assert!(
        err.starts_with("note: --jobs 100000 is more than the ")
            && err.contains(" threads a search"),
        "{err:?}"
    );
}

/// A derivation estimated above `--derivation-limit` is not written: the
/// verdict stands with its exit status, standard error says how large the
/// derivation is and how to get it, and `none` lifts the limit;
/// `--tree never` leaves the tree out without a word, and into a pipe
/// `--tree auto` writes it whatever its size.
#[test]
fn derivation_limit_and_tree_switch() {
    let sequent = "A, A -o B |- B";
    let verdict = "provable (MLL, classical, net engine)\n";
    let (status, out, err) = linlog(&["prove", "--derivation-limit", "100", sequent], "");
    assert_eq!((status, out.as_str()), (0, verdict));
    assert_eq!(
        err,
        "the derivation is not written: its 3 inferences with 29 characters of sequents are \
         estimated at 616 B, over the limit of 100 B; --format json writes the proof itself, \
         --derivation-limit SIZE raises the limit and --derivation-limit none lifts it\n"
    );
    let (status, out, err) = linlog(
        &[
            "prove",
            "--format",
            "latex",
            "--derivation-limit",
            "1KiB",
            sequent,
        ],
        "",
    );
    assert_eq!(status, 0);
    assert!(
        out.contains("\\begin{prooftree}") && err.is_empty(),
        "{out}{err}"
    );

    let (_, whole, _) = linlog(&["prove", sequent], "");
    for when in ["always", "auto"] {
        let (status, out, err) = linlog(
            &[
                "prove",
                "--tree",
                when,
                "--derivation-limit",
                "none",
                sequent,
            ],
            "",
        );
        assert_eq!((status, out, err), (0, whole.clone(), String::new()));
    }
    let (status, out, err) = linlog(&["prove", "--tree", "never", sequent], "");
    assert_eq!((status, out.as_str(), err.as_str()), (0, verdict, ""));

    // `check` is under the same limit.
    let (_, proof, _) = linlog(&["prove", "--format", "json", sequent], "");
    let (status, out, err) = linlog(&["check", "--derivation-limit", "100"], &proof);
    assert_eq!(
        (status, out.as_str()),
        (0, "valid proof of ⊢ ~A, A ⊗ ~B, B (classical)\n")
    );
    assert!(
        err.starts_with("the derivation is not written: its 3 inferences"),
        "{err}"
    );
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
            "unprovable (MLL, classical, net engine): the count equation fails: a provable \
             one-sided sequent of MLL with 0 ⊗, 1 ⅋, 0 1 and 0 ⊥ has exactly 0 − 1 − 0 + 0 + 2 \
             = 1 formulas, and this one has 3",
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
            "unknown (MLL, classical, focus engine): the recursion limit of 1 was reached \
             after …; raise it with --recursion-limit N",
        ),
        (
            &["prove", "-q", "--fragment", "mall", "A |- A"],
            0,
            "provable (MALL as asserted, classical, focus engine)",
        ),
        (
            &[
                "prove",
                "-q",
                "--copies",
                "0",
                "--forward-copies",
                "0",
                "!A |- A",
            ],
            3,
            "unknown (MELL, classical, focus engine): the copy bound of 0 was reached after \
             …; raise it with --copies N, or lift it with --copies none to deepen it while the \
             time limit lasts",
        ),
        (
            &["prove", "-q", "-a", "A, B |- A"],
            0,
            "provable (MLL, classical affine, focus engine)",
        ),
    ] {
        let (code, out, err) = linlog(args, "");
        assert_eq!(
            (code, timeless(&out), err),
            (status, format!("{line}\n"), String::new())
        );
    }

    let (status, out, _) = linlog(
        &[
            "prove",
            "-q",
            "--stats",
            "--deterministic",
            "|- a * b, ~a par ~b",
        ],
        "",
    );
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

/// Intuitionistic mode: the verdict line names the intuitionistic fragment
/// and the engine, the derivation is two-sided, `check -i` reads what
/// `prove -i` writes and prints the sequent two-sided, and a sequent with
/// no intuitionistic reading is an error that names the subformula.
#[test]
fn intuitionistic_mode() {
    let (status, out, _) = linlog(&["prove", "-i", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "provable (IMLL, intuitionistic, net engine)\n\
         ───── ax   ───── ax\n\
         A ⊢ A      B ⊢ B\n\
         ──────────────── ⊸L\n\
         \x20 A, A ⊸ B ⊢ B\n"
    );
    for (args, status, line) in [
        (
            &["prove", "-i", "-q", "A & B |- A"][..],
            0,
            "provable (IALL, intuitionistic, additive engine)",
        ),
        (
            &["prove", "-i", "-q", "!A, !(A -o B) |- !B"],
            0,
            "provable (IMELL, intuitionistic, two-sided engine)",
        ),
        (
            &["prove", "-i", "-a", "-q", "A, B |- A"],
            0,
            "provable (IMLL, intuitionistic affine, two-sided engine)",
        ),
        (
            &["prove", "-i", "-q", "--engine", "two-sided", "A |- A"],
            0,
            "provable (IMLL, intuitionistic, two-sided engine)",
        ),
        (
            &["prove", "-i", "-q", "(A -o B) -o A |- A"],
            1,
            "unprovable (IMLL, intuitionistic, net engine): ~A occurs 1 more time than A, so \
             they cannot all meet in axioms",
        ),
        (
            &["prove", "-q", "--engine", "additive", "|- A & B, ~A + ~B"],
            0,
            "provable (ALL, classical, additive engine)",
        ),
    ] {
        let (got, out, err) = linlog(args, "");
        assert_eq!(got, status, "{args:?}: {err}");
        assert_eq!(out.lines().next(), Some(line), "{args:?}");
    }
    for (args, error) in [
        (
            &["prove", "-i", "|- A par B"][..],
            "not an intuitionistic sequent: the subformula A ⅋ B is neither an intuitionistic \
             formula nor the negation of one",
        ),
        (
            &["prove", "-i", "A |- B, C"],
            "both B and C can only be the goal",
        ),
        (
            &["prove", "-i", "--engine", "focus", "A |- A"],
            "the focus engine does not search in intuitionistic mode",
        ),
        (
            &["prove", "-i", "--mix", "A |- A"],
            "Mix has no intuitionistic form",
        ),
        (
            &["prove", "--engine", "additive", "A, B |- A * B"],
            "the additive engine decides a sequent of two additive-only formulas, not 3 formulas of MLL",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }

    let (status, json, _) = linlog(&["prove", "-i", "--format", "json", "A & B |- B"], "");
    assert_eq!(status, 0);
    assert!(
        json.starts_with(r#"{"verdict":"proved","fragment":"IALL","#),
        "{json}"
    );
    let (status, out, _) = linlog(&["check", "-i"], &json);
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "valid proof of A & B ⊢ B (intuitionistic)\n\
         \x20 ───── ax\n\
         \x20 B ⊢ B\n\
         ───────── &L₂\n\
         A & B ⊢ B\n"
    );
    // The classical proof of a sequent classical linear logic proves and
    // intuitionistic linear logic does not is an invalid proof under -i.
    let schellinx = "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)";
    let (status, json, _) = linlog(&["prove", "--format", "json", schellinx], "");
    assert_eq!(status, 0);
    let (status, out, _) = linlog(&["check", "-i", "-q"], &json);
    assert_eq!(status, 1);
    assert!(
        out.contains("a sequent of the rule has 2 formulas on the right of ⊢ instead of one"),
        "{out}"
    );

    let (status, out, _) = linlog(&["seq", "print", "-i", "A, A -o B |- B"], "");
    assert_eq!((status, out.as_str()), (0, "A, A ⊸ B ⊢ B\n"));
    let (status, _, err) = linlog(&["seq", "print", "-i", "A, B |-"], "");
    assert_eq!(status, 2);
    assert!(err.contains("no formula can be the goal"), "{err}");
    let (status, out, _) = linlog(&["seq", "fragment", "-i", "!A |- A & 1"], "");
    assert_eq!((status, out.as_str()), (0, "ILL\n"));
}

/// `interact` reads commands from standard input: rules are listed and
/// applied by goal and position, refused with a reason, undone, goals are
/// closed by the search, the session is saved and resumed, and the finished
/// proof is checked, which decides the exit status.
#[test]
fn interactive_session() {
    let dir = std::env::temp_dir().join(format!("linlog-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let session = dir.join("session.json");
    let proof = dir.join("proof.json");
    let script = format!(
        "goals\nrules 0 1\napply 0 1 par\napply 0 1 * 0\napply 1 0 ax\nundo\nsave {s}\nquit\n",
        s = session.display()
    );
    let (status, out, err) = linlog(&["interact", "A, A -o B |- B"], &script);
    assert_eq!(
        out,
        format!(
            "goal 0: ⊢ 0: ~A, 1: A ⊗ ~B, 2: B\n\
             ⊗ (with a split)\n\
             error: the rule ⅋ does not act on formula 1\n\
             opened goal 1: ⊢ 0: ~A, 1: A\n\
             opened goal 2: ⊢ 0: ~B, 1: B\n\
             closed\n\
             reopened goal 1: ⊢ 0: ~A, 1: A\n\
             session written to {}\n",
            session.display()
        )
    );
    assert_eq!((status, err.as_str()), (1, ""));
    let script = format!(
        "close 1\napply 2 1 ax\nshow\nproof {p}\n",
        p = proof.display()
    );
    let (status, out, err) = linlog(&["interact", "--state", session.to_str().unwrap()], &script);
    assert_eq!(
        out,
        format!(
            "goal 1: proved (MLL, classical, focus engine)\n\
             closed; no goal is open: `proof` checks the proof\n\
             ─────── ax   ─────── ax\n\
             ⊢ ~A, A      ⊢ ~B, B\n\
             ──────────────────── ⊗\n\
            \x20 ⊢ ~A, A ⊗ ~B, B\n\
             valid proof written to {}\n",
            proof.display()
        )
    );
    assert_eq!((status, err.as_str()), (0, ""));
    let (status, _, _) = linlog(&["check", "-q", proof.to_str().unwrap()], "");
    assert_eq!(status, 0);
    std::fs::remove_dir_all(&dir).unwrap();

    // Two-sided: the goals show both sides, the rules their ILL names, and
    // a split that leaves two goals on a side is refused.
    let script = "goals\nrules 0 1\napply 0 1 -oL 2\napply 0 1 -oL 0\nclose\nproof\n";
    let (status, out, err) = linlog(&["interact", "-i", "A, A -o B |- B"], script);
    assert_eq!(
        out,
        "goal 0: 0: A, 1: A ⊸ B ⊢ 2: B\n\
         ⊸L (with a split)\n\
         error: a premise would have 2 formulas on the right of ⊢ instead of one\n\
         opened goal 1: 0: A ⊢ 1: A\n\
         opened goal 2: 0: B ⊢ 1: B\n\
         goal 1: proved (IMLL, intuitionistic, two-sided engine)\n\
         goal 2: proved (IMLL, intuitionistic, two-sided engine)\n\
         no goal is open: `proof` checks the proof\n\
         valid proof (intuitionistic)\n\
         ───── ax   ───── ax\n\
         A ⊢ A      B ⊢ B\n\
         ──────────────── ⊸L\n\
        \x20 A, A ⊸ B ⊢ B\n"
    );
    assert_eq!((status, err.as_str()), (0, ""));
    let (status, _, err) = linlog(&["interact"], "");
    assert_eq!(status, 2);
    assert!(err.contains("no sequent given"), "{err}");
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
            "proof nets exist in linear mode only",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }
}

/// `--format latex` and `--format typst` print the verdict as a comment and
/// the derivation as a proof tree, for `prove`, `check` and the session's
/// `show`, and the sequent for `seq print`; `--standalone` makes a document
/// and is refused for the other formats.
#[test]
fn latex_and_typst_formats() {
    let (status, out, _) = linlog(&["prove", "--format", "latex", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "% provable (MLL, classical, net engine)\n\
         \\begin{prooftree}\n\
         \\infer0[$\\mathrm{ax}$]{\\vdash A^\\bot, A}\n\
         \\infer0[$\\mathrm{ax}$]{\\vdash B^\\bot, B}\n\
         \\infer2[$\\otimes$]{\\vdash A^\\bot, A \\otimes B^\\bot, B}\n\
         \\end{prooftree}\n"
    );
    let (_, json, _) = linlog(&["prove", "-i", "--format", "json", "A, A -o B |- B"], "");
    let (status, out, _) = linlog(&["check", "-i", "--format", "typst", "--standalone"], &json);
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "// valid proof of A, A ⊸ B ⊢ B (intuitionistic)\n\
             #import \"@preview/curryst:0.6.0\": prooftree, rule\n"
        ),
        "{out}"
    );
    let printed = linlog(
        &["seq", "print", "-i", "--format", "latex", "A, A -o B |- B"],
        "",
    );
    assert_eq!(
        printed,
        (0, "$A, A \\multimap B \\vdash B$\n".into(), String::new())
    );
    let (status, out, err) = linlog(&["prove", "--standalone", "A |- A"], "");
    assert_eq!((status, out.as_str()), (2, ""));
    assert!(err.contains("--standalone needs --format latex"), "{err}");
    let (status, out, _) = linlog(&["interact", "A |- A"], "show latex\nquit\n");
    assert_eq!(status, 1);
    assert_eq!(
        out,
        "\\begin{prooftree}\n\
         \\hypo{\\vdots}\n\
         \\infer[no rule]1{\\vdash A^\\bot, A}\n\
         \\end{prooftree}\n"
    );
}

/// `--format rocq` prints the verdict as a Rocq comment and the
/// derivation as a lemma with its proof script for `prove` and `check`;
/// `--standalone` adds the import; a proof with Mix has no certificate.
#[test]
fn rocq_format() {
    let (status, out, _) = linlog(&["prove", "--format", "rocq", "A |- A"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "(* provable (MLL, classical, net engine) *)\n\
         Lemma certificate (A : formula) : ll [dual A; A].\nProof.\nax_expansion.\nQed.\n"
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A * B |- B * A"], "");
    let (status, out, _) = linlog(&["check", "--format", "rocq", "--standalone"], &json);
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "(* valid proof of ⊢ ~A ⅋ ~B, B ⊗ A (classical) *)\n\
             From NanoYalla Require Import macroll.\n\nLemma certificate (A B : formula)"
        ),
        "{out}"
    );
    assert!(out.contains("apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).\n"));
    let (status, out, err) = linlog(&["prove", "--mix", "--format", "rocq", "A, B |- A, B"], "");
    assert_eq!((status, out.as_str()), (2, ""));
    assert!(err.contains("NanoYalla has no Mix rule"), "{err}");
}

/// `--format svg` and `--format net-svg` print the verdict as an XML
/// comment, with no `--` in it, and the derivation or the proof net as an
/// SVG document, for `prove`, `check`, `seq print` and the session's
/// `show`; an SVG is always a document, so `--standalone` is refused.
#[test]
fn svg_formats() {
    let (status, out, _) = linlog(&["prove", "--format", "svg", "A |- A"], "");
    assert_eq!(status, 0);
    assert!(
        out.starts_with("<!-- provable (MLL, classical, net engine) -->\n<svg ")
            && out.ends_with("</svg>\n"),
        "{out}"
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A * B |- B * A"], "");
    let (status, out, _) = linlog(&["check", "--format", "net-svg"], &json);
    assert_eq!(status, 0);
    assert!(out.contains("<circle id=\"o0\""), "{out}");
    let bounds = ["--copies", "0", "--forward-copies", "0"];
    let args = [&["prove", "--format", "svg"], &bounds[..], &["|- ?A"]].concat();
    let (status, out, _) = linlog(&args, "");
    assert_eq!(status, 3);
    assert!(out.contains("while the time limit lasts -->"), "{out}");
    let (status, out, _) = linlog(&["seq", "print", "--format", "svg", "A |- A"], "");
    assert_eq!(status, 0);
    assert!(out.contains("<title>⊢ A⊥, A</title>"), "{out}");
    let (status, _, err) = linlog(&["prove", "--format", "svg", "--standalone", "A |- A"], "");
    assert_eq!(status, 2);
    assert!(err.contains("--standalone needs --format latex"), "{err}");
    let (_, out, _) = linlog(&["interact", "A |- A"], "show svg\nquit\n");
    assert!(
        out.starts_with("<svg ") && out.contains(">⋮</text>"),
        "{out}"
    );
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
    for command in ["prove", "check", "interact", "seq"] {
        assert!(out.contains(&format!("\n  {command} ")), "{command}: {out}");
    }
    let (_, out, _) = linlog(&["seq", "--help"], "");
    for command in ["print", "json", "fragment"] {
        assert!(out.contains(&format!("\n  {command} ")), "{command}: {out}");
    }
}

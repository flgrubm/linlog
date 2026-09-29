# Step 4 report: the library API and the CLI

Session of 2026-09-29, from `plan/04-api-and-cli.md`. Wires the focused
engine of step 3 into a command line program and settles the library's
front door, realising the user-facing half of plan decisions D8 (detection
with overrides), D9 (three-valued outcomes with statistics) and D11 (the
clock and threads live in the CLI).

## Outcome

The command is now called `linlog` (the package is still `linlog-cli`). It
has three commands. `linlog prove` decides a sequent and prints the verdict
line, the derivation and, on request, the statistics, or the whole outcome as
JSON; its exit status is 0 for provable, 1 for unprovable, 3 for unknown and
2 for an error. `linlog check` runs the independent checker on a proof file,
and the JSON that `prove` writes is such a file. `linlog seq print|json|fragment`
prints, serializes or classifies a sequent. `core` gained a JSON form for
`Fragment`, `Mode` and `Outcome`, `CheckError::describe` for messages with
formulas, error messages that say what is wrong with the input, and crate
docs that show the common path as a doc test. All checks pass at the last
change: `cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace` (52 unit tests in core with 2 ignored, 9 parse and
8 serialize integration tests, 8 doc tests, 2 unit tests and 4 integration
tests in the CLI), `cargo test -p linlog --no-default-features`,
`cargo hack check --feature-powerset -p linlog`, `cargo deny check`
(advisories, bans, licenses and sources all ok) and `nix flake check`
("all checks passed!"). `nix run . -- prove -q "A |- A"` finds the binary.

Ten changes, in order (`jj log`):

1. **Give fragments, modes and search outcomes a JSON form.**
   `core/src/serialize/search.rs`, the new `Outcome::mode` field, and the
   format pinned in `core/tests/serialize.rs`.
2. **Describe check errors with formulas.** `CheckError::describe(&forest)`
   returns a `Described` value whose `Display` prints formulas where the
   plain `Display` prints occurrence ids.
3. **Make error messages say what is wrong with the input.** The messages of
   `Error` and `ParseError`, which a JSON or text input reaches.
4. **Redesign the CLI around prove, check and seq.** `cli/src/*`, the
   `[[bin]]` named `linlog`, `meta.mainProgram` in `modules/workspace.nix`,
   and the constants `Options::DEFAULT_MEMO_LIMIT` and
   `Options::DEFAULT_RECURSION_LIMIT` in core.
5. **Stop a search on Ctrl-C with ctrlc.** The one new dependency.
6. **Show the common path in the crate docs.** `core/src/lib.rs`.
7. **Test the CLI end to end.** `cli/tests/cli.rs`.
8. **Document usage and the CLI's invariants.** The README usage section,
   CLAUDE.md, the new `.claude/rules/cli.md` and `.claude/rules/core.md`.
9. **Refuse to check intuitionistic proofs instead of calling them invalid.**
10. This report, with a line added to `.claude/rules/cli.md`.

## The CLI

```
linlog prove [SEQUENT] [-f PATH] [--json-input]
             [-i|--intuitionistic] [-a|--affine] [--mix]
             [--fragment mll|mll-units|all|mall|mell|ll] [--engine auto|focus]
             [--timeout DURATION] [--memo-limit N] [--recursion-limit N]
             [--format text|json] [-o PATH] [-q|--quiet] [--stats]
linlog check [PROOF] [-i] [-a] [--mix] [--format text|json] [-o PATH] [-q]
linlog seq print    [SEQUENT] [-f PATH] [--json-input] [-o PATH]
linlog seq json     [SEQUENT] [-f PATH] [--json-input] [--optimize] [-o PATH]
linlog seq fragment [SEQUENT] [-f PATH] [--json-input]
```

A sequent comes from the argument, from `--file` (where `-` means standard
input), or from standard input; when standard input is a terminal and
nothing else is given, the command says so instead of waiting. `--copies N`
is parsed but hidden from the help, and refused with "--copies bounds the
copies of ? formulas, which no engine searches yet". Every command that reads a
sequent prints the syntax after its help.

One example per command, run in this session (`cargo run -q -p linlog-cli --`
in place of `linlog`):

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B
$ echo $?
0
$ linlog prove -q "A * B |- B * A, C"
unprovable (MLL, classical, focus engine): the search was exhaustive      # exit 1
$ linlog prove --timeout 1s -q -f three-partition-refutation.txt
unknown (MALL, classical, focus engine): the time limit of 1s was reached   # exit 3
$ linlog prove --mix --stats "|- A par B, ~A, ~B"
provable (MLL, classical with Mix, focus engine)
─────── ax   ─────── ax
⊢ A, ~A      ⊢ B, ~B
──────────────────── mix
   ⊢ A, B, ~A, ~B
   ─────────────── ⅋
   ⊢ A ⅋ B, ~A, ~B
stable sequents visited: 3 (0 from the memo)
memo entries at most: 3
splits examined: 4
time: 47.10µs
$ linlog prove "!A |- A"
error: no engine for MELL in classical mode yet                            # exit 2
$ linlog prove "A * (B |- )"
error: cannot parse the sequent
  A * (B |- )
          ^ unexpected "-"
$ linlog prove --mix --format json "|- A par B, ~A, ~B" | linlog check
invalid proof of ⊢ A ⅋ B, ~A, ~B (classical): node 2 (mix from 0, 1) with premises ⊢ A, ~A and ⊢ B, ~B: the mode forbids the rule
$ linlog prove --format json "A |- A" | linlog check --quiet
valid proof of ⊢ ~A, A (classical)
$ linlog seq print "A * B -o C |- ~C -o ~(A * B)"
⊢ (A ⊗ B) ⊗ ~C, C ⅋ (~A ⅋ ~B)
$ linlog seq json "A |- A"
{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]}
$ linlog seq fragment "A & B |- 1"
MALL
```

The three-partition input is the refutable instance of step 3's slow test
(items 1, 1, 1, 3, 1, 1 into two bins of size 4); `--stats` on it reports
73 728 stable sequents and 100 million splits in 1.01 s. Pressing Ctrl-C
(tested by sending SIGINT after one second) prints `unknown (…):
interrupted` with the statistics and exits 3.

## The JSON forms

- `Fragment` is its name: `"MLL"`, `"MLL with units"`, `"ALL"`, `"MALL"`,
  `"MELL"`, `"LL"`. Reading a name back gives the named fragment, which
  contains every fragment of that name, so the trip is lossy towards the
  larger fragment. As an assertion that only switches off prunes and never
  refuses the sequent it came from. The other choice, a list of connective
  classes, would round-trip exactly but is not what a person reading the
  output wants, and nothing reads a fragment back yet.
- `Mode` is `{"intuitionistic": false, "affine": true, "mix": false}`.
- `Outcome` serializes, but does not deserialize, since it is output only:

  ```json
  {"verdict":"proved","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},
   "engine":"focus","statistics":{"nodes":2,"memo_hits":0,"memo_entries":2,"splits":1},
   "sequent":{…},"proof":[{"ax":[3,4]},{"ax":[2,0]},{"⊗":[1,1,0]}]}
  ```

  `reason` appears only for `unknown`, as a snake_case tag (`"stopped"`,
  `"recursion_limit"`, `{"context_too_wide": 70}`). For `proved`, the
  proof's own keys `sequent` and `proof` are flattened into the outcome.
  serde ignores unknown keys, so the whole outcome deserializes as a `Proof`,
  and that is how `check` reads `prove`'s output with no extraction step.
  All three verdicts are pinned in `core/tests/serialize.rs`.

## The library surface

What a user of the `linlog` crate writes, all re-exported from the root:

- `"…".parse::<Sequent>()`, `sequent.fragment()`, `Display` for both.
- `prove(&sequent, mode, &Options::default())` and `prove_until(…, stop)`,
  which return `Outcome { verdict, fragment, mode, engine, statistics }`.
- `Mode { intuitionistic, affine, mix }`, built from named fields or from
  `Mode::CLASSICAL.with_mix()`. `Options::default().memo_limit(n)
  .recursion_limit(n).engine(…).fragment(…)`, with the defaults as the
  public constants `Options::DEFAULT_MEMO_LIMIT` and
  `Options::DEFAULT_RECURSION_LIMIT`.
- `proof.derivation()?.to_string()`, `proof.check(mode)`, and
  `err.describe(proof.forest())` for a `CheckError` with formulas.
- serde for `Sequent`, `Proof`, `Fragment`, `Mode` and (serialize only)
  `Outcome`, behind the `serialize` feature.

The crate docs in `lib.rs` run this path as two doc tests (parse, fragment,
prove, the derivation asserted, a JSON round trip and check; then a mode
from named flags, a builder with an asserted fragment and a stop closure).
The review of the exports found nothing to remove; `Described`, the return
type of `describe`, is exported from `linlog::proofs` next to `CheckError`.

## What a later step does to add an engine or an output format

- **An engine** (steps 6, 7, 8): an `Engine` variant in `core` with its name
  in `Display`, which is also its JSON form; a row in `prove_until`; then in
  `cli/src/argument_parsing.rs` a variant of `EngineArg` with a doc comment,
  which becomes its `--help` line, and its arm in
  `From<EngineArg> for Option<Engine>`. The verdict line and the JSON pick
  up the name without any other change.
- **An output format** (steps 5, 9, 10, 11: `net`, `latex`, `typst`, `svg`,
  `rocq`): a variant of `Format` in `argument_parsing.rs` with a doc comment,
  and its arm in the `match format` of `prove` and of `check_text` in
  `cli/src/prove.rs`. A format that renders the derivation builds it inside
  the `on_large_stack` closure, as the text format does, because the
  builder recurses to the proof's height.
- **The copy bound** (step 7): remove `hide = true` from `--copies` and the
  refusal at the top of `prove`, and pass the value to the `Options` setter
  that step 7 adds.
- **A new `Reason` variant or `Statistics` field**: its line in the proxies
  of `core/src/serialize/search.rs`. The compiler does not catch a missing
  statistics field there, because the remote derive lists the fields it
  serializes. The verdict line prints `Reason`'s `Display`.

## Decisions where the prompt left room

- **The binary is `linlog`**, through a `[[bin]]` section in
  `cli/Cargo.toml` and `meta.mainProgram` in `modules/workspace.nix`, so the
  README's commands are literal. The package keeps its name, so
  `cargo run -p linlog-cli`, the devshell's `launch`, the permission rules
  and `nix build` are unchanged.
- **The prompt's shape, kept almost as proposed.** Changes from it:
  `--format` lists only `text` and `json`, because values that always fail
  would make `--help` lie; the extension point above is one variant and one
  match arm. `--engine` offers `auto` and `focus` for the same reason.
  `--fragment` spells MLL with units as `mll-units` rather than `mll1`,
  because the name is what the help and the verdict line say. `check` takes
  a file path (or standard input), not the JSON text as an argument,
  because a proof is too long to type. `seq` keeps the names `print`,
  `json` and `fragment`; `--optimize` stays on `seq json` only, where it
  still means something: it canonicalises a hand-written JSON arena, which
  parsing text always does.
- **Exit statuses follow grep and diff**: 0 for yes, 1 for no, 2 for
  trouble, which is also clap's status for bad arguments, and 3 for
  unknown. `if linlog prove -q …` reads naturally in a shell.
- **`--quiet` prints the verdict line**, the one with the fragment and the
  engine, rather than a bare word. `--quiet` and `--stats` are
  text-format options. JSON always carries the counters and never the time
  (core has no clock, and a time would make the output irreproducible).
  `--quiet` with `--format json` has no effect, which its help says.
- **`check` takes the mode from its flags**, as the prompt says, even though
  `prove`'s JSON now carries a `mode` key: an explicit mode keeps the
  independent check independent. A mode the checker cannot check
  (intuitionistic) is an error with exit 2, not an invalid proof.
- **The search always runs on a spawned thread**, not only when the limit
  exceeds the default. One code path, and the thread costs nothing
  measurable. The stack is `recursion_limit × STACK_PER_LEVEL`, at least
  8 MiB, where `STACK_PER_LEVEL` is 4 KiB in debug builds and 1 KiB in
  release. That is twice step 3's measured per-level cost, to leave room
  for the derivation builder and renderer on the same thread. Measured: a
  3000-level `⊗` chain is proved with `--recursion-limit 20000` in debug and
  release builds, and a 1000-level chain's derivation (35 MB of text) renders
  in a debug build without overflow.
- **The stop closure reads the clock every 1024 polls.** At about six
  million stable sequents a second, one clock read per poll would cost a
  noticeable share of the search, and 1024 polls are well under a
  millisecond. The CLI records why it stopped, so the verdict line says
  "the time limit of 1s was reached" or "interrupted" rather than the
  engine's generic "the search was stopped".
- **Durations are parsed by hand** (`parse_duration`: a decimal number and
  `ms`, `s`, `m`/`min` or `h`, bare numbers being seconds). The grammar the
  prompt asks for is fifteen lines with a test, and a combined form such as
  `1h30m` was not requested, so no crate was added. humantime would have
  been the candidate; its maintenance status was not checked in this
  session.
- **Ctrl-C through `ctrlc`** (MIT or Apache-2.0; on Linux it adds `nix`,
  MIT). The first Ctrl-C sets a flag the stop closure polls, so the verdict
  is unknown and `--stats` still reports; the second exits with 130. The
  crate-source-explorer confirmed at the pinned version that the handler
  runs on a thread of its own once per signal (so `exit` is safe there),
  that a second `set_handler` returns an error rather than panicking, and
  that the crate has no default features.
- **The CLI tests use `std::process::Command`**, not `assert_cmd`. Cargo
  provides the binary's path as `CARGO_BIN_EXE_linlog`, and the helper that
  runs it with standard input and collects the status and both streams is
  a dozen lines, while `assert_cmd` would bring its own dependency tree for
  four tests.
- **Parse errors point at the input**: the CLI prints the input with a caret
  under the failing character, and an empty input says that the empty
  sequent is written `|-`. The library's `ParseError` prints
  `unexpected "-" at byte 8`, since it does not hold the input.
- **Error messages** of `Error` now start in lower case like the step 3
  ones, and name the thing in the input rather than the index arithmetic:
  "term 1 refers to term 2, but a subterm must come before the terms that
  use it" instead of "Term index not decreasing: … 2 >= 1". The one test
  that pinned an old message changed with it.

## Deviations from the prompt

- `--format` does not register `latex`, `typst`, `svg`, `rocq` and `net`;
  the prompt allowed "or leave a clear extension point", and that is what
  exists (above).
- `Outcome` gained a field, `mode`. It is `#[non_exhaustive]`, so this is
  not a breaking change, and the JSON is only re-checkable when it carries
  the mode.
- No fresh-context reviewer ran. This step changes no checker, criterion or
  prune: `describe` only formats, and the checker's verdicts are untouched
  (its tests pass unchanged, and the new message test asserts the formula
  rendering of an existing error).
- No decision in `plan/README.md` turned out wrong. D11 held: `core` still
  has no clock and spawns no thread.

## Open questions and follow-ups

- **Rendering very large derivations is slow**, and this is not a stack
  problem: every line of the text tree is as wide as the conclusion, so a
  3000-level chain produces hundreds of megabytes of text and did not finish
  within two minutes in a debug build. `--quiet` avoids it. A width limit or
  a different layout for huge proofs is a question for the export steps.
- **`check` could suggest the flag** when the problem is `Forbidden` ("the
  mode forbids the rule; was the proof found with --mix?"), or fall back to
  the file's `mode` key when no mode flag is given. Neither was done,
  because the prompt fixes the mode to the flags.
- **`--quiet --format json`** could drop the proof from the JSON, for batch
  runs that want verdicts only. That needs an option on the outcome's
  serialization, and nobody has asked for it.
- **The time is not in the JSON**; step 13's harness measures its own.
- `NoEngine`'s message does not list what does have an engine; it would go
  stale with every step that adds one.

## For later steps

- Step 5 (proof nets): `--format net` is a `Format` variant (above). The
  net's text rendering belongs in `core`; the CLI only picks it.
- Step 7: `--copies` is waiting, hidden. `Error::NoEngine` for exponentials
  becomes a dispatch row, and the CLI needs no change beyond `--copies`.
- Step 8: `check -i` is refused in the CLI before the checker says
  `Problem::Intuitionistic`; remove that refusal when the checker handles
  intuitionistic mode.
- Steps 9 to 11: each format is a `Format` variant, built on the search
  thread if it recurses over the derivation.
- Step 13: `--stats` gives stable sequents, memo hits, memo peak, splits and
  time; `--format json` gives the counts without the time.

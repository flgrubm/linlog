# Step 4: the library API and the CLI

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item (clap shows them as `--help`), the verification table, no pushing.
Read before you start:

- `plan/README.md` (D8, D9, D11 and the review protocol) and every report in
  `plan/reports/`.
- `cli/src/**` (the subcommand tree exists but `main.rs` does not dispatch),
  `core/src/lib.rs` and the `search` and `proofs` modules' public items.
- `.claude/rules/core.md`.

## What step 3 left you

`plan/reports/03-focused-engine.md`, "The public API" and "For step 4":
`prove` and `prove_until(&sequent, mode, &options, stop)` where `stop` is
polled once per stable sequent (the deadline closure and a Ctrl-C flag go
there; `core` has no clock); `Options` with private fields and setters
(`memo_limit`, `recursion_limit`, `engine`, `fragment`); `Outcome { verdict,
fragment, engine, statistics }`; `Verdict::{Proved(Box<Proof>), Unprovable,
Unknown(Reason)}`; `Reason` and `Engine` with `Display`;
`Error::{NoEngine, FragmentMismatch}` as the user-facing refusals. The
engine recurses on the calling thread: one level costs under 2 KiB in
debug and under 512 bytes in release, and the default recursion limit of
2048 fits an 8 MiB stack, so spawn the search thread with a stack sized
from `--recursion-limit` when it exceeds the default, and run the
derivation builder on the same thread.

## Goal

A CLI and a library surface that a person understands at first read:
`linlog prove` decides sequents through the engines that exist (only the
focused engine so far; later steps add rows to the dispatch and output
formats without touching the CLI's shape), and the existing `seq` commands
work. The user's stated priorities: clear and easy to understand for humans,
then performance.

## What to build

1. **Library surface.** Review what `core` exports and make the common path
   short and documented with examples in `lib.rs`'s crate docs: parse a
   sequent, detect its fragment, prove it with default options, print the
   derivation, serialize the proof. Builder-style `Options` with sensible
   defaults; `Mode` constructed from named flags. Errors through the crate's
   `Error` with `thiserror`, messages a user can act on. Nothing in `core`
   uses threads, the clock or the OS unconditionally (D11): the CLI supplies
   the deadline closure and, if the engine needs a big stack, spawns the
   search thread.
2. **CLI shape.** Redesign the clap tree in `argument_parsing.rs` so it
   reads well; the current `seq pretty|serialize prose|json` split into
   input-format subcommands is not it. A shape to start from (change it if
   you find better, and say why in the report):
   - `linlog prove [SEQUENT] [--file PATH]` with input from the argument, a
     file or stdin; `--json-input` to read a serialized sequent instead of
     prose.
   - Mode flags: `--intuitionistic`/`-i`, `--affine`/`-a`, `--mix`.
   - Overrides: `--fragment <mll|mll1|mall|mell|ll|…>` asserts that the
     input lies in the fragment (error otherwise) and uses that row of the
     dispatch table; `--engine <focus|net|additive|auto>` forces an engine.
   - Limits: `--copies N` (the copy bound of step 7; accept it now and
     reject it with a clear message until step 7 lands, or hide it until
     then), `--timeout DURATION` (a human duration; find a small,
     well-maintained crate or parse `10s`/`2m` yourself), `--memo-limit`.
   - Output: `--format <text|json|latex|typst|svg|rocq|net>` where only
     `text` and `json` exist now (the others are steps 5, 9, 10 and 11;
     register the names or leave a clear extension point), `--output PATH`,
     `--quiet` for the verdict only, `--stats` for the statistics, and a
     non-zero exit code for `Unprovable` and another for `Unknown` so scripts
     can branch.
   - `linlog seq print|json` (rename as you see fit): parse and pretty-print
     or serialize a sequent; `linlog seq fragment` prints the detected
     fragment; keep `--optimize` only if it still means something.
   - `linlog check PROOF` runs the checker on a serialized proof.
     Deserialization does not run the checker (the file carries no mode),
     so this command takes the mode flags and calls `Proof::check`; a
     `CheckError` prints with occurrence ids only, so add the method step
     2's report suggests that renders it with formulas through the forest,
     and use it here and in `prove`.
   The `--help` text is the documentation: every argument's doc comment says
   what it does and, where not obvious, why one would use it.
3. **Output text.** The default output for `Proved` is the derivation
   rendering of step 2 (`proof.derivation()?.to_string()`) preceded by one
   line with the verdict, the fragment detected and the engine used (so a
   user sees the auto-detection at work). The derivation builder recurses
   to the derivation's height and unfolds shared subproofs into a tree, so
   run it on the same large-stack thread as the search and let `--quiet`
   skip it;
   `Unprovable` and `Unknown` say why in one line. `--json` output carries
   the outcome, the fragment, the engine, the statistics and the proof.
   `Fragment` and `Mode` have no serde yet (step 1's report): give them a
   wire form here, readable names rather than raw flags (`"MALL"`,
   `{"intuitionistic": false, "affine": true, "mix": false}` or similar),
   behind the `serialize` feature in `core`, and pin it in
   `core/tests/serialize.rs` like the sequent format.
4. **Tests.** CLI integration tests (`cli/tests/`) that run the binary on a
   few sequents and check verdicts, exit codes and that `--help` mentions
   every subcommand; use a test crate the ecosystem uses for this if it
   earns its place (`assert_cmd` is the usual one; adopt through the
   `new-tool` skill, as a dev-dependency), else `std::process::Command`.
5. **Documentation.** README: a usage section with real commands and their
   output. CLAUDE.md: the CLI paragraph (`argument_parsing.rs` no longer
   "not dispatched"), the `launch` command if its meaning changed.
   `.claude/rules/core.md` if the API surface moved.

## Constraints

- No `unsafe`; a dependency only where it earns its place, scoped to `cli`
  where only the CLI needs it.
- Keep `core`'s API free of CLI concepts (no clap types, no exit codes).
- Do not implement engines or exports; wire what exists and leave clean
  extension points for steps 5 to 11.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo deny check` if a dependency changed,
`nix flake check` at the end (`jj st` first), and `cargo run -p linlog-cli --
prove "A, A -o B |- B"` and a few more by hand, pasted into the report.

## Deliverables

- Thematic jj commits ("Redesign the CLI around prove", "Print the
  derivation and the statistics", "Test the CLI end to end", "Document
  usage", …).
- `plan/reports/04-api-and-cli.md`: the final CLI tree with one example per
  subcommand, the library entry points, what a later step must do to add an
  engine or an output format, decisions, deviations, open questions.

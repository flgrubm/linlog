---
paths:
  - "cli/**"
---

# linlog-cli: the `linlog` command

Loaded when a file under `cli/` is read. The package is `linlog-cli`, the
binary `linlog` (`[[bin]]` in `cli/Cargo.toml`; `meta.mainProgram` in
`modules/workspace.nix` so that `nix run` finds it).

## Layout

- `argument_parsing.rs`: the clap tree. Doc comments are the `--help`
  text: the first paragraph is the short help (no trailing period, clap's
  style), later paragraphs the long help. The sequent syntax is one
  `SYNTAX` string shown after the help of every command that reads a
  sequent. `Cli::command().debug_assert()` in its tests catches
  inconsistent definitions.
- `lib.rs`: the package's library `linlog_cli`, which holds everything
  (public modules, so rustdoc documents them next to the core crate under
  a name of their own): dispatch, the exit status, the Ctrl-C flag, and
  the parse error with a caret under the failing character. `main.rs` is
  one call into it and is `doc = false`: the binary is named `linlog` like
  the core crate, and documenting it would overwrite the library's docs.
- `io.rs`: input from the argument, `--file` (`-` is standard input) or
  standard input, refused when standard input is a terminal; output to
  `--output` or standard output.
- `prove.rs`: `prove` and `check`, and the `net` format's refusal of
  sequents outside unit-free MLL and of affine or intuitionistic mode
  (`nets_exist`), before the search runs. `--copies` (default
  `Options::DEFAULT_COPIES`) is the per-branch copy bound of the focused
  engine's iterative deepening; `unknown … the copy bound of N was reached`
  is exit status 3 like every other unknown. `--format net` prints the net the
  net engine found (`Outcome::net`) and otherwise the net read off the
  proof; `--stats` prints the counters of the engine that ran
  (`statistics`, one arm per engine with its own counters).

## Invariants

- **Exit status**: 0 proved, valid or done; 1 unprovable or invalid; 2 an
  error, the same status clap uses for bad arguments; 3 unknown. Scripts
  depend on it, and `cli/tests/cli.rs` pins it.
- **The search runs on its own thread** (`on_large_stack`) with a stack of
  `recursion_limit × STACK_PER_LEVEL`, at least 8 MiB. `STACK_PER_LEVEL` is
  twice the engine's measured cost per level (4 KiB unoptimized, 1 KiB
  optimized), because the derivation builder and its renderer, which also
  recurse to the proof's height, run on the same thread. A 1000-level
  `⊗` chain renders on it without overflow in a debug build.
- **The stop closure looks at the clock and the Ctrl-C flag every 1024
  polls** (`POLLS_PER_CLOCK`): the engine polls once per stable sequent,
  a few million times a second, and reading the clock every time would cost
  a noticeable share. The CLI knows why the search stopped (`Stop`), so the
  verdict line says "the time limit of 10s was reached" or "interrupted"
  instead of `Reason::Stopped`'s generic phrase.
- **Ctrl-C** (`ctrlc`, whose handler runs on a thread of its own once per
  signal): the first sets a flag the search polls, so the outcome is
  unknown and `--stats` still prints; the second exits with 130. The
  handler is installed by `prove` only.
- **JSON output is core's `Outcome` serialization**, unchanged; `check`
  reads it as a `Proof` because the proof's keys are flattened into it.
  The time is not in the JSON (core has no clock, and the output stays
  reproducible); `--stats` prints it as text.
- `check` takes the mode from its flags, never from the file's `mode` key.
  A mode the checker refuses (`Problem::Intuitionistic`) is an error, exit
  2, not an invalid proof; drop that refusal once the checker handles the
  mode.

## Extension points

- **An engine**: a variant of `EngineArg` with a doc comment (its `--help`
  line) and its arm in `From<EngineArg> for Option<Engine>`. The verdict
  line prints `Engine`'s `Display`; `statistics` in `prove.rs` gets an arm
  only if the engine has counters of its own, as the net engine has.
- **An output format** (`latex`, `typst`, `svg`, `rocq`): a variant of
  `Format` and its arm in `prove`'s and `check_text`'s `match format`, as
  `net` has. A format that renders the derivation builds it inside the
  `on_large_stack` closure, as the text format does; `net` builds the net
  there too, though desequentialization does not recurse.
- **A new `Reason`**: its arm in `verdict_line`, which turns a generic
  phrase into advice (`RecursionLimit` and `CopyBound` name the flag to
  raise); the default arm prints `Reason`'s `Display`.
- Stay out of `core`'s way: no clap types or exit statuses in `core`, and
  the CLI never re-implements what `core` computes (fragment names, the
  mode's words, the JSON form).

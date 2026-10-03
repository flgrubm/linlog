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
- `interact.rs`: `interact`, a line-based session over `Interactive`:
  the state comes from the sequent argument or `--state FILE` (a session
  `save` wrote; the mode is then the file's), the commands from standard
  input (`goals`, `rules`, `apply`, `undo`, `close`, `show [latex|typst|svg]`,
  `proof`, `save`, `load`, `help`, `quit`; `HELP` is the list; `show`
  with a format prints the export fragment of the partial derivation, or
  its SVG document),
  every command's output or `error: …` goes to standard output and the
  session goes on,
  and the whole loop runs inside `on_large_stack` so that `close` and the
  derivation drawing have the stack `prove` has. `goal_line` prints a goal
  with the position of every formula, two-sided under the reading.
- `prove.rs`: `prove` and `check`, and the `net` and `net-svg` formats'
  refusal of
  sequents outside unit-free MLL and of affine mode (`nets_exist`), before
  the search runs; in intuitionistic mode the net printed is the one of
  the one-sided sequent. `--copies` (default
  `Options::DEFAULT_COPIES`) is the per-branch copy bound of the focused
  engine's iterative deepening; `unknown … the copy bound of N was reached`
  is exit status 3 like every other unknown. `--bias auto|rarer|factors`
  (`BiasArg`, on `prove` and `interact`) is `Options::bias`: how the
  focused engines pick each atom's positive literal, never what is
  provable; `auto` on a sequent with exponentials runs a search under
  each rule, and `--forward-copies` (default
  `Options::DEFAULT_FORWARD_COPIES`, on both commands too) is
  `Options::forward_copies`, the forward search's own copy bound on
  Horn programs. A test that pins a copy bound's message sets both
  bounds, or names a bias. `--format net` prints the net the
  net engine found (`Outcome::net`) and otherwise the net read off the
  proof; `--stats` prints the counters of the engine that ran
  (`statistics`, one arm per engine with its own counters). `--format
  latex` and `--format typst` print the verdict line and the statistics
  as comments of the target (`note`) and the derivation through
  `linlog::export` (`derivation(proof, mode, format, form)`, which also
  draws the text tree); `--standalone` makes the derivation a document and
  is refused, exit 2, for the other formats (`form`). `--format svg`
  draws the derivation and `--format net-svg` the net (`net_in`, the
  same net `net` prints) through `linlog::export::svg` with the default
  `Style`, the verdict and statistics as XML comments (`note`, which
  turns every `-` into `‐`, since a comment cannot hold `--` and the
  advice names flags). `--standalone` is refused for them too: an SVG is
  always a whole document, so the flag would do nothing. An unprovable
  sequent prints the comment alone, which is not an XML document; the
  exit status says why. `--format rocq` prints the verdict and statistics
  as `(* … *)` comments and the derivation as `linlog::export::rocq`
  writes it with the default `Options` (lemma `certificate`), taking
  `--standalone` for the file with the import; in intuitionistic mode the
  two-sided derivation is passed and certified one-sided; a proof with
  Mix or affine weakening is exit 2 with the library's `Unsupported`
  message, after the search. `seq print --format` (`SequentFormat`, with
  `svg`) prints through `sequent_in`, which `sequent_text` wraps. There
  is no `net` subcommand: the CLI's nets are those of proofs, which
  `prove` and `check` draw.

## Invariants

- **Exit status**: 0 proved, valid or done; 1 unprovable or invalid; 2 an
  error, the same status clap uses for bad arguments; 3 unknown. Scripts
  depend on it, and `cli/tests/cli.rs` pins it.
- **The search runs on its own thread** (`on_large_stack`) with the stack
  core's `Options::stack_size` computes for the recursion limit: twice
  the engine's measured cost per level (4 KiB unoptimized, 1 KiB
  optimized), at least 8 MiB, because the derivation builder and its
  renderer, which also recurse to the proof's height, run on the same
  thread. A 1000-level `⊗` chain renders on it without overflow in a
  debug build. The parallel search sizes its pool's threads by the same
  function, so the CLI computes no stack size of its own.
- **The stop closure looks at the clock and the Ctrl-C flag every 1024
  polls on one thread** (`POLLS_PER_CLOCK`, `polls_per_clock`): the
  engine polls once per stable sequent, a few million times a second,
  and reading the clock every time would cost a noticeable share. With
  several threads core's driver polls the closure once a millisecond on
  the calling thread and every poll looks, or a time limit would be off
  by seconds. The CLI knows why the search stopped (`Stop`), so the
  verdict line says "the time limit of 10s was reached" or "interrupted"
  instead of `Reason::Stopped`'s generic phrase.
- **`--jobs` defaults to the machine's parallelism** (`default_jobs`) and
  `--deterministic` overrides it with one thread, on `prove` and
  `interact`: the sequential engines are what a pinned output (a test's
  `--stats` counts, a proof compared across runs) needs, since a parallel
  run's counts add every thread's and its proof is the first found. The
  CLI enables core's `parallel` feature in `cli/Cargo.toml`.
- **Every proof reported has passed the checker**: the library checks it
  before `prove_until` returns (`Options::check`), so `--quiet` and
  `--format json` are checked like the drawn formats; `--no-check`
  switches it off. A proof the checker rejects is `Error::Rejected`, exit
  status 2: a defect to report, not a verdict.
- **Ctrl-C** (`ctrlc`, whose handler runs on a thread of its own once per
  signal): the first sets a flag the search polls, so the outcome is
  unknown and `--stats` still prints; the second exits with 130. The
  handler is installed by `prove` only.
- **JSON output is core's `Outcome` serialization**, unchanged; `check`
  reads it as a `Proof` because the proof's keys are flattened into it.
  The time is not in the JSON (core has no clock, and the output stays
  reproducible); `--stats` prints it as text.
- `check` takes the mode from its flags, never from the file's `mode` key.
- **`interact` needs the sequent as an argument or `--file`**: standard
  input carries the commands, so the fallback to standard input that
  `SequentInput` gives the other commands is refused with a message. Its
  exit status is 0 only when the session ends with a finished proof that
  checks (`Interactive::proof`), 1 otherwise; a refused command is not an
  error of the session. Rule names on the command line are what
  `Rule::from_str` accepts: the usual spellings and ASCII ones (`*`,
  `par`, `+1`, `-oL`, `&L1`, …). `close` clears the Ctrl-C flag first
  (`clear_interrupt`), since a stopped search must not stop the next one.
- **Intuitionistic mode is a matter of presentation in the CLI**: the
  verdict line and the JSON name the fragment through
  `Fragment::name_in(mode)` (`IMLL`, `ILL`, …); `prove -i` and `check -i`
  print the two-sided derivation (`Proof::two_sided_derivation`) and
  `check -i` the sequent two-sided (`sequent_text`); `seq print -i` and
  `seq fragment -i` do the same for a bare sequent. A sequent with no
  intuitionistic reading is exit 2 with core's `ShapeError` described with
  formulas (`describe` in `prove.rs`, which rebuilds the forest for that);
  a classical proof file checked with `-i` is an invalid proof, exit 1.

## Extension points

- **An engine**: a variant of `EngineArg` with a doc comment (its `--help`
  line) and its arm in `From<EngineArg> for Option<Engine>`. The verdict
  line prints `Engine`'s `Display`; `statistics` in `prove.rs` gets an arm
  only if the engine has counters of its own, as the net and additive
  engines have (`two-sided` shares the focus engine's).
- **An `interact` command**: an arm in `Session::command`, a line in
  `HELP`, and the session test in `cli/tests/cli.rs`, which pins the
  exact output of a scripted session.
- **An output format** (`lean`, say): a variant of `Format` and its
  arm in `prove`'s and `check_text`'s `match format`, as `net`, `latex`,
  `typst` and `rocq` have. A format that renders the derivation builds it inside
  the `on_large_stack` closure, as the text format does (an arm in
  `derivation`); `net` builds the net there too, though
  desequentialization does not recurse. A format with a document form
  takes `--standalone` (`form` lists which formats have one), a net
  format builds the net in `net_in`, and one
  whose output is a source file writes the verdict as its comment
  (`note`), so that the output still compiles.
- **A new `Reason`**: its arm in `verdict_line`, which turns a generic
  phrase into advice (`RecursionLimit` and `CopyBound` name the flag to
  raise); the default arm prints `Reason`'s `Display`.
- Stay out of `core`'s way: no clap types or exit statuses in `core`, and
  the CLI never re-implements what `core` computes (fragment names, the
  mode's words, the JSON form).

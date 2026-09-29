---
paths:
  - "bench/**"
---

# linlog-bench: the benchmark harness

Loaded when a file under `bench/` is read. The package is `linlog-bench`,
the binary `linlog-bench` (`doc = false`: a command, not an API). It
depends on core with `parse` and `parallel` only, and adds no dependency
beyond clap and anyhow, which the CLI already has.

## Layout

- `src/main.rs`: the clap tree (`run`, `summary`, `families`, and the
  hidden `one`); doc comments are the `--help` text, as in the CLI.
- `src/problems.rs`: the three sources. The parent lists `Reference`s
  (family and size, an LLTP path, a problem file and line) and never
  parses a problem file's formulas; the child `load`s the one its `id`
  names. An LLTP problem is intuitionistic when a component of its path
  is `ILL` (the library's layout), classical otherwise; its expected
  verdict is its header's `Status`, and a run in another mode than the
  problem's own (`--modes classical` on an ILL problem) has none, since
  the verdict may differ there. A problem file's line is `name; mode;
  expected; copies; sequent`; the name's part before `/` is the family.
- `src/run.rs`: the parent (`run`) and the child (`one`). One child
  process per run: the child loads the problem, builds the forest (for
  `occurrences` and `multiplicity`), times `prove_until` alone with a
  deadline in its stop closure (the clock read every 64 polls on one
  thread, every poll on a pool, as the CLI does), checks the proof outside
  the timed part, and prints the 16-field tail of the CSV row. The parent
  kills a child that outlives its limit by a tenth plus five seconds and
  writes the row itself (`reason` `killed`, or `crash (status): <last
  stderr line>`).
- `src/summary.rs`: Markdown from CSV. A problem counts once per
  configuration (family, mode, requested engine, jobs, portfolio, test
  period): the first run's verdict, the median of the runs' times.
  `refused` rows (a forced engine that does not apply) are dropped.
- `problems/slow-tests.txt`: problems of the engine reports' timing tables
  that no family generates (the Partition instances of the net engine's
  first table, the chain with a token over, the parallel cancellation
  case). `baseline.sh`: the command that regenerates `results/*.csv` and
  `RESULTS.md`.

## Invariants

- **The CSV columns are the interface** (`run::HEADER`): `summary` reads
  them by name, and the committed `results/*.csv` are what a later change
  is compared with, so add columns at the end of the tail and never
  rename one. Fields never contain commas (`clean` turns them into `;`),
  so the files are split on commas without quoting.
- **`verdict` and `reason`**: `proved`, `unprovable`, `unknown` (reasons
  `timeout`, `copy_bound`, `context_too_wide`, `recursion_limit`,
  `killed`, `crash …`), `refused` (`NetFragment`, `NetMode`, `EngineMode`,
  `NotAdditive`, `IntuitionisticMix`: the configuration does not apply)
  and `error` (every other `Error`, a parse failure, a missing reading;
  these are findings, not configurations). `checked` is `ok` or the
  checker's message for a proof of the roots.
- **`time_ms` is the search alone**: `prove_until`, forest construction
  and pool start-up included, parsing, the proof check and process start
  excluded. `nodes` on a parallel run is the sum over the threads.
- **Timings mean something only from `baseline.sh`** on an otherwise idle
  machine: the sequential streams are pinned to performance cores with
  `taskset`, the parallel runs run alone. The `bench` flake check runs the
  harness for its verdicts only (a `MISMATCH` fails it), never for times.
- **A mismatch is a verdict against the known one.** For a generated
  family it is a bug in an engine or in the family's construction; for an
  LLTP problem it may be the header (two KLE headers contradict their
  problems; see the step 14 report). Keep the families' claims derived
  from the combinatorial problem, never from an engine.
- **The LLTP library is not in the repository** (GPL-3.0): `nix build
  .#lltp -o bench/lltp` fetches it, `bench/lltp` is ignored.
- **`baseline.sh` resumes**: every `run` appends with `--resume`, which
  skips a problem and configuration the CSV already has, so rerunning the
  script finishes an interrupted baseline and reruns only rows deleted
  from the files; `rm -rf bench/results` starts over. It caps every
  process at 16 GiB of address space (`prlimit`), since the additive
  path's memo is unbounded (depth 18 of the `additive` family needs about
  130 GB) and a child that swaps slows every other stream.

## Extension points

- **A family**: an entry of `linlog::families::FAMILIES` (name, summary,
  default sizes, instances per size, generator) with its verdict proved
  by construction; `verdicts_as_constructed` then covers its smallest
  size. Pick the default sizes so that the largest one times out at the
  baseline's limit and the others do not.
- **A problem source**: a function in `problems.rs` that lists
  `Reference`s and an arm of `load`.
- **A configuration axis** (a new `Options` knob): a `RunArgs` flag, its
  argument in `child`, a `OneArgs` field applied in `tail`, a column of
  the parent's prefix, and the label in `summary`'s `config`.

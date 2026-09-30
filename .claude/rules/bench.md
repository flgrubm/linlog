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
  configuration (CSV file, family, mode, requested engine, jobs, portfolio, test
  period): the first run's verdict, the median of the runs' times.
  `refused` rows (a forced engine that does not apply) are dropped. The
  CSV file's name is part of the configuration's label, since each file
  of the baseline is one `run` with options of its own (a copy bound, a
  recursion limit, a longer time limit) that the columns alone do not
  tell apart.
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
- **`time_ms` is wall-clock time of the search alone** (`Instant` around
  `prove_until`): forest construction and pool start-up included, parsing,
  the proof check and process start excluded; the time limit is
  wall-clock too. `cpu_ms` is the process's CPU time over the same span
  (`/proc/self/stat`, all threads, in 10 ms ticks), and `wait_ms` the
  time the calling thread was ready but waited for a CPU
  (`/proc/thread-self/schedstat`, nanoseconds): on a sequential run a
  `wait_ms` well above zero says another process slowed it down, which is
  how a run on a shared machine shows itself; on a parallel run the
  calling thread mostly sleeps and its wait says little. `nodes` on a
  parallel run is the sum over the threads.
- **Timings mean something only from `baseline.sh`** on an otherwise idle
  machine: the sequential streams are pinned to performance cores with
  `taskset`, the parallel runs run alone. The `bench` flake check runs the
  harness for its verdicts only (a `MISMATCH` fails it), never for times.
- **A mismatch is a verdict against the known one.** For a generated
  family it is a bug in an engine or in the family's construction; for an
  LLTP problem it may be the header (the intuitionistic headers of
  KLE013 and KLE065 of `KLE-cbn` and of SYN001 contradict their problems;
  see the step 14 report). Keep the families' claims derived
  from the combinatorial problem, never from an engine.
- **The LLTP library is not in the repository** (GPL-3.0): `nix build
  .#lltp -o bench/lltp` fetches it, `bench/lltp` is ignored.
- **`baseline.sh`** is meant to run as `bench/baseline.sh --detach
  --fresh`: `--detach` starts it as the systemd user unit
  `linlog-baseline` (24 GiB and no swap for the unit, `OOMPolicy=continue`
  so the kernel kills a runaway child alone, no core dumps, its own
  target directory `target/baseline`), because a session that dies takes
  its terminal's processes with it (a reviewer's scratch program once ran
  the machine out of memory and systemd failed the whole terminal scope,
  the baseline with it). It refuses to start when the load average is
  above 1 or a scheduled job other than the trivial ones is due within nine
  hours (`nix-gc` at midnight, `nix-optimise` before four, backups), unless
  given `--force`; RESULTS.md records the starting load and the jobs that
  fired, and the journal gets every stream's last progress line (with the
  harness's estimate of the time left), the load and the other processes
  using a CPU every ten minutes. Every `run` appends with `--resume`, so
  rerunning finishes an interrupted baseline; `--fresh` deletes
  `bench/results` first. Each process is capped at 12 GiB of address
  space (`prlimit`): the additive path's memo is unbounded (depth 18 of
  the `additive` family needs about 130 GB) and parsing the library's
  largest files takes over 15 GB, and two such processes at once stay
  within the unit's limit; the classical LLTP pass runs `--reverse` so
  that the two LLTP passes do not parse those files at the same time.
- **The machine** (an Intel Core Ultra X9 388H laptop, host `wired`, its
  configuration in the author's system flake): 16 physical cores and no
  SMT, of three kinds: CPUs 0–3 performance cores (5.1 GHz, own L2),
  4–11 efficiency cores (4.0 GHz, two clusters of four sharing an L2),
  12–15 low-power efficiency cores (3.7 GHz, shared L2 and no share of
  the L3). The sequential streams are pinned to 0–3; "every core" in the
  parallel stage includes the slow low-power ones, so speedups flatten
  above eight threads for a reason of the hardware. It throttles
  thermally under long loads (the package counter is recorded per run
  and in the journal), turbo stays on (off, the night would not fit),
  TLP's power profile is `performance` on mains. A detached run keeps the
  user's other slices (`app.slice`, `session.slice`, `background.slice`)
  on CPUs 4–15 for its whole duration and gives them back in the unit's
  `ExecStopPost` (`baseline.sh --unshield`), since a stopped unit's
  processes can be killed before a trap of theirs runs; it holds a
  `sleep:idle:handle-lid-switch` inhibitor (logind suspends on a closed
  lid even on mains here, the idle manager on battery) and refuses to
  start on battery. The system's own services and kernel threads it
  cannot move; `sudo systemctl set-property --runtime system.slice
  AllowedCPUs=4-15` (and `init.scope`) does, until `AllowedCPUs=` or a
  reboot. The summary marks with `†` a sequential problem whose run
  waited for a CPU for over 1 % of its time.
- **Scratch programs next to a running baseline** (a reviewer's checker, an
  exploratory run) go in a scope of their own, `systemd-run --user
  --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 4-15 …`, off the
  performance cores the sequential streams are pinned to, and nothing
  heavy runs during the parallel stage.

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

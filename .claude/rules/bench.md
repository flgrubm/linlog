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
  kills a child that outlives its limit by `--grace` seconds (by default a
  tenth of the limit and five) and writes the row itself (`reason` `killed`, or `crash (status): <last
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
  case). `baseline.sh`: the command that takes a baseline into
  `results/DAY/` (`*.csv`, `starts.txt`, `RESULTS.md`) and copies its
  tables to `bench/RESULTS.md`.

## Invariants

- **The CSV columns are the interface** (`run::HEADER`): `summary` reads
  them by name, and the committed `results/*/*.csv` are what a later change
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
  `taskset`, the parallel runs run alone. Four streams at once cost a
  stream 3–11 % against the same run alone (shared L3, heat), the same
  for every sequential row, which is why stage 3 takes its own one-thread
  rows. The kill at the limit plus the grace counts from
  the child's start, parse included, so a `killed` row on a file of
  megabytes may be its parse; on a small file it is a search that did
  not stop. The `bench` flake check runs the
  harness for its verdicts only (a `MISMATCH` fails it), never for times.
- **A mismatch is a verdict against the known one.** For a generated
  family it is a bug in an engine or in the family's construction; for an
  LLTP problem it may be the header (the intuitionistic headers of
  KLE013 and KLE065 of `KLE-cbn` and of SYN001 contradict their problems;
  see the step 14 report). Keep the families' claims derived
  from the combinatorial problem, never from an engine.
- **The LLTP library is not in the repository** (GPL-3.0): `nix build
  .#lltp -o bench/lltp` fetches it, `bench/lltp` is ignored.
- **`baseline.sh`** is meant to run as `bench/baseline.sh --arm
  --fresh` in the slot the machine is the benchmark's (`slot`, 20:00 to
  07:00, `--slot=HH:MM-HH:MM` for another). `--arm` sets two transient
  user timers: `linlog-baseline.timer` starts the unit at the slot's
  start, or at once inside it, and `linlog-baseline-stop.timer` stops
  it at the slot's end whatever its state (`ExecStopPost` then gives the
  cores back and starts again the user timers the unit stopped for the
  run, such as `obsidian-snapshot`, listed in
  `$XDG_RUNTIME_DIR/linlog-baseline-timers`; the inhibitor dies with the
  unit). The timers do not survive a reboot or the end of the user's
  session manager: arm after the last reboot, and look at `systemctl
  --user list-timers` before leaving the machine. System timers and
  services need root: the author stops them. The unit waits for an
  idle machine (on mains, a load average of at most 1) until the slot's
  end minus `estimate` (10 h 30 min), then starts regardless and says so in
  `starts.txt`: a night not used is worse than rows marked as disturbed.
  `--detach` starts the unit at once and never stops it. The unit,
  `linlog-baseline`, runs the script with `--force` (40 GiB and no swap,
  `OOMPolicy=continue` so the kernel kills a runaway child alone, no core
  dumps, its own target directory `target/baseline`), because a session
  that dies takes its terminal's processes with it (a reviewer's scratch
  program once ran the machine out of memory and systemd failed the
  whole terminal scope, the baseline with it). Run by hand, the script
  refuses to start on battery, when the load average is above 1 or when
  a scheduled job other than the trivial ones is due within the estimate
  (`nix-gc` at midnight, `nix-optimise` before four, `obsidian-snapshot`
  at 23:00, backups), unless given `--force`; `--arm` names the jobs due
  in the slot instead. The journal gets every stream's last progress
  line (with the harness's estimate of the time left), the load and the
  other processes using a CPU every ten minutes. Each process is capped
  at 12 GiB of address space (`prlimit`; 16 and 32 GiB in stage 4): the
  check of a child's proof takes a bitset of the forest's width per proof
  node (7.6 GB at depth 16 of the `additive` family, about 130 GB at
  depth 18; the additive search itself stays under 0.1 GB since its memo
  has a cap), and three such processes at once stay within the unit's
  limit. Loading is cheap by comparison: the library's largest
  file (103 MB, 30 million occurrences) loads in 16 s with 2 GB. The
  classical LLTP pass runs `--reverse` so that the two LLTP passes do not
  load those files at the same time.
- **Stage 4 reruns what more room lets finish**, from `bench/reruns.txt`
  (lines `FILE FAMILY/NAME`: the CSV file of the run that was killed or
  crashed, and the problem), into `FILE-generous.csv`, with `--grace`
  600 and 16 GiB on one thread and `--grace` 60 and 32 GiB on every
  core. The list is chosen from measurements, not from the rows: its
  header says how, and a later baseline reruns the same list so that the
  two compare. The grace lets a search that misses its stop run on, so
  a rerun's `time_ms` can exceed its limit many times over, and
  `summary` counts a verdict found that late as solved (two Petri nets
  proved after 21.6 s and 552 s under 5 s in the first baseline):
  compare these files by verdict, reason and time. `--only` matches
  `FAMILY/NAME`, which names one translation of an LLTP problem exactly
  (the three translations share file names).
- **The library is repaired in one byte**: the flake's `lltp` package
  turns the only tab in the library, in `ILL/ILLTP-SYJ-01/SYJ206+1.018.p`,
  into the closing parenthesis it replaced, so that every file of the
  library loads. Every header reads: 111 ILLTP-SYJ problems say
  `Unsolved` and have no expected verdict, and 25 files contradict their
  problems (23 headers, and two more translations of one of them; see
  the step 14 report).
- **Every baseline keeps a directory of its own**, `results/DAY/`, DAY
  the day it started, so that two baselines (before and after a
  performance pass) sit side by side. A run without `--fresh` resumes
  the latest directory that has no `RESULTS.md` (a baseline not
  finished): every `run` appends with `--resume` and skips the
  configurations its CSV file has, so the script run again on another
  night finishes a baseline the slot's end stopped. `--fresh` deletes
  the day's directory only. Every start appends a line to `starts.txt`:
  the time, the commit the binary is built from (`@-`, and the files `@`
  changes outside the results, read with `--ignore-working-copy` since a
  snapshot from the unit would sign a commit), the load and whether the
  start was forced. The finished run writes `RESULTS.md` into the
  directory, its header listing those lines, and copies it to
  `bench/RESULTS.md`, which is always the latest baseline's. Two
  baselines compare only under the same script, slot and settings, and a
  resumed one only if every start measured the same engines: the first
  baseline's second start names a later commit, which differs from the
  first in `plan/` and in the harness's kill and filter alone (`core/`,
  `cli/`, the Cargo files and the toolchain are identical).
- **The machine** (an Intel Core Ultra X9 388H laptop, host `wired`, its
  configuration in the author's system flake): 16 physical cores and no
  SMT, of three kinds: CPUs 0–3 performance cores (5.1 GHz, own L2),
  4–11 efficiency cores (4.0 GHz, two clusters of four sharing an L2),
  12–15 low-power efficiency cores (3.7 GHz, shared L2 and no share of
  the L3). The sequential streams are pinned to 0–3; "every core" in the
  parallel stage includes the slow low-power ones; on the families that
  scale, speedups still grew from eight threads to sixteen in the first
  baseline (the unsolvable 3-Partition 6.0× to 7.8×, the net engine's
  Partition table 6.3–7.0× to 11.8–13.6×). It throttles thermally under
  long loads (the package counter is recorded per run and in the
  journal; 11 283 s of throttling over the first baseline's 9 h 21 min,
  most of it in the all-core stage), turbo stays on (off, the night would not fit),
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

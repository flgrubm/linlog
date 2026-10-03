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
  `occurrences` and `multiplicity`), prints the line `loaded`, times
  `prove_until` alone with a deadline in its stop closure (a flag that a
  thread of its own raises at the limit, as the CLI's is: a clock read
  every 64 polls was seconds late on the large nets, where a poll comes
  many milliseconds after the last)
  and with the library's own check off (`Options::check(false)`), prints
  the 22-field tail of the CSV row with the search's verdict, and for a
  proof checks it outside the timed part and prints the tail once more
  with `checked` and `check_ms`. The parent takes the last tail. It kills
  a child that has not said `loaded` within `--load-limit` seconds
  (`DEFAULT_LOAD_LIMIT`, 120) or that outlives its time limit by `--grace`
  seconds (by default a tenth of the limit and five) *counted from that
  line*, so a file of a hundred megabytes gets its whole limit for the
  search. A child that dies leaves its last tail with `checked` saying how
  (`failed: killed`, `failed: crash (status): <line>`): it died in the
  check, and the verdict it printed stands. One that dies before any tail
  gets the parent's row (`reason` `killed`, `killed while loading`, or
  `crash (status): <line>`, the line being the last of its error output
  that is not a `note:`, so a panic's message and not the hint about
  backtraces; the whole error output goes to the parent's standard error,
  which is the run's log).
- `src/summary.rs`: Markdown from CSV. A problem counts once per
  configuration (CSV file, family, mode, requested engine, jobs, test
  period, and in a file from before the portfolio was removed its
  `portfolio` column, which `run` no longer writes): the first run's
  verdict, the median of the runs' times.
  `refused` rows (a forced engine that does not apply) are dropped. The
  CSV file's name is part of the configuration's label, since each file
  of the baseline is one `run` with options of its own (a copy bound, a
  recursion limit, a longer time limit) that the columns alone do not
  tell apart.
- `src/compare.rs`: `summary --before DIR` (every file against the file
  of the same name in an earlier baseline) and `summary --against FILE`
  (every other file against one of the same baseline, keyed without the
  file's name): the verdicts that differ, what the first decides and the
  second does not (with the outcomes of the files that have no
  counterpart beside it), a row per group (file, family, configuration)
  and the generated problems side by side. A verdict after the time
  limit counts as decided and late. `bench/COMPARISON.md` is its output
  for the two baselines, under a summary written by hand.
- `problems/slow-tests.txt`: problems of the engine reports' timing tables
  that no family generates (the Partition instances of the net engine's
  first table, the chain with a token over, the parallel cancellation
  case). `baseline.sh`: the command that takes a baseline into
  `results/DAY/` (`*.csv`, `starts.txt`, `RESULTS.md`) and copies its
  tables to `bench/RESULTS.md`. `targets.sh`: the target set of the
  focused engine's performance pass, into `targets/LABEL.csv` (below).

## Invariants

- **The CSV columns are the interface** (`run::HEADER`): `summary` reads
  them by name, and the committed `results/*/*.csv` are what a later change
  is compared with, so add columns at the end of the tail and never
  rename one (`bias` came after the first baseline: a file
  without it ran everything with `auto`, which is how `--resume` and
  `summary` read it; `forward_copies`, the last, came with the default
  bias that runs two searches on a sequent with exponentials: a file
  without it ran `auto` as the rarer-literal search alone, so `--resume`
  takes none of its rows as done). One column was taken out, between
  `jobs` and `test_period`: `portfolio`, with the option it recorded;
  the summaries read it where a file has it (`true` is part of the
  configuration's label), and `--resume` refuses a file whose header is
  not `HEADER`, since the rows it would append do not fit. Fields never contain commas (`clean` turns them into `;`),
  so the files are split on commas without quoting.
- **`verdict` and `reason`**: `proved`, `unprovable`, `unknown` (reasons
  `timeout`, `copy_bound`, `context_too_wide`, `recursion_limit`,
  `killed`, `crash …`), `refused` (`NetFragment`, `NetMode`, `EngineMode`,
  `NotAdditive`, `IntuitionisticMix`: the configuration does not apply)
  and `error` (every other `Error`, a parse failure, a missing reading;
  these are findings, not configurations). `checked` is `ok` or the
  checker's message for a proof of the roots, or `failed: …` with how the
  child died in the check; `check_ms` is the wall-clock time of the check
  alone.
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
  the end of the child's load (before: from its start, parse included,
  so that a `killed` row of the first two baselines on a file of
  megabytes may be its parse): a `killed` row is a search that did not
  stop, `killed while loading` a load past `--load-limit`. The `bench` flake check runs the
  harness for its verdicts only (a `MISMATCH` fails it), never for times.
- **A mismatch is a verdict against the known one.** For a generated
  family it is a bug in an engine or in the family's construction; for an
  LLTP problem it may be the header (the intuitionistic headers of
  KLE013 and KLE065 of `KLE-cbn` and of SYN001 contradict their problems;
  see the step 14 report, and the count of such files below). Keep the families' claims derived
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
  end minus `estimate` (10 h), then starts regardless and says so in
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
  classical LLTP pass runs `--reverse` so that it does not load those
  files at the same time as the passes in order. Stage 1 has the
  intuitionistic library under each bias alone (`lltp-rarer`, `--bias
  rarer`; `lltp-forward`, `--bias factors --copies 30`) in a stream each,
  the families after the default intuitionistic pass and the engines
  after the classical one. The later stages take their LLTP problems
  (`ended`) from the first baseline's intuitionistic pass (`reference`,
  `results/2026-09-30/`), not from the night's own, so that every row has
  its counterpart; a later baseline keeps that reference.
- **`wait_ms` says nothing on a default run with exponentials.** The
  default runs its second search on a thread of its own, and while that
  search runs alone the calling thread wakes every millisecond to poll
  the caller's stop; pinned to one core, each wake-up queues behind the
  running search, and `wait_ms` counts it. The default LLTP passes of the
  second baseline show 11.5 % of their time as waits (5 191 problems
  `†`), the passes under one bias 0.07 % and 0.14 %. Read a night's
  disturbance off the rows with one search.
- **A `crash` row on a large net is the check of a proof the search
  found.** In the second baseline 76 runs crashed with `memory allocation
  … failed`. 58 of them are 14 Petri nets of tens of thousands of
  transitions with a firing sequence of one step (BART, TokenRing-40 and
  -50, Philosophers-10000, GPPP-1000-1000 and others; 14 per default
  pass and in the forward pass, 3 under `--bias rarer`, 13 in
  `lltp-recursion`), which the first baseline ended at the recursion or
  width limit within milliseconds. The search now proves each in 50 ms
  to 2 s within 250 MB (`linlog prove --quiet --stats` on the sequent),
  and the child's check of the proof (`proofs::check::derive`, which
  keeps what every node derives, on a sequent of 65 000 `!` clauses a
  list of that length per node) fills the 12 GiB in a few seconds: the
  row says `unknown` where the search's answer was `proved`. The others
  are the search's own memory: four Philosophers-10000 nets under the
  recursion limit of 16 384 (the counts of a split, per level of a deep
  recursion), `qbf/48#0`, whose memo grows for 290 s, and the largest
  SYJ files on a pool, as before; the focused engine forced onto
  `additive/16` crashed in both baselines (which of the two it is was
  not looked into). To tell which a crash is, run
  the row alone with core dumps on and read the trace (`coredumpctl
  info`), or the sequent through the command with `--quiet`.
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
  `Unsolved` and have no expected verdict, and 28 files contradict their
  problems: the 25 of the first baseline (23 headers, and two more
  translations of one of them; see the step 14 report), and three that
  the second baseline's larger copy bounds refute (`lltp-copies-10`,
  `lltp-forward`): KLE069 in `KLE-01` and KLE078 and KLE086 in `KLE-cbn`,
  whose translated formulas have a classical countermodel (the
  translation lost a negation of the ILTP original).
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

## The target set

- **`bench/targets.sh LABEL`** runs what the focused engine's performance
  pass is judged on into `bench/targets/LABEL.csv` (outside
  `bench/results`, which a baseline's `--fresh` deletes): the families
  the first baseline showed it losing on at the sizes named in the script
  (3-Partition, Partition, QBF, Mix, the Petri-net counters in both
  modes, `growing`, `chain`, the wide sequents), the problem file, and a
  fixed sample of 113 intuitionistic LLTP problems listed by name: 24
  each of those the first baseline's pass ended at the time limit, the
  copy bound, the recursion limit and the width limit (every 35th to
  41st by name of those with at most 200 000 occurrences), 13 Petri nets
  whose search missed its stop there (25 s of grace), and 4 under a
  recursion limit of 16 384 (two proved long after their limit, one
  whose arena outgrew the memory, one killed; 55 s of grace). 165 runs
  with 300 s for a family or file problem and 5 s for an LLTP one, runs
  under 10 s three times.
- **It runs by day**: two streams, each pinned to a performance core
  (`cores`, 2 and 3), as the user unit `linlog-targets` with 8 GiB and no
  swap for all of it and 6 GiB of address space per process, on a copy of
  the binary (`target/targets/LABEL/`), so that builds meanwhile do not
  touch it. About 35 minutes at the engines of the first baseline, about
  20 after the pass (Mix at ten and eleven pairs is most of it). The
  streams write `LABEL.1.csv` and `LABEL.2.csv`, which a second start
  resumes, and the script joins them at the end.
- **What compares**: the counters first. On one thread `nodes`, `splits`,
  `memo_hits` and `memo_entries` of a decided run are a function of the
  input and the engine, on any machine under any load; `before.csv`
  reproduces the first baseline's on every row both decide. `cpu_ms`
  second, where `wait_ms` is small; `time_ms` is reported and not argued
  from. A run that ends at a limit has counters that depend on when the
  limit fell. `splits` changed its meaning with the pass: steps of the
  split searches (a member assigned and the counts tested) and forced
  splits, where it was submasks enumerated.
- **The four labels** of the pass are committed: `before` (the engines
  of the first baseline), `after-search` (after the changes to what the
  engine searches), `after` (after the changes to what a unit of
  search costs, which leave the counters of every decided row as they
  were) and `after-bias` (the default bias running both searches on a
  sequent with exponentials: the rows without exponentials have the
  counters of `after`, those with them the sum over the turns of both
  searches); `bench/TARGETS.md` has the table. A later change that must
  not alter the search runs the script under a label of its own and
  compares with `after-bias.csv`, or with `after.csv` under an explicit
  `--bias`; trial labels `scratch-*` are ignored by jj. `baseline-2026-10-02`
  is the set-up check of the second baseline: the engine it measured
  reproduces every decided row of `after-bias.csv`. `after-check` is the
  first label with the column `check_ms` (the rewritten checker, and the
  search checking every proof unless the harness switches that off):
  the same counters again, and the check at 0.3 % of the search time
  over the proved runs. `after-limits` is the first without the
  `portfolio` column (the polls inside forced chains and the set-up,
  the cursors of a chain's lookups, the copies ranked in one pass): the
  same counters again. `after-memory` is the first with the column
  `memory_limit` (the search under a bound of one gibibyte, the memo as
  records in chunks, the kept arena collected): the 99 decided rows
  have the counters of `after-limits`, and the rows over a second are 8
  to 18 % faster (`mix` at 10: 151 s before, 125 s).

## Extension points

- **A family**: an entry of `linlog::families::FAMILIES` (name, summary,
  default sizes, instances per size, generator) with its verdict proved
  by construction; `verdicts_as_constructed` then covers its smallest
  size. Pick the default sizes so that the largest one times out at the
  baseline's limit and the others do not. Three families have no such
  size since the performance pass: `3-partition-no` is refuted in 971
  stable sequents at every bin size, the wide sequents meet the recursion
  limit at 2 048 literals first, and the counter is proved at 64 tokens
  in 81 s (128 was not tried, as each doubling multiplies the time by
  about forty).
- **A problem source**: a function in `problems.rs` that lists
  `Reference`s and an arm of `load`.
- **A configuration axis** (a new `Options` knob): a `RunArgs` flag, its
  argument in `child`, a `OneArgs` field applied in `tail`, a column at
  the end of the tail (also filled in by `died` and by the error rows),
  the key of `finished`, and the label in `summary`'s `config`; `--bias`,
  `--forward-copies` and `--memory-limit` are the models. The last is
  the search's memory bound in bytes (`memory_limit`, the last column):
  absent it is the library's default, 0 is no bound, and a file from
  before the column ran without one, which is how `finished` and the
  summary read an empty field. The reasons `memory_limit` and
  `index_limit` are the two the bound added. After it come
  `copies_reached` (`Statistics::copies`: how far the deepening got) and
  `pool_after` (`--pool-after SECONDS`: the child searches on one thread
  that long and then with a pool of `--jobs` − 1 threads (at least two)
  beside it, the first to decide answering, each with half the memory,
  adding the counters of both, as the command does by default; empty for
  threads from the start, which is what a file from before the column
  ran). `--copies
  none` (`run::Bound`) is a search without a bound, written `none` in
  the `copies` column; without the flag the copy bound is still the
  problem's, else 3, so the harness's own defaults did not move when the
  command's did. Every LLTP pass of `baseline.sh` and `targets.sh` names
  `--copies 3` all the same, so that a later change of the harness's
  default leaves them the passes of the earlier baselines; the families
  keep their own bounds. `baseline.sh`'s stage 3 ends with
  `lltp-default`: the intuitionistic library under the command's
  default (`--copies none --jobs all --pool-after 0.1 --timeout 2`),
  about an hour and a quarter, since some two thousand problems wait the
  whole limit.

## Heap profiles

What a search allocates is read with heaptrack from the flake's
nixpkgs, which is not in the devshell (its closure is a gigabyte of Qt
for a viewer nobody here uses):

```sh
CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --locked -p linlog-bench --target-dir target/symbols
H=$(nix build --no-link --print-out-paths --inputs-from . nixpkgs#heaptrack)/bin
systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 5 \
  $H/heaptrack --record-only -o OUT target/symbols/release/linlog-bench one \
  --problem family:qbf:48:0 --mode given --engine auto --jobs 1 --timeout 20
$H/heaptrack_print -f OUT.zst -a 0 -T 0 -n 8 -s 1        # the summary and the peaks
$H/heaptrack_print -f OUT.zst -t 0 -p 0 -a 0 -T 0 --flamegraph-cost-type peak -F stacks.txt
```

- **`--record-only`, always**: without it `heaptrack` opens its viewer
  when the run ends, on the desktop of whoever sits there.
- The merged backtraces of `heaptrack_print` end in the allocator's own
  frames; what says which structure a byte belongs to is the collapsed
  stack file (`-F`, one line per stack, root first, the cost last; cost
  types `peak` and `allocations`), summed by its innermost `linlog`
  frame with a few lines of awk. `-t 0` keeps the type names, without
  which every method reads `<>::insert`. A file of a run that
  allocates per node has millions of lines and takes minutes to sum.
- A run under heaptrack is slower by the allocations it makes, so a
  profile of five seconds is not five seconds of search: compare bytes
  per entry and allocations per stable sequent, not totals.

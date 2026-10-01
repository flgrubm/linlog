# Step 14 report: benchmarks, LLTP input and the hard families

Sessions of 2026-09-30 and 2026-10-01. The baseline is taken: the night
of 2026-09-30 to 10-01, from 21:23 to 06:45 (9 h 21 min), every stage
complete, on commit `b53cb17c6831`, on the 16-core Intel Core Ultra X9
388H, on mains, with turbo on. Its rows are `bench/results/2026-09-30/`
and its tables `bench/RESULTS.md`. Nothing in the night disturbed it
measurably: sequential runs waited for a CPU 0.07 % of their time, and no
scheduled job ran. It found no wrong verdict of an engine. It did find:
nine more LLTP headers that contradict their problems, a stop the
parallel path misses on small Petri nets, five aborts at sixteen threads
on the largest problems, and one hole in the script (the net engine's
cubes on MLL 3-Partition were never run). All four are below.

The step was done in two runs. The first session built everything and
could not take the baseline (see "The runs"). The second checked the
harness and the machine, made a baseline keep a directory of its own,
made the run start and stop unattended within the night, took it, and
wrote this report.

## Outcome

- **LLTP input**: `linlog::lltp::read` (feature `parse`) reads the LLTP
  library's `fof(name, role, formula).` files into a `Sequent` and the
  status the header claims. Every file of the library reads except
  SYJ206+1.018, which is malformed in the library itself (one
  parenthesis short). `Reading::new` refused none of the 4 495
  intuitionistic problems.
- **Families**: `linlog::families` (feature `parse`) generates 17 problem
  families at any size, each instance with a verdict known from the
  problem it encodes: 3-Partition as a Horn program with `&` (Kanovich
  style) and as Lincoln and Winkler's two-literal MLL sequent, each
  solvable and unsolvable; Matsuoka's Partition, solvable and unsolvable;
  random 3-QBF under a lock-and-key encoding into MALL; wide sequents with
  every literal once to four times; tensor pairs under Mix; the Petri-net
  counter and its unreachable variant; the growing context; the chain of
  implications with additive choice; identities of additive trees. The
  engines' tests use these generators; the ignored timing tests
  (`speedups` twice, `three_partition_refuted`, `horn_programs_slow`,
  `illtp_style_slow`, `partition_instances_slow`) are retired, since the
  harness measures them.
- **The harness**: the workspace crate `bench/` (package and binary
  `linlog-bench`) runs families, LLTP files and problem files in every
  mode, engine and thread count asked for, one child process per run with
  a wall-clock limit and a kill after it, one CSV row per run (verdict,
  reason, whether the proof passed the checker, engine, fragment, sizes,
  wall time, CPU time, run-queue wait, the engine's counters), and
  summarises CSV files as Markdown: problems solved within the limit per
  family and configuration, and a time per problem and configuration.
- **The baseline script**: `bench/baseline.sh --arm --fresh` arms two
  transient user timers, one that starts the run at 20:00 (or at once
  inside the slot) and one that stops it at 07:00. The run executes as a
  memory-capped systemd user unit that waits for an idle machine. Each
  baseline goes into `bench/results/DAY/`, named by the day it started,
  with `starts.txt` naming the commit measured and `RESULTS.md` its
  tables; `bench/RESULTS.md` copies the latest.
- **The baseline**: taken, every stage complete, before the 07:00 stop.
  Its numbers are below, under "The baseline's numbers".
- **Checks**: the flake's `bench` check runs the harness on the smallest
  instance of every family and on the committed problem file and fails
  on any verdict against a known one; the `lltp` package fetches the
  library at a pinned commit.
- **One engine fix** (a deviation, below): the focused engine polls its
  stop condition during long split enumerations; before, a 2 s limit ran
  past five minutes.

Commits of the first run, in order: "Read LLTP problems", "Generate the
hard families", "Poll the stop condition during long split
enumerations", "Add the benchmark harness", "Document the benchmarks",
"Detach the baseline and check that the machine is idle", "Extend the
baseline to an overnight run", "Record CPU time and run-queue wait per
run", the first version of this report, "Shield the benchmark cores and
record the machine's state", and its amendment. Of the second: "Keep
every baseline in a directory of its own and run it unattended in the
night slot", "Record the baseline", "Drop the preliminary rows the
baseline supersedes", "Run the net engine's cubes on MLL 3-Partition
too, and estimate the baseline from its night", and this report.

## The night: what the unit and the administrator did

The second session armed the run at 21:22 with `bench/baseline.sh --arm
--fresh`. By then the slot had begun, so the start timer fired at once.
The unit waited one minute, until the load average fell below 1 after
the session's own checks, and started at 21:23 with a load average of
0.57. It then:

- kept the user slices off CPUs 0 to 3;
- stopped the user timer `obsidian-snapshot` (23:00);
- built nothing, since the binary was current;
- ran the three stages;
- wrote `RESULTS.md` at 06:45.

At the end, its `ExecStopPost` gave the slices every core back and
started `obsidian-snapshot.timer` again, which caught up on its missed
run at once. The stop timer fired at 07:00 on an inactive unit.

The administrator pasted the evening block at 23:02. It stopped the
system timers `nix-gc` (due 00:02) and `nix-optimise` (due 03:54) and the
two sync clients, and kept `system.slice` and `init.scope` off the
performance cores. So stage 1 (21:23 to 23:10) ran with syncthing and
insync running and with the system's services free to use CPUs 0 to 3.
The rows show no effect of that (see "The machine and its noise"). The
morning block, pasted at 07:37, undid all of it, and the two system
timers caught up on their missed runs then. No scheduled job ran during
the baseline.

| stage | what | estimated | took |
|---|---|---|---|
| 1 | every family at its sizes (300 s each, fast runs thrice); focus against net on the MLL families and the problem file; the net engine's test period 1, 2, 8, 16; the intuitionistic mode of the counter, chain and 3-Partition families; the whole LLTP library intuitionistically and classically (5 s each); four streams pinned to CPUs 0–3 | 1 h 40 | 1 h 47 (21:23–23:10) |
| 2 | the largest sizes that time out at 300 s, once each with 20 min; the LLTP problems that ended at the copy bound again with a bound of 10, those at the recursion limit again with 16 384; four streams | 1 h 30 | 1 h 03 (23:10–00:13) |
| 3 | alone: the hard families on 1, 2, 4, 8 and 16 threads (120 s), 2 h 41; the net engine's cubes on the Partition table, 14 min; the LLTP problems that timed out or took 50 ms or more on 16 threads, 1 h 48, and the same with the portfolio, 1 h 49 | 5 h | 6 h 32 (00:13–06:45) |

The script now estimates 9 h 45 min (it said 8 h 30), so an armed run
starts by 21:15 at the latest. The order already puts the sequential
stages, which step 15 compares with first, before the parallel one, so a
run that overruns loses only part of stage 3, which resumes on another
night.

**For step 16, before leaving** (the morning block undoes it):

```sh
sudo systemctl stop nix-gc.timer nix-optimise.timer syncthing.service
sudo systemctl set-property --runtime system.slice AllowedCPUs=4-15
sudo systemctl set-property --runtime init.scope AllowedCPUs=4-15
systemctl --user stop app-insync@autostart.service
```

**In the morning, after 07:00:**

```sh
sudo systemctl set-property --runtime system.slice AllowedCPUs=
sudo systemctl set-property --runtime init.scope AllowedCPUs=
sudo systemctl start nix-gc.timer nix-optimise.timer syncthing.service
systemctl --user start app-insync@autostart.service
```

## The machine and its noise

The author asked whether the processor's cores and the system's services
can distort the numbers. The facts come from `lscpu`, sysfs, `systemctl`
and the system flake's `power.nix` and `idle-lock.nix`. The second
session checked them again before the night and found them as described.

- **16 physical cores, no SMT**, of three kinds: CPUs 0–3 are performance
  cores (up to 5.1 GHz, a 3 MB L2 each), 4–11 efficiency cores (4.0 GHz,
  two clusters of four sharing a 4 MB L2), 12–15 low-power efficiency cores
  (3.7 GHz, one shared L2 and no share of the 18 MB L3). The kernel
  reports SMT as not supported, and every CPU is its own sibling, so the
  pinned streams share no core.
- **The four sequential streams share the L3 and the package's power and
  heat**, so one stream is slower than it would be alone. The baseline
  measures it: a run in stage 1's streams takes 3 to 11 % longer than the
  same run alone in stage 3 (the unsolvable 3-Partition with bins of four
  49.8 s against 45.1 s, QBF 20 #0 87.2 s against 78.4 s, Partition with
  six items 12.7 s against 12.2 s, `wide-m4` 32 43.4 s against 42.3 s).
  The effect is the same for every sequential row of a stage, so their
  comparisons hold. The speedups compare with stage 3's own one-thread
  rows, which ran alone. The one exception is the net engine, whose
  one-thread rows come from stage 1 (see the follow-ups).
- **"Every core" is heterogeneous.** Sixteen threads include the four
  low-power cores, which are slower and outside the L3. On the families
  that scale, the speedups still grew from eight threads to sixteen:
  the unsolvable 3-Partition from 6.0× to 7.8×, and the net engine's
  Partition table from 6.3–7.0× to 11.8–13.6×.
- **Heat.**
  - The package throttled 473 691 times over the run, for 11 283 s in
    all, against 9 h 21 min of wall time. Stage 1's four streams cost
    about 25 000 events and 300 s per ten minutes. The all-core LLTP runs
    cost fewer events but 300 to 500 s per ten minutes.
  - Turbo stayed on: without it the night's work would not fit the slot.
  - The settings were platform profile `performance`, governor
    `powersave`, energy preference `balance_performance` and turbo on.
    This is TLP's profile on mains, which lets the hardware pick the
    clock. `RESULTS.md` records them.
  - A later baseline is comparable to this one only under the same
    settings.
- **Background services.**
  - The system runs NetworkManager, syncthing, journald, udisks, fwupd,
    bluetooth, the keyboard remapper and the nix daemon. The user
    session runs the compositor, the editor daemon, insync (a Google Drive
    client), pipewire, portals and the idle manager.
  - At rest they cost a few percent of one core, in short bursts. The
    journal's ten-minute samples of other processes using a CPU named
    only the compositor, the night-light daemon and the idle Claude Code
    session, at 1 to 8 % of one core each.
  - The unit moves the user's other slices (`app.slice`, `session.slice`,
    `background.slice`) onto CPUs 4–15 for its whole duration. It stops
    the user timers due during the run. It undoes both in its
    `ExecStopPost`, which runs however the run ends.
  - The system slice needs root (the evening block), and kernel threads
    and interrupts stay where the kernel puts them.
  - Every run records `wait_ms`, the time its thread was ready but
    waiting for a CPU. Over the 3 324 sequential runs above a second,
    the waits sum to 22.3 s of 32 636 s, 0.07 %. The largest single wait
    was 10 ms, on a 635 ms run of `wide-m2/2048` on the net engine. That
    is the one problem marked `†` (a wait above 1 % of its run) in the
    per-problem tables, of the five the summary counts.
  - Runs under two seconds are repeated three times and the median
    taken.
- **Scheduled jobs**: none ran during the run. In the slot are `nix-gc`
  (00:02), `nix-optimise` (03:54), `obsidian-snapshot` (23:00, a user
  timer the unit pauses), `logrotate` (hourly) and `fwupd-refresh`; the
  last two are negligible. `borgbackup` (midday) and `fstrim` (Mondays)
  fall outside the slot.
- **Suspend.**
  - The idle manager (stasis) suspends only on battery, after 20 minutes.
  - Logind suspends on a closed lid even on mains (`HandleLidSwitch` is
    `suspend`, with no rule for external power).
  - The unit holds a `sleep:idle:handle-lid-switch` inhibitor, and an
    armed run waits for mains. The journal line says `BATTERY` if the
    power goes during the run; it never did.

What remains uncontrolled: the kernel's own threads and interrupts, and
the package's heat. A disturbed instance can be rerun on a later night:
delete its rows and run the script again without `--fresh`, and it
resumes.

## How it runs

```sh
nix build .#lltp -o bench/lltp                       # the library, 1.1 GB, once
cargo run --release -p linlog-bench -- families      # the families and their sizes
cargo run --release -p linlog-bench -- run --family partition-no=3,4 \
  --engines focus,net --jobs 1,4 --timeout 10 --output runs.csv
cargo run --release -p linlog-bench -- run --lltp bench/lltp/ILL/KLE-cbn --modes given,classical
cargo run --release -p linlog-bench -- summary runs.csv
bench/baseline.sh --arm --fresh                      # the baseline, in tonight's slot
systemctl --user list-timers                         # linlog-baseline and its stop
```

`run` takes `--family NAME[=SIZES]` (or `--all-families`), `--lltp PATH`
(a problem under a directory named `ILL` is intuitionistic),
`--problems FILE` (lines `name; mode; expected; copies; sequent`, as
`bench/problems/slow-tests.txt`), `--only TEXT,…` (which filters every
source, families included), `--reverse`, `--modes
given,classical,intuitionistic`, `--engines auto,focus,net,two-sided,
additive`, `--jobs 1,2,…,all`, `--portfolio`, `--copies`,
`--test-period`, `--recursion-limit`, `--timeout`, `--repeat` with
`--repeat-under`, `--output` with `--append` and `--resume`.
`baseline.sh` takes `--arm` (with `--slot=HH:MM-HH:MM`, by default
20:00-07:00), `--detach` (at once, never stopped), `--fresh` and
`--force`.

**Timing.** `time_ms` is wall-clock time (`Instant`) around `prove_until`
in the child: forest construction and pool start-up included, parsing,
the proof check and process start excluded; the time limit is wall-clock
too, polled by the stop closure. The parent kills a child that outlives
the limit by a tenth plus five seconds, counted from the child's start,
parsing included. `cpu_ms` is the process's CPU time over the same span
(all threads, 10 ms ticks) and `wait_ms` the time the calling thread was
ready but waited for a CPU.

## The baseline's numbers

From `bench/RESULTS.md` and the rows under `bench/results/2026-09-30/`.
A time is the median of up to three runs (runs under 2 s are repeated).
`>` marks a time limit reached, `✗` a refutation, `?` an unknown with its
reason. Every verdict on a generated family agrees with its
construction.

**Families, one thread, default engine** (stage 1 at 300 s in four
streams; the next sizes in stage 2 at 1 200 s, also in four streams).
The largest size of every family times out at 300 s and the others do
not, as intended:

| family | largest decided in 300 s | next size, 1 200 s |
|---|---|---|
| 3-partition-no (Horn, bins of b) | b = 4: 49.8 s (two-sided 25.4 s) | b = 5: refuted in 302.4 s |
| partition-yes (n items) | n = 6: 12.7 s | n = 7: > 1 200 s |
| partition-no | n = 4: 114 ms | n = 5: refuted in 811.5 s |
| qbf (n variables, 4 instances) | n = 16: 1.45–7.9 s; n = 20: 87.2, 89.3 and 123.6 s, the fourth > 300 s | n = 24: all > 300 s, #0 > 1 200 s |
| wide-m3 (focus, k literals) | k = 30: 10.6 s | k = 36: proved in 697 s |
| wide-m4 (focus) | k = 32: 43.4 s | k = 36: proved in 685 s |
| mix (k pairs under Mix) | k = 10: 214 s (×10–11 per pair) | k = 11: > 1 200 s |
| counter (n tokens) | n = 8: 124 ms (two-sided 15.4 ms) | n = 16: > 1 200 s |
| counter-over | copy bound at n = 8 after 493 ms (two-sided 18.4 ms) | n = 16: > 300 s |
| growing (bound b) | copy bound up to b = 256 (79 ms) | b = 1024: recursion limit after 0.57 s |
| chain (k clauses) | k = 256: 2.8 s, the same two-sided | – |
| additive (depth d, additive engine) | d = 16: 528 ms | – |
| 3-partition-yes, 3-partition-mll-*, wide-m1, wide-m2 | every size, at most 0.62 s (the wide ones at 2 048 literals, on the net engine) | – |

From the problem file: the cancellation case (3-Partition with bins of
four, proved) takes the focused engine 50.7 s.

**Intuitionistic against classical** on the same sequents: the counter
with 8 tokens 15.4 ms against 124.0 ms (8.1×), its unreachable variant
18.4 ms against 492.7 ms (26.8×), the unsolvable Horn 3-Partition with
bins of four 25.4 s against 49.8 s (2.0×), the solvable one with twelve
bins 16.5 ms against 30.1 ms (1.8×), the chain the same both ways. On the
LLTP library, every one of the 4 495 intuitionistic problems ran in both
modes at 5 s:

- Both modes decide 731 problems, with never a different verdict.
- Six Petri nets are decided only intuitionistically (the classical
  search times out), and none only classically.
- Where both take a millisecond or more (79 problems, 77 of them Petri
  nets), the classical search is 1.36× slower in the median, from 0.89×
  to 9.6×.

So the two-sided constraint pays across ILLTP, most on Horn-like
problems, as step 8 measured on the counter.

**Focus against net on unit-free MLL**, 60 s, stage 1:

| problem | literal multiplicity | focus | net |
|---|---|---|---|
| 3-partition-mll-no, b = 4 / 5 / 6 | 8 / 10 / 12 | 15 µs / 18 µs / 17 µs | 1.80 s / 56.3 s / > 60 s |
| 3-partition-mll-yes, b = 8 / 12 | 16 / 24 | 33 µs / 37 µs | 5.55 s / > 60 s |
| partition-yes, n = 4 / 5 / 6 | 8 / 12 / 16 | 2.3 ms / 169 ms / 12.2 s | 2.53 s / > 60 s / > 60 s |
| partition table 2-3-2-1 / 1-2-5 / 3-3-3-1 | 8 / 8 / 8 | 6.1 ms / 5.8 ms / 112 ms | 4.58 s / 3.66 s / > 60 s |
| wide-m1, k = 32 / 256 / 2048 | 1 | 45.5 s / too wide / too wide | 196 µs / 9.3 ms / 638 ms |
| wide-m3, k = 30 | 3 | 10.9 s | 182 µs |
| wide-m4, k = 28 | 4 | 2.75 s | 290 µs |

The net engine loses by two to six orders of magnitude wherever equal
literals sit in one `⊗` or `⅋` tree (every Horn encoding), and wins by
as much where they are spread over conclusions (the wide sequents). The
focused engine cannot split a context of more than 63 members at all.
Forced onto the additive family, the focused engine takes 6.2 s at
depth 14 and aborts after 12.2 s at depth 16, most likely at the 12 GiB
cap per process (only the last line of its error output is kept), where
the additive engine needs 528 ms.

**The net engine's test period** (by default every link up to 200
occurrences, every fourth above), one thread:

| problem (occurrences) | default | 1 | 2 | 8 | 16 |
|---|---|---|---|---|---|
| 3-partition-mll-no/4 | 1.80 s | 1.80 s | 1.43 s | 5.01 s | 22.0 s |
| 3-partition-mll-no/5 | 56.3 s | 56.3 s | 51.4 s | > 60 s | > 60 s |
| partition-yes/4 | 2.53 s | 2.76 s | 6.11 s | > 60 s | > 60 s |
| partition-no/3 | 3.45 s | 3.49 s | 8.11 s | > 60 s | > 60 s |
| wide-m1/2048 (14 335) | 638 ms | 1.35 s | 856 ms | 521 ms | 460 ms |

On the small Horn structures, a period of 2 is 1.1–1.3× faster on one
family and 2.3–2.4× slower on the others, and 8 or 16 lose by up to an
order of magnitude or time out. On the large wide structures, 16 is
1.39× faster than the default of 4. The default holds. A longer period
above a few thousand occurrences is a small follow-up, and no CLI flag
is needed (`Options::test_period` serves library users).

**Threads**, stage 3, alone on the machine, 120 s (the one-thread column
is the same stage's):

| problem | 1 | 2 | 4 | 8 | 16 |
|---|---|---|---|---|---|
| 3-partition-no/4 | 45.1 s | 23.5 s (1.9×) | 12.7 s (3.6×) | 7.45 s (6.1×) | 5.80 s (7.8×) |
| 3-partition-no/5 | > 120 s (302 s in stage 2) | > 120 s | 77.7 s | 46.7 s | 37.1 s |
| partition-yes/5 | 192 ms | 192 ms | 196 ms | 30.5 ms (6.3×) | 17.5 ms (10.9×) |
| partition-yes/6 | 12.2 s | 11.8 s | 8.70 s (1.4×) | 4.21 s (2.9×) | 1.33 s (9.2×) |
| partition-no/4 | 127 ms | 73.9 ms (1.7×) | 45.9 ms (2.8×) | 29.4 ms (4.3×) | 26.9 ms (4.7×) |
| qbf/20#0 | 78.4 s | 43.0 s (1.8×) | 43.2 s | 43.9 s | 43.4 s (1.8×) |
| qbf/20#1 | 109 s | 61.4 s (1.8×) | 61.7 s | 62.0 s | 61.1 s (1.8×) |
| mix/9 | 18.0 s | 18.0 s | 16.7 s | 16.0 s (1.1×) | 16.9 s |
| counter/8 | 130 ms | 156 ms | 127 ms | 105 ms (1.2×) | 109 ms |
| counter/2 | 13 µs | 99 µs | 126 µs | 253 µs | 479 µs |
| counter-over/8 (to its bound) | 550 ms | 503 ms | 390 ms | 355 ms (1.5×) | 323 ms (1.7×) |
| wide-m3/30 | 11.1 s | 10.7 s | 8.34 s (1.3×) | 7.87 s (1.4×) | 8.24 s |
| wide-m4/32 | 42.3 s | 42.0 s | 32.7 s (1.3×) | 32.8 s | 35.5 s |
| 3-partition-yes/12 | 37.7 ms | 29.6 ms | 29.3 ms (1.3×) | 29.5 ms | 30.4 ms |

None of the parallel runs decides `3-partition-no` beyond b = 5,
`partition-yes/7`, `partition-no/5`, `qbf/24`, `mix` at 10 or 11,
`counter/16` or the next sizes of `wide`, within 120 s.

The net engine's cubes on the Partition table, against its one-thread
run in stage 1 (four streams, so the speedups flatter it by up to a
tenth):

| problem | 1 | 2 | 4 | 8 | 16 |
|---|---|---|---|---|---|
| 1-2-5 | 3.66 s | 2.04 s (1.8×) | 1.02 s (3.6×) | 582 ms (6.3×) | 310 ms (11.8×) |
| 2-3-2-1 | 4.58 s | 2.27 s (2.0×) | 1.19 s (3.8×) | 654 ms (7.0×) | 338 ms (13.6×) |
| 1-1-1-5 | 7.50 s | 3.91 s (1.9×) | 2.02 s (3.7×) | 1.11 s (6.8×) | 611 ms (12.3×) |
| 1-2-3-4-5-5 | > 60 s | > 60 s | 3.5 ms | > 60 s | > 60 s |
| 2-2-2-2-2-2-9-1 | > 60 s (focus too) | > 60 s | > 60 s | 861 ms | 955 ms |

The last two rows depend on which cube the solution lies in, not on the
thread count. The MLL 3-Partition instances that were meant to be run
here were not (see the follow-ups).

**LLTP, one thread, 5 s**, intuitionistic pass over all 4 495 problems:

- 638 proved and 99 refuted (16.4 %).
- Unknown: 984 at the time limit, 983 at the recursion limit of 2 048,
  898 at the copy bound of 3, 845 with a context of more than 63 formulas
  to split, and 47 killed (the largest files, whose parse alone takes
  most of the kill's grace). One file is malformed.
- Per collection: KLE-IMP-CONJ 162 of 222, its ALT 27 and NON-THEOREMS
  22 (all); KLE-01, cbn and cbv 50, 68 and 67 of 88; ILLTP-SYN 11, 16 and
  16 of 19; ILLTP-SYJ 11, 34 and 39 of 252; ILLTP-LCL 1 of 6; misc 3 of 3;
  the Petri nets 210 of 3 137.

The classical pass over the same files plus the 17 CLL problems decides
742 problems (the 731 shared, 11 of the CLL problems).

**Raised limits, one thread, 5 s**:

- Of the 898 problems that ended at the copy bound of 3, a bound of 10
  decides 314: 289 proved and 25 refuted. 177 stay at the bound and 407
  time out. Every problem of KLE-IMP-CONJ is decided now. Nine of the
  refutations contradict their headers, and the header is wrong in every
  case (below).
- Of the 983 problems that ended at the recursion limit of 2 048, a limit
  of 16 384 decides 56 Petri nets, all proved. 154 stay at the limit,
  401 reach a context too wide to split, 363 time out and 9 are killed.

**All cores and the portfolio on LLTP**, 5 s, over the 1 102
intuitionistic problems that timed out, were killed or took 50 ms or
more on one thread:

- 16 threads decide 32 of them, against 28 on one thread: 11 gained, 7
  lost.
- On the 21 that both decide, 16 threads are 2.2× slower in the median
  (from 6.7× slower to 5.1× faster).
- 209 runs are killed at the kill time of 10.5 s. Of these, 163 timed
  out cleanly at 5 s on one thread and 3 were decided there (in 0.4,
  0.8 and 4.1 s). All 166 are small files, at most 0.18 MB, so this is
  a search that does not stop on time, not a parse.
- 5 runs abort with a panic on three of the largest SYJ problems (8 to
  12 million occurrences), where one thread times out at 5 s or is
  killed while parsing.
- The portfolio decides 34, the 32 and two more Petri nets. Its times on
  the 32 shared match all-cores within 0.69–1.27× (median 1.01).

**Mismatches with the headers**: 23 verdicts contradict what an LLTP
header claims, every one of them the library's doing, none an
engine's. The 5 s pass finds the 14 the first session found:

- KLE065 (three translations) and SYJ212+1.001 (cbn) are marked
  Non-Theorem, but linlog's proofs pass the checker and LLTP's own result
  files prove them too.
- SYN001 (three) is marked intuitionistic Non-Theorem and is proved with
  checked proofs.
- KLE013 (three) and SYN041 (three) are marked Theorem but have `0` as
  their goal and no hypothesis that can produce `0`.
- SYN915 in its cbn translation turns `$true` into an atom `T`.

The copy bound of 10 finds nine more, all refutations of problems marked
Theorem: KLE017 (cbn and cbv), KLE069 (cbn and cbv), KLE078 (cbv),
KLE088 (cbn), SYJ103 (cbv), and SYJ105+1.003 and +1.004 (cbv). Each has
a classical countermodel once linearity is forgotten (`!` erased, `⊸` as
implication, `⊗` and `&` as conjunction, `⊕` as disjunction, `0` as
falsity). This was checked by truth table for all nine; for KLE017, `a`
true and `b` false. A sequent provable in ILL stays provable classically
under that map, so none of the nine is a theorem. Their KLE-01 and
remaining translations stay at the copy bound, which is consistent. The
copy-10 run also proves SYJ212+1.001 in its other two translations,
whose header is the one already known to be wrong.

## What the numbers say about each engine

- **The focused engine** decides every Horn encoding the net engine
  cannot, but it enumerates `⊗` splits.
  - The unsolvable 3-Partition grows from 49.8 s at bins of four to
    302 s at five.
  - The unsolvable Partition grows from 114 ms at four items to 811 s at
    five.
  - Mix grows about tenfold per pair (214 s at ten).
  - QBF over 20 variables takes about 100 s.
  - A context of more than 63 formulas is out of reach (845 LLTP
    problems).
  - The copy bound and the recursion limit stop as many LLTP problems as
    the time limit does (898 and 983 against 984). Raising them decides
    a third of the first (314) and 6 % of the second (56); most of the
    rest then run into the time limit or the 63-member limit instead.
- **The net engine** is linear on distinct atoms and wide contexts (14 335
  occurrences in 0.64 s) and exponential in the symmetry of equal
  literals within one tree. Its cubes scale almost linearly to eight
  threads and keep scaling to sixteen (11.8–13.6×), which reproduces
  step 13's 7.4× and 6.7× at eight on the Partition table (6.3–7.0×
  here).
- **The parallel focused engine** reproduces step 13's 6.0× on the
  unsolvable 3-Partition at eight threads (6.1× here) and reaches 7.8× at
  sixteen. It also scales on Partition, up to 9–11× at sixteen.
  Elsewhere it gains little or nothing:
  - QBF gains 1.8× from the second thread and nothing from more, which
    is what and-parallel `&` premises (QBF's `∀`) give and the cubes over
    choices do not.
  - Mix and the counter programs gain nothing (1.1–1.2×). Step 13's
    1.35× on the counter becomes 1.2× here.
  - The unreachable counter gains 1.5–1.7×.
  - The wide sequents gain 1.3–1.4×.
  - Every problem under a millisecond loses to the pool's start-up
    (the counter with two tokens: 13 µs on one thread, 479 µs on
    sixteen).
  - On LLTP, sixteen threads are a net loss: 2.2× slower in the median,
    with 11 problems gained against 7 lost, and 166 runs that overrun
    their limit.
- **The portfolio** shows no consistent gain on LLTP (the same times
  within 0.69–1.27×, two more Petri nets decided), as on step 13's
  families.
- **The two-sided engine** gains 2× to 27× on the Horn-like families and
  1.36× in the median on ILLTP's Petri nets over the classical search of
  the same sequents, and never decides less.
- **The additive path** decides the depth-16 identity (131 071-leaf
  trees) in 0.53 s but holds a memo of about 8 GB doing it. Its memo has
  no cap, which is a follow-up.

## Decisions

- **The LLTP reader lives in core behind `parse`**: the web front end
  could accept LLTP files as the CLI could, and the reader is a few dozen
  lines over the crate's own parser, since LLTP's connectives and
  precedences are the crate's (checked on every mixed-operator formula of
  the library). Atom names with `-` or `.` (Petri-net places such as
  `P-start_1_1`, `merge.s00001061.input`) are mapped to `‿` and `·`,
  which the crate's identifiers accept. The status is the first `Status
  (intuit.)` or `Status (linear)` comment's, else the plain one's, since
  the translated ILLTP problems carry their classical source's status
  first. The mode is not in the file: the harness takes it from the
  library's `ILL`/`CLL` layout. The CLI does not read LLTP files; that is
  a one-flag follow-up if wanted.
- **The families live in core behind `parse`**, public, because the
  engines' tests need the same encodings the harness runs (the prompt
  asked to move the test-private generators rather than duplicate them).
  Every verdict is derived from the encoded problem (subset sums, QBF
  evaluation, 3-Partition by construction) or a construction argument,
  never from an engine.
- **The harness is a crate of its own**, not `linlog bench`: it adds a
  process-per-run driver, CSV and summaries that no CLI user needs, and
  it takes no dependency beyond clap and anyhow, which the workspace had.
- **A child process per run**, not a thread: a stack overflow, an
  allocation failure or a search that stops polling costs one row
  (`crash`, `killed`), not the run; the parent never parses a problem.
- **The LLTP library is fetched on demand**, not committed: it is
  GPL-3.0 (redistribution is allowed with the licence, but the repository
  is EUPL-1.2 and admits no GPL dependency), and 1.1 GB. The flake's
  `lltp` package pins commit `e0394fb` with its Petri-net archives
  unpacked; `bench/lltp` is its ignored link.
- **The QBF family is a lock-and-key encoding in the spirit of Lincoln,
  Mitchell, Scedrov and Shankar, not their encoding verbatim**: `∀x` is
  `(~tx & ~fx) ⅋ S`, `∃x` is `((~tx ⅋ ~kx) ⊕ (~fx ⅋ ~kx)) ⅋ (kx ⊗ S)`,
  whose rest needs the key the choice releases, and a clause is the `⊕`
  of `(tx ⊗ ⊤)` or `(fx ⊗ ⊤)`. Random instances follow Gent and Walsh's
  model A (two existential literals per clause, 1.5 clauses per
  variable), which keeps them from being false outright; about half are
  true.
- **The Mix family wraps each tensor pair in `⊕ 0`**: in MLL the count
  equation refutes the bare pairs at once, so the `3^k` partition cost the
  step 3 report described only shows in MALL.
- **The dispatch threshold stays at two** (`NET_MULTIPLICITY`). The
  measurement above suggested raising it to four (the wide sequents at
  three and four occurrences run 10⁴× faster on the net engine, LLTP's
  multiplicative problems at three and four are tens of microseconds
  either way), and the reviewer then produced a sequent at multiplicity
  four, blocks of equal literals in one `⊗` tree with a defect in the
  last, that the net engine does not finish in 10 s and the focused
  engine refutes in 20 ms. Multiplicity is the wrong feature; the right
  one is equal literals under one pure `⊗` or `⅋` tree, which is also
  what the net engine's leaf symmetry break would remove. Recorded in
  `.claude/rules/core.md`.
- **Configuration labels carry the CSV file's name**: every file of the
  baseline is one `run` with its own options (a copy bound, a recursion
  limit, a longer limit), which the columns alone do not tell apart.
- **Options (D15)**: the harness adds no output a user configures in the
  library; its flags map onto `search::Options` as the CLI's do, and the
  LLTP reader and the families have no options.
- **A baseline's directory is named by the day it started, and a run
  without `--fresh` resumes the latest directory without `RESULTS.md`.**
  The directory needs no state file: a finished baseline is the one with
  its tables written. `--fresh` deletes the day's directory and nothing
  else. `--arm` and `--detach` create the directory before starting the
  unit, so the unit's own lookup finds it.
- **The commit is recorded per start, in `starts.txt`**, together with
  the load and whether the start was forced. A resumed baseline lists
  every start, so a resumption on another commit shows itself in
  `RESULTS.md`'s header instead of being refused, which would waste the
  night. The commit is `@-` plus the files `@` changes outside the
  results, read with `jj --ignore-working-copy`. Under the author's
  `signing.behavior = "own"`, a snapshot from inside the unit would sign
  a commit with nobody at the pinentry. The arming command snapshots
  first, interactively.
- **Unattended start and stop by two transient user timers**, not by a
  long-lived waiting process.
  - `linlog-baseline.timer` starts the unit at the slot's start. It
    starts it at once when the slot has begun, since a calendar time in
    the past never fires.
  - `linlog-baseline-stop.timer` runs `systemctl --user stop` on the
    unit at the slot's end, so the stop holds whatever the script is
    doing, and `ExecStopPost` undoes the shield and the paused timers.
  - The wait for an idle machine is in the unit, so the inhibitor holds
    from 20:00. Its test is the script's own: on mains and a load
    average of at most 1, checked every minute until the last start that
    still fits.
  - The slot is an option (`--slot`), and the estimate a variable in the
    script, set from the night's run.
- **The unit pauses the user's own timers due during the run** and
  restores them, since that needs no root. The system timers and the
  sync clients stay in the author's block, since stopping them needs
  root or concerns the author's own applications.
- **The `parallel-net` run is split in two** after the baseline, since
  its `--only partition-table` filtered out the MLL 3-Partition family it
  was meant to run. It also gains a one-thread column taken alone. The
  fix adds rows to the next baseline and changes none of this one's.

## Deviations, with reasons

- **The split poll** ("Poll the stop condition during long split
  enumerations"): the prompt says to measure the engines, not fix them,
  and the step-13 prompt took the stop closure's polling as reliable. It
  was not: the focused engine polled only at stable sequents, and a
  `⊗` split whose premises fail in focus visits none, so a 20-token Petri
  net ran past five minutes under a 2 s limit (60 million splits in one
  stable sequent). Every such run would have cost the harness its kill
  grace and the LLTP pass hours, and `linlog prove --timeout` had the
  same hole. The fix polls once every 4 096 splits (`SPLITS_PER_POLL`) in
  the two enumeration loops, returning `Stopped` exactly as a stable
  sequent's poll does; the reviewer argued it verdict-neutral, and with
  it the same run stops at 2.02 s.
- **Matsuoka's 3D-Matching encoding is not implemented**: the paper was
  not at hand, and the Partition encoding's buy-back trick does not carry
  over (its two rounds are symmetric only because the two halves are).
  Lincoln and Winkler's two-literal 3-Partition is the second MLL family
  instead.
- **Partition items are drawn from 1 to n** (not larger), so that the net
  engine still decides the small sizes at all.
- **The script changed after the baseline** (the split `parallel-net`
  run and the estimate), so step 16 runs a script that adds rows to this
  one's. No row of this baseline is affected.

## The runs

1. **A first run** started at 01:39 on 2026-09-30 as four pinned
   streams. At 02:03 the kernel's OOM killer stopped a reviewer's scratch
   checker at 62 GB, and systemd failed the whole terminal scope, which
   ended the session and the benchmark with it.
2. **A second run**, as a systemd unit from 08:12, shared the machine
   with other work and was stopped at 09:34 on request. Its rows were the
   report's preliminary numbers and were committed as `bench/preliminary/`.
   They are removed now, since the baseline supersedes them. The two
   agree exactly: on the 1 543 problems both decided (families, engines,
   intuitionistic mode, both LLTP passes), the verdicts and the `nodes`
   and `splits` are the same, and no problem is decided in one run only.
   The sequential counters are as machine-independent as claimed.
3. **The run planned for that night** was cancelled.

The harness answers all three:

- The baseline runs detached, with 24 GiB and no swap for the unit and
  `OOMPolicy=continue`.
- Every process is capped at 12 GiB.
- The run starts and stops itself within the slot and waits for an idle
  machine.
- Every run records its CPU time and run-queue wait.

**The fourth run is the baseline**, taken on the night of 2026-09-30 to
10-01 by the second session:

- By day it checked the script against this report and the rules and
  inspected the machine. The CPU topology, frequency settings, power,
  services, timers and suspend settings were as described, and
  throttling since that morning's boot was negligible (36 events).
- It made the two changes above (a directory per baseline, unattended
  start and stop).
- It tested the armed path three times for a few minutes, under an
  artificial load:
  - The unit waited for an idle machine and started at its latest start
    regardless, recording that it did.
  - A second run without `--fresh` resumed the same directory without
    duplicating a configuration.
  - The stop timer stopped the unit at the slot's end. The slices got
    every core back, the inhibitor was released, and the paused user
    timer was scheduled again.
- It armed the baseline at 21:22. The run took 9 h 21 min and finished
  before the stop.

## Review

A fresh-context reviewer, in two rounds, during the first run of the
step:

- **First round: the family claims, the split poll, the LLTP reader.**
  - The QBF encoding was argued sound and complete, and checked against
    its own evaluator on 2 000 instances (n up to 12), with an
    independent naive MALL prover agreeing on 1 200 of them.
  - The other families were brute-forced where small (the item lists for
    bins of 4 to 39, the Partition sizes) and argued where not.
  - About 240 engine runs gave no contradicting verdict, and the poll was
    argued verdict-neutral.
  - Findings, all acted on: the additive family at depth 18 needs about
    130 GB (the family stops at 16 and every process is capped); the
    reader took a translated problem's classical status (39 files would
    have raised false mismatches); Petri-net names with dots did not
    parse (6 files); annotations were misread; some sizes looped (the
    generators now panic on them).
- **Second round: the fixes and a raise of the dispatch threshold.**
  - It confirmed the reader against its previous version on every file
    up to 2 MB: no sequent read differently, and the six nets parse.
  - It confirmed the size guards.
  - It produced the counterexample that kept the threshold at two.
  - It pointed out that a problem's expected verdict was compared with
    runs in another mode: the harness now keeps it only for the problem's
    own mode.
- **Minor findings it named, left as they are:**
  - `Status (Intuitionistic)` is matched case-sensitively; this only
    affects five classical problems, where the plain status is the right
    one.
  - A quoted clause name containing `%` would swallow the next clause;
    no LLTP file has one.
  - The poll counter can skip a multiple of 4 096 when splits are counted
    outside the loops; that costs a poll, never a verdict.

The second run's changes are to a shell script and the documentation,
nothing soundness-critical, and had no reviewer. They were tested as
"The runs" says. The nine new header contradictions were checked
independently of the engines, by the countermodels above.

## Open questions and follow-ups; what step 15 must beat

- **The focused engine's targets**, sequential, from stage 1 and 2 (four
  streams) and stage 3 (alone):
  - the unsolvable 3-Partition, 49.8 s at bins of four (45.1 s alone)
    and 302 s at five;
  - Partition, 12.7 s at six items (proved) and over 1 200 s at seven;
    the unsolvable one 114 ms at four items and 811 s at five;
  - QBF, 87 to 124 s at 20 variables, one instance over 300 s, and every
    instance at 24 over 300 s;
  - Mix, 214 s at ten pairs, over 1 200 s at eleven;
  - the counter, 124 ms at 8 tokens (15.4 ms two-sided), over 1 200 s
    at 16; its unreachable variant, 493 ms to its bound at 8;
  - the wide sequents on the focused engine, 697 s and 685 s at 36
    literals;
  - the cancellation case, 50.7 s.

  These are where the per-problem atom bias, branch-and-bound splits, a
  canonical choice among identical members and nested forced factors
  should show. The counters (`nodes`, `splits`, `memo_hits`,
  `memo_entries`) of every sequential row are machine-independent: a
  run of step 15 at this commit must reproduce them.
- **The LLTP limits**: 984 problems stop at the time limit, 983 at the
  recursion limit, 898 at the copy bound and 845 at the 63-member limit.
  Raising the bound to 10 decides 314 of the 898. Raising the recursion
  limit to 16 384 decides 56 of the 983 and turns 401 of them into the
  63-member limit. The lazy contexts or the branch-and-bound split are
  what the last needs.
- **A stop the parallel path misses** (recorded in
  `.claude/rules/core.md`): at sixteen threads, 166 small Petri nets ran
  on past the harness's kill at 10.5 s, where one thread stops every one
  of them at 5 s. Something in the parallel focused engine does not poll
  its flags during long work, as the sequential engine did not before
  the split poll. This is for the parallel follow-ups. The CLI's `prove`
  on a pool runs the same engine, so its time limit can presumably
  overrun the same way.
- **Aborts at sixteen threads**: three of the largest SYJ problems (8 to
  12 million occurrences) abort with a panic at sixteen threads (5
  runs). A rerun of one at sixteen threads on four cores under 8 GB
  timed out cleanly, so the cause is likely the memory that sixteen
  workers fill under the 12 GiB cap (the unit's peak was 11.9 GB), but
  that is unconfirmed: the harness keeps only the last line of a
  child's error output. Keeping the whole of it for a crash row is a
  small harness follow-up. The focused engine forced onto the additive
  family at depth 16 aborts the same way, after 12.2 s.
- **The parallel engines on LLTP and the portfolio**: sixteen threads
  lose on LLTP (2.2× slower in the median, 11 gained against 7 lost);
  the portfolio gains nothing measurable. Whether to keep the portfolio
  is now the performance pass's call on these numbers.
- **The net engine's cubes on MLL 3-Partition were not measured**: the
  script's `--only partition-table` filtered the family out of its run.
  The script is fixed for step 16, which takes those rows and a
  one-thread column alone for the net engine.
- **The net engine's test period**: a period of 16 above a few thousand
  occurrences is 1.39× faster on the wide sequents; a small follow-up.
- **The additive memo is unbounded** (8 GB at depth 16). It needs a cap,
  or no memo for `&` pairs.
- **LLTP data**:
  - the 23 header contradictions above are worth reporting upstream;
  - SYJ206+1.018 is malformed;
  - the largest SYJ files (up to 103 MB) take longer to parse than a run
    gets.
- **D3** (index width): nothing in these numbers points at the `u32`
  indices; the limits that bind are the recursion depth and the split
  width.

## What step 16 must repeat for its numbers to compare

- **The same script, run the same way**: `bench/baseline.sh --arm
  --fresh` from the devshell, with the commit to measure checked out
  and the working copy clean. The baseline then goes into its own
  directory beside `bench/results/2026-09-30/`.
  - Its only differences from this run are the split `parallel-net` run
    (rows this one lacks) and the new estimate (an earlier latest
    start).
  - Compare rows by configuration: the CSV file, family, problem, mode,
    requested engine, jobs, portfolio and test period.
- **The same machine and settings**:
  - the 16-core Intel Core Ultra X9 388H laptop, on mains, lid open;
  - platform profile `performance`, governor `powersave`, energy
    preference `balance_performance`, turbo on. `RESULTS.md` records
    these; a difference makes the times incomparable.
- **The same inputs**: the LLTP library at the flake's pinned commit
  (`nix build .#lltp -o bench/lltp`). The toolchain is whatever
  `rust-toolchain.toml` names at the measured commit; a toolchain bump
  between the two baselines is a difference to name.
- **The evening block pasted before 20:00**, not after the run has
  started as on this night. Stage 1 of this baseline ran with the sync
  clients on and the system's services free to use the performance
  cores. Its rows show no sign of it, but an exact repetition removes
  the question.
- **Compare like with like**:
  - sequential rows by their counters first (deterministic) and their
    times second;
  - parallel rows only against the same stage's one-thread rows;
  - LLTP passes by the counts of decided problems per collection and
    reason.

## Verification

- **The second session, by day** (2026-09-30, before the night):
  - `cargo clippy --workspace --all-targets -- --deny warnings` is clean.
  - `cargo test --workspace` passes, 114 core tests among them.
  - `nix flake check` printed "all checks passed!" on the script and
    documentation changes. After the last script edit (the paused user
    timers), the `treefmt` check, the only one that reads the script, was
    rebuilt and passed.
  - The three armed test runs behaved as "The runs" says.
- **The baseline**: complete, `RESULTS.md` written at 06:45. It reports
  no mismatch on any generated family. Each of the 23 LLTP mismatches
  is a header that contradicts its problem: 14 known from the first
  run, 9 new ones refuted by classical countermodels.
- **In the morning** (2026-10-01): `nix flake check` printed "all checks
  passed!" with the results, the script fix and this report in the
  tree.
- **Against the preliminary rows**: the same verdicts and counters on
  every one of the 1 543 problems both decided.

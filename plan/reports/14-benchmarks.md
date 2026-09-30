# Step 14 report: benchmarks, LLTP input and the hard families

Sessions of 2026-09-30. The code is done and checked; the baseline itself
is not: it was scheduled for the night of 2026-09-30 to 10-01, and this
report gives the preliminary numbers of an interrupted run on a shared
machine, marked as such.

**Note of the planning session, 2026-09-30:** the night's run was
cancelled, since the machine is no longer free. The step is therefore
incomplete in one respect: no baseline is recorded. "Tonight" below
describes a run that did not happen and stays as the procedure for the
night it does (step 14c of the plan, after 14b). The rows behind
"Preliminary numbers" are committed as `bench/preliminary/*.csv` with
their summary in `bench/preliminary/README.md`.

## Outcome

- **LLTP input**: `linlog::lltp::read` (feature `parse`) reads the LLTP
  library's `fof(name, role, formula).` files into a `Sequent` and the
  status the header claims. Every file of the library up to 2 MB reads
  (4 336 files, checked by the reviewer; the six Petri nets with dotted
  names since the fix below), except SYJ206+1.018, which is malformed in
  the library itself (one parenthesis short); the 222 larger files, up to
  103 MB, are left to the runs, where the largest outgrow the time or
  memory a run gets. `Reading::new` refused none of the 4 022
  intuitionistic problems the partial run reached.
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
- **The baseline**: `bench/baseline.sh --detach --fresh` runs it as a
  systemd user unit and writes `bench/results/*.csv` and
  `bench/RESULTS.md`. Not yet run to its end; see "Tonight".
- **Checks**: the flake's new `bench` check runs the harness on the
  smallest instance of every family and on the committed problem file and
  fails on any verdict against a known one; the `lltp` package fetches
  the library at a pinned commit.
- **One engine fix** (a deviation, below): the focused engine now polls
  its stop condition during long split enumerations; before, a 2 s limit
  ran past five minutes.

Commits, in order: "Read LLTP problems", "Generate the hard families",
"Poll the stop condition during long split enumerations", "Add the
benchmark harness", "Document the benchmarks", "Detach the baseline and
check that the machine is idle", "Extend the baseline to an overnight
run", "Record CPU time and run-queue wait per run", this report, "Shield
the benchmark cores and record the machine's state", and its amendment.

## Tonight: what the administrator does

The machine is the baseline's from about 20:00 to 07:00. The run takes
about eight and a half hours.

0. Keep the laptop on mains power with its lid open and its vents free
   (see "The machine and its noise": it throttles thermally, and a closed
   lid is now inhibited but still worse for cooling). Optionally, as root,
   keep the system's own services off the performance cores for the
   night: `sudo systemctl set-property --runtime system.slice
   AllowedCPUs=4-15` and the same for `init.scope`, undone in the morning
   with `AllowedCPUs=` (or by a reboot); and pause the two sync clients,
   which may hash files for minutes when something changes: `sudo
   systemctl stop syncthing` and `systemctl --user stop
   app-insync@autostart.service`, started again in the morning.
1. Pause the scheduled jobs that fall into the slot: `nix-gc` (daily at
   00:00, up to 5 min later; 20–80 s, one core, 1–3 GB read) and
   `nix-optimise` (03:45, up to 30 min later; 15–40 s, 5 GB read, up to
   5 GB of memory), and the user timer `obsidian-snapshot` (23:00, a
   commit of the vault):
   `sudo systemctl stop nix-gc.timer nix-optimise.timer` and
   `systemctl --user stop obsidian-snapshot.timer`, started again in the
   morning (both system timers are `Persistent`, so a missed run happens
   when they start). The other timers (`logrotate`, `fwupd-refresh`,
   `systemd-tmpfiles-clean`, the midday `borgbackup` at idle priority,
   `fstrim` on Mondays) are negligible or outside the slot. The script
   refuses to start while a non-negligible timer is due within nine hours
   or the load average is above 1, unless given `--force`.
2. From the devshell, with the library fetched once (`nix build .#lltp -o
   bench/lltp`): `bench/baseline.sh --detach --fresh`. Follow it with
   `journalctl --user -fu linlog-baseline`: every ten minutes every run's
   last progress line with its estimate of the time left, the load, and
   whatever else uses a CPU. `systemctl --user stop linlog-baseline` stops
   it; running the script again without `--fresh` finishes an interrupted
   baseline, since every run skips what its CSV file already has.
3. In the morning: `bench/results/*.csv` (now tracked; the progress logs
   are not) and `bench/RESULTS.md`, whose header records the duration, the
   load at the start and the scheduled jobs that fired, are committed as
   "Record the baseline", and the numbers below are replaced by the
   night's in this report.

The stages (the sequential ones in four streams pinned to the performance
cores, CPUs 0 to 3 of the Intel Core Ultra X9 388H; the parallel ones
alone):

| stage | what | about |
|---|---|---|
| 1 | every family at its sizes (300 s each, fast runs thrice); focus against net on the MLL families and the problem file; the net engine's test period 1, 2, 8, 16; the intuitionistic mode of the counter, chain and 3-Partition families; the whole LLTP library intuitionistically and classically (5 s each) | 1 h 40 |
| 2 | the largest sizes that time out at 300 s, once each with 20 min; the LLTP problems that ended at the copy bound again with a bound of 10; those that ended at the recursion limit again with a limit of 16384 | 1 h 30 |
| 3 | the hard families on 1 (alone, the speedups' baseline), 2, 4, 8 and 16 threads (120 s); the net engine's cubes on the Partition table and the MLL 3-Partition; the LLTP problems that timed out or took 50 ms or more, on 16 threads with and without the portfolio | 5 h |

Stage 2 and the LLTP part of stage 3 rerun only the problems a knob can
change (the reruns select them from stage 1's rows), which is what keeps
the night to eight hours rather than fourteen.

## The machine and its noise

The author asked whether the processor's cores and the system's services
can distort the numbers. The facts, from `lscpu`, sysfs and the system
flake's `power.nix` and `idle-lock.nix`:

- **16 physical cores, no SMT**, of three kinds: CPUs 0–3 are performance
  cores (up to 5.1 GHz, a 3 MB L2 each), 4–11 efficiency cores (4.0 GHz,
  two clusters of four sharing a 4 MB L2), 12–15 low-power efficiency cores
  (3.7 GHz, one shared L2 and no share of the 18 MB L3). "Eight physical
  cores" would matter if the pinned streams shared cores through SMT; they
  do not. The kernel reports SMT as not supported, and every CPU is its
  own sibling.
- **The four sequential streams share the L3 and the package's power and
  heat**, so one stream is somewhat slower than it would be alone (less
  turbo headroom). The effect is the same for every sequential row of a
  stage, so their comparisons hold. For the speedups it does not hold,
  since those compare a run under four-stream load with a run alone:
  stage 3 therefore runs the hard families on one thread too, alone,
  which is the baseline its parallel columns compare against.
- **"Every core" is heterogeneous.** Sixteen threads include the four
  low-power cores, which are slower and outside the L3. Speedups flatten
  above eight threads for that reason, not only because of the search,
  hence the eight-thread column.
- **Heat.**
  - The package has throttled 406 438 times, for 6 258 s in all, since
    the last boot. That uptime included the morning's runs, and this is a
    laptop under long all-core load.
  - Throttling lowers the clocks in stage 3 above all, and it is not
    constant over a night.
  - Turbo stays on: without it the night's work would take roughly twice
    as long and not fit the slot.
  - The script records the throttling instead: the package's throttle
    count and throttled seconds go into the journal every ten minutes and
    into `RESULTS.md` for the whole run, with the platform profile,
    governor, energy preference and turbo state (today `performance`,
    `powersave`, `balance_performance`, on: TLP's profile on mains, which
    lets the hardware pick the clock).
  - A later baseline is comparable to this one only under the same
    settings, which is why they are recorded.
- **Background services.**
  - The system runs NetworkManager, syncthing, journald, udisks, fwupd
    and the nix daemon; the user session runs the compositor, the editor
    daemon, insync (a Google Drive client), pipewire and portals. At rest
    they cost a few percent of one core, in short bursts; the sync
    clients can hash files for minutes when something changes.
  - A detached run moves the user's other slices (`app.slice`,
    `session.slice`, `background.slice`) onto CPUs 4–15 for its whole
    duration. It gives them back in the unit's `ExecStopPost`, which runs
    however the run ends (a stop killed the script before its own trap
    could). The system slice needs root (step 0 above), and kernel
    threads and interrupts stay where the kernel puts them.
  - Every run records `wait_ms`, the time its thread was ready but
    waiting for a CPU. The summary marks with `†` every sequential
    problem whose run waited for over 1 % of its time, and counts them at
    the top of `RESULTS.md`, so a disturbed number shows itself.
  - Runs under two seconds are repeated three times and the median taken,
    which absorbs a burst. Longer runs are taken once, and a few
    milliseconds of noise are well under 1 % of them.
- **Scheduled jobs**: see step 1 above. The script refuses to start while
  one is due, and records those that fired.
- **Suspend.**
  - The idle manager suspends only on battery, after 20 minutes.
  - Logind suspends on a closed lid even on mains (`HandleLidSwitch=
    suspend`, no rule for external power).
  - The unit holds a `sleep:idle:handle-lid-switch` inhibitor, and the
    script refuses to start on battery. The journal line says `BATTERY`
    if the power goes during the run.

What remains uncontrolled: the kernel's own threads and interrupts, the
package's heat, and whatever a service decides to do outside the pinned
cores during stage 3. The records above make all three visible after the
fact. A disturbed family instance can be rerun alone the next morning by
deleting its rows and running the script again: it resumes.

## How it runs

```sh
nix build .#lltp -o bench/lltp                       # the library, 1.1 GB, once
cargo run --release -p linlog-bench -- families      # the families and their sizes
cargo run --release -p linlog-bench -- run --family partition-no=3,4 \
  --engines focus,net --jobs 1,4 --timeout 10 --output runs.csv
cargo run --release -p linlog-bench -- run --lltp bench/lltp/ILL/KLE-cbn --modes given,classical
cargo run --release -p linlog-bench -- summary runs.csv
bench/baseline.sh --detach --fresh                   # the baseline, as above
```

`run` takes `--family NAME[=SIZES]` (or `--all-families`), `--lltp PATH`
(a problem under a directory named `ILL` is intuitionistic),
`--problems FILE` (lines `name; mode; expected; copies; sequent`, as
`bench/problems/slow-tests.txt`), `--only TEXT,…`, `--reverse`, `--modes
given,classical,intuitionistic`, `--engines auto,focus,net,two-sided,
additive`, `--jobs 1,2,…,all`, `--portfolio`, `--copies`,
`--test-period`, `--recursion-limit`, `--timeout`, `--repeat` with
`--repeat-under`, `--output` with `--append` and `--resume`.

**Timing.** `time_ms` is wall-clock time (`Instant`) around `prove_until`
in the child: forest construction and pool start-up included, parsing,
the proof check and process start excluded; the time limit is wall-clock
too, polled by the stop closure, and the parent kills a child that
outlives it by a tenth plus five seconds. `cpu_ms` is the process's CPU
time over the same span (all threads, 10 ms ticks) and `wait_ms` the time
the calling thread was ready but waited for a CPU, so a sequential run
that another process slowed down shows itself; the morning's run predates
these two columns.

## Preliminary numbers

From the run of 2026-09-30, 08:12 to 09:34, stopped before its end: the
four sequential streams pinned to CPUs 0 to 3, the machine shared with an
editor and an OCaml language server (light, but not idle), and a release
build without the CPU-time columns. Complete: every family on one thread,
the engine comparison, the test period, the intuitionistic mode; cut off:
the LLTP passes at 4 022 of 4 495 problems (intuitionistic) and 3 792 of
4 512 (classical); not started: the parallel runs, the portfolio, the
raised copy bound. Treat every time below as an upper bound with some
noise; the verdicts do not depend on the machine.

**Families, one thread, default engine, 300 s** (the largest size of every
family times out and the others do not, as intended, except `wide-m4`,
whose size 36 is added for tonight):

| family | largest decided | first undecided |
|---|---|---|
| 3-partition-no (Horn, bins of b) | b = 4: 52.6 s | b = 5 |
| partition-yes (n items) | n = 6: 12.9 s | n = 7 |
| partition-no | n = 4: 115 ms | n = 5 |
| qbf (n variables, 4 instances) | n = 16: 1.5–8.1 s; n = 20: 88–125 s for three | one n = 20, all n = 24 |
| wide-m3 (focus) | k = 30: 10.8 s | k = 36 |
| wide-m4 (focus) | k = 32: 44.7 s | (k = 36 tonight) |
| mix (k pairs under Mix) | k = 10: 217 s (×9–10 per pair) | k = 11 |
| counter (n tokens) | n = 8: 129 ms | n = 16 |
| counter-over | copy bound at n = 8 after 492 ms | n = 16 |
| growing (bound b) | copy bound up to b = 256 | b = 1024: recursion limit after 0.6 s |
| chain (k clauses) | k = 256: 2.9 s | – |
| additive (depth d) | d = 16: 533 ms | – |
| 3-partition-yes, 3-partition-mll-*, wide-m1, wide-m2 | every size, milliseconds or less | – |

**Intuitionistic against classical** on the same sequents: the counter
with 8 tokens 16.4 ms against 128.6 ms (7.8×, step 8 measured 6×), its
unreachable variant 18.5 ms against 492 ms (27×), the unsolvable Horn
3-Partition with bins of four 25.5 s against 52.6 s (2.1×), the chain
the same both ways. Across the LLTP problems reached in both modes the
two engines decide the same problems, except six Petri nets decided only
intuitionistically, never a different verdict, and where both take over a
millisecond the classical search is 1.47× slower in the median (65 Petri
nets). So the two-sided constraint pays across ILLTP, most on Horn-like
problems.

**Focus against net on unit-free MLL**, 60 s:

| problem | literal multiplicity | focus | net |
|---|---|---|---|
| 3-partition-mll-no, b = 4 / 5 | 8 / 10 | 17 µs / 17 µs | 1.84 s / 57.2 s |
| 3-partition-mll-yes, b = 8 / 12 | 16 / 24 | 35 µs / 36 µs | 5.87 s / > 60 s |
| partition-yes, n = 4 / 5 / 6 | 8 / 12 / 16 | 2.4 ms / 183 ms / 12.6 s | 2.76 s / > 60 s / > 60 s |
| partition table 2-3-2-1 / 1-2-5 | 8 / 8 | 5.9 ms / 5.8 ms | 4.52 s / 3.67 s |
| wide-m1, k = 32 / 256 / 2048 | 1 | 45.6 s / too wide / too wide | 194 µs / 9.3 ms / 639 ms |
| wide-m3, k = 30 | 3 | 10.9 s | 163 µs |
| wide-m4, k = 28 | 4 | 2.75 s | 267 µs |

The net engine loses by two to five orders of magnitude wherever equal
literals sit in one `⊗` or `⅋` tree (every Horn encoding) and wins by as
much where they are spread over conclusions (the wide sequents); the
focused engine cannot split a context of more than 63 members at all.

**The net engine's test period** (default: every link up to 200
occurrences, every fourth above): on the small Horn structures a period
of 2 is 1.3× faster on one family and 2.3× slower on another, and 8 or 16
are three to ten times slower or time out; on the wide structures of 1 791
and 14 335 occurrences a period of 8 or 16 is 1.3–1.4× faster than the
default of 4 (450 ms against 639 ms at 14 335). The default holds; a
longer period on large structures is a small follow-up, and no CLI flag
is needed (`Options::test_period` serves library users).

**LLTP, one thread, 5 s** (intuitionistic pass, 4 022 of 4 495 problems):
612 proved and 99 refuted (18 %); unknown: 888 at the copy bound of 3,
895 at the recursion limit of 2 048 (Petri nets whose markings are long
tensors), 651 with a context of more than 63 formulas to split (Petri
nets), 835 at the time limit, 41 killed (the largest files, whose parse
alone exceeds the limit); 1 malformed. Per collection: KLE-IMP-CONJ 162 of
222 (every other one at the copy bound), its ALT and NON-THEOREMS all 49,
KLE-01/cbn/cbv 50, 68 and 67 of 88, ILLTP-SYN 11, 16 and 16 of 19,
ILLTP-SYJ 11, 34 and 39 of 252, ILLTP-LCL 1 of 6, the Petri nets 184 of the
2 664 reached. The classical pass decides the same problems in the
collections it reached.

**Mismatches with the headers**: 14 verdicts contradict what an LLTP
header claims, all of them the library's doing, none an engine's:
KLE065 (three translations) and SYJ212+1.001 (cbn) are marked
Non-Theorem, but linlog's proofs pass the checker and LLTP's own result
files prove them too; SYN001 (three translations) is marked
intuitionistic Non-Theorem and is proved with checked proofs as well;
KLE013 (three) and SYN041 (three) are marked Theorem but have `0` as
their goal and no hypothesis that can produce `0`, so no proof exists
(LLTP's own prover times out on them); SYN915 in its cbn translation turns
`$true` into an atom `T`, which LLTP's own results call false too.

## What the numbers say about each engine

- **The focused engine** decides every Horn encoding the net engine
  cannot, but it enumerates `⊗` splits: the unsolvable 3-Partition with
  bins of five, Partition with seven items, Mix over eleven pairs, and
  QBF over 24 variables do not finish in 300 s, and a context of more
  than 63 formulas is out of reach. On the counter program the memo
  answers millions of visits from thousands of entries (step 7's
  observation, unchanged). One stable sequent of a 20-token Petri net
  enumerated 60 million splits in 2 s, which is where the missing poll
  was found.
- **The net engine** is linear on distinct atoms and wide contexts
  (14 335 occurrences in 0.64 s) and exponential in the symmetry of equal
  literals within one tree.
- **The two-sided engine** gains 2× to 27× on the Horn-like families and
  1.5× in the median on ILLTP's Petri nets over the classical search of
  the same sequents.
- **The additive path** decides the depth-16 identity (131 071-leaf
  trees) in 0.53 s but holds a memo of about 8 GB doing it; depth 18
  would need about 130 GB (the reviewer measured it), so the family stops
  at 16. Its memo has no cap: a follow-up.
- **Parallel runs and the portfolio**: not measured yet; stage 3 tonight.

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
- **The baseline is not in this report yet**, for the reasons below.

## The runs, and why the baseline is pending

A first run started at 01:39 as four pinned streams. At 02:03 the
kernel's OOM killer stopped a reviewer's scratch checker at 62 GB, and
systemd failed the whole terminal scope, which ended the session and the
benchmark with it. A second run, started in the morning as a systemd unit
from 08:12, shared the machine with other work and was stopped at 09:34
on request; its numbers are the preliminary ones above. The harness now
guards against both: the baseline runs detached (`--detach`, with 24 GiB
and no swap for the unit and `OOMPolicy=continue`), every process is
capped at 12 GiB (`prlimit`), the script refuses a busy machine or a
scheduled job due during the run, and every run records its CPU time and
run-queue wait. Scratch programs next to a baseline go into a memory-
capped scope of their own on the efficiency cores (`systemd-run --user
--scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 4-15 …`).

## Review

A fresh-context reviewer, in two rounds:

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

## Open questions and follow-ups; what step 14b must beat

- **The candidates of 15a' and 15b' against these rows**:
  - For the focused engine, the refutations that take seconds to minutes
    today: `3-partition-no/4` (52.6 s, with `/5` undecided), `mix/10`
    (217 s) and `qbf/20` (88–125 s); also `counter/16`, which step 7 and
    this step leave undecided, and `counter-over/8` (0.49 s to its bound).
    These are where the atom bias per problem, branch-and-bound splits, a
    canonical choice among identical members and nested forced factors
    should show.
  - For the net engine, the Partition table and `3-partition-mll-no/5`
    (57 s), for leaf symmetry breaking. Beyond that, the routing feature
    above.
  - Tonight's stage 2 adds the times of the next sizes with 20 minutes
    each.
- **The recursion limit binds on 895 Petri nets** (a marking of a hundred
  tokens is a tensor chain as deep), and **the 63-member limit on 651**.
  Stage 2 tonight measures the first with a limit of 16 384. The second
  needs the lazy contexts or the branch-and-bound split.
- **The additive memo is unbounded** (8 GB at depth 16). It needs a cap,
  or no memo for `&` pairs.
- **The portfolio, and all-cores on LLTP**: stage 3 tonight; the step-13
  report's speedups are what the parallel columns should reproduce
  (3-Partition refuted 6.0× on 8 threads, the net engine's Partition
  instances 7.4× and 6.7×).
- **LLTP data**:
  - the 14 header contradictions above are worth reporting upstream;
  - SYJ206+1.018 is malformed;
  - the largest SYJ files (up to 103 MB) take longer to parse than a run
    gets.
- **D3** (index width): nothing in these numbers points at the `u32`
  indices; the limits that bind are the recursion depth and the split
  width.

## Verification

- **Clippy and the tests** (`cargo clippy --workspace --all-targets --
  --deny warnings`, `cargo test --workspace`) pass at the last code
  change, with 114 core tests.
- **The flake:** `nix flake check` passed ("all checks passed!") at
  "Extend the baseline to an overnight run". After "Record CPU time and
  run-queue wait per run", the `bench` and `clippy` checks were rebuilt
  and passed.
- **The detached path:** it was started once and stopped after a minute.
  The unit's limits, the pinning, the 12 GiB caps, the reversed classical
  pass and the time-left estimates were as intended.
- **Shielding and inhibition:** tested the same way, twice.
  - The first test found that the slices kept their restriction after a
    stop: the script's trap was killed.
  - The restore now lives in `ExecStopPost`. The second test showed the
    slices restricted during the run and restored after the stop, the
    inhibitor listed by `systemd-inhibit --list`, and the unit in
    `linlog.slice`.
- **The refusal path:** the script refused at 09:43 with the midday
  backup due.
- **The harness verdicts:** no mismatch on any generated family in either
  run.
- **Not yet verified:** the baseline itself, which runs tonight.

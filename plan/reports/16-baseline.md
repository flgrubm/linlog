# Step 16 report: the baseline again, and what the performance pass changed

Session of 2026-10-02 and the morning of 2026-10-03. The second baseline
is taken, complete, in one night on one commit. It ran from 23:20 to
07:24 (8 h 4 min) and measured commit `e5d930cf12dd`, whose engines are
those of step 15's last code commit. The results are in
`bench/results/2026-10-02/` and the comparison with the first baseline
is in `bench/COMPARISON.md`.

## Outcome

- **No verdict differs.** On every problem that both baselines decide in
  the same configuration, the verdict is the same, in every file.
- **No loss.** Every problem the first baseline decided, the second
  decides too, in every file, within the limit wherever the first did.
  The comparison prints both lists ("Verdicts that differ", "Decided by
  before, not by after"), and both are empty. An awk count over the raw
  rows confirms it.
- **The gain is in the Petri nets.**
  - The intuitionistic LLTP pass decides 2 049 of 4 495 problems within
    5 s, up from 737. The Petri nets go from 210 to 1 520 of 3 137.
  - The classical pass decides 2 008, up from 742.
  - Outside the nets, two problems are gained (one each in LCL-cbn and
    SYJ-cbn) and nothing changes: those collections still end at the
    copy bound of 3.
- **The families became easy.** Every size the first baseline timed out
  on is decided now, mostly in microseconds; only `mix` at eleven pairs
  is not. The families got larger sizes; the new largest that time out
  at 300 s are listed below. Two families have no size that times out,
  and the counter's largest size, 64 tokens, is proved in 81 s.
- **The default keeps its contract almost everywhere.** It decides
  everything the backward search alone (`--bias rarer`) decides, except
  two nets near the limit. Against the forward search alone at 30
  copies, the default leaves 67 nets undecided at the time limit and
  288 problems outside the nets at its copy bound of 3.
- **New defects, none of them a wrong verdict:**
  - **Memory.** 76 runs ran out of their 12 GiB of address space. Most
    are large nets that the first baseline stopped at the recursion or
    width limit within milliseconds, which the search now takes on and
    fills 12 GiB with in 4 to 6 s.
  - **A missed stop in the forward search.** It misses its stop on the
    GPPP-1000 nets: killed past 10.5 s under a 5 s limit, or proved up to
    a second late.
  - **A missed stop on a pool.** Sixteen threads still miss their stop on
    sixteen small SYJ problems in the cbv translation, as they did in
    the first baseline.
- **Threads and the portfolio.**
  - Every core against one thread, on the 1 102 LLTP problems of the
    all-cores run: 37 more decided, none lost, 2.9 times slower in the
    median on the problems both decide.
  - The portfolio gains nothing again: 10 gained, 6 lost, the same time.
  - On the families the speedups are mostly gone.

Commits, in order:
- "Extend the families past the sizes the focused engine now decides at once"
- "Take the reruns of a baseline from the first one's problems, and pass the library under each bias alone"
- "Record the target set on the engine of the second baseline"
- "Record the baseline after the performance pass"
- "Compare two baselines in the summary"
- "Compare the baseline after the performance pass with the first"
- the documentation
- this report

## How the night was taken, and how it compares

**The same set-up as the first baseline:**
- the same machine, the 16-core Intel Core Ultra X9 388H laptop, on mains;
- the same settings: platform profile `performance`, governor `powersave`,
  energy preference `balance_performance`, turbo on (`RESULTS.md`
  records them);
- the same toolchain and lock files: `rust-toolchain.toml`, `Cargo.lock`
  and `flake.lock` are unchanged since `b53cb17c6831`;
- the same LLTP library, at the same store path;
- the same timers in the slot.

The author pasted the evening block before the start: nix-gc,
nix-optimise and syncthing were stopped and the system slices kept off
CPUs 0 to 3. The unit started with a load average of 0.24, and no
scheduled job ran during the run.

**The slot.** The author gave the machine from 20:30 with no end
cutoff. The run was armed for 20:30, then postponed twice at the
author's request (to 21:00, then 21:30) and disarmed. When the author
said to start, it was armed two minutes ahead with a safety stop at
13:18. The unit waited for an idle machine both times it fired before
the postponements and measured nothing then. The results directory was
created empty and recreated by `--fresh` on every arming, so
`starts.txt` has the one start.

**The set-up check.** Before the night, `bench/targets.sh
baseline-2026-10-02` ran on two pinned cores (twelve minutes). It
reproduced every one of the 99 decided rows of
`bench/targets/after-bias.csv`: the same verdict, stable sequents,
splits, memo hits and memo entries. The code changes since that file
were two doc comments.

**The dry run.** Before arming, a copy of the modified script ran end to
end on efficiency cores with every limit cut to 0.2 s and every problem
list cut to two entries (a wrapper around the harness), in under three
minutes, with exit 0. No row crashed, errored or was killed, and no
verdict contradicted a known one. `nix flake check` passed on the
commit measured.

**The durations.** The estimate was arithmetic: step 15's sampled times
per ending category of the first baseline, scaled by the category sizes,
and the families' new sizes from one-minute checks. It said about 3.5 h
for stage 1, 1 h for stage 2, a little over 3 h for stage 3 and minutes
for stage 4. The script's `estimate` became 10 h as an upper bound.

| stage | took |
|---|---|
| 1, four streams: the two default LLTP passes, the two passes under one bias, families, engines, test periods, intuitionistic | 3 h 34 min (23:20–02:54) |
| 2, four streams: the long runs, `lltp-copies-10`, `lltp-recursion` | 1 h 03 min |
| 3, alone: the thread table, the net engine's cubes, every core and the portfolio on LLTP | 3 h 19 min |
| 4, the reruns of `bench/reruns.txt` | 8 min |

The first baseline took 9 h 21 min for stages 1 to 3 and 1 h 28 min for
its supplement.

**Throttling.** The package throttled 523 047 times for 7 074 s, against
11 283 s over the first baseline's longer night.

**The CPU waits do not measure disturbance here.** On the default runs
of a sequent with exponentials, `wait_ms` stops saying what it said on
the first night:
- The default runs its second search on a thread of its own. While that
  search runs alone, the calling thread wakes every millisecond to poll
  the caller's stop condition.
- Pinned to one core, each wake-up queues behind the running search, and
  `wait_ms` counts the queueing.
- The default LLTP passes show 11.5 % of their time as waits, and the
  summary marks 5 191 problems with `†`.
- The passes with one search show 0.07 % (`--bias rarer`) and 0.14 %
  (`--bias factors`), and families, engines and the long runs 0.05 to
  0.11 %: no other process slowed the night.

So the `†` marks of `RESULTS.md` on those rows say nothing.
`.claude/rules/bench.md` now says so. Whether the polling costs the
search measurable time on one core was not measured.

## What changed in the script, and why

Only what "What step 15 left you" names:

- **The reruns take the first baseline's problems.** `ended` reads
  `reference`, which is `bench/results/2026-09-30/`. So
  `lltp-copies-10`, `lltp-recursion`, `lltp-all-cores` and
  `lltp-portfolio` ran on exactly the first baseline's problems: 1 003,
  995, 1 102 and 1 102 rows, every one with a counterpart.
- **Two passes under one bias.**
  - `lltp-rarer` (`--bias rarer`): the backward search alone.
  - `lltp-forward` (`--bias factors --copies 30`): the forward search
    alone at the default's forward bound.

  These two are the longest passes, so each has a stream of its own
  from the start. The families run after the default intuitionistic
  pass and the engines after the classical one. That kept stage 1 at
  about 3.5 h, where putting both after the families and engines would
  have made it about 4.5 h. Every sequential row still ran in four
  pinned streams, as on the first night.
- **Larger sizes.** They were found by arithmetic from `bench/TARGETS.md`
  and one-minute probes on one performance core, capped.
  - New default sizes: `partition-yes` 12, 16, 20, 24, 28;
    `partition-no` 9, 12, 14, 15; `qbf` 32, 40, 44, 48; `counter` and
    `counter-over` 32, 64; `wide-m3` and `wide-m4` 256, 1 024, 2 048.
  - The explicit lists of the script gained a selection of these: the
    engines, the long runs and the thread table.
  - Every old size stays. `3-partition-no` keeps its sizes: the probe
    showed 971 stable sequents at every bin size up to 48 (0.78 ms), so
    no size of it times out.
- **`bench/reruns.txt` unchanged**, as the prompt asks, so that the
  `-generous` files compare.

## The comparison

`linlog-bench summary --before DIR FILES` and `--against FILE FILES`
(`bench/src/compare.rs`) print it. `bench/COMPARISON.md` has their
output under a hand-written "At a glance". The main tables follow.

### Families, one thread, 300 s

| family | largest decided, first | largest decided, second | a size both decide |
|---|---|---|---|
| 3-partition-no | b = 4: 49.8 s | every size, under 1 ms | b = 4: 49.8 s → 299 µs |
| partition-yes | n = 6: 12.7 s | n = 24: 118 s (28 > 1 200 s) | n = 6: 12.7 s → 130 µs |
| partition-no | n = 4: 114 ms (5 in 811 s at 1 200 s) | n = 12: 3.87 s (14, 15 > 300 s; 15 > 1 200 s) | n = 5: 811 s → 477 µs |
| qbf | n = 20: three of four, 87–124 s | n = 48: three of four, 230–282 s | n = 20 #0: 87.2 s → 7.3 ms |
| wide-m3 | k = 30: 10.6 s | k = 1 024: 55 ms (2 048: recursion limit) | k = 36: 697 s → 157 µs |
| wide-m4 | k = 32: 43.4 s | k = 1 024: 52 ms (2 048: recursion limit) | k = 36: 685 s → 144 µs |
| mix | k = 10: 214 s | k = 10: 146 s (11 > 1 200 s) | k = 9: 19.8 s → 14.1 s |
| counter | n = 8: 124 ms | n = 64: 80.7 s (two-sided 268 ms) | n = 8: 124 ms → 87 µs |
| counter-over | copy bound throughout | refuted to 16; copy bound at 32 and 64 (85 s) | – |
| chain, additive, growing | as constructed | the same verdicts, up to 8.8 times faster | chain/256: 2.82 s → 321 ms |

New sizes that time out at 300 s: `partition-yes` 28, `partition-no` 14
and 15, `qbf` 48 #3, `mix` 11. Three families have none:
- `3-partition-no`, constant in the bin size;
- the wide sequents, at the recursion limit at 2 048 literals;
- the counter, proved at 64 tokens. 128 tokens was not tried, since each
  doubling multiplies the time by about forty.

`qbf/48#0` ran out of memory after 290 s, both at 300 s and at 1 200 s.

**Focus against net** (`engines.csv`, 60 s). The focused engine now
matches the net engine on the wide sequents from 8 to 1 024 literals,
within a factor of three and faster from 256 on (`wide-m3/256`: 3.7 ms
against 9.7 ms). On every Horn encoding it takes microseconds where the
net engine takes seconds or times out: Partition with six items 130 µs
against over 60 s, MLL 3-Partition with five bins 20 µs against 57.7 s.
It loses only at 2 048 literals, where it meets the recursion limit and
the net engine takes 0.64 s. The net engine's own times are those of the
first baseline within 1 to 4 % on most rows and 12 % at most, its code
unchanged. Forced onto
`additive/16`, the focused engine still runs out of memory (9.9 s, 12.2
s before).

### LLTP, one thread, 5 s

| file | first | second | Petri nets |
|---|--:|--:|---|
| `lltp-intuitionistic` | 737 | 2 049 (and 2 late) | 210 → 1 520 |
| `lltp-classical` | 742 | 2 008 (and 2 late) | 204 → 1 468 |
| `lltp-copies-10` | 411 | 458 | 59 → 89 |
| `lltp-recursion` | 56 | 280 | all nets |

Per collection, in the intuitionistic pass:

| collection | problems | first | second | where the others end now |
|---|--:|--:|--:|---|
| KLE (six directories) | 535 | 396 | 396 | the copy bound |
| ILLTP-SYJ (three translations) | 756 | 84 | 85 | 567 copy bound, 54 recursion limit, 42 time, 8 killed (the largest files' load) |
| ILLTP-SYN | 57 | 43 | 43 | the copy bound |
| ILLTP-LCL | 6 | 1 | 2 | the copy bound |
| misc, Non-theorems | 4 | 3 | 3 | the copy bound |
| Petri nets | 3 137 | 210 | 1 520 | 1 594 time, 2 copy bound, 7 killed, 14 out of memory |

Where the first baseline's endings went:

| first baseline | problems | second baseline |
|---|--:|---|
| decided | 737 | all decided |
| time limit | 984 | 529 decided, 455 time limit |
| copy bound | 898 | 91 decided, 727 copy bound, 80 time limit |
| recursion limit | 983 | 280 decided, 636 time limit, 54 recursion limit, 13 out of memory |
| too wide to split | 845 | 381 decided and 2 late, 454 time limit, 7 killed, 1 out of memory |
| killed | 47 | 29 decided, 10 time limit, 8 killed |
| not loading (the repaired file) | 1 | time limit |

Outside the nets the times are 1.0 to 1.4 times the first baseline's in
the median per collection, and 2.7 times in SYJ-cbn. These are problems
decided in microseconds to milliseconds. The likely cost is the default
starting its second thread on a sequent with exponentials; I did not
check it. The nets that both decide are 1.9 times faster in the median
(0.52×).

### The bias

The intuitionistic library, one thread, 5 s:

| | default | `--bias rarer` | `--bias factors --copies 30` |
|---|--:|--:|--:|
| decided | 2 049 | 969 | 2 393 |
| Petri nets decided | 1 520 | 442 | 1 576 |
| nets it decides and the default does not | – | 2 | 67 |
| nets the default decides and it does not | – | 1 080 | 11 |
| the default's time over its own, median on the nets both decide | – | 1.29× (440) | 1.64× (1 507) |
| other collections decided | 529 | 527 | 817 |
| of these, not decided by the default | – | 0 | 288 (the default at its copy bound) |

**Whether the contract holds at scale: not entirely.** The default
decides everything the backward search alone decides, except two nets:

- `ResAllocation_RAS-C-100_5_1`: `rarer` 2.71 s, the default over 5 s;
- `Diffusion2D_2D8_gradient_40x40_100_5_1`: `rarer` 4.50 s, the default
  over 5 s.

The second is the loss at the limit that step 15 foresaw (more than two
thirds of the limit). The first is not: 2.71 s is 54 % of the limit.
The two searches' shares are counted in work, and a unit of work costs
different time in the two searches.

**The rows closest to the limit held.** The NeighborGrid nets are proved
by the default in 3.80 to 4.00 s, against 1.45 s in the first baseline,
2.30 s under `rarer` alone and 3.39 s under the forward search alone.
`UtahNoC_5_1` and `_10_1` take 0.93 s.

**What the alternation costs.** On the nets both decide, 1.29 times the
backward search alone and 1.64 times the forward search alone, in the
median.

**What it leaves.**
- 67 nets that the forward search alone decides within 5 s and the
  default does not.
- 288 problems outside the nets that the forward search decides at 30
  copies. The default keeps the forward search to `--copies` there,
  since they are no Horn programs. That is the copy-bound question
  below, not the bias's.

### The raised limits

- **`lltp-copies-10`.** Its rows outside the nets are what the copy bound
  of 3 leaves. A bound of 10 decides 369 of these 832 problems, against
  352 in the first baseline; 293 time out and 170 stay at the bound. The
  forward search at 30 copies decides 288 such problems on its own. For
  step 17's question whether `--copies` should rise, the answer from this
  data is that a larger bound decides about four in ten of what 3 leaves
  within 5 s.
- **`lltp-recursion`.** 280 nets decided, against 56. The raised
  recursion limit matters little now: the default pass at 2 048 decides
  the same 280 of these problems, since only 54 rows still reach the
  limit.

### Threads and the portfolio

**The thread table** (stage 3, alone, 120 s; times on 1 / 2 / 4 / 8 / 16
threads):

| problem | 1 | 2 | 4 | 8 | 16 |
|---|--:|--:|--:|--:|--:|
| partition-no/12 | 4.01 s | 2.83 s | 1.73 s | 1.20 s | 0.85 s (4.7×) |
| partition-yes/24 | 106 s | > 120 s | > 120 s | > 120 s | > 120 s |
| qbf/44#0 | 72.5 s | 54.9 s | 56.0 s | 55.4 s | 55.0 s (1.3×) |
| counter/64 | 78.4 s | 102 s | 76.9 s | 22.6 s | 23.9 s (3.3×) |
| counter/32 | 2.18 s | 2.70 s | 1.98 s | 0.63 s | 0.70 s |
| mix/9 | 13.9 s | 17.4 s | 17.9 s | 18.6 s | 21.2 s |

The old sizes are decided in microseconds to milliseconds on every
thread count, with nothing for a pool to share out.

**Every core on LLTP** (the 1 102 problems), the default against itself
on one thread:
- 623 decided within 5 s on sixteen threads against 586 on one;
- 37 gained, none lost;
- 2.88 times slower in the median on the 586 both decide (quartiles
  0.37 and 6.91).

So the two searches on a pool of eight threads each lose no verdict, but
cost time on most small problems. In the first baseline, by the same
count: 29 against 28, 1.5 times slower.

**The portfolio:**
- 627 against 623, 10 gained and 6 lost;
- the same time (1.01×, quartiles 0.90 and 1.11).

It shows no gain for the second time, so step 15's recommendation to
remove it stands on two measurements.

### The first night's kills and crashes, and the generous files

| file | killed or crashed in the first baseline | now |
|---|--:|---|
| `lltp-intuitionistic` | 47 | 29 decided, 10 time limit, 8 killed (the largest files' load, as before) |
| `lltp-classical` | 50 | 32 decided, 10 time limit, 8 killed |
| `lltp-recursion` | 9 | 5 decided, 4 time limit |
| `lltp-all-cores` | 214 | 119 decided, 68 time limit, 1 copy bound, 23 killed, 3 out of memory |
| `lltp-portfolio` | 214 | 121 decided, 66 time limit, 1 copy bound, 23 killed, 3 out of memory |

**The generous files** (by verdict, reason and time):
- **`lltp-intuitionistic-generous`:** 11 of its 28 reruns proved, all
  within the limit.
- **`lltp-classical-generous`:** 15 of 34 proved, all within the limit.
- **`lltp-recursion-generous`:** 5 of 9 proved within the limit, where
  the first baseline had only two TokenRing proofs, after 21.6 s and
  552 s.
- **The all-cores and portfolio reruns:** end as before, at the time
  limit or the copy bound.
- **No generous run ends past its limit by more than 0.7 s.**

**Late verdicts** in the regular files:
- **On one thread:** the two GPPP-1000 nets, 0.7 to 1.0 s late, in each
  pass that runs the forward search.
- **On a pool:** four, between 9 and 228 ms late.

**The latest stops without a verdict:**
- **One thread:** 3.1 s past a 5 s limit (`PaceMaker_20_1` under
  `rarer`).
- **A pool:** 1.6 s past it.
- **First baseline, for comparison:** up to 552 s.

## What got worse, or did not improve

- **Out of memory, 76 runs:**
  - 14 nets in each default pass and in the forward pass, 3 in `rarer`,
    17 in `lltp-recursion`: BART, TokenRing-40 and -50,
    Philosophers-10000, AirplaneLD-pt-4000, GPPP-1000-1000 and others.
    The first baseline stopped them at the recursion or width limit in
    milliseconds; now they fill 12 GiB within 4 to 6 s.
  - `qbf/48#0`, after 290 s.
  - The largest SYJ files on a pool, as before.

  The memo's cap counts entries, not bytes.
- **The forward search misses its stop on the GPPP-1000 nets.** On one
  thread: 7 to 11 per pass killed past 10.5 s under a 5 s limit, 2 late
  proofs. Never under `--bias rarer`.
- **Sixteen SYJ202 and SYJ208 problems in cbv** (2 000 to 12 000
  occurrences) are killed on sixteen threads in both baselines, where
  one thread stops at 5.1 to 5.4 s.
- **`partition-yes/24` times out on every pool** where one thread
  proves it in 106 s. It is a new size, so there is no first-baseline
  counterpart.
- **Two contract losses at the limit**, above.
- **Small times outside the nets** are up to 1.4 times slower in the
  median (2.7 times in SYJ-cbn).
- **Mix** improved by a third (214 s → 146 s at ten pairs) and no more.
- **`wait_ms`** no longer flags disturbance on default runs with
  exponentials.

## What the numbers say the next engine step should be

1. **Robustness on large nets first.** Two defects take verdicts from
   real problems:
   - the memory of the search, a cap in bytes or a smaller
     representation of the memo's keys;
   - the stops: the forward search on the GPPP nets, and a pool on the
     SYJ cbv problems.

   Each is a precise target with named instances.
2. **The copy bound.** A default of 3 leaves 727 intuitionistic problems
   at the bound. A bound of 10 decides 369 of the 832 the first baseline
   left there, and 30 copies of forward search decide 288 of them, all
   within 5 s. Iterative deepening of `--copies` within the time limit
   (the focused engine deepens already) would take these without a flag.
   Step 17 decides.
3. **The parallel default.** Every core is 2.9 times slower in the median
   on the LLTP problems and gains 37 of 1 102. A sequential first
   attempt, or a smaller default, is supported by both baselines. The
   portfolio can go.
4. **The forward search's efficiency on nets.**
   - 1 594 nets still end at the time limit under the default.
   - The alternation costs 1.6 times the forward search alone.
   - The forward search alone decides 67 nets the default does not.

   This, not a new engine, is where the nets' remaining verdicts are,
   before a Petri-net reachability route.
5. **The net engine as a default route** matters only at 2 048 literals
   of the wide sequents. A loop for chains of free splits in the focused
   engine would take that case.

`plan/later.md` has these where the candidates' premises changed: the
net engine, the inverse method, the Petri-net route, the parallel and
the focused-engine follow-ups.

## Decisions

- **Two comparison modes in `summary`, not a separate command.** `--before
  DIR` matches files by name across baselines; `--against FILE` matches
  problems across the files of one baseline, which is what the bias table
  needs. Both print the same four sections. A late verdict counts as
  decided and is marked late in every count, as the prompt asks.
- **No library option was added.** The comparison is the harness's, a
  command-line tool with no front end beyond it, so D15 does not apply.
- **`bench/COMPARISON.md` beside `bench/RESULTS.md`.** `RESULTS.md` is
  regenerated by every baseline and must stay the latest's tables.
- **The reruns' reference is a fixed path in the script.** A later
  baseline that should compare with the second rather than the first
  changes `reference`.

## Deviations and assumptions

- **Stage 1's layout** differs from the prompt's suggestion: the bias
  passes have a stream each from the start. The reason is above; the
  rows of every stage still ran in four pinned streams.
- **The bias passes ran in stage 1 against the default passes** of the
  same night, as the prompt asks, not against the first baseline's.
- **The probes for the sizes** were the one-minute checks the prompt
  names, on cores 0 and 1 in capped scopes, while the target set ran on
  cores 2 and 3. The target set compares counters, which load does not
  change.
- **Some size choices missed their mark:**
  - `counter` and `counter-over` at 64 do not time out (81 s and 85 s).
  - `qbf/48#0` runs out of memory before its limit.
  - `partition-no` at 14 already times out, so 15 adds a second timeout.

  I did not iterate further by day.
- **The CPU-wait artefact** was found after the night. The disturbance
  check therefore rests on the rows with one search.

## Verification

- **Before the night:**
  - the target set reproduced `after-bias.csv` on all 99 decided rows;
  - the dry run of the whole script passed;
  - `nix flake check` passed (exit 0).
- **In the morning:**
  - **The journal:** one start at 23:20, load 0.24, no job fired, the
    stage times above, `finished 2026-10-03 07:24`.
  - **Verdicts and losses:** the comparison's verdict and loss lists
    were cross-checked with an awk count over the raw rows of every file
    (none differ, none lost).
  - **Code checks:** `cargo clippy --workspace --all-targets -- --deny
    warnings` and `cargo test -p linlog-bench` pass.
  - **The README's example of `summary --before`** is the command's
    output.
  - `nix flake check` after the results, the documentation and this
    report passed (exit 0).

# Step 19 report: the search keeps its time limit

Session of 2026-10-03, by day on the shared machine. The time limit of
the command and the stop condition of the library are now honoured
within half a second on every call the step names, on one thread and on
a pool, counted from the moment the command starts: the calls of the
assessment that missed their limits by half a minute to several minutes
end 0.03 to 0.36 s after them, and one of them is no longer a stop at
all, because the search behind it became four hundred times faster. A
pool of the net engine costs what one thread costs where the links are
forced, and is not slower than before on the baseline's rows. `--jobs`
is bounded, the portfolio is gone, and the `agree` test asserts what the
rules promise. One thing keeps the goal from holding without exception:
a search whose memo is full takes 0.15 to 0.55 s to free it, depending
on the core, both when it returns and, unpolled, whenever it empties
the memo in the middle of a search, so a stop can be up to about a
second late in the worst case. That is the memo's design and is handed
to step 20.

| the assessment's call | before | now |
|---|---|---|
| R6 `prove -i --jobs 1 --timeout 5s` on `GPPP_G-PPP-1000-10_10_1` | "provable" after 140 s | "provable" after 0.47 s (the search 0.33 s), the same 568 stable sequents and 4 754 641 splits |
| R7 the same flags on `SYJ202+1.008` (cbv), `--jobs 2` | ended after 46 s | 5.36 s |
| R7 `--jobs 4` | not ended after 150 s | 5.32 s |
| R7 `--jobs 1` | 5.17 s | 5.21 s |
| R10 `prove -i --jobs 1 --timeout 1s` on `SYJ212+1.020` (cbv, 95 MB) | ended after 88 s | 1.03 s: "unknown: the time limit of 1s was reached while the sequent was read", exit status 3 |
| the same with `--timeout 20s` (the parse takes 12.4 s) | – | 20.22 s |
| R13 `wide-m1` at 256 pairs, one thread and two | 8 ms and 1.2 s (512 and 131 328 links); on the cores used here 11.8 ms and 1.71 s | 12.1 ms and 11.9 ms, 512 links each (four threads: 11.4 ms) |
| R13 at 512 pairs | 33 ms and 9.5 s (1 024 and 524 800 links); here 45.6 ms and 13.6 s | 47.4 ms and 46.3 ms, 1 024 links each (four threads: 49.3 ms) |
| `prove --jobs 10000 "A |- A"` | 183 s of processor time | 3 ms, with a note that the search uses 16 threads |

All in scopes capped at 8 GiB without swap, pinned to the cores 4 to 7
(one of them for one thread), which are the machine's middle cores
(4.0 GHz, against 5.1 GHz for the cores 0 to 3 and 3.7 GHz for 12 to
15), with the release build of this step's last code commit; R6, R7 and
R10 are wall-clock times of the whole command, the R13 rows the
harness's. `nix flake check` passes at that commit, as do `cargo test
--workspace` (131 tests of the core crate with the `parallel` feature),
`cargo test -p linlog` without it (123), clippy, and `cargo hack` over
every feature alone and every pair.

## The cause of each miss

1. **The pool on the SYJ problem (R7): a stopped premise of `&` went on
   starting premises.** Found before anything around it was changed, by
   counting on the two-thread run: one second into the search, with the
   stop flag not yet raised, 5.3 million stable sequents had been
   answered "stopped" at their poll and 578 had really been searched; a
   backtrace of one of them was a tower of `with_parallel` calls. The
   `&` rule on a pool starts its right premise on a worker without
   waiting for the left one, and the asynchronous phase polls nowhere:
   only its stable sequents do. So a worker whose flags were raised (by
   a sibling's failure, which cancels it, or by the caller's stop) still
   decomposed its list, met the next `&` and started both premises, each
   of which did the same. With `n` `&` in one asynchronous phase that is
   `2ⁿ` workers, each ended by its first stable sequent. The cbv
   translation of `SYJ202+1.008` has such a tower; the first failed
   premise sets the cascade off, long before any limit, so the time
   limit was not what was missed: the search was busy with cancelled
   work when the limit came, and the scope could not end before the
   cascade had. Fixed by one poll at the entry of `with_parallel`. The
   baselines' kills on sixteen threads (SYJ202 and SYJ208 in cbv) are
   the same shape; they were not rerun.
2. **Forced chains (R6): three things, of which the poll was the
   least.** `forced_splits` and `literal_tensor` did not poll; they do
   now, once every 4 096 splits and literals. But on the GPPP net the
   chain is not where the time went. (a) Every lookup of a dual scanned
   the literal's list of occurrences from its head (`dual_in`). (b) The
   copy heuristic (`meets`) compared every literal below every formula
   of the unrestricted zone with every member of the linear zone: a
   quarter of a second per stable sequent on a marking of thousands of
   tokens under clauses of thousands of literals. (c) The command read
   its clock once every 1 024 polls, and this search makes 568 polls in
   all without the chain's (some 1 700 with them), so the limit was
   looked at once at most. With (a) and
   (b) linear the net is proved in 0.30 s, with the counters the
   assessment recorded for the run of 140 s.
3. **The command's clock (R6, R10, and the harness).** The stop closure
   of `prove`, of a session's `close` and of the harness's child looked
   at the clock every 1 024 polls (the harness every 64) on one thread.
   That is exact where the engine polls millions of times a second and
   wrong by `1 024 ×` the gap between two polls where a poll is slow: on
   a forest of 27.8 million occurrences a stable sequent takes about
   30 ms, which is the 32.8 s that the search of R10 reported under a
   limit of 1 s. The set-up before the first poll, which the assessment
   suspected, took 1.27 s of it. The harness's 5.2 to 7.5 s on
   five-second limits for the large nets are the same arithmetic with 64.
4. **The wait in `alternate`.** Of the two searches that alternate on
   one core, the one on the calling thread waited for the other's slice
   on a condition variable without a timeout. A slice is counted in
   work, and with `meets` as it was a slice of the backward search could
   take seconds.
5. **Before the first poll (R10).** The parse of the 95 MB file takes
   12.4 s and was outside the limit; the forest takes 0.43 s; the
   search's set-up took 1.27 s, 1.04 s of it `Counts::new`.
6. **The net engine's cubes (R13).** As the assessment said: the cubes
   were the branches of the first `d` links, enumerated from the root
   for `d = 1, 2, …` until there were sixteen per thread; with forced
   links there is one branch at every depth.

## What polls where

The rules file has the list under "Proof search: the front door"; in
short:

- **The focused engine** polls at every stable sequent, every 4 096
  steps of a split search (both as before), every 4 096 forced splits
  and literals closed in place (`poll_forced`, new), and on a pool at
  every `&` (`with_parallel`, new). The chain's poll passes no work to
  the stop condition, so the slices of the two alternating searches and
  with them every counter of a decided run are what they were.
- **The net engine** polls at every literal chosen (as before) and at
  every exact test that fails (new): a run of failed candidates chooses
  no literal.
- **The set-up** of a search on a forest of 65 536 occurrences or more
  polls after the dispatch, after the classes, after the plan and
  inside the counts every 65 536 occurrences (`set_up_stopped`,
  `Counts::new_until`). On `SYJ212+1.020` the first poll comes after
  0.12 s and no two are more than 0.2 s apart. A smaller forest is
  polled by the engines alone, so a stop condition that counts its
  polls sees exactly what it saw.
- **The two alternating searches**: the calling thread wakes once a
  millisecond while the other search has its turn and polls the
  caller's condition for the polls that search made (`pass_polling`).
- **A pool**: the driver polls the caller's condition once a
  millisecond on the calling thread, as before.
- **Not polled, measured**: the check of the proof found and the size
  pass of a derivation. On the largest proof the engines find in the
  library (`SYJ202+1.005` in cbv, 566 490 inferences) they take 22 ms
  and 39 ms; on the largest Petri nets proved (`PaceMaker_1_1`,
  `CloudDeployment_deploy_7_b_1_1`, `Dekker_dekker-200_1_1`,
  `ARMCacheCoherence_1_1`) 3 to 6 ms and 5 to 8 ms. Neither can pass the
  slack, so neither is polled.
- **Not polled, and what a stop is late by**: `Forest::new` for a caller
  of `prove_until` (0.43 s on the largest file; the command builds the
  forest under its own limit and calls `prove_goal`), and the freeing
  of a full memo: 0.15 to 0.35 s on the cores 4 to 7 for a memo at its
  default cap of 2²⁰ entries, which is all of R7's lateness, and 0.55 s
  on the machine's slowest cores (12 to 15). It happens when the search
  returns and, unpolled, in the middle of a search whenever a full memo
  is emptied (`qbf/40#1`: a gap of 0.51 s between two polls every two
  seconds). This is the one place where the goal is not met in the
  worst case; see the open questions.

## The limit counts from the start

`cli/src/limit.rs` has `Deadline`, the command's time limit as a flag: a
thread sleeps until the limit has passed and raises an `AtomicBool`, and
the stop closure is two loads, the Ctrl-C flag and this one, asked at
every poll. `prove` starts the deadline at its first line and runs the
load (reading, parsing, building the forest) through `Deadline::within`:
under a limit on a thread of a main thread's stack that the command
stops waiting for when the limit passes. The parser cannot be stopped
from inside, so the command answers without it, "unknown: the time limit
of 1s was reached while the sequent was read", exit status 3, and the
thread ends with the process. The line is the output in every format
but JSON, where it goes to standard error and standard output stays
empty, since an outcome without a sequent has no JSON form. A session's
`--timeout` stays a `close`'s; its sequent is read outside any limit.
The harness's child has the same flag.

For a library caller: the stop closure is polled from the first pass
after the dispatch (above); `prove_until`'s doc comment now says what is
polled, what is not, and that a condition must not ration itself by
counting polls.

## The net engine on a pool

The root engine keeps a list of cubes, each the links of a branch
nobody has followed, in the order in which one thread would reach them,
starting from the one cube without a link. While there are fewer than
sixteen per thread it makes a pass over the list that replaces every
cube in place by the branches of its next choice, where a link of a
literal with exactly one admissible partner is no choice and is
followed at once
(`Frame::forced`, `Engine::choices`; `choose` already counted the
partners of the literal it picks). Nothing is searched twice; a sequent
whose links are all forced is decided by the first expansion, which is
the sequential search link for link. This changes what a pool searches
(the cubes are other cubes), never what one thread does: the sequential
counters of `3-partition-mll-no/4` are the second baseline's (729 157
literals chosen, 1 542 036 links).

**The thread table's net rows of the second baseline** (`3-partition-
mll-no` at 4 and 5, and the thirteen Partition instances of
`bench/problems/slow-tests.txt`, net engine forced, 1, 2 and 4 threads),
through the harness on the cores 4 to 7, "before" being the build of the
commit the step started from on the same cores an hour earlier. Times
in milliseconds:

| problem | threads | before | after | |
|---|--:|--:|--:|--:|
| `3-partition-mll-no/4` (refuted) | 1 | 2 148 | 2 042 | −4.9 % |
| | 2 | 1 058 | 1 029 | −2.7 % |
| | 4 | 575 | 525 | −8.8 % |
| `3-partition-mll-no/5` (refuted) | 1 | 70 467 | 67 542 | −4.2 % |
| | 2 | 33 647 | 34 710 | +3.2 % |
| | 4 | 18 396 | 17 591 | −4.4 % |
| Partition 1, 2, 5 (refuted) | 1 | 4 552 | 4 531 | −0.5 % |
| | 2 | 2 267 | 2 223 | −1.9 % |
| | 4 | 1 174 | 1 128 | −3.9 % |
| Partition 1, 1, 1, 5 (refuted) | 1 | 9 185 | 8 881 | −3.3 % |
| | 2 | 4 707 | 4 631 | −1.6 % |
| | 4 | 2 398 | 2 328 | −2.9 % |
| Partition 1, 1, 2, 4 (proved) | 1 | 1 277 | 1 276 | −0.1 % |
| | 2 | 619 | 629 | +1.6 % |
| | 4 | 294 | 298 | +1.4 % |
| Partition 2, 3, 2, 1 (proved) | 1 | 5 573 | 5 247 | −5.8 % |
| | 2 | 2 655 | 2 638 | −0.7 % |
| | 4 | 1 314 | 1 254 | −4.6 % |

The one-thread rows, whose code did not change, move by −5.8 to +4.0 %
between the two runs, so the pool's rows (−8.8 to +3.2 %) are within
what this machine does by day; the scaling is the baseline's (2.0 and
3.9 to 4.0 times on two and four threads for the refutations). The five
smaller instances take under 65 ms and are as fast or faster (Partition
2, 1, 1 on four threads: 1.2 ms against 4.3 ms); the four that no
thread count decides within 15 s visit as many literals as before, and
Partition 1, 2, 3, 4, 5, 5 is proved on four threads in 2.1 ms (3.4 ms)
and on no fewer, as before. A refutation on a pool now chooses exactly
the literals one thread chooses (729 157 and 16 202 533 on the two
3-Partition rows, where the old cubes chose 8 more).

## The options, and how each front end sets them (D15, D16)

| option | type and default | the command | the web front end, another wrapper |
|---|---|---|---|
| how many threads a search may use | `search::Options::jobs(usize)`, default 1; `Options::MAX_JOBS` = 256 is the most the options name, and a search starts no more threads than `std::thread::available_parallelism()` reports | `--jobs N` on `prove` and `interact` (default: the machine's parallelism, which is step 21's to change); a larger `N` is taken as the bound with a note on standard error | a field of the search options once they have serde (step 23); on wasm there is no pool and the field has no effect |
| the time limit | none in the library, which has no clock: the stop closure of `prove_until` and `prove_goal` | `--timeout DURATION` on `prove` (from the command's start) and `interact` (per `close`), no default yet (step 21) | its own timer, or a count of the work done, behind the same closure; `Deadline` in `cli/src/limit.rs` is thirty lines to copy where there are threads |

`Options::portfolio` is removed, so the search options have one field
less. The cadences are named constants and not options, as before:
`SPLITS_PER_POLL`, `FORCED_PER_POLL`, `SET_UP_POLL`, `CUBES_PER_THREAD`
and the driver's millisecond.

## Measurements

**The target set** (`bench/targets.sh after-limits`, once, 165 runs on
its two pinned cores, at the commit that removed the portfolio, so with
every change to the focused engine): all 99 rows that
`bench/targets/after-bias.csv` decides have its verdict, `nodes`,
`splits`, `memo_hits` and `memo_entries`; the 66 undecided rows have its
verdict and reason; the rows both decide take 177 s together against
189 s. The file is committed with `bench/TARGETS.md`.

**The set-up on the library's largest file** (`SYJ212+1.020` in cbv,
27 787 233 occurrences, one core): read 0.05 s, parse 12.4 s, forest
0.43 s; with a stop that fires 0.6 s and 1.5 s into the search, the
search returns 0.002 and 0.10 s later and the longest gap between two
polls is 0.12 and 0.20 s.

**The families' verdicts** (`linlog-bench run --all-families --timeout
5`, 123 runs on core 12, as the verification table asks for a change
under `bench/`): no mismatch; 59 proofs, all `checked` `ok`; 35
refuted, 22 at the time limit, 4 at the copy bound, 3 at the recursion
limit. The latest stop is 0.49 s after its limit (`qbf/40#1`) and
eleven QBF instances are 0.33 to 0.49 s late, all with the memo at its
cap: a scratch program on that instance measured 0.55 s from the poll
at which the stop fired to the return of `prove_until`.

**The twelve latest stops of the second baseline** (its intuitionistic
LLTP run, where they ended 5.61 to 6.11 s into a limit of 5 s:
`DrinkVendingMachine` at 10, `TokenRing-50`, `PaceMaker`,
`GPPP-1000-100`, `Dekker-200`), through the harness on core 12: 5.006
to 5.120 s.

**The review.** A fresh-context reviewer read the two changes to the
parallel paths (the poll in `with_parallel`, the net engine's cubes in
all three of their versions) against the merge rules and the sequential
engines, and ran a differential fuzzer of its own outside the
repository, on the cores 8 to 11 with at most four threads, every
search under a five-second stop, 18.9 minutes of runs in all. One thread
against two, three and four:

- *The focused and the two-sided engine*: 369 057 sequents (random
  provable ones and their mutants, towers of `&` in one asynchronous
  phase, classical, with Mix, affine, intuitionistic and intuitionistic
  affine; copy bounds 1 to 3, memo limits 0, 16 and the default, a
  quarter with a recursion limit of 24 to 63), 1 107 171 parallel runs,
  49 cut by the stop: no contradiction, no rejected proof. 681
  differences at the copy bound and 3 at the recursion limit, which the
  contract allows; three sequents the pool refuted where one thread was
  at its bound were checked by hand and are unprovable.
- *The net engine*, forced, with every test period from 1 to 5 and each
  sequent also decided by the focused engine: 376 209 sequents and
  1 128 627 parallel runs on the final cubes (and 363 406 sequents on
  the intermediate version), literals of multiplicity 1 to 4, up to ten
  pairs: no mismatch, every net correct.
- *The two cost claims*: forty roots `a & b` are refuted with 2 to 6
  stable sequents on two and four threads at every size from 10 to 40;
  `wide(k, 1)` has equal statistics on one, two and four threads for
  `k` = 16, 64 and 256.

It answered the questions it was given with "sound" (the cubes
partition what is left; `forced` is exact and soundness does not rest
on it; a seed reproduces the root's state; `choices` is kept through
every path) and found one defect and one delay:

- *A pool answered "stopped" without a stop.* The last match of
  `with_parallel` took the left premise's reason first. When the right
  premise ends at the recursion limit it cancels the left one, which
  returns `Stopped`, and that was the `&`'s answer, all the way to the
  caller. The match is from step 13; the new poll adds one more way for
  a cancelled premise to return `Stopped`. Fixed here, since a step
  about stops should not leave a stop that nobody asked for: `Stopped`
  gives way to the other premise's reason, as it does in
  `Collected::take`; `a_cancelled_premise_is_no_stop` pins the
  reviewer's probe, which fails without the fix.
- *A premise's failure cancels the other only once its worker has left
  its nested scopes*, and a thread waiting there may run a stolen task
  of the sibling premise that nothing cancels. Seen once in three runs
  of one probe: an answer that came only when the reviewer's own stop
  fired, 5 s in. Not fixed; in the open questions.

It also noted that a seed's links count in `Statistics::links` once per
pass and cube (`wide(16, 2)`: 32 links on one thread, 908 on two), which
is the cost of the seeds made visible, and that its recursion limits
were rarely reached by random sequents (10 of 369 057), so those paths
were exercised by its probes alone.

## Decisions

- **One poll at the `&`, not a poll in the asynchronous phase.** The
  phase's loop is linear in the formulas it decomposes; what was
  exponential was the fan-out, and the fan-out is in one function.
- **The chain's poll passes no work.** The step forbids changing what
  one thread searches, and the slices of the two alternating searches
  are counted in work reported at polls; a poll that reported work
  would move every slice boundary after the first chain.
- **`dual_from` with a cursor per chain** rather than a change of what a
  lookup returns: it finds the occurrence `dual_in` found, so no counter
  moves. A chain's lookups of one literal cost its list once; the first
  lookup of each chain still passes over the occurrences that earlier
  stable sequents consumed, so "linear in what it returns" holds per
  chain and not over a whole branch (a branch of `n` chains over a list
  of `k` consumed occurrences costs `n·k`; on the GPPP net that is
  nothing).
- **`meets` made linear, although the step names no such thing.** The
  limit cannot be kept on the GPPP nets with a quarter of a second
  between two polls and more on larger markings, and a poll inside a
  quadratic loop would have kept the limit and left the net at 140 s.
  The marks give the answer the comparison gave; the work counted for
  a slice is the old formula.
- **The set-up polls only on large forests.** Unconditional polls would
  be simpler to state, and would change at which poll a counting
  condition fires on every small sequent, the tests' among them.
- **A flag and a timer thread in the command, not a cleverer cadence.**
  Any rule of the form "look every `n` polls" is late by `n` gaps, and
  the gap between polls varies by five orders of magnitude between
  problems and within one run.
- **The load is abandoned, the search is not.** The parser has no hook,
  so under a limit the load runs on a thread the command can leave.
  The search is stopped cooperatively and its outcome waited for: the
  verdict line then has the fragment, the engine and the statistics,
  and a miss inside the library shows as a late answer instead of
  hiding behind a backstop. What that costs is the lateness above.
- **The forest is built under the command's limit**, so the command
  calls `prove_goal` where it called `prove_until`.
- **The bound on threads is the machine's parallelism**, as GNU `sort
  --parallel` and Z3's thread cap have it: more threads than processors
  only take turns. It is in the library (`parallel::threads`) so that
  every front end has it, with a constant bound in the options
  (`MAX_JOBS`) for a platform that does not tell. The command clamps
  and says so rather than refusing, so that a script written for a
  larger machine runs on a smaller one. The harness refuses, because
  its rows record the count.
- **The portfolio's column is gone from new files** and read in old
  ones; `--resume` refuses a file with another header, since the rows
  it appends would not fit.
- **Cubes split in place, not found by deepening.** Deepening without
  the forced links would have fixed `wide-m1` and left every sequent
  with one surviving branch per choice quadratic. The first version
  here kept the cubes in a queue and put a cube's branches at its end;
  the measurement of the thread table's rows showed two threads slower
  than one on a provable Partition instance (2.2 s and 253 852 literals
  chosen against 1.3 s and 109 627), because the workers no longer took
  the cubes in the order of the search. Splitting in place keeps that
  order, and with it the bound: the cube with one thread's proof is
  taken no later than one thread reaches it.

## Deviations and assumptions

- More commits than the prompt lists: the linear copy ranking, the
  harness child's flag, the target set's record and the documentation
  are changes of their own, and two are corrections of this step's own
  work that its measurements and its review asked for (the cubes'
  order, and the stop that was no stop).
- **"Linear in what it returns"** is read per chain (above).
- **"A pool is never slower than one thread by more than its start"**
  holds for the net engine where the links are forced, and for stops.
  It is not a property of the focused engine's pool in general, whose
  and-parallel premises and cubes search what one thread might skip;
  the rules file says what a pool promises.
- **"On every problem of the library"** was verified on the instances
  the step names and on the twelve latest stops of the second
  baseline's intuitionistic run, not on the whole library: at five
  seconds a problem that is hours of a core and was not asked for by
  day. The third baseline (step 31) will show it.
- **A session's sequent is read outside the limit**; only `prove`
  counts from its start.
- The R13 rows were taken on the cores 4 to 7, which are slower than
  the cores of the baseline nights; "before" is the build of the commit
  the step started from on the same cores.

## Open questions and follow-ups

- **A stop is late by the freeing of a full memo**, and a search is
  unpolled while it empties one: 0.15 to 0.55 s each, depending on the
  core, so that in the worst case (the limit passes while a full memo
  is being emptied, and the memo is full again at the return) a stop is
  about a second late on the slowest cores. This is the memo's design
  (two allocations per entry, `HashMap::clear` in `Memo::insert`), not a
  missing poll. Keys in one allocation halve it; keys in an arena that
  is dropped whole, or handing the full table to a thread to free,
  remove it. Left for step 20, which gives the memo its bound in bytes
  and will touch the same structure; it also costs memo-bound searches
  a fifth of their time today (0.5 s of every 2.5 s on `qbf/40#1`).
- **`Forest::new` is not polled**, nor are single passes over the
  forest (0.1 to 0.2 s each at 27.8 million occurrences).
- **The first lookup of each chain rescans the consumed head of a
  list.** A cursor per branch instead of per chain would need the
  context's history.
- **The pool's speculative work** on `&` towers is bounded by the
  cancellation now, not by anything else; and nested `with_parallel`
  scopes still stack on a waiting thread (the rules' note on the stack).
- **A cancellation at a `&` can come late** (the review's second
  point): the flag is raised when the premise's worker returns, after
  its nested scopes have ended, and a stolen task of the sibling
  premise runs uncancelled meanwhile. Raising the flag where the
  failure is first known, or polling the `&`'s flag from the stolen
  task's chain, would close it; it needs the flags' structure changed
  and a measurement, so it was left.
- **An error of one premise cancels the other**, so a pool answers
  `RecursionLimit` where the other premise would have failed and one
  thread answers `Unprovable`. Cancelling on a failure only would give
  the pool one thread's answer there.
- A fuzz run of the parallel paths with recursion limits of 4 to 16
  (the review's suggestion): its limits of 24 to 63 were almost never
  reached.
- **Where the list of cubes stays short** the root engine does the
  whole search with a seed per cube, and the workers wait.
- **The exports cannot be stopped inside** (step 22, as step 18 left
  it).
- `PaceMaker_20_1` under `--bias rarer` (3.1 s late on one thread in
  `plan/later.md`) was not rerun under that flag.
- The harness's `time_ms` still starts after the child's load, and its
  kill counts from `loaded`; whether `bench/reruns.txt` is still needed
  was not looked at.
- `cargo doc --document-private-items` fails on a link to `Shared` in
  `focus/memo.rs` that resolves only with the `parallel` feature; it
  did before this step, and the flake's `doc` check, which documents
  the public items, passes.

## The commits

In order, on top of "Plan: review step 18" (one commit of the planning
session, "Plan: step 29 leaves a reference prover in the repository",
landed between them and is not this step's):

1. Stop a stopped premise of & on the pool before it starts its own
2. Poll the stop inside a chain of forced splits, and look its duals up
   from a cursor
3. Rank the copies of a stable sequent in one pass over its members
4. Poll the caller's stop while the calling thread waits for the other
   search's slice
5. Poll the stop between the passes that set a search up
6. Remove the portfolio of worker orders (with the `agree` test)
7. Count the command's time limit from its start, as a flag a timer
   raises (with the command's `timeout` test)
8. Stop the harness's child by a flag a timer raises
9. Split the net engine's search into cubes without searching one twice
10. Bound the threads a search uses by those the machine runs at once
11. Document where a search polls, what a pool promises and the
    command's limit
12. Record the target set after the stops
13. Keep the net engine's cubes in the order of the search, split in
    whole passes
14. Let a cancelled premise's stop give way to the other premise's
    reason
15. This report.

Nothing is pushed.

## What steps 20 and 21 must know

- **20 (memory and inputs).** Under `--timeout` the sequent is read and
  parsed on the thread `load`, whose stack is 8 MiB like a main
  thread's: the deep-nesting abort (R11) is the same abort on another
  thread. A load that is abandoned is never freed; the process ends.
  `Counts::new_until` is the set-up's longest pass and the place of the
  quadratic rows. A stop's lateness is the memory freed (above), so a
  bound on the memo's bytes is also a bound on that. `MAX_JOBS` and the
  machine's parallelism bound the pool's threads, each with a stack of
  `Options::stack_size()` and, in the focused engine, state per worker.
  The harness refuses `--jobs` above the process's CPU set.
- **21 (defaults).** A default time limit is `args.timeout` with a
  default: `Deadline::start` takes it from there, the load is under it,
  and "while the sequent was read" is the line a newcomer with a huge
  file would see. "One thread first, then the pool" is `jobs` in
  `cli/src/argument_parsing.rs` (which already clamps and notes) and
  `default_jobs`. The net engine on a pool no longer costs anything on
  the wide sequents it is the default for, which was one reason against
  a pool by default; the focused engine's pool on small problems is
  still milliseconds slower than one thread.
- **Quantifiers (D17).** Two new structures are indexed by the forest's
  lists of a literal's occurrences (`2·atom + sign`): a chain's
  `Cursors` and the stamps of `mark_literals`. With terms a dual is a
  unifiable literal, so both become indexed by predicate symbol and
  sign, and a lookup filters by unification after the index; neither
  type changes shape. The polls are independent of the logic.

## From the review (2026-10-03)

The planning session read the changes to the parallel paths, the net
engine's cubes and the command's limit, and ran the command and the
harness itself, in capped scopes on pinned cores. Nothing in the report
had to be corrected.

- **The named calls**, with the release build of the step's last
  commit: R6 0.39 s with the counters above (core 4); R7 5.15, 5.36 and
  5.51 s on one, two and four threads (cores 4 to 7; the last is 0.01 s
  over the half second, by the memo's freeing); R10 1.00 s in every
  format, the line in the output file where one is named, and 16.13 s
  under a limit of 16 s that ends in the search (core 12);
  `wide-m1` at 1 024 pairs 196, 193 and 195 ms with the same 2 048
  links on one, two and four threads; `PaceMaker_20_1` under `--bias
  rarer`, 3.1 s late before, 5.07 s; a session's `close` under a limit
  of 2 s, 2.37 s.
- **A stop on a pool is never a refutation**: three provable problems
  (two Partition instances with the net engine forced and with the
  default, `SYJ202+1.005` in cbv two-sided) on two and four threads
  under twelve limits from 5 ms to 0.9 s, three times each, 288 runs:
  115 proved, 173 unknown, none unprovable.
- **The pool on the library**: the 1 358 intuitionistic problems outside
  the nets on four threads under one second, through the harness on the
  cores 8 to 11: 434 proved and all `checked` `ok`, 101 refuted, no
  contradiction with the second baseline's one-thread verdicts, no kill,
  the latest stop 0.43 s after its limit (`SYJ202+1.007` in cbv, memo
  full).
- One defect of wording, fixed in the review: `--jobs 4` on a process
  confined to one core noted "the 1 threads a search uses".
- **One thread on the whole library** (4 512 problems under `ILL` and
  `CLL`, one second each, through the harness in two halves on the cores
  4 to 7): 1 868 proved and all `checked` `ok`, 104 refuted, no
  contradiction with the second baseline, no kill (the nine `GPPP` nets
  that were killed five seconds past the limit at the review of step 18
  are proved or stopped in time), and the latest stop 0.43 s after its
  limit; the ten stops more than 0.3 s late are all `SYJ202` and
  `SYJ208` in cbv, whose memo is full.

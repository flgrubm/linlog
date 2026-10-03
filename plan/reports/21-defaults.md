# Step 21 report: the defaults a user meets

`linlog prove SEQUENT` without flags now deepens its copy bound with no
upper end while a time limit of two seconds lasts. It searches on one
thread for a tenth of a second, and then a pool of the machine's other
cores searches beside that thread, the first of the two to decide
answering. An "unknown" says which bound or limit ended the search,
after how long, at which copy bound, and which flag to try. An
"unprovable" says why where the counts of the sequent tell: an atom
whose literals cannot all meet in axioms, or the count equation of the
multiplicatives. Otherwise it says that the search was exhaustive.
Every default is a named constant with a flag and a line of help; every
explicit flag means what it meant.

On the two named sets, at five seconds on two pinned performance cores,
under the scheme committed last (the single thread kept beside the
pool):

| at 5 s | 1 003 of `lltp-copies-10` | 1 342 recorded |
|---|--:|--:|
| decided by today's default (copies 3, one thread) | 188 | 526 |
| of those, left undecided by the new default | 1 | 0 |
| decided by `--copies 10` or the forward pass at 30 copies | 477 | 814 |
| of those, left undecided by the new default | 3 | 2 |
| decided by the new default | 485 | 822 |
| of those, by none of the other passes | 11 | 10 |

On the recorded set the new default decides 822 problems, where the
Maude prover the library records decides 659 in five minutes each. Both
decide 657, with the same verdict on every one. The two that only Maude
decides are `SYJ201+1.003` and `SYJ203+1.009` in the `01` translation,
in 277 s and 85 s. The three problems missed across both sets are:
- `SYJ212+1.014` and `SYJ212+1.015` in `cbn`, which the forward search
  refutes in 1.3 and 3.5 s: without a copy bound the backward search
  keeps its share of the time;
- the Petri net `AutoFlight_afcs_06_b_10_1`, which today's default proves
  at 3.45 s: on two cores, three threads share them.

Neither is met in full: one row of today's default is lost, and three
rows of the larger bounds are missed (Open questions).

The search on one thread under explicit flags did not change: the target
set has the counters of `after-memory` on every decided row (below).

## Outcome, item by item

1. **The copy bound deepens while the budget lasts.** `Options::copies`
   takes an `Option<u32>`: `None` makes `Engine::run` go on to the next
   level until a level decides, the stop fires or a limit binds
   (`Options::copy_bound` is `u32::MAX`, which the inclusive range of
   levels reaches without a wrap and no search reaches, since every
   level visits a stable sequent and polls there).
   `Statistics::copies` is the level reached, of two searches the
   larger. `Some(n)` is today's cap. `Unprovable` keeps its meaning:
   a level that ended without a cut. The library's own default keeps a
   cap of 3 (`DEFAULT_COPIES`): `prove` has no stop condition, and a
   search without a bound ends only when it decides. The command's
   default is `None` under its time limit.
2. **A default time limit**, `DEFAULT_TIMEOUT` = 2 s in
   `cli/src/argument_parsing.rs`, `--timeout DURATION|none`, on `prove`
   and on every `close` of a session. The value is the knee of the
   tables below: from 2 s to 5 s the recorded set gains seven decided
   problems (0.9 %), while some 470 unknowns each wait two and a half
   times as long. On a terminal, a search that runs longer than half a
   second writes one line on standard error (`Notice`: how long it may
   run, whether it deepens, the flag) and takes it back when the answer
   comes.
3. **One thread first.** Without `--jobs`, one thread searches for
   `DEFAULT_POOL_AFTER` (100 ms, `--pool-after DURATION`). If it has not
   decided, a pool of the other threads (`jobs − 1`, at least two)
   searches beside it, each within the whole of `--memory-limit` (so the
   default may hold twice the bound), and the first to decide stops the
   other (`alone_first` in `cli/src/prove.rs`, its
   twin in `bench/src/run.rs`). `--jobs N` runs N threads from the start
   unless `--pool-after` is given too; `--deterministic` runs one thread
   throughout. Whether the second attempt may reuse anything of the
   first: it reuses all of it, because the first goes on. The first
   version restarted the search on the pool and lost three problems that
   one thread proves in two seconds; both schemes are measured below.
4. **What "unknown" says** (`unknown` in `cli/src/prove.rs`, shared with
   the session). For each `Reason`:
   - a timeout: "the time limit of 2s was reached at a copy bound of 512;
     --timeout DURATION gives the search longer";
   - Ctrl-C: "interrupted after 1.32s at a copy bound of 4";
   - the copy bound: "the copy bound of 3 was reached after 1.79ms; raise
     it with --copies N, or lift it with --copies none to deepen it while
     the time limit lasts";
   - the recursion limit: "the recursion limit of 2048 was reached after
     1.01s at a copy bound of 512; raise it with --recursion-limit N";
   - the memory limit: "the memory limit of 1 GiB was reached after …;
     raise it with --memory-limit SIZE";
   - the index limit: the reason with the time, since no flag raises it.

   The copy bound is named where the engine deepened one, and `--stats`
   prints "copy bound reached" where the fragment has exponentials. The
   refused check of a proof (`Error::Unchecked`) names its limit in
   binary units ("64 MiB", where it said "67108864 bytes").
5. **Why a sequent is unprovable.** `Verdict::Unprovable(Refutation)`,
   which is `#[non_exhaustive]`, has three variants:
   - `Exhausted`;
   - `Unbalanced { atom, name, least, most }`;
   - `Equation { formulas, tensors, pars, ones, bottoms, mix }`, with
     `needed()`.

   `prove_goal` computes it on every refutation of every engine
   (`focus::refutation`): the focused engine's own `Counts` and `Rules`,
   tallied over the goal. The command prints its `Display`; the JSON
   carries `refutation` (`"exhausted"`, `{"unbalanced": {"atom", "least",
   "most"}}`, `{"equation": {…, "needed", …}}`). Examples: `⊢ A ⅋ B, ~A,
   ~B` gives "the count equation fails: a provable one-sided sequent of
   MLL has exactly #⊗ − #⅋ − #1 + #⊥ + 2 formulas, here 0 − 1 − 0 + 0 +
   2 = 1, and this one has 3"; `A ⊢ B` gives "~A occurs 1 more time than
   A in the one-sided sequent, so they cannot all meet in axioms".
   Nothing is guessed: without a count that rules the proof out the
   answer is `Exhausted`.
6. **The harness and the tests name their bounds.** The harness's own
   defaults did not move: without `--copies`, a run takes the problem's
   bound, else 3, and its threads from the start. New axes are `--copies
   none` and `--pool-after SECONDS`, and new columns `copies_reached`
   and `pool_after`. Every LLTP pass of `bench/baseline.sh` and
   `bench/targets.sh` names `--copies 3`. Stage 3 of the baseline ends
   with `lltp-default`, the intuitionistic library under the command's
   default at its limit, for step 31. The command's tests that pin an
   unknown compare through `timeless`, which writes the time as `…`;
   those that pin a bound's message name it.
7. **Documentation**: README's usage section (the deepening, the limit,
   the threads, the refutations; every example run against the binary),
   the help texts, `.claude/rules/core.md`, `cli.md`, `bench.md`,
   CLAUDE.md.

## The defaults, their reasons, and how each front end sets them (D15, D16)

| default | value and reason | the library | the command | the web front end, another wrapper |
|---|---|---|---|---|
| copy bound | none in the command: no fixed bound is right for every provable sequent; 3 in the library, whose `prove` has no stop | `Options::copies(Option<u32>)`, `DEFAULT_COPIES` = 3 | `--copies N\|none` on `prove` and `interact`, default `none` | `copies: None` beside its own timer; a field of the options' serde form (step 23) |
| time limit | 2 s, from the tables: the knee between late proofs and every unknown's wait | none (D11): the caller's stop closure | `--timeout DURATION\|none`, `DEFAULT_TIMEOUT` | its own timer behind the stop closure, the same 2 s as a start |
| one thread first | 100 ms: a small sequent is decided in microseconds; a pool costs milliseconds to start and makes the proof depend on timing; the single thread goes on beside the pool, which can search worse than one thread | `Options::jobs`; the race is two `prove_goal` calls and a flag | `--pool-after DURATION` (`DEFAULT_POOL_AFTER`), `--jobs N`, `--deterministic` | wasm has no pool: one thread; a wrapper with threads copies `alone_first` (forty lines) |
| the notice on a terminal | after half a second, taken back with the answer | – | `NOTICE_AFTER`, standard error only when it is a terminal | a progress indicator of its own |
| refutation | always computed on `Unprovable` | `Refutation`, its `Display` | the verdict line | the JSON's `refutation` |
| unknown's words | the bound, the time, the copy bound, the flag | `Reason`, `Statistics::copies` | `unknown` | its own words from the same values |

Named constants that are not options: `NOTICE_AFTER`, and the pool's
size (the other threads, at least two).

**Quantifiers (D17).** `Refutation::Unbalanced` names an atom. With
terms it names a predicate symbol: an axiom pairs literals of one
predicate whatever their arguments, so the balance per symbol stays a
necessary condition. The equation is unchanged, since a quantifier
weighs nothing in it. The copy bound counts copies of `?` formulas per
branch. A bound on the depth of the terms an instance introduces would
be a second deepening beside it, in `Options` next to `copies`, and its
level would go in `Statistics` beside `copies`.

## Measurements

All runs were through the harness in capped user units on the
performance cores 2 and 3, by day, `--jobs 2 --pool-after 0.1 --copies
none --timeout 5`, one child at a time. A row's time is the search after
the load. The comparisons are with the second baseline
(`bench/results/2026-10-02/`), taken at night on one performance core
each:
- `lltp-intuitionistic`, today's default: a copy bound of 3;
- `lltp-copies-10`, the bound 10;
- `lltp-forward`, `--bias factors --copies 30`.

The rows are in `bench/defaults/`.

**The sets.**
- The first is the 1 003 problems of `lltp-copies-10.csv`.
- The second is the 1 342 problems that the result files of the Maude
  prover in the LLTP library cover. Step 17 counted 1 342 from the files'
  1 345 lines less three files not in the library (`SYN007+1.014` in
  three translations). Nine of the files end without a line break,
  which first left nine problems out of my list; they were run with the
  rerun, and are in the race's rows only. 829 problems are in both sets.

### The first scheme: the search restarted on the pool at 100 ms

| at 5 s | 1 003 of `lltp-copies-10` | 1 333 recorded |
|---|--:|--:|
| decided by today's default (copies 3) | 188 | 518 |
| of those, left undecided by the new default | 0 | 0 |
| decided by `--copies 10` or the forward pass at 30 | 477 | 805 |
| of those, left undecided by the new default | 5 | 5 |
| decided by the new default | 483 (392 proved, 91 refuted) | 810 (667 proved, 143 refuted) |
| decided by it alone of the four | 11 | 10 |
| decided within the first 100 ms, on one thread | 415 | 760 |

Every proof was checked (`checked` `ok`), and no verdict contradicts
another pass. The five losses are the same in both sets:
`SYJ204+1.014` and `SYJ204+1.015` in the `01` translation,
`SYJ204+1.015` in `cbv`, and `SYJ212+1.014` and `SYJ212+1.015` in `cbn`.
The other passes decide them in 1.3 to 3.5 s. A probe the author allowed
(eight runs on the cores 4 and 5) separated two causes:

| configuration, no copy bound, 5 s | `SYJ204+1.014` (01) | `SYJ212+1.014` (cbn) |
|---|---|---|
| forward search alone, one thread | proved, 1.96 s, 15 708 917 stable sequents, level 9 | refuted, 1.30 s, 35 826 |
| default bias, one thread | proved, 1.95 s, the same 15 708 917 | unknown, 183 171 |
| default bias, pool of 2 from the start | unknown, 66 777 494 | unknown, 374 272 |
| forward search, pool of 2 | unknown, 66 451 665 | unknown, 295 647 |

On `SYJ204` the two rules agree on every atom, so the default runs the
forward search alone, and one thread proves it. The pool, which
replaced that thread at 100 ms, visits four times the stable sequents
without a proof: the focused engine's pool is not a superset of one
thread. On `SYJ212` in `cbn` one thread loses it too. Without a bound
the backward search never ends and keeps its share of the work, where
`--copies 10` ends it at level 10 and hands the core to the forward
search. In time that share is far more than the two thirds it is in
work. That is the price of the deepening default, and the identity of
the rules file says only that the default decides it given time.

### The second scheme: the single thread kept beside the pool

The rerun took the 681 problems where the schemes can differ (those the
restart did not decide within 100 ms, and the nine missing ones), with
half the memory bound for each of the two searches as first built. The
eight problems on which the two schemes then differed were run again
under the final build, with the whole bound each:

| on the rerun's problems, 5 s | 590 of the first set | 573 of the recorded set |
|---|--:|--:|
| decided by the restart | 70 | 50 |
| decided by the race, the eight rechecked | 72 | 53 |
| only by the race | 4 | 3 |
| only by the restart | 2 | 0 |
| the time of the race less the restart's, on the problems both decide | median 0 ms (−3.0 to +1.9 s) | median −69 ms (−3.3 to +1.2 s) |

The eight, restart against race (half the bound) against race (whole
bound):

| problem | restart | race, half | race, whole |
|---|---|---|---|
| `SYJ204+1.014` (01) | unknown | proved 1.50 s | proved 1.45 s |
| `SYJ204+1.015` (01) | unknown | proved 2.48 s | proved 2.46 s |
| `SYJ204+1.015` (cbv) | unknown | proved 2.49 s | proved 2.47 s |
| `SYJ208+1.005` (cbv) | unknown | refuted 0.11 s | refuted 0.11 s |
| `SYJ208+1.007` (cbv) | unknown | refuted 4.34 s | unknown |
| `SYJ212+1.013` (cbn) | refuted 0.52 s | unknown | refuted 0.76 s |
| `SYJ205+1.008` (01) | proved 4.55 s | unknown | unknown |
| `AutoFlight_afcs_06_b_10_1` | proved 3.03 s | unknown | unknown |

Three things are visible here:
- The `SYJ204` proofs are the pool's loss that the race repairs.
- `SYJ212+1.013` lost to the halved bound: its forest is wide, every
  memo entry is large, and a quarter of the bound per search starved the
  memo. That is why each search now has the whole bound.
- The last two are lost to three threads on two cores: the pool's
  forward search runs at two thirds of its speed. `SYJ208+1.007`, which
  ends near the limit, comes and goes between runs.

On a machine of sixteen cores the single thread is one in seventeen.

### What a longer budget buys

From the rows of the race, as every limit would cut them. An unknown
waits the whole budget unless a limit of its own ended it sooner (a
recursion or memory limit):

| budget | first set: decided | its unknowns, each waiting it | recorded: decided | its unknowns waiting it | their wait in all |
|---|--:|--:|--:|--:|--:|
| 0.1 s | 415 | 588 | 769 | 519 | 52 s |
| 0.5 s | 452 | 551 | 797 | 491 | 246 s |
| 1 s | 467 | 536 | 808 | 476 | 480 s |
| 2 s | 475 | 528 | 815 | 468 | 941 s |
| 3 s | 482 | 521 | 819 | 464 | 1 397 s |
| 5 s | 485 | 518 | 822 | 461 | 2 310 s |

The recorded set's unknowns that do not wait the budget ended at the
recursion limit (55) or the memory limit (8) sooner.

At 2 s the recorded set has nearly every problem a longer budget
decides. Nine in ten are decided within the first 100 ms on one thread.
The rest are a few tens of late proofs against some 470 unknowns that
each wait the whole budget. So the default is 2 s, as the planning
count proposed, and not the 10 s of step 17.

### Against the Maude prover

| on the 1 342 recorded problems | decided |
|---|--:|
| the Maude prover, 300 s | 659 (620 proved, 39 refuted) |
| linlog's default before this step (copies 3), 5 s | 526 |
| linlog's four passes of the second baseline together, 5 s each | 814 |
| linlog's new default, 5 s, two cores | 822 (677 proved, 145 refuted) |
| decided by Maude and the new default | 657, the same verdict on every one |
| only by Maude | 2 (`SYJ201+1.003`, `SYJ203+1.009` in `01`, 277 s and 85 s) |
| only by the new default | 165 |

The copy bound of 3 was the whole difference to that prover (step 17).
The default without one now decides more than the four passes did
together, and every proof was checked.

### The target set

`bench/targets.sh after-defaults` (165 runs on its two pinned cores,
by day, after the last code commit): all 99 rows that
`bench/targets/after-memory.csv` decides have its verdict, `nodes`,
`splits`, `memo_hits` and `memo_entries`; the 66 undecided rows have its
verdicts and reasons. The CPU time of the decided rows is 149.7 s
against 146.6 s (+2.1 %; `nix flake check` ran beside part of it).

### The families

`linlog-bench run --all-families --timeout 5` on core 12 gave 123 runs:
- 59 proved, all `checked` `ok`;
- 36 refuted;
- unknown: 21 at the time limit, 4 at the copy bound, 3 at the
  recursion limit;
- no mismatch.

These are the counts of step 20.

## Decisions

- **The library's default keeps a bound, the command's has none.** A
  library function without a stop condition must end. The command, the
  web front end and any wrapper with a clock lift it, and the rules
  file says why.
- **The time limit is the command's constant, not the library's**
  (D11). Its value comes from the tables. The web front end, whose user
  waits in a browser, decides its own with the same tables.
- **The single thread goes on beside the pool.** Restarting it on the
  pool was the first version, measured above, and lost three problems
  that one thread decides. The cost of keeping it is one thread more
  than the machine's parallelism, and twice the memory bound at most:
  with half the bound each, the first build lost a wide sequent whose
  memo starved. A pool of one would be the single thread's search
  again, so the pool has at least two threads.
- **The notice is by time, not by level.** The library reports no
  progress, and a level-based notice would need a callback in every
  engine. A wait of half a second at a terminal is what "never silent"
  asks to cover.
- **The refutation is computed after the verdict, from the engine's
  own counts.** No counter of a search moves; the cost is one more
  `Counts` pass on a refuted sequent. The argument that the goal's sums
  refute is in the rules file. A fresh-context reviewer derived a
  stronger one (every `&`-slice of a proof pairs the literals of an atom
  with rows exactly), checked 395 000 goals, about 95 000 of them refuted
  by the counts, against the engine and against a brute-force prover of
  its own for MALL with Mix, and found none provable.
- **`Interactive::close_with`** grafts a proof found outside `close`,
  so that a session races the same way and the switch never stops a
  graft. The first version passed the session's stop to the graft
  (fixed in "Go on with every thread when only the switch stopped a
  session's graft", then replaced by the race).
- **The CSV files of the step's runs are in `bench/defaults/`**, not in
  `bench/results/`, where a directory without `RESULTS.md` is what
  `baseline.sh` resumes.

## Deviations and assumptions

- **Two runs became three.** The named runs measured the restarting
  scheme. Their losses changed the scheme, and the author allowed a
  probe (eight runs, about a minute) and a rerun of the 681 problems
  where the schemes can differ: those not decided within 100 ms, plus
  the nine missing ones.
- "On two pinned performance cores" is read as one child at a time with
  `--jobs 2`. The race then runs three threads on two cores: the single
  thread shares a core with the pool.
- "Unknown at 2 s" is read off the 5 s rows by time, not run at 2 s:
  the search under the default does not depend on its limit, only its
  end does.
- The library's `Options::default()` keeps a copy bound of 3, which the
  step's wording ("by default the search goes on") could be read
  against. The reason is the first decision.
- The harness's rows are the search after the load. The command's limit
  also counts the parse, which on the library's largest files is
  seconds.

## Open questions and follow-ups

- **On two cores neither scheme meets both of the step's conditions.**
  The restart lost no row of today's default but missed five of the
  larger bounds' rows. The race misses three and loses one of today's,
  `AutoFlight_afcs_06_b_10_1` (proved at 3.45 s by today's default),
  to three threads on two cores. A pool of `jobs − 1` threads with no
  pool below two would oversubscribe nothing, and make the default on
  two cores one thread throughout: one thread loses the same net to the
  backward search's share. Measured on more cores, at the third
  baseline (`lltp-default`, every core), the question is whether the
  race keeps today's rows there. I kept the race because its cost is
  bounded by one thread's share and the pool's loss is not (four times
  the stable sequents without a proof).

- **The forward search's refutations under the default bias without a
  bound** (`SYJ212+1.014`, `SYJ212+1.015` in `cbn`). The backward search
  keeps a share that in time is far above its two thirds of the work.
  A share that adapts once one search has proved levels without a
  decision is a question of the engine's scheduling (step 29).
- **The focused engine's pool can search far worse than one thread**
  (`SYJ204+1.014`: 66 million stable sequents on two threads against 16
  million to a proof on one). The race hides it in the command; the
  cause is the pool's and step 29's.
- **The refutation's counting pass polls the caller's stop** on a
  forest of 65 536 occurrences or more. A stop there, the switch's
  included, leaves `Exhausted`: true, but the reason then depends on
  timing (the reviewer's finding).
- **"copy bound reached" is the larger of the two searches**: with
  `--copies 3` on a Horn program it says 30, the forward search's bound.
- The derivation view's errors still name their limits in bytes, and
  the harness's `--pool-after` panics on a negative or NaN value, as
  `--timeout` does.
- **The third baseline no longer fits one night.** With `lltp-default`
  it is about eleven and a half hours against a slot of eleven; the
  script finishes it on the next night (step 31).
- `Interactive::close` itself runs one search with the threads it is
  given; the race is the command's.

## Verification

- `cargo clippy --workspace --all-targets -- --deny warnings`: clean.
- `cargo test --workspace` passes; `cargo test -p linlog --features
  parallel --lib` has 154 tests (two ignored).
- `cargo hack check --each-feature -p linlog`: 11 runs.
- `cargo hack check --feature-powerset --depth 2 -p linlog`: 39 runs.
- Both `cargo hack` runs are without a warning.

`nix flake check` passed (all checks, x86_64-linux) on the tree of the
last code commit, "Give the single thread and the pool beside it the
whole memory bound each".

Every README example was run against the release binary. The output
matches, except where the README shows another machine or terminal:
- the note of `--jobs 10000` names 16 threads, where this run had 4
  cores;
- the 30-column terminal example needs that terminal;
- output files.

## The commits

On "Plan: review step 20", in order:

1. Deepen the copy bound without a cap where none is set
2. Say why an unprovable sequent is unprovable where its counts tell
3. Name the memory limit of a refused check in binary units
4. Deepen while a default time limit lasts, on one thread first
5. Run the harness without a copy bound and with one thread first
6. Name the copy bound of every LLTP pass, and add a pass under the default
7. Print the copy bound reached only where the fragment has exponentials
8. Document the defaults a call without flags gets
9. Go on with every thread when only the switch stopped a session's graft
10. Word the refutations in one-sided terms, with the equation spelt out
    (the reviewer's points)
11. Close a session's goal with a proof found outside close
12. Keep the single thread searching beside the pool that joins it
13. Give the single thread and the pool beside it the whole memory bound
    each
14. Keep the rows that chose the command's defaults
15. Take the target set after the defaults
16. This report, and the step's entry in the plan

Commit 9 fixed a defect of commit 4 found while writing the session's
part; commit 12 replaced it. Nothing was pushed.

## From the review (2026-10-03)

The planning session read the deepening and its bound, the refutation
against the engine's `Rules`, the race in the command, the session and
the harness, and `Interactive::close_with`. It ran the command and the
harness itself, in capped scopes on pinned cores (4 to 7 for single
calls, as a machine of four cores sees them).

- **The pool honoured a stop up to 15 s late on large Petri nets, and
  the default met it on every one** (fixed in the review: "Skip the
  queued alternatives of a choice once a flag above them is raised").
  `GlobalResAllocation_galloc_res-5_100_1` was proved by a call without
  flags after 16.4 s under its limit of 2 s, where one thread proves it
  in 0.14 s. A choice on the pool queues a task per alternative, and a
  stable sequent of a net has hundreds. Each task built its worker
  before its first poll, which copies the branch's stack of keys (two
  bitsets of the forest's width each) and hashes every key again. So
  once the single thread had decided, or the stop had come, the pool
  went through its backlog for seconds and the race waited for it. The
  defect is the pool's and older than this step. The default exposed
  it: it starts a pool after 100 ms on every problem that one thread
  has not decided by then. The step's runs could not show it, since on
  two cores the pool runs the two searches side by side and no cubes.
  Now a task polls its flags before it builds a worker. A task skipped
  for an ancestor's flag records a stop, never a failure
  (`Collected::skip`, `a_skipped_alternative_is_no_failure`); one
  skipped for the choice's own flag records nothing, since that flag is
  raised only once a proof or an error is recorded. A fresh-context
  reviewer found it sound. Afterwards: the default proves the net in
  0.33 s; `--jobs 4 --copies 3` in 0.31 s where it took 17.5 s; a stop
  at 500 ms comes at 0.52 s where it came at 21.8 s. The rules file
  states it beside the same rule for `&`.
- **Items 1 and 5, read closely.** The deepening ends at `u32::MAX`
  without a wrap and would answer "unknown" there; a copy is spent only
  above budget 0. `LCL181+1` reaches 1.3 million levels in two seconds
  on one thread, so "hours at the least" for four billion holds, if
  only just. The refutation reads rows signed by the literal, not by
  the bias, under the same `Rules` the engine prunes with. The
  asynchronous phase keeps the goal's sums, so every refutation it
  prints is a reason the engine itself used. Checked by hand: the
  equation with and without Mix (Mix here has no empty sequent, so "at
  least" is right), the hull across `⊕`, the balance in intuitionistic
  mode, affine and `⊤` as `Exhausted`, and the JSON of each.
- **The target set** (`after-defaults.csv`): all 99 decided rows have
  the counters of `after-bias.csv` and of `after-memory.csv`, and the 66
  others the same verdicts.
- **By hand**: the growing sequent `!(A -o A * A), A |- ?B` under every
  thread setting ends at the recursion limit at a copy bound of 512
  within half a second, and `--copies 3` keeps its old answer; Ctrl-C
  says "interrupted after 703 ms at a copy bound of 512"; a session's
  `close` races and keeps its own limit. The README examples that
  depend on time (the limit of 2 s at a copy bound of 512, the affine
  one at 18, the limit of 1 s at 12) give those numbers here too. On
  the 19 largest problems of earlier reviews, the default's peak memory
  is 2.05 GB on `SYJ206+1.016` in its `01` translation: two searches,
  each within the whole bound, as the help says.
- **The whole library under the default on four cores**, as a machine
  of four cores runs it: the 4 512 problems of `ILL` and `CLL` through
  the harness at `--copies none --pool-after 0.1 --jobs 4 --timeout 2`,
  in two detached halves (cores 4 to 7 in order, 8 to 11 reversed)
  until they met, with the step's binary. 2 252 proved, all `checked`
  `ok`; 153 refuted; 2 107 unknown (2 001 at the time limit, 54 at the
  recursion limit, 9 at the memory limit, 43 killed). The nine
  verdicts against a header are the known wrong headers (`SYJ212+1.001`,
  `SYN001+1` and `KLE065+1`, each in three translations). 191 rows ran
  past 2.1 s, 7.25 s at the most, and the 43 killed outran their grace,
  all of them on the pool. Run again with the fix, every one of the
  191 ends by 2.11 s; five of them are now proved in 0.2 to 0.46 s
  (`GlobalResAllocation_galloc_res-5_100_1` and four
  `BridgeAndVehicles` nets, `ok`), and none is killed.
- **Harness help**: `--pool-after` said the pool "takes over afresh";
  it now says that it searches beside the single thread ("Say that the
  harness's single thread goes on beside the pool").
- **Assigned**: to 22, the new words a user reads (the refutations, the
  "unknown" sentence, the notice) for localisation, and "copy bound
  reached" of two searches; to 23, `Refutation::Unbalanced` holding a
  name the forest has, `Statistics::copies`, and `close_with` taking a
  proof of any forest; to 24, the race written twice, its memory of
  twice the bound, three threads under `--jobs 2`, and the harness's
  panics on NaN; to 29, the worker's copy per task, `SYJ204` on the
  pool, `LCL181+1` on one thread and the backward search's unbounded
  share; to 31, whether the race keeps every row of the old default on
  sixteen cores, and the latest stop of `lltp-default`.

# Step 19: the search keeps its time limit

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: section 1.2 (D6, D11, D13), section
  2.6 (R6, R7, R10, R13), section 1.4, and the author's answers.
- `plan/reports/18-bounded-proofs.md`, for what step 18 changed around
  the search.
- `plan/README.md`: D9, D11, D16, D19, and the Status entries of steps 13,
  15 (both sessions) and 17.
- `plan/later.md`: "Follow-ups: the focused engine" (the missed stop)
  and "Follow-ups: parallel search".
- `.claude/rules/core.md`: "Proof search: the front door" (the stop
  closure), "The default bias with exponentials is two searches", "The
  parallel runtime", "The net engine".
- `core/src/search/mod.rs`, `search/parallel.rs`, `search/net.rs`
  (`net::parallel`), `search/focus/mod.rs` (`forced_splits`,
  `literal_tensor`, `dual_in`, `poll_splits`, `turns`),
  `search/focus/parallel.rs` (`alternate`, `Baton`, `race`),
  `cli/src/prove.rs`, `cli/src/argument_parsing.rs`.

## What step 17 left you

A caller's time limit is a stop closure the engines poll. Four places do
not poll, or poll too rarely, and one path is slow where it should be
fast:

- **Forced chains.** `forced_splits` and `literal_tensor` count splits
  and never call `poll_splits`; `dual_in` rescans a literal's occurrences
  from the head, quadratic in equal tokens. `linlog prove -i --jobs 1
  --timeout 5s` on `GPPP_G-PPP-1000-10_10_1` answers after 140 s.
- **A pool, on a small problem.** `prove -i --jobs 4 --timeout 5s` on
  `ILLTP-SYJ-cbv/SYJ202+1.008` (19 KB) had not ended after 150 s; with
  `--jobs 2` it ends after 46 s and reports 268 million stable sequents
  with 578 memo entries; with `--jobs 1` after 5.2 s. The cause is not
  known. Both baselines saw it at sixteen threads only, as a kill.
- **The calling thread of the two alternating searches** waits for the
  backward search's slice without polling the caller's stop.
- **Before the first poll**: the set-up of a search on a forest of
  millions of occurrences, and in the command the parse, which the limit
  does not cover at all (88 s under `--timeout 1s` on the library's
  largest file, 33 s of it reported by the search).
- **The net engine's cubes** are quadratic on the sequents the net engine
  is the default for: `wide-m1` at 512 pairs takes 1 024 links and 33 ms
  on one thread, 524 800 links and 9.5 s on two. The enumeration never
  reaches its count of cubes where every link is forced, and starts
  again at every depth.

Step 18 added work after the search that no stop reaches inside the
library: `prove_goal` ends with the check of the proof it found
(`Options::check`; 14 ms on the largest net measured, one pass), and a
caller that asks for a derivation pays a pass for its size first. The
command polls its own condition again while a derivation is built and
written (`halt` in `cli/src/prove.rs`). The harness's child prints
`loaded` before it searches, the parent counts its kill from that line,
and at the review of step 18 the harness still reported 5.2 to 7.5 s on
five-second limits for the larger `TokenRing-50` and
`DrinkVendingMachine` nets.

Also: `--jobs` has no bound (10 000 threads on `A |- A` cost 183
CPU-seconds), `Options::portfolio` has shown no gain in two baselines,
and the second assertion of `focus::parallel::tests::agree` claims what
`.claude/rules/core.md` calls false.

## Goal

A time limit is kept to within a fraction of a second, on one thread and
on a pool, on every problem of the library, counted from the moment the
command starts; and a pool is never slower than one thread by more than
its start.

## What to build

1. **The stops**, each with the instance above as its test and a
   statement of where the engine now polls: the forced chains (and
   `dual_in` in time linear in what it returns); the wait in
   `alternate`; the search's set-up, polled or bounded; the check of the
   proof found and the size pass of a derivation, measured on the
   library's largest proofs and polled if either can pass the slack;
   whatever the pool's miss turns out to be. Find that cause before you fix anything
   around it, and say in the report what it was.
2. **The limit counts from the start.** The command's limit covers
   reading and parsing the sequent; a library caller's stop closure is
   polled from the first work a search does.
3. **The net engine on a pool** costs no more than the sequential
   search where the links are forced, and keeps the scaling the
   baselines measured on the Horn encodings (`bench/results/2026-10-02/
   parallel-net.csv`).
4. **`--jobs`** is bounded by something a user can reason about, with
   an error or a clamp that says so; the library's `Options::jobs`
   likewise.
5. **`Options::portfolio` is removed**, with its flag, its harness
   column kept readable in old files, and its tests.
6. **The `agree` test** asserts the contract of the rules file and no
   more.
7. **Tests** for a stop inside a forced chain, on a pool on the SYJ
   instance's shape, and for the command's `--timeout` with its message
   and exit status, which no test passes today.
8. **Documentation**: the rules file (where every engine polls, what a
   pool promises), CLAUDE.md, README, the help texts.

## Constraints

- What is searched does not change on one thread: every decided row of
  `bench/targets.sh` keeps the counters of `bench/targets/
  after-bias.csv`. A fix that must change the search (the net cubes on a
  pool do) says so and is measured.
- Memory bounds and input boundaries are step 20's; the defaults (which
  thread count, which time limit) are step 21's. Do not move them here.
- No `unsafe`; no lock held across a recursive call.
- Runs on more than four cores are not this step's to make; R7 shows at
  two and four threads.

## Verification

The checks of CLAUDE.md's table, `cargo test --workspace` with and
without the `parallel` feature, `bench/targets.sh` once, `nix flake
check` at the end. Then, in capped scopes on pinned cores: R6, R7 (at
two and four threads), R10 and R13 of the assessment, each ending within
its limit plus half a second; the thread table's net rows of the second
baseline at 1, 2 and 4 threads, no slower than before; and a
fresh-context reviewer's differential run of the parallel paths against
the sequential ones, as step 13 had.

## Deliverables

- Thematic jj commits, one per stop, then the cubes, the bound on
  `--jobs`, the portfolio's removal, the test.
- `plan/reports/19-time-limits.md`: the cause of each miss, what polls
  where, the measurements, decisions, deviations, open questions, what
  steps 20 and 21 must know.

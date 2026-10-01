# Step 15: performance pass on the focused engine

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D7, D8, D9, D10) and every report in `plan/reports/`,
  above all `14-benchmarks.md` ("The baseline's numbers", "What the numbers
  say about each engine", "Open questions and follow-ups; what step 15
  must beat"), `03-focused-engine.md` ("Performance observations", "Open
  questions and follow-ups"), `07-exponentials.md` ("Timings", "Open
  questions and follow-ups"), `08-intuitionistic.md` and `13-parallel.md`
  ("The focused engine on the pool", "Speedups").
- `proof-search-specifications.md`: the MLL, MALL and MELL sections on
  splitting, the count invariants, focusing and the atom bias, and
  "Cross-cutting engineering notes".
- `plan/later.md` § "Follow-ups: the focused engine" (the candidates this
  step takes up).
- `.claude/rules/core.md` (the focused engine's invariants: every prune
  there has an argument you must not break) and `.claude/rules/bench.md`.
- `core/src/search/focus/**`, `core/src/search/additive.rs`,
  `core/src/occurrences/mod.rs` (the forest's atom bias), `bench/**`.

## Where step 14 left things

The harness exists and is checked: `linlog-bench run` runs generated
families (`linlog::families`, 17 of them with verdicts known by
construction), LLTP files (`nix build .#lltp -o bench/lltp`) and problem
files, one child process per run, one CSV row per run with the verdict,
wall time, CPU time, run-queue wait and the engine's counters;
`linlog-bench summary` turns CSV files into tables, one column per
configuration, the CSV file's name being part of the configuration. The
baseline was taken in step 14, on a night the machine was the
benchmark's: its rows are in `bench/results/<day>/` and its tables in
`bench/RESULTS.md`, whose header names the commit it measured. That is
the record of the engines before your changes, and step 16 takes the
baseline again after them, on another such night. By day the machine is
shared, and you work by day.

So this step measures differently from a baseline, and the rule binds
everything below:

- **The engine's counters are the primary evidence.** On one thread the
  search is deterministic, so `nodes` (stable sequents visited), `splits`,
  `memo_hits` and `memo_entries` are the same on any machine under any
  load. A change that halves the splits of an instance has done so
  whatever the clock says.
- **CPU time is the secondary evidence**: `cpu_ms` of a sequential run
  pinned to one performance core (`taskset -c 3`, say; CPUs 0 to 3 are
  the performance cores), the median of three runs for anything under ten
  seconds, and a run whose `wait_ms` is over a few percent of its time is
  repeated rather than believed. Wall time is reported but not argued
  from.
- **The machine is shared.** At most two cores busy with measurements at
  a time, no run over five minutes (`--timeout 300`), nothing on every
  core, no `bench/baseline.sh`: the whole-library and all-core numbers
  are step 16's. Every measurement and every scratch
  program, yours or a sub-agent's, runs in a memory-capped scope of its
  own (`systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0
  taskset -c 2-3 …`): an unbounded scratch checker took 62 GB during step
  14 and the kernel killed the whole terminal with the session in it.
  Tell every sub-agent this in its brief, with the reason.
- **Before and after are measured by you, in this session**, on the same
  set with the same command, not read off the baseline: its times were
  taken under other conditions. Its counters are another matter: a
  sequential row of yours at the commit you start from must reproduce the
  `nodes` and `splits` of the same row in the baseline, which is a check
  on your set-up worth making once.

## Goal

The focused engine decides, or decides much faster, the instances the
benchmarks showed it losing on, without changing a single verdict: fewer
`⊗` splits examined, fewer stable sequents visited, no limit on the width
of a context it can split. Every change is measured before and after on a
fixed target set, keeps every proof passing the checker, and is argued
sound in `.claude/rules/core.md`.

## What to build

1. **The target set and its command**, before any engine change: a script
   (`bench/targets.sh`, or a named set the harness knows) that runs, on
   one pinned core with the caps above, the instances this step is judged
   on and writes one CSV file per label outside `bench/results` (which
   the baseline's `--fresh` deletes), so that `linlog-bench summary
   before.csv after.csv` prints them side by side. The set, from the
   report: `3-partition-no` at bins of 4 (52.6 s, 3.9 billion splits by
   step 3's count) and 5 (undecided), `partition-yes` at 5, 6 and 7,
   `partition-no` at 4 and 5, `mix` at 8 to 11, `qbf` at 16 and 20,
   `counter` at 8 and 16 and `counter-over` at 8, both classically and
   intuitionistically, `growing`, `chain`, `wide-m3` at 24 and 30 and
   `wide-m4` at 24 and 28 on the focus engine (the dispatch sends both
   there), the rows of `bench/problems/slow-tests.txt`, and a fixed
   sample of LLTP problems: a few dozen each of those the report counts
   as ending at the time limit, at the copy bound, at the recursion limit
   and at the 63-member limit (pick them by name from a first pass and
   list the names in the script, so the sample is the same before and
   after). Sizes that time out before stay in the set: deciding them is
   the point. Take the LLTP names from the baseline's
   `lltp-intuitionistic.csv`, whose `reason` column says how each problem
   ended. Commit the script, the before file and, at the end, the after
   file and a short table (`bench/TARGETS.md`).
2. **Splits without enumeration.** Today `split` moves members one at a
   time through all `2^n` submasks in Gray-code order
   (`enumerate_split`, `submasks`) and tests the counts after every flip;
   one stable sequent of a Petri net examined 60 million splits, and a
   context of more than 63 members is refused (`MAX_SPLIT`,
   `Reason::ContextTooWide`, which stops 651 LLTP problems). Replace the
   enumeration by a search over the members that prunes on the running
   tallies: decide the members in an order that fixes an atom's count
   early, and cut a partial assignment as soon as the interval check or
   the count equation cannot be met by any completion of it (bounds from
   what the undecided members can still contribute per atom). The set of
   splits whose both sides pass the counts must be exactly the set the
   enumeration passes to `premises` today, in a deterministic order; only
   the rejected ones stop being visited. No mask of the whole context, so
   no width limit: `ContextTooWide` should disappear or move out of
   reach, and say which. Keep the forced splits (`forced_side`) first,
   the goal on the consequent's side in intuitionistic mode, the split
   poll (`SPLITS_PER_POLL`), Mix's enumeration on the same code, the
   parallel chunking (`split_parallel` fixes the first members' sides per
   task: the chunks must still partition the splits), and
   `focus::split_passes` equal to what the engine prunes, since the
   interactive state lends it to clients.
3. **Identical members.** Hypotheses that are the same formula are
   distinct occurrences, so choosing `k` of `n` equal members for a side
   gives `C(n, k)` splits and as many memo keys where there is one up to
   renaming: the counter program answers millions of visits from
   thousands of entries, and sixteen tokens do not finish. Two parts,
   separately committed and measured: (a) in a split, among members that
   are interchangeable, send the lowest ids left, so only the number
   taken varies; (b) the memo's failures keyed up to renaming of
   interchangeable occurrences (a failure is invariant under it; a proof
   is not, since it names occurrences, so a `Proved` entry stays keyed by
   occurrences or is renamed on a hit, whichever you can argue).
   "Interchangeable" needs a definition you prove: the same term is
   necessary, and check what else is (the zone, the intuitionistic
   position, membership in the branch stack's keys, the copy bookkeeping
   of `?` formulas). The same canonical choice applies to focus
   candidates: two equal members are one candidate.
4. **The atom bias per problem.** The forest gives every atom a polarity
   once (`Forest::bias`), and the rule in force makes the bodies of Horn
   clauses negative, so their `⊗` splits are enumerated instead of
   forced, which is the 3-Partition refutation's cost
   (`.claude/rules/core.md`, "The atom bias hurts Horn clauses"). Focusing
   is complete for every assignment of polarities to atoms, so the bias
   is a free choice per problem: find one that makes more splits forced
   on the Horn-like families without losing on the others (a choice from
   the shape of the sequent, not a search over biases), keep it a
   function of the input so the run stays deterministic, and measure it
   on the whole target set. If no single rule wins everywhere, an
   `Options` knob with the better default is acceptable, and then the
   harness gets the axis (`.claude/rules/bench.md`, "A configuration
   axis").
5. **Smaller candidates, each only if the counters say so**: a forced
   rule for a `⊗` factor that is itself a tensor of positive literals; a
   restart of each copy-bound level from the frontier of sequents the
   level below exhausted, instead of re-exploring from the root; a hash
   per branch-stack entry for the loop check; memo keys stored in an
   arena. Measure each alone; drop the ones that do not pay and say so.
6. **Two limits the LLTP pass hit.** The additive path's memo is
   unbounded (8 GB at depth 16 of the `additive` family): give it the cap
   the focused memo has (`Options::memo_limit`) or drop entries in a way
   you can argue, and measure that depth 16 still decides. The recursion
   limit of 2 048 stops 895 Petri nets whose markings are long tensor
   chains: find out on the sample what depth they need, whether the
   engine can avoid a level of recursion per link of a `⅋` or `⊗` chain,
   and otherwise whether the default should rise (the stack grows with
   it, `Options::stack_size`); decide from the numbers.
7. **Documentation**: `.claude/rules/core.md` gets, for every prune and
   every canonical choice you add, the statement and the argument, and
   the stale numbers there are replaced; `.claude/rules/bench.md` the
   target set; README only where behaviour a user sees changed (a limit
   gone, an option added).

## Constraints

- No verdict changes, in any mode, on any input: `Proved` stays `Proved`,
  `Unprovable` stays `Unprovable`, and `Unknown` may only become decided.
  Every proof passes the checker. The generator tests, the parallel
  agreement tests and the flake's `bench` check (every family's smallest
  size against its known verdict) must pass after every commit.
- Each of items 2, 3a, 3b and 4 is soundness-critical: before you call
  one done, a fresh-context reviewer compares the engine before and after
  the change on random sequents in every mode (the classical and
  intuitionistic generators in `search::generate`, mutants included,
  linear, affine and Mix, several copy bounds), thousands of cases, with
  the scratch programs capped as above and the enumerations bounded by
  size. A differential run against the previous commit's binary is the
  simplest form.
- The engines are changed, not forked: no second split routine kept
  beside the first, no option that switches an optimisation off except
  the bias knob if item 4 needs it.
- The parallel path keeps working and keeps its guarantee (another proof,
  never another verdict); its speedups are not re-measured here, since
  that needs the machine, but one run on four threads of two targets
  confirms nothing regressed badly.
- No `unsafe`, no new dependency, determinism on one thread as before
  (`--deterministic` counts are a function of the input).

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --each-feature -p linlog` and
`cargo hack check --feature-powerset --depth 2 -p linlog` if a feature
gate moved, `nix flake check` at the end (`jj st` first), and the target
set before and after with the table pasted into the report. Report the
counters and the CPU times faithfully, including the targets nothing
helped and the candidates that did not pay.

## Deliverables

- Thematic jj commits, one per optimisation, each naming what it does
  ("Search the splits of a tensor by their counts", "Choose among equal
  members canonically", "Key memo failures up to renaming", "Pick the
  atom bias from the sequent's shape", "Cap the additive memo", …), each
  building and passing its tests alone.
- `plan/reports/15-performance.md`: the before and after table
  (counters first, CPU time second), what each change contributed, the
  soundness argument of each in a paragraph with a pointer to the rules
  file, what did not pay, which targets remain undecided and why, which rows
  of the baseline step 16 should look at first, and what is left for the
  net engine's pruning and the inverse method, both candidates in
  `plan/later.md`.

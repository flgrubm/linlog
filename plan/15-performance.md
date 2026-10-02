# Step 15: performance pass on the focused engine

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing.

This step is being run a second time, for the one thing its first session
left open and named as the largest gain left on the table: the default
atom bias when the sequent has exponentials. Everything else the step
asked for is built, reviewed and accepted; do not redo any of it. Read
before you start:

- `plan/reports/15-performance.md`, the first session's report, which is
  yours to extend: "Outcome", "What each change contributed" (item 4),
  "The reviews", "Targets that remain undecided, and why", "For step 16".
- `.claude/rules/core.md`: "Atom bias", "The copy budget", "Memo contract
  with the bound", "Complete failures are keyed up to interchangeable
  members", "The loop check", "A factor that is a tensor of positive
  literals forces its side too", and "The parallel runtime".
- `plan/README.md`: D7, D8 with the paragraph under its table, D9, D11,
  D15, and the Status entry of step 15's review.
- `plan/later.md`, "Follow-ups: the focused engine", its first entry.
- `core/src/search/focus/**`, `core/src/search/mod.rs` (`Options`,
  `Bias`, `run`'s deepening), `core/src/occurrences/mod.rs`
  (`Forest::bias_under`), `bench/targets.sh`, `.claude/rules/bench.md`.
- "What the step first asked for" at the end of this prompt, for how
  this step measures and what its constraints are; they hold for you.

## Where the step stands

`Options::bias` has three values. `Rarer` makes the literal of an atom
with fewer occurrences positive; `Factors` makes the one positive that is
more often a direct factor of a `⊗`, so that more splits are forced, and
on Horn-like hypotheses under `!` (a Petri net) that is forward chaining.
`Auto`, the default, is `Factors` without exponentials and `Rarer` with
them or in affine mode. The reason for `Rarer` with exponentials is the
copy bound: a forward chain takes one copy per step on a single branch,
so within the default bound of 3 the factor rule turns proofs into
`Reason::CopyBound` (the counter family at every size), which a default
must not do. What that default leaves unused, measured on the target
set's LLTP sample (109 intuitionistic problems, 5 s):

| | proved | refuted | copy bound or recursion limit | time limit |
|---|--:|--:|--:|--:|
| `rarer` (the default), bound 3 | 11 | 0 | 30 | 68 |
| `factors`, bound 3 | 23 | 0 | 78 | 8 |
| `factors`, bound 10 | 48 | 1 | 23 | 37 |

(From the first session's report, the third column by subtraction; take
the three runs again as your `before`.) `factors` at bound 3 loses 3 of the default's 11
proofs to the bound and answers 45 of its 68 timeouts at the bound within
milliseconds; at bound 10 it has every one of the 11. And one problem
shows what the default costs outside the sample: the planning session
ran the engine on every LLTP row the first baseline had decided (1 890
rows of its three sequential passes, 5 s), and all keep their verdict
but the Petri net `ILL/petri-nets/MCC/IBM5964_1_1.p`, which the engine
before the first session proved in 1.8 s and 14 million stable
sequents, and which now takes 42 s and 256 million under `rarer` (the
order of the search changed) and 0.08 ms under `factors`.

## Goal

A default that a user never has to second-guess: with exponentials,
`Bias::Auto` decides what either rule decides, as far as the limits
allow, without the user knowing that forward chaining wants its copy
bound raised, and without ever answering less than today's default does
under the same options. The second baseline (step 16, a night) then
measures the engine users get.

## What to build

1. **The combined default.** For a sequent with exponentials in linear
   mode, classical and intuitionistic, `Bias::Auto` uses both rules. The
   design is yours; what it must satisfy:
   - **Never less than today.** With no stop condition firing, `Auto`
     never answers `Unknown` where `Bias::Rarer` under the same options
     decides, and never another verdict. This is a contract, stated in
     the rules file and tested, not a tendency.
   - **The gain is taken.** What `Factors` decides is decided by `Auto`
     too, the proofs that need more copies on one branch than
     `Options::copies` included as far as you can argue it: say what
     bound the forward search runs under and why (a bound of its own, a
     count that treats a chain of forced steps differently, a multiple
     of `Options::copies`), keep the documented meaning of
     `Options::copies` for the search it describes today, and keep
     `Reason::CopyBound(n)` truthful about what was tried. If it needs a
     knob, it is a field of `Options` with a default (D15), a flag of
     the CLI and an axis of the harness; no constant a user might want
     to move.
   - **Neither rule starves the other under a limit.** Eight of the
     sampled nets time out under `Factors` as well, and `Rarer` runs
     into the limit on most, so "one, then the other" spends a time
     limit on the first. The crate has no clock (D11): the turns are
     counted in the engine's own units (stable sequents, split steps),
     on budgets that grow, so that a run is a function of the input on
     one thread and a problem either rule decides quickly is decided
     quickly. Say what the scheme costs against the better rule alone,
     in the worst case and on the targets.
   - **What is shared between the two searches is only what holds under
     both.** Provability does not depend on the bias, so a proof and a
     complete failure are facts for both; a failure cut by the budget is
     a statement about one rule's search space under one budget, and the
     loop check is about one branch of one search. Argue each thing you
     share in `.claude/rules/core.md`; when in doubt, do not share.
   - **`Unprovable` keeps its meaning**: a level searched to its end
     without a cut, under either rule, since focusing is complete for
     every bias.
2. **What stays as it is, as the oracle.** `Bias::Rarer` and
   `Bias::Factors` named explicitly search exactly as they do now, and
   `Auto` without exponentials and in affine mode is what it is now: on
   the target set the counters (`nodes`, `splits`, `memo_hits`,
   `memo_entries`) of every decided row without exponentials are
   identical to `bench/targets/after.csv`, and a run of the LLTP sample
   under each explicit bias reproduces what you took as `before`.
3. **On a pool.** The two searches are independent, which is what a
   pool is good at, and `Options::portfolio`, which reorders
   alternatives per worker, has never shown a gain. If the combined
   default falls out naturally as the two rules side by side when
   `jobs` is above one, build that, and say what should become of the
   portfolio; if it does not, leave the parallel path on the scheme of
   item 1 and say so. Either way the parallel guarantee stands: another
   proof, never another decided verdict.
4. **Measured.** `bench/targets.sh after-bias`, committed with a column
   in `bench/TARGETS.md`: no row that `after.csv` decides is lost, the
   LLTP sample decides at least what the two explicit runs decide
   between them at the default bound, and the report says how close it
   comes to `factors` at bound 10. The check the planning session made
   is yours to repeat, since it is what found the lost net: every LLTP
   row the first baseline decided, in `lltp-intuitionistic.csv`,
   `lltp-classical.csv` and `lltp-copies-10.csv` under
   `bench/results/2026-09-30/`, run again at 5 s on one pinned
   performance core (`run --lltp … --only` with the `FAMILY/NAME` of
   the decided rows; three runs of about a minute each, named here, so
   they are measurements you may make), and every one decided with its
   verdict, `IBM5964_1_1` among them.
5. **Documentation.** `.claude/rules/core.md` ("Atom bias" rewritten for
   the default as it is, with the contract and the argument for what is
   shared), the doc comments of `Bias` and `Options::bias`, the CLI's
   `--bias` help, README's paragraph on the bias with its example
   regenerated, `.claude/rules/bench.md` and `bench/TARGETS.md` for the
   new label.

## Constraints

- The first run's constraints hold (below): no verdict changes, every
  proof passes the checker, no `unsafe`, no new dependency, determinism
  on one thread, the engine changed and not forked.
- This is soundness-relevant. Before you call it done, a fresh-context
  reviewer compares the engine at your head with the engine at the
  commit you start from, on generated sequents with exponentials in
  classical and intuitionistic linear mode (the generators of
  `search::generate`, mutants included, copy bounds 0 to 3 and the
  generator's own, memo limits default, 0 and 2, one thread and four):
  no `Proved` against an `Unprovable`, every proof checked, and, which
  is this session's own contract, no case decided at the start and
  `Unknown` at the head without a stop. With a stop condition that
  fires after a random number of polls: no contradiction, and a
  decided verdict only where the unstopped run has the same. Scratch
  programs capped and pinned as the first run's.
- By day on a shared machine: at most two cores busy with measurements,
  no run over five minutes, nothing on every core, no
  `bench/baseline.sh`; every measurement and scratch program in a
  memory-capped scope (`systemd-run --user --scope -p MemoryMax=8G -p
  MemorySwapMax=0 taskset -c 2-3 …`), and every sub-agent told so with
  the reason. A measurement this prompt does not name is asked for
  first.
- Commit as you go, each commit building and passing its tests alone:
  the signing key's passphrase is cached for two hours at most, and a
  session that leaves its commits to the end finds that it cannot make
  them (CLAUDE.md, "Version control").

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --each-feature -p linlog` and
`cargo hack check --feature-powerset --depth 2 -p linlog` if a feature
gate moved, `cargo run --release -p linlog-bench -- run --all-families
--timeout 5` for the verdicts, the target set under its new label, the
check of item 4, and `nix flake check` at the end (`jj st` first).

## Deliverables

- Thematic jj commits ("Decide a sequent with exponentials under both
  biases", "Record the target set under the combined default", …).
- `plan/reports/15-performance.md`, extended, not rewritten: a section
  "The default bias" (the scheme and why, the contract and its argument,
  what is shared, what it costs, the numbers of item 4, the review), the
  summary at the top and "For step 16" brought up to date (which passes
  under an explicit bias the second baseline still needs as the record
  of the two components, and which rows to look at), and the follow-up
  in "Targets that remain undecided" turned into what is left.

## What the step first asked for

Kept for how the step measures, its constraints and what the pass was
for. All of it is built; the report says how.

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

### Where step 14 left things

The harness exists and is checked: `linlog-bench run` runs generated
families (`linlog::families`, 17 of them with verdicts known by
construction), LLTP files (`nix build .#lltp -o bench/lltp`) and problem
files, one child process per run, one CSV row per run with the verdict,
wall time, CPU time, run-queue wait and the engine's counters;
`linlog-bench summary` turns CSV files into tables, one column per
configuration, the CSV file's name being part of the configuration. The
baseline was taken in step 14 on the night of 2026-09-30, when the
machine was the benchmark's, on commit `b53cb17c6831`, and completed by
a supplement on the night of 2026-10-01 (the net engine's cubes on MLL
3-Partition and its one-thread Partition table, which the first night's
filter had dropped, and a fourth stage that ran the killed or crashed
runs of `bench/reruns.txt` again with more grace and memory, into
`*-generous.csv`): its rows are in `bench/results/2026-09-30/`, whose
`starts.txt` names both starts, and its tables in `bench/RESULTS.md`.
The engines were identical on both nights. That is the record of the
engines before your changes, and step 16 takes the baseline again after
them, on another such night. By day the machine is shared, and you work
by day.

### What the baseline found in the engines

Beyond the times, the night turned up defects of the focused engine that
this step's changes must remove, since they are in the code the step
rewrites (the report's "Open questions and follow-ups" and
`.claude/rules/core.md`, "Such places exist, in the split enumeration"):

- **The stop is missed inside a long split enumeration.** 90 one-thread
  LLTP runs on files under 2 MB ran past the harness's kill at 10.5 s
  under a 5 s limit; the Petri net `ILL/petri-nets/MCC/AutoFlight_afcs_05_a_1_1.p`
  examines 1.8 billion splits at one stable sequent and stops after
  242 s. Given 600 s before the kill, 37 such runs stop 10 s to 506 s
  after their start, and two Petri nets are proved after their limit,
  with checked proofs (`TokenRing-15-unfolded_1_1` at 21.6 s and
  `TokenRing-20-unfolded_1_1` at 552 s, under 5 s and a recursion limit
  of 16 384; `lltp-recursion-generous.csv`): a missed stop is also a
  late verdict. The poll every `SPLITS_PER_POLL` splits does not reach
  that loop. `linlog prove --timeout` runs the same engine, so a user's time
  limit overruns the same way. After item 2 the split search must poll
  the stop condition at a bounded interval of work wherever it loops,
  and a run of that problem under a 2 s limit must end within a few
  seconds; pin that with one test on a generated instance that shows the
  same behaviour, not on the library file.
- **The proof arena grows without bound under a failing enumeration.**
  Nine probed runs aborted with `memory allocation of 17179869184 bytes
  failed`, the backtrace showing `Engine::push` growing the proof-node
  arena from `initial` inside `enumerate_split`, at about 110 MB a
  second: nodes are pushed for premises that the split then rejects and
  are never reclaimed. Seven more one-thread reruns aborted the same way
  in the supplement, after 53 s to 469 s under 16 GiB (which allows the
  arena no further doubling than 12 GiB did). Two aborts that looked
  alike are not this: the five at sixteen threads on the largest SYJ
  problems end cleanly with 32 GiB (12 GiB was too little for a pool on
  8 to 15 million occurrences), and for the focused engine forced onto
  the additive family at depth 16 the harness does not say which
  allocation fails, since it keeps only the last line of a crashed
  child's error output. Nodes a failed branch pushed
  must not stay in the arena (truncate on backtrack, or push only once a
  split's premises are proved; on the shared arena of the parallel path
  argue what is safe), and the memory of a refutation must stay bounded
  by the memo, not by the splits examined.
- **At sixteen threads, 166 small Petri nets that one thread stops at
  5 s ran past the kill** (`lltp-all-cores.csv`, `reason` `killed`): the
  same miss, reached through the workers. The fix above must hold on
  the pool too; the four-thread runs the constraints ask for are where
  to see it.
- **The additive memo is unbounded**, as item 6 says.

### How this step measures

By day the machine is shared, so this step measures differently from a
baseline, and the rule binds everything below:

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

### Goal

The focused engine decides, or decides much faster, the instances the
benchmarks showed it losing on, without changing a single verdict: fewer
`⊗` splits examined, fewer stable sequents visited, no limit on the width
of a context it can split. Every change is measured before and after on a
fixed target set, keeps every proof passing the checker, and is argued
sound in `.claude/rules/core.md`. Last, and within a stated bound, the
cost of a unit of search: a few changes of representation that a profile
points at and the counters prove behaviour-neutral (item 7).

### What to build

1. **The target set and its command**, before any engine change: a script
   (`bench/targets.sh`, or a named set the harness knows) that runs, on
   one pinned core with the caps above, the instances this step is judged
   on and writes one CSV file per label outside `bench/results` (which
   the baseline's `--fresh` deletes), so that `linlog-bench summary
   before.csv after.csv` prints them side by side. The set, from the
   report: `3-partition-no` at bins of 4 (49.8 s in the baseline, 3.9
   billion splits by step 3's count) and 5 (302 s), `partition-yes` at
   5, 6 and 7 (the last over 1 200 s), `partition-no` at 4 and 5 (811 s),
   `mix` at 8 to 11 (214 s at ten), `qbf` at 16 and 20,
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
   ended, and add the Petri nets of `bench/reruns.txt` whose enumeration
   misses its stop (`AutoFlight_afcs_05_a_1_1` first), which the
   `*-generous.csv` rows time, with the two TokenRing nets above under
   `--recursion-limit 16384`, which are provable and should become
   quick. A target that today runs past its limit is run with a kill
   you can afford (`--grace`) and counted as such. One small change to
   the harness belongs here, since the arena defect's evidence is a
   backtrace: a crashed child's whole error output goes to the run's
   log, where today only its last line survives, in the `reason`
   column. Commit the script, the before file and, at the end, the
   after file and a short table (`bench/TARGETS.md`).
2. **Splits without enumeration.** Today `split` moves members one at a
   time through all `2^n` submasks in Gray-code order
   (`enumerate_split`, `submasks`) and tests the counts after every flip;
   one stable sequent of a Petri net examined 60 million splits, and a
   context of more than 63 members is refused (`MAX_SPLIT`,
   `Reason::ContextTooWide`, which stops 845 LLTP problems, and 401 more
   once the recursion limit is raised). Replace the
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
   limit of 2 048 stops 983 LLTP problems, Petri nets whose markings are
   long tensor chains (a limit of 16 384 decides 56 of them, leaves 154
   at the limit and sends 401 to the width limit of item 2): find out on
   the sample what depth they need, whether the
   engine can avoid a level of recursion per link of a `⅋` or `⊗` chain,
   and otherwise whether the default should rise (the stack grows with
   it, `Options::stack_size`); decide from the numbers.
7. **Constant factors, last and bounded.** Everything above changes how
   much the engine searches; this item is about what a unit of search
   costs, and it is deliberately small. When items 2 to 6 are committed
   and measured, profile the targets that still take a second or more: a
   sampling profiler on a release build with debug symbols, set through
   the environment for that build (`CARGO_PROFILE_RELEASE_DEBUG=true`),
   not in `Cargo.toml`. None is installed, so take one from the flake's
   nixpkgs for the session (`nix shell --inputs-from . nixpkgs#…`),
   capped and pinned like every other measurement. Then take at most
   three changes of representation, each only if all of this holds:
   - it is local, behind the API of a type that exists, with no new type
     parameter through the engines (D2 and D3 removed those on purpose);
   - it leaves `nodes`, `splits`, `memo_hits` and `memo_entries` of
     every sequential target bit-identical, which is the whole proof
     that it changed no behaviour and the reason such a change needs no
     reviewer;
   - it lowers the pinned CPU time of the targets it concerns by a tenth
     or more, median of three runs.

   The first candidate is the one D3 names and nobody built: `OccSet` is
   a `Box<[u64]>` at every size, so every set over a forest of at most
   64 occurrences is a heap allocation and an indirection where one word
   inline would do. Others a profile may show: the memo key's layout and
   hashing, allocation in the split search of item 2, and the release
   profile itself (`lto`, `codegen-units`), which costs build time in
   every check, so say what it costs. A candidate that fails a condition
   is not taken. What the profile shows and you do not take goes into
   the report as a ranked list (the hot spot, its share of the samples,
   the change it suggests), which the code-audit candidate of
   `plan/later.md` starts from. Here CPU time is the evidence and the
   identical counters the guard, the reverse of the items above. Do not
   let this item grow: it is the last thing the step does and not a
   refactoring pass.
8. **Documentation**: `.claude/rules/core.md` gets, for every prune and
   every canonical choice you add, the statement and the argument, and
   the stale numbers there are replaced; `.claude/rules/bench.md` the
   target set; README only where behaviour a user sees changed (a limit
   gone, an option added).

### Constraints

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

### Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --each-feature -p linlog` and
`cargo hack check --feature-powerset --depth 2 -p linlog` if a feature
gate moved, `nix flake check` at the end (`jj st` first), and the target
set before and after with the table pasted into the report. Report the
counters and the CPU times faithfully, including the targets nothing
helped and the candidates that did not pay.

### Deliverables

- Thematic jj commits, one per optimisation, each naming what it does
  ("Search the splits of a tensor by their counts", "Choose among equal
  members canonically", "Key memo failures up to renaming", "Pick the
  atom bias from the sequent's shape", "Cap the additive memo", "Keep
  small occurrence sets inline", …), each building and passing its tests
  alone.
- The target set under three labels: before, after items 2 to 6, and
  after item 7, so that step 16 can tell what the search changes gained
  from what the representation did.
- `plan/reports/15-performance.md`: the before and after table
  (counters first, CPU time second), what each change contributed, the
  profile of item 7 with its ranked list of what was not taken, the
  soundness argument of each in a paragraph with a pointer to the rules
  file, what did not pay, which targets remain undecided and why, which rows
  of the baseline step 16 should look at first, and what is left for the
  net engine's pruning and the inverse method, both candidates in
  `plan/later.md`.

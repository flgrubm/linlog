# Step 14: benchmarks, LLTP input and the hard families

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing.

This step is being run a second time, to be completed. Its first session
built and committed everything the step asked for except the one thing
the step is for: the baseline was never taken. You finish it. Read before
you start:

- `plan/reports/14-benchmarks.md`, the first session's report, which is
  yours to finish: "Tonight: what the administrator does", "The machine
  and its noise", "How it runs", "Preliminary numbers", "The runs, and why
  the baseline is pending", "Open questions and follow-ups".
- `.claude/rules/bench.md` (the harness's invariants and the machine),
  `bench/baseline.sh` from its first line to its last, `bench/src/**`,
  `modules/bench.nix`.
- `plan/README.md`: the step table, D15, and the Status entries of
  2026-09-30.
- "What the step first asked for" at the end of this prompt, for what the
  numbers are meant to answer.

## Where the step stands

Built, reviewed and committed, all checks passing; do not rebuild any of
it: the LLTP reader (`linlog::lltp`), seventeen problem families with
verdicts known by construction (`linlog::families`), the harness
(`linlog-bench`: a child process per run, one CSV row per run with the
verdict, the engine's counters, wall time, CPU time and run-queue wait;
Markdown summaries), the flake's `bench` check and `lltp` package, the
documentation, and one engine fix (the focused engine polls its stop
condition every 4 096 splits).

Not done: the baseline. A first run died after twenty minutes when a
reviewer's scratch program took 62 GB and the kernel's OOM killer took
the terminal with it. A second shared the machine with other work and was
stopped; its rows are in `bench/preliminary/` and are what the report
calls preliminary. A third, planned for a night, was cancelled. The first
session answered those failures in the harness: the baseline runs as a
detached systemd user unit with a memory limit and no swap, every child
is capped, the unit keeps the author's other slices off the performance
cores and gives them back however it ends, holds a sleep and lid
inhibitor, refuses to start on battery, on a busy machine or with a
scheduled job due, records CPU time, run-queue wait, throttling and the
power settings, and resumes an interrupted run. Those guards were tested
for a minute each, never through a night.

## The slot

The machine is the benchmark's from 20:00 to 07:00, as the author has
said, tonight for this step and on a later night for step 16, which takes
the baseline again after the performance pass (step 15). Outside that
slot the machine is shared and the rules in "How to work on this step"
below apply: capped scopes for scratch programs, nothing on every core,
no baseline.

## What remains

1. **Check the harness by day**, on the shared machine, in small runs
   only. Read the script against the report and the rules file, and run
   the detached path for a minute as the first session did (the shield,
   the inhibitor, the restore on stop). Inspect the machine again, as the
   author asked the first session to: the CPU topology, the governor,
   turbo and the thermal state, the services running, the sync clients,
   the suspend settings, and every timer, system and user, that is due
   between 20:00 and 07:00 (`systemctl list-timers --all`, with and
   without `--user`). Compare with "The machine and its noise" and correct
   the script or the report where the machine differs from what they say.
   Fix what you find; do not redesign what works.
2. **Two baselines must be able to sit side by side.** Step 16 takes the
   baseline again, so a baseline goes into a directory of its own named by
   the day it started (`bench/results/2026-09-30/*.csv` and its
   `RESULTS.md`), `--fresh` deletes only the directory it is about to
   write, a resumed run finds its own directory, and `bench/RESULTS.md` is
   the latest baseline's tables. The header of a baseline's `RESULTS.md`
   also names the commit the binary was built from, since comparing two
   baselines means comparing two commits. Make the smallest change to the
   script that gives this, and bring `.claude/rules/bench.md`, README and
   CLAUDE.md in line.
3. **The run starts and stops by itself.** Nobody is at the machine at
   20:00. The run starts unattended at 20:00 (a transient systemd user
   timer, or an option of the script that arms one), or as soon after as
   the machine is idle by the script's own test, and no later than the
   last moment from which its estimated duration still ends before 07:00;
   at that moment it starts whatever the load, and records that it did,
   since a night not used is worse than rows marked as disturbed. The
   unit stops by 07:00 whatever its state, gives the cores back, releases
   the inhibitor and leaves a state that the script, run again without
   `--fresh`, finishes on another night. What needs no root the unit does
   and undoes itself (the user timers in the slot, the user slices). If
   the estimate does not fit the slot with a margin, say which stage
   would be cut and order the stages so that the sequential ones, which
   step 15 needs, come first.
4. **What only the author can do**, in one block to paste before leaving
   and one to paste in the morning: the system timers to stop and start
   again (`nix-gc`, `nix-optimise` and whatever item 1 finds), the
   optional restriction of the system's own services to the efficiency
   cores, the sync clients, and mains power with the lid open. Give the
   blocks in your last message before the night, exactly as they are to
   be typed.
5. **Do not wait through the night.** Once the start is armed, confirm it
   (`systemctl --user list-timers`), give the author the blocks of item 4
   and the time the run should end, tell them to come back after 07:00
   and say "continue", and end your turn. Do not poll overnight: the unit
   writes its progress to the journal, and every wake-up of this session
   costs tokens and adds nothing.
6. **In the morning.** Read the journal of the run: how long it took,
   the load it started with, the jobs that fired, the throttling, how many
   rows waited for a CPU. If the run did not finish, say what is missing
   and leave the rest for a night; do not run it by day. Commit the
   results directory and `bench/RESULTS.md` as "Record the baseline".
   Remove `bench/preliminary/`, which the baseline supersedes. In the
   report, replace the preliminary numbers by the night's, turn "Tonight"
   and "why the baseline is pending" into what happened, and bring "What
   the numbers say about each engine" and "what step 15 must beat" up to
   the measured rows: the parallel columns against step 13's speedups,
   the portfolio, the raised copy bound and recursion limit on LLTP,
   classical against intuitionistic. Then correct every place that says
   no baseline exists (`CLAUDE.md`, `README.md`, `.claude/rules/core.md`,
   `.claude/rules/bench.md`; search for "not been taken" and
   "preliminary").

## Constraints

- The engines are not touched: the baseline measures the commit that is
  checked out at 20:00, and step 15 changes the engines afterwards.
- No dependency is added; the script stays a script.
- Report the numbers faithfully, including the rows marked as disturbed
  and the families where every engine times out.

## Verification

By day: `cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace` and `nix flake check` (`jj st` first) after the
script and documentation changes, and the one-minute detached run. In the
morning: `nix flake check` again after the results and the report are in.

## Deliverables

- Thematic jj commits ("Keep every baseline in a directory of its own",
  "Start and stop the baseline unattended", "Record the baseline",
  "Report the baseline", …).
- `plan/reports/14-benchmarks.md`, finished: the baseline's tables and
  what they say, how long the run took and on what hardware and settings,
  what disturbed it, what step 15 must beat, and what step 16 must
  repeat exactly for its numbers to be comparable.

## What the step first asked for

Kept for what the measurements are meant to answer. Everything below is
built except the baseline run; "the machine is yours" in its verification
paragraph is now "The slot" above.

### Goal

A benchmark harness that reads the LLTP/ILLTP problem format and linlog's
own, runs each problem with a timeout on a fresh engine, records the
outcome, wall time, node count and memo size as CSV, and reports
solved-within-timeout counts per family; the hard families from the spec
generated on demand; and a place where the numbers are tracked so a later
change shows its effect.

### What step 13 left you

`plan/reports/13-parallel.md`. The search runs on a pool when
`Options::jobs` is above one (`--jobs N`, `-j`, default every core in the
CLI; `--deterministic` is one thread): the focused engine as nested
fork-join over the first two choices of a branch with and-parallel `&`
premises, a sharded memo and one proof arena shared by the workers; the
net engine as cubes of the first links pulled from a counter; the
additive path sequential. What the harness must respect: the sequential
engines are the reference, so every problem runs with `jobs` at one as
its baseline row, and the parallel columns (2, 4, 8, every core) are
further rows or columns of the same table, wall time being the number
that matters there since `Statistics::nodes` on a parallel run is the
sum over all threads (up to twice the sequential count on memo-bound
families, which is the duplicated exploration the report measured, not
the memo's locks); a pool is built per call, milliseconds, so problems
that take under a millisecond lose on any pool and the table should show
that rather than hide it; `Options::portfolio` exists and showed no
consistent gain on the report's families, so it gets a column only if a
family here disagrees. The stop closure of `prove_until` is polled on the
calling thread once a millisecond on a parallel run and once per stable
sequent on a sequential one, so a per-problem deadline through the
closure is reliable in both cases, and a fresh thread per problem is
still right for the recursion limit's stack (`Options::stack_size`) and
for isolating a runaway. The report's speedup table (3-Partition refuted
6.0× on 8 threads, the net engine's Partition instances 7.4× and 6.7×,
the counter programs 1.35× or a loss) is the number a parallel column
here should reproduce, and the ignored `speedups` tests in
`core/src/search/focus/parallel.rs` and `core/src/search/net.rs` hold
the generators for those families (`three_partition`, `counter`,
`partition`, test-private today): move them into the family generators
of item 2 rather than writing them again, and retire the ignored tests
once the harness covers them. For the performance pass (step 15), the
report names the duplicated exploration of and-parallel `&` premises and
of copies run as alternatives as the thing to measure first on
memo-bound families.

### What to build

1. **LLTP reader**: the `fof(...)` syntax used by LLTP/ILLTP (read the
   repository's format description and a few files; WebFetch is allowed),
   parsed into `Sequent` with mode flags (ILL problems are two-sided and
   run with `Mode::INTUITIONISTIC`; the parser already lowers `Γ ⊢ A`, and
   `Reading::new` must accept every ILLTP problem, so report any it
   refuses). Put it behind the `parse` feature in `core` or in the bench
   crate; decide by whether the web front end could want it.
2. **Family generators**: Kanovich's Horn encoding of 3-Partition and
   Matsuoka's encodings for MLL, the LMSS QBF encoding for MALL (Chaudhuri's
   qbf suites are the model), and a Petri-net style !-Horn family for MELL;
   each as a function from a size to a `Sequent` with a known verdict.
3. **The harness**: `linlog bench` or a `bench/` workspace crate (D4 allows
   it; choose by whether the CLI should carry the extra dependencies), with
   a per-problem timeout (a fresh thread or process per problem so a hung
   engine cannot take the run down), CSV output, a summary table, and the
   options of `prove` (engine, jobs, copies). Follow the spec's harness
   description.
4. **Problem sets**: a small curated set committed under `bench/problems/`
   (license permitting; LLTP's license must allow redistribution, else
   fetch on demand and document), the generated families at a few sizes,
   and the sequents from the earlier steps' slow tests.
5. **The engine routing and its knobs.** The net engine wins on distinct
   atoms and wide contexts (linear where the focused engine enumerates
   splits) and loses by orders of magnitude on Horn encodings with
   repeated literals (step 6's report, "Timings"), so the dispatch routes
   unit-free MLL to it only when no literal occurs more than twice
   (`NET_MULTIPLICITY` in `core/src/search/mod.rs`). Measure that
   threshold on the harness, and `Options::test_period` (the exact test's
   cadence; expose it in the CLI if the numbers want it), and record what
   the performance pass should try first: leaf symmetry breaking for pure
   `⊗`/`⅋` trees of equal literals (sound by the argument in
   `.claude/rules/core.md`; the spec forbade it), a per-atom balance over
   the `⊗`-skeleton components of a partial structure. The portfolio
   exists since step 13 (`Options::portfolio`, off by default) and the
   report's families showed no gain from it; measure it on the LLTP
   problems before the performance pass decides whether to keep it.
6. **The exponential families and their follow-ups.** Step 7's report
   ("Timings", "Open questions and follow-ups") measured the counter
   program `!(a ⊗ a ⊸ b), !(b ⊗ b ⊸ c), !(c ⊗ c ⊸ d), a^8 ⊢ d` (millions of
   memo hits on thousands of entries, because the eight `a` hypotheses
   are distinct occurrences of one formula) and a growing-context family;
   sixteen tokens did not finish. Put both in the harness at several
   sizes, with the ILLTP Petri-net problems when they are read, so that
   the performance pass can measure a canonical choice among identical
   members, a nested forced rule for a factor that is a tensor of positive
   literals, a per-level restart from the frontier of exhausted sequents
   instead of re-exploring, and a hashed loop check. Run the ILL problems
   and the two-sided forms of these families both classically and with
   `-i`: step 8's report measured the counter program at 21 ms two-sided
   against 126 ms classically (the ignored test `illtp_style_slow` in
   `core/src/search/focus/mod.rs`), and the table should say whether that
   holds across ILLTP. The additive engine (two additive-only formulas) is
   a row of its own; it is linear in the product of the sizes and should
   never appear among the timeouts.
7. **The known slow cases as benchmark families.** Step 3's report
   ("Performance observations", "Open questions and follow-ups") names
   them: refuting a wide sequent under Mix costs about `3^k`; refuting an
   unsolvable 3-Partition instance takes 55 s for bins of size 4 (3.9
   billion splits) because the atom bias makes the Horn clauses' bodies
   negative and every `⊗` split is enumerated. Make both families part of
   the harness at several sizes so that the performance pass after this
   step (branch-and-bound splitting instead of Gray-code enumeration, a
   per-problem atom bias, the tighter counts the report lists, memo keys in
   an arena) has numbers to beat. Measure, do not fix here.
8. **Tracking**: a `bench/RESULTS.md` (or CSV) with the current numbers per
   family and engine, and the command that regenerates it; not a CI job
   (timings on shared runners are noise), but a `nix flake check` entry
   that runs the harness on a tiny set to keep it building.
9. **Documentation**: README (how to run the benchmarks), CLAUDE.md
   (commands, the bench crate), `.claude/rules/core.md` if formats were
   added.

### Constraints

- No `unsafe`; dependencies scoped to the bench crate where only it needs
  them.
- Deterministic problem generation (seeded).
- Report numbers faithfully, including families where the engines time out.


### Verification, as first written

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo deny check` if a dependency changed,
`nix flake check` at the end (`jj st` first), and the baseline run in
release mode with the table pasted into the report. The machine is yours
for this step (the author is not using it otherwise), so the baseline may
be fuller than a quick pass: every family at three or four sizes rather
than two, per-problem timeouts of a few minutes on the hard families
rather than seconds, every engine that applies and the thread counts 1,
2, 4 and every core, the ILL problems both ways, and the whole LLTP set
you were able to fetch. Keep the whole run to a few hours of wall time
in total, not a day: pick sizes so that the largest instance of each
family times out and the rest do not, run the long instances once and
the short ones a few times for a median, and run the harness in the
background while you write the report so the machine is never idle. Say
in the report how long the run took and on what hardware. A run that
would exceed that budget is trimmed by dropping the largest size, never
by shortening the timeouts below what the families need to show their
scaling.

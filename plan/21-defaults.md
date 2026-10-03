# Step 21: the defaults a user meets

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: sections 2.1, 2.2, 2.5, 5.2 and 5.3,
  questions 2, 3 and 8 with the author's answers.
- `plan/reports/19-time-limits.md` and `20-memory-and-boundaries.md`.
- `plan/README.md`: D8, D9, D15, D16 and D19, and the Status entries of
  step 15's second session and of step 17.
- `plan/notes/distribution.md`, "Threads and time limits elsewhere".
- `.claude/rules/core.md`: "The copy budget", "The default bias with
  exponentials is two searches"; `.claude/rules/cli.md`.
- `core/src/search/mod.rs` (`Options`, the constants, `Reason`),
  `search/focus/mod.rs` (`run`, `plan`, `chains`), `cli/src/prove.rs`,
  `cli/src/interact.rs`, `cli/src/argument_parsing.rs`,
  `bench/src/run.rs`.

## What steps 17 to 20 left you

`linlog prove SEQUENT` without flags searches within three copies of a
`?` formula per branch, on every core, with no time limit. The baselines
and the LLTP library's own results say what that costs:

- The copy bound of 3 is the whole difference to the one prover the
  library records. On its 1 342 problems that prover decides 659 in five
  minutes each, linlog's default 526 in 5 s; the 196 it loses are all at
  the bound, and with `--copies 10` or the forward search at 30 copies
  it loses three and decides 158 that prover does not. Of the 832
  problems the first baseline left at the bound, a bound of 10 decides
  369 in 5 s, and 293 that answered "unknown" in a fraction of a
  millisecond run into the time limit instead. So the bound cannot
  simply rise.
- What a deepening default costs is the wait for an "unknown" (the
  planning session's count from the second baseline's rows, at the
  review of step 17). Outside the nets, 288 problems that the default
  leaves at its bound are decided by `--copies 10` or by the forward
  search at 30 copies: 195 of them within 1 ms, 242 within 100 ms, 269
  within 1 s, 282 within 2 s, all within 4.2 s. And 445 are decided by
  neither: today each answers "unknown" in a fraction of a millisecond,
  and under a deepening default each takes the whole budget. The author
  said on 2026-10-02 that an undecided sequent must not be slow. So the
  budget is a trade between a few late proofs and every such wait, and
  the rows put its knee near two seconds, not ten.
- Without a time limit 1 594 Petri nets of the library run until
  interrupted.
- A pool costs milliseconds on small problems (1.1 ms against 4.6 ms in
  the median) and gains seconds on large ones; every defect step 17
  found in the search was on the pool. One thread is the reference every
  review tested. Step 19 removed those defects (the net engine on a pool
  now costs what one thread costs where the links are forced, and the
  pool keeps its limit), and at its review the 1 358 problems of the
  library outside the nets ran on four threads under a limit of one
  second without a contradiction, a kill or a stop more than 0.43 s
  late. Two things remain on the pool, both step 29's: a cancellation at
  a `&` that can come late, and an error of one premise that cancels the
  other, so that a pool may answer "recursion limit" where one thread
  refutes.
- The command's limit is a flag that a timer thread raises
  (`cli/src/limit.rs`, `Deadline`), counted from the command's start
  with the reading and parsing under it; a default limit is
  `args.timeout` with a default. A sequent too large to read within it
  is answered "unknown: the time limit of … was reached while the
  sequent was read", which a newcomer with a large file will meet and
  must be able to act on. `--jobs` above the machine's parallelism is
  clamped with a note (`jobs` in `cli/src/argument_parsing.rs`).

- Step 20 gave a search a bound of one gibibyte by default
  (`Options::DEFAULT_MEMORY_LIMIT`, `--memory-limit`), the sequent a
  bound of fifty million occurrences, and the check and the derivation
  the same memory bound. A search whose memo cycles within the bound
  does not answer "unknown" by it: it goes on until a time limit ends
  it, so the default time limit is what makes such a search answer at
  all (`TokenRing-40-unfolded_100_1` ran for 150 s within 144 MiB). A
  memo starved by a small bound makes a search slow rather than
  "unknown". Three new ways to end need their words:
  `Reason::MemoryLimit`, `Reason::IndexLimit`, and `Error::Unchecked`
  (the proof was found and its check was refused for memory), whose
  message today names the limit in raw bytes (`1073741824 bytes`). Of
  the default bias's two searches the reason reported is the backward
  one's, also where the forward one ended at its memory bound. On a
  pool the kept arena is not collected, so a pool reaches the bound
  sooner than one thread. The harness's rows run under one gibibyte
  unless `--memory-limit 0` is passed.

The author's decision (2026-10-03), which binds this step and every
later one (D16): sensible defaults, and the ability to tune every one of
them. Steps 19 and 20 made the time limit and the memory bound hold;
this step sets what a call without flags does.

## Goal

A user who types a sequent and nothing else gets, within a default
budget of time and memory, the best answer the suite can give in that
budget, and when the answer is "unknown" or "unprovable", a sentence that
says what was tried and what to try. Every default is an option with a
name, a flag and a line of documentation.

## What to build

1. **The copy bound deepens while the budget lasts.** By default the
   search goes on to the next bound until it decides, the time budget
   ends or the memory bound binds, and reports the bound it reached.
   `--copies N` remains a cap for whoever wants one, with today's
   meaning. Work out what this does to the two searches of the default
   bias and to the forward bound (`Options::forward_copies`), and keep
   the contract of `.claude/rules/core.md`: with no limit firing the
   default decides whatever either explicit search decides.
   `Unprovable` keeps its meaning exactly.
2. **A default time limit** for the command, lifted or changed by
   `--timeout`: decide it from the rows, by what a longer budget still
   decides against what every "unknown" then waits (step 17 proposed
   10 s; the count above speaks for about 2 s; the report shows the
   table for the values you compared). While it deepens past the first
   bound on a terminal, the command may say so on standard error, so
   that a wait is never silent. The library keeps no clock (D11): the
   budget is the caller's stop closure, as before.
3. **One thread first.** By default the command tries the sequential
   engines for a short budget (a tenth of a second proposed) and takes
   the pool only if that has not answered; `--jobs N` and
   `--deterministic` keep their meaning. Decide whether the second
   attempt may reuse anything of the first, and measure both.
4. **What "unknown" says**: which bound was reached, how long was
   searched, and which flag to try, for each `Reason`.
5. **Why a sequent is unprovable**, where the engine knows more than
   "the search was exhaustive": the atom whose counts cannot balance,
   the count equation that fails, each as a value of the library that
   the command prints and the JSON carries. Nothing is guessed: where
   the refutation is the exhausted search, it says so.
6. **The harness and the tests name their bounds**, so that the
   baselines stay comparable and no test depends on the machine's speed:
   `bench/baseline.sh`'s passes run under the explicit flags they ran
   under, and a pass under the new default is added for step 31.
7. **Documentation**: README's usage section (the default's behaviour,
   every flag that tunes it), the help texts, the rules files.

## Constraints

- Every explicit flag means what it meant; only the call without flags
  changes.
- An answer near the limit depends on the machine. Nothing pinned (a
  test, a snapshot, a README example) may rest on that: pinned calls
  name `--copies`, `--timeout` and `--deterministic` as needed.
- The search on one thread under explicit flags does not change:
  `bench/targets.sh` keeps its counters.

## Measurement

This step names two runs by day, each detached in a capped unit on two
pinned performance cores, about an hour and a half together: the 1 003
problems of `bench/results/2026-10-02/lltp-copies-10.csv` and the 1 342
problems the LLTP result files cover, under the new default at 5 s and
at the default limit. The default must decide what `--copies 10` and
the forward pass at 30 copies decide within the same time, and lose
none of the rows today's default decides. Anything else is asked for
first.

## Verification

The checks of CLAUDE.md's table, both `cargo hack` runs,
`bench/targets.sh`, `linlog-bench run --all-families --timeout 5`, `nix
flake check`, and every README example run against the binary.

## Deliverables

- Thematic jj commits.
- `plan/reports/21-defaults.md`: each default, its reason, its option
  and how each front end sets it (D15, D16); the two runs' tables;
  decisions, deviations, open questions.

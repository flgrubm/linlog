# Step 20 report: the search keeps its memory, and the inputs their bounds

A search now holds at most `Options::memory_limit` bytes (one gibibyte
by default, `--memory-limit` in the command): its memo is emptied when it
no longer fits, the proofs only the memo referred to are collected, and
when that is not enough the answer is `Unknown(Reason::MemoryLimit)`.
No input ends the process by an abort: the parser, every formula
printer, sequentialization and the session's translation into terms no
longer recurse on the input, a sequent that unfolds beyond
`--occurrence-limit` is refused before it is unfolded, and the small
aborts of the assessment are errors. The search itself did not change:
the counters of the rows compared so far are identical, and it is
faster, not slower (`mix` at 9 pairs 14.4 s before and 13.0 s after on
one pinned core; `qbf/48#0` a third more stable sequents a second).

This report was written before the step was finished, at the author's
request; "Where this session stopped" at its end says what is committed,
what was running and what was not started. Item 7 (the checker and the
derivation within the bound) is a sub-agent's work that was not yet
merged when this was written.

## Outcome, item by item

1. **The memory bound** (`core/src/search/memory.rs`, `Options::memory_limit`,
   `Reason::MemoryLimit(bytes)`). An `Account`, one per search, counts
   the bytes that everything which grows has allocated, by capacity. The
   memo's entries are records in chunks with an index of the memo's own
   (`core/src/search/focus/memo.rs`), so the bound counts them, an entry
   costs no allocation, emptying a table resets a count, and dropping a
   full one frees some hundreds of blocks. A stop on `qbf/40#1` with its
   memo full comes 14, 16 and 21 ms after the limit on the machine's
   three kinds of core (76, 126 and 203 ms before, single runs).
   `Options::memo_limit` stays, as the finer knob (see Decisions).
2. **The arena** (`Arena::collect` in `core/src/search/focus/mod.rs`) gives
   back what an emptied memo no longer refers to, in the sequential
   engine; reaching its index limit is `Unknown(Reason::IndexLimit)`.
   `qbf/48#0` ran for 120 s on one core between 360 and 402 MB at every
   ten-second sample, where it grew by 15 MB a second.
3. **`Counts::new`** (`core/src/search/focus/counts.rs`) writes its rows in
   place, charges them to the account as they grow, polls inside the
   merges, and refuses past 2³² entries. The rows are still quadratic in
   the worst case, because a row is: the item's "bounded and polled".
   The quadratic *time* of the assessment's R11 turned out to be here
   too, for nested exponentials (found by the parser's sub-agent): the
   first pass walked the subtree of every `!` and `?`; it is one pass now.
   A `Tally` and a `Split` are as wide as the atoms that have rows, which
   on a Petri net is none: the Philosophers-10000 nets no longer run out
   of memory under a recursion limit of 16 384.
4. **The forest's bound** (`Forest::DEFAULT_LIMIT`, fifty million;
   `Forest::within`, `Forest::from_owned`, `Sequent::occurrences`,
   `Options::occurrence_limit`, `--occurrence-limit`). The assessment's
   D5 (427 bytes of JSON, 67 108 863 occurrences) is refused in 0.01 s
   within 5 MB by `prove`, `seq print` and `seq fragment`, exit status 2.
5. **Depth.** The text parser is a hand-written precedence parser with
   one explicit stack that writes straight into the arena; chumsky is
   removed. Every formula printer walks with a stack of its own
   (`sequents::fmt::Walk`), sequentialization and the session's
   `proof()` likewise. A formula nested 100 000 deep is parsed, printed
   and "proved" (refuted in 0.03 s) by the command; tests at 100 000 run
   on stacks of 256 KiB.
6. **The small aborts**: the caret past column 65 535 (a window of the
   line with the character's number), the set operations on different
   widths, `ProofStructure::link` validated at the public boundary.
7. **The checker and the derivation within the bound**: in progress, see
   the end of this report.
8. **Documentation**: `.claude/rules/core.md` ("The memory bound": what
   counts and what does not; the arena's collection and what it relies
   on; the memo's layout; the counts), `cli.md`, `bench.md` (the
   harness's axis, how heap profiles are taken), `CLAUDE.md`,
   `README.md`, the help texts.

## The bound's design, and what it counts

- **One account per search**, shared by the workers of a pool
  (an atomic count), with half the bound for each of the two searches of
  the default bias, so that neither search's memory decides what the
  other keeps and each remains the search it would be alone.
- **Counted**: the focused memo, the kept arena and every pending stack,
  the branch stack of keys, the pool buffers of the recursion (charged
  when made; the growable lists when given back), the counts and the
  classes of the set-up, the additive path's memo and arena. **Not
  counted**: the forest and the sequent (the occurrence limit bounds
  them), the proof returned, a tally's `touched` lists, a context's
  extra list, the table a collection uses while it runs, thread stacks,
  the net engine (linear in the forest), the allocator's overhead.
- **The order of answers**: a memo with no room for a new key is emptied
  and the arena collected; a search over its bound empties the memo,
  collects, and then gives the memo's memory back; `MemoryLimit` when
  what cannot be emptied is still over the bound, or an empty memo
  cannot have its first chunk. The memo leaves an eighth of the bound
  free, and a relief does not squeeze the arena to fit: both were
  learnt from R8, where a first version spent every node copying its
  arena (170 stable sequents a second instead of 400 000).
- **The collection** renames the kept ids in the pending nodes, in the
  ids rules *hold* across the search of a second premise (`with`,
  `premises`, `parts`), and in the entry being recorded. A missed id
  gives a wrong proof, which the checker rejects, never a verdict.
  `proofs_survive_collections` runs generated sequents under memos of
  one, two and five entries and checks every proof (over a thousand).

## The options, and how each front end sets them (D15, D16)

| option | type and default | the command | the web front end, another wrapper |
|---|---|---|---|
| the memory a search may hold | `search::Options::memory_limit(Option<u64>)`, `DEFAULT_MEMORY_LIMIT` = 1 GiB, `None` for no bound | `--memory-limit SIZE\|none` on `prove` and `interact` | a field of the search options once they have serde (step 23); a browser tab has about 2 GiB, so the default suits it |
| the occurrences a sequent may unfold to | `search::Options::occurrence_limit(u64)`, `DEFAULT_OCCURRENCE_LIMIT` = `Forest::DEFAULT_LIMIT` = 50 000 000; `Forest::within(&sequent, limit)` | `--occurrence-limit N\|none` on every command that reads a sequent | `Sequent::occurrences()` before it builds anything, then `Forest::within` |
| the entries of the memo | `Options::memo_limit(usize)`, unchanged | `--memo-limit N` | as before |
| the harness's bound | `linlog-bench run --memory-limit BYTES` (0: none), column `memory_limit` | – | – |

Named constants that are not options: the memo's chunk (a mebibyte, or
a sixteenth of the room under a small bound), the eighth the memo
leaves free, the hysteresis of the arena's shrinking. They define how
the bound is kept, not what a user chooses.

**Quantifiers (D17).** The memo's record holds a key as two bitsets of
the forest's width plus the extra copies; with terms a key becomes
occurrences under substitutions, so the record's variable part (today
the extras) is where instances go, and the chunk layout takes records
of any stride. The account is independent of the logic.

## Measurements

Heap profiles (heaptrack from the flake's nixpkgs, release build with
debug symbols, core 5, capped at 8 GiB): R8 is `SYJ202+1.008` in cbv for
5 s, `qbf/48#0` for 20 s, the net `TokenRing-40-unfolded_100_1` for 20 s.
A run under heaptrack is slower by its allocations, so the "after" runs
got further: compare bytes per entry and allocation counts.

| | before | after |
|---|---|---|
| R8: peak heap | 278.8 MB at 408 461 entries | 648.9 MB at 1 048 576 entries |
| R8: the memo | keys 202.6 MB (72.7 %) in two allocations each, table 34.1 MB (12.2 %): 580 bytes an entry | chunks 553.7 MB (85.3 %) in 1 051 allocations: 528 bytes an entry, 8 more for the index |
| R8: the arena | kept 33.5 MB (12.0 %), pending 8.4 MB | kept 67.1 MB, pending 16.8 MB, the collection's table 11.2 MB |
| R8: allocations | 3 021 294 | 2 368 503, of which 2 363 750 the branch stack's copies, fixed after this profile (not profiled again) |
| `qbf/48#0`: peak heap | 525.7 MB | 412.5 MB |
| `qbf/48#0`: the memo | keys 278.9 MB (53.1 %), table 204.5 MB at a doubling (38.9 %) | chunks 352.3 MB (85.4 %): 336 bytes an entry |
| `qbf/48#0`: the arena | kept 41.9 MB (8.0 %) and growing without end | kept 41.9 MB, collected at every emptying |
| `qbf/48#0`: allocations | 18 277 410 (10.2 million copies of zones in the asynchronous phase) | 5 315 |
| the net: peak heap | 130.4 MB | 146.4 MB |
| the net: allocations | 5 347 597 | 733 356 (525 288 in `Forest::build`, 196 951 in `lltp::read`) |

What the profile showed that nobody had asked: two derived `clone_from`
allocated at every copy of a zone and of a branch-stack key, behind the
rules' "no allocation per node once warm".

Times, each pair on the same pinned core with the binaries of before
and after (single runs unless said):

| what | before | after |
|---|---|---|
| `mix` at 9, refuted (core 2), CPU | 14.36 s, 36 MB | 13.04 s, 12 MB; same 129 009 092 stable sequents, hits, entries and splits |
| `chain` at 256, `qbf/20#2`, `growing` at 1 024 (core 2), CPU | 350, 40, 280 ms | 310, 30, 290 ms; same counters |
| `qbf/48#0`, 20 s (core 4) | 9.08 million stable sequents, peak 752 MB | 12.08 million, peak 393 MB |
| R8, 5 s, default options (core 4) | 4.19 million stable sequents, peak 883 MB, stop 0.20 s late | 7.82 million, peak 943 MB, stop 0.07 s late |
| a stop on `qbf/40#1`, memo full, cores 2, 6, 13 | 76, 126, 203 ms late | 14, 16, 21 ms late |

The bound, in scopes capped at twice the bound plus 8 MiB, core 4:

| R8 under | ends | peak of the process |
|---|---|---|
| 2 GiB | at the time limit of 5 s | 1 003 MiB |
| 256 MiB | "unknown" by the bound after 2.49 s | 253 MiB |
| 16 MiB | by the bound after 0.20 s | 19.8 MiB |
| 4 MiB | by the bound after 0.06 s | 8.1 MiB |
| 1 MiB | by the bound after 0.02 s | 5.4 MiB |

The process's peak is the count plus what holds the input and the
program, about 4.5 MB here. `qbf/48#0` for 120 s on core 4: between
360 and 402 MB at each of eleven samples ten seconds apart, 73 million
stable sequents. The five Philosophers-10000 nets under a recursion
limit of 16 384 and 5 s, in scopes of 2 GiB: `_1_1` proved and checked
in 0.15 s within 75 MB, the other four "unknown" at the time limit at
a peak of 982 MiB; the second baseline has a crash in all five rows.
R11: `seq fragment`, `prove` and `seq print` on a formula nested
100 000 deep take 0.01, 0.03 and 0.02 s (`prove` answers "unprovable");
the parser's sub-agent measured the old binary at 0.05 s for depth
30 000, so the assessment's 2.4 s did not reproduce, and found the
quadratic pass in the counts instead (`!` nested 80 000 deep: 2.2 s of
`prove_goal` before the first poll).

The parser against chumsky, by its sub-agent: 1.1 million generated
inputs, valid and invalid, and the 4 556 LLTP files of at most 1 MB:
the same sequent or the same error position on every one.

## Decisions

- **The memo's limit in entries stays**, as the finer knob beside the
  bound in bytes: the pinned counters of every memo-bound run depend on
  where a full table is emptied, a table that fits the cache can beat
  one that fits the memory (the additive path's measurement), and the
  tests empty tables of a few entries on purpose.
- **A record layout and an index of the memo's own, no new
  dependency.** The table is never iterated and never deletes, so
  linear probing over record numbers is thirty lines; `hashbrown`'s
  raw table would have been a direct dependency for the same thing.
- **The arena is collected, not left to grow**: the profile showed it at
  a tenth of the memo and growing without end on `qbf/48#0`, which is
  what "15 MB a second" was. Not on a pool, where workers hold ids
  nobody can rename; there it counts toward the bound.
- **Half the bound per search of the default bias**, for the identity
  the rules promise of each search.
- **One gibibyte by default**: no decided row of the target set held
  more than 100 MB of memo in its last run, and a laptop and a browser
  tab both have it.
- **`Reason::IndexLimit`** beside `MemoryLimit`: a search without a
  bound that outgrows a `u32` index says that, not a limit the user
  lifted.
- **Fifty million occurrences**: above the library's largest file
  (27.8 million), below the 67 million of the assessment's file.
- **A hand-written parser instead of a depth limit**: a limit that is
  safe on every caller's stack would have been a few hundred levels.
- **heaptrack is not in the devshell**: its closure is a gigabyte for a
  viewer; `.claude/rules/bench.md` has the recipe.

## Deviations and assumptions

- heaptrack opened its viewer on the author's desktop at the end of the
  first two recordings, because it does so unless told `--record-only`;
  the windows were closed and the flag is in the rules.
- A sub-agent ran a scratch test outside its capped scope for about ten
  minutes on its two cores (a generator that doubled a formula at
  every round); it stopped it and reported it. Nothing else was
  affected.
- The author committed to the README in the shared working copy
  meanwhile ("Say that linlog is inspired by Click & coLLecT"); it sits
  between this step's commits.
- "Unknown by the bound" is reached when what cannot be emptied passes
  the bound, not when the memo first fills it: a search whose memo
  cycles within the bound goes on until its time limit. R8 gets there
  because its branch holds millions of proof nodes.
- A proof file and a session's state are read under the default
  occurrence limit whatever the flag says: serde's `Deserialize` takes
  no options.

## Open questions and follow-ups

- **The zones of a key are bitsets of the forest's width**, which is
  nearly all of an entry on a forest of thousands of occurrences; a
  sparse form would multiply what a bound holds.
- **A memo starved by a small bound makes a search slow rather than
  "unknown"**; whether "unknown" should come earlier is step 21's
  question of what "unknown" tells the user.
- **The pool's arena is not collected.**
- `Forest::build` allocates a vector per occurrence; `Forest::lca` is a
  parent walk; `OccSet::insert` with an id beyond the width panics (an
  API misuse, not an input).
- The merged reason of the default bias's two searches is the backward
  search's, so a forward search that ended at its memory bound is not
  what the user reads.
- `plan/` still names chumsky in its history.

## The commits

"Copy an occurrence set into its own words"; "Bound a search's memory
in bytes"; "Refuse a forest beyond a limit on occurrences"; "Parse
sequents without recursion"; "Remove chumsky"; "Point at a parse error
in an input of any length"; "Print formulas without recursion";
"Sequentialize a net without recursion"; "Define the set operations on
sets of different widths"; "Validate a link at the public boundary";
"Give the command its limits on memory and occurrences"; "Make the
memory bound an axis of the harness"; "Document the memory bound and
the limits on the input"; "Translate a session's derivation into its
term without recursion".

## Where this session stopped

**Done and committed** (the fourteen changes above): items 1 to 6 and
the documentation of item 8 for them; the harness's axis. Checked on
the merged tree: `cargo clippy --workspace --all-targets -- --deny
warnings` clean; `cargo test --workspace` green; `cargo hack check
--each-feature` (11 of 11) and `--feature-powerset --depth 2` without
an error, both before the last commit ("Translate a session's
derivation…"), which touched no feature gate.

**In progress when this was written:**

- **Item 7**, by a sub-agent in the jj workspace `step20-checker`
  (`target/ws/checker`), not merged: five changes there, "Build a
  derivation on a stack of its own", "Keep every integer of the checker
  and of the size in range", "Bound the memory a check of a proof
  holds", "Document the checker's bound, its integers and the
  derivation builder", "Count what the derivation's record keeps
  against its pass". Its fresh-context reviewer had not reported. Its
  interface, as briefed: `linlog::DEFAULT_MEMORY_LIMIT`,
  `Proof::check_within(mode, Option<u64>)`, a refusal distinguishable
  from an invalid proof, `ViewOptions::memory`, a `ViewError` that says
  which bound it was.
- **R9** (`TokenRing-40-unfolded_100_1` under 256 MiB and 64 MiB, in
  scopes of twice that, guard 150 s) was running; no result yet.
- A last heap profile of R8 on the final layout failed to start (a
  mistake in the command line); the allocation count after the branch
  stack's fix is therefore not measured.

**Not started:**

- Rebasing the checker's changes onto this chain; calling
  `check_within` with `Options::memory_limit` at the end of
  `prove_goal`; `--memory-limit` on `check`; the derivation under the
  memory bound in the command (not built when its estimate passes the
  bound even with `--derivation-limit none`, with a line that names the
  bound); the megabyte proof file through the command;
  `TokenRing-50-unfolded_1_1` with `--derivation-limit none`.
- `bench/targets.sh` (the full comparison of counters, and the two
  percent on the rows over a second: so far four rows by hand).
- `cargo deny check` (a dependency changed: chumsky out, unicode-ident
  direct; the sub-agent ran `licenses bans sources`), both `cargo hack`
  runs and `nix flake check` on the final tree.
- The plan's Status entry, `plan/later.md` (the follow-up "The search's
  own memory" is done; the new ones above), and "what step 21 must
  know", which in short is: the default bound is `DEFAULT_MEMORY_LIMIT`
  and `args.memory_limit`; a default time limit is what ends a search
  whose memo cycles within the bound; `Reason::MemoryLimit` and
  `IndexLimit` need their words in whatever "unknown" will tell a user;
  and the harness's rows now run under one gibibyte unless
  `--memory-limit 0` is passed.

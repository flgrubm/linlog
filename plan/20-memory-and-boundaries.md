# Step 20: the search keeps its memory, and the inputs their bounds

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: section 1.2 (D5, D7, D10), section 2.6
  (R8, R9, R11), and the author's answers.
- `plan/reports/18-bounded-proofs.md` and `19-time-limits.md`.
- `plan/README.md`: D9, D15, D16, and the Status entries of steps 15 and
  17.
- `plan/later.md`: "Follow-ups: the focused engine" ("The search's own
  memory").
- `.claude/rules/core.md`: "Memo contract with the bound", "The proof
  arena has two parts", "Counts", "The additive fast path", "Parsing",
  "Sequents are arena-allocated DAGs", "The occurrence forest".
- `core/src/search/focus/memo.rs`, `focus/mod.rs` (`Arena`, the pools of
  scratch buffers), `focus/counts.rs` (`Counts::new`, `Split`),
  `search/additive.rs`, `search/mod.rs` (`Options`, `Reason`),
  `core/src/parse/mod.rs`, `core/src/occurrences/mod.rs` and `set.rs`,
  `core/src/nets/mod.rs` (`link`), `cli/src/lib.rs` (the caret).

## What step 17 left you

Nothing bounds the memory of a search. The memo's cap
(`Options::DEFAULT_MEMO_LIMIT`, 2²⁰) counts entries: on a forest of
65 000 occurrences an entry is two sets of 8 KB, so the cap is 16 GiB,
and the default with exponentials runs two searches. A small ILLTP
problem fills the cap in five seconds at 918 MB; `qbf/48#0` grows by
15 MB a second until a doubling of its table fails. The kept arena never
shrinks after the memo is cleared and panics at 2³¹ nodes. `Counts::new`
builds rows quadratic in a tensor of distinct atoms before any poll, and
a `Split` of six per-atom arrays is taken per level of recursion (four
Philosophers nets under a raised recursion limit run out there).

At the inputs: a JSON sequent of 433 bytes whose subterms are shared 24
times over unfolds to a forest that takes 1.9 GiB (the forest refuses
only at 2³² occurrences); a formula nested 100 000 deep aborts the
process with a stack overflow on the main thread, and one nested 30 000
deep takes seconds before any search (something is quadratic in the
depth); a parse error past column 65 535 panics in the caret's
formatting; `OccSet::is_subset` and `is_disjoint` truncate silently on
sets of different widths, and the public `ProofStructure::link` only
debug-asserts its arguments.

## Goal

A search that would exhaust memory answers "unknown" with a reason
instead, within a bound the caller sets and a default that suits a
laptop; and no input, however large, deep or malformed, ends the process
other than by an error with exit status 2.

## What to build

1. **A memory bound in bytes** for a search (D15, D16): one option with a
   sensible default and a flag, covering the memo (both searches of the
   default bias together), the kept arena and the scratch that grows
   with the recursion; `Reason` gains the variant that names it, with its
   line in the command, the JSON and the harness. The memo's limit in
   entries stays as the finer knob or goes; say which and why. Emptying
   the memo when it is full remains the first answer, as today, and
   "unknown" the last.
2. **The arena** gives back what a cleared memo no longer refers to, or
   the report shows why that is not worth its cost; reaching its index
   limit is `Unknown`, not a panic.
3. **`Counts::new`** in memory linear in the forest, or bounded and
   polled.
4. **A bound on the forest**: a sequent whose unfolding passes a limit on
   occurrences (an option, with a default far above the library's
   largest problem, 15 million) is refused before it is built, from JSON
   and from text alike.
5. **Depth**: parsing, lowering, printing and every other walk over a
   formula either does not recurse or refuses a depth it cannot handle
   with an error; the quadratic step is found and removed. A test at
   100 000.
6. **The small aborts**: the caret past column 65 535; set operations on
   different widths; `ProofStructure::link` validated at the public
   boundary.
7. **Documentation**: the rules file (what counts toward the bound, what
   does not), README's limits, the help texts.

## Constraints

- The search does not change: every decided row of `bench/targets.sh`
  keeps its counters, and the bound's bookkeeping costs under two
  percent of pinned CPU time on the rows over a second (measure it).
- `Reason` and `Options` are `#[non_exhaustive]`; the JSON of an
  `Outcome` gains a reason and loses nothing.
- The defaults a user meets without flags (the time limit, the deepening,
  the thread count) are step 21's; this step gives the memory bound its
  default and no more.

## Verification

The checks of CLAUDE.md's table, both `cargo hack` runs,
`bench/targets.sh` once, `nix flake check`. Then, in scopes capped at
twice the bound under test: R8 and R9 of the assessment ending "unknown"
by the bound with the memory under it; `qbf/48#0` for 120 s on one
pinned core, its memory flat; the four Philosophers-10000 nets of
`lltp-recursion.csv` under `--recursion-limit 16384`; D5's file refused
in milliseconds; R11 an error.

## Deliverables

- Thematic jj commits.
- `plan/reports/20-memory-and-boundaries.md`: the bound's design and
  what it counts, the options and how each front end sets them (D15),
  measurements, decisions, deviations, open questions, what step 21 must
  know.

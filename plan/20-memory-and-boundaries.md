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

## What step 18 and its review left you

The checker's zones are counters now, and the review found them wrapping
in a release build: a proof file of 131 nodes was a "valid proof" of
`⊢ !⊥, 1` (`plan/reports/18-bounded-proofs.md`, "Corrections from the
review"). The review's fix refuses a zone that the rest of the proof
cannot consume (`Pass::within`, `Problem::Surplus`), which bounds every
zone by the goal plus twice the nodes and makes the counters exact. What
is left there:

- **The clone per reader.** A node read by several others is cloned for
  all but the last, and the clones live until their own readers come, so
  a proof file of about a megabyte (a zone of thousands of distinct
  members, read by thousands of nodes that are all derived before any
  is consumed) can make the pass hold nodes × zone, gigabytes.
- **The other integers** of the pass and of the size estimate
  (`readers`, `outputs`, the weights an observer adds up, `Sub`'s
  fields) were not argued one by one.
- **The lifted limit.** `--derivation-limit none` on a proof whose
  derivation is larger than the machine's memory ends with the kernel's
  kill and no verdict written, though the verdict was known before
  anything was built.
- **The builder's recursion** is as deep as the derivation is high; the
  command runs it on the search's stack. A derivation 8 000 high was
  built; nothing says where it stops.

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
7. **The checker and the derivation within the bound.** `linlog check`
   on any proof file, and the check inside a search, stay within the
   memory bound or end with an error that names it: the states the pass
   holds are counted, or made persistent so that a reader's copy costs
   nothing; a test with a file of about a megabyte that takes gigabytes
   today. Every integer of the pass and of the size estimate either
   cannot reach its limit, with the reason written where it is declared,
   or saturates into a refusal; no arithmetic of the checker may wrap in
   any build. The memory bound also holds for what is built after the
   search: a derivation whose estimate passes it is not built even with
   `--derivation-limit none`, the verdict stands and the line says which
   bound it was. The derivation builder gets an explicit stack, or
   refuses a height its stack cannot take, by `Size::height`.
8. **Documentation**: the rules file (what counts toward the bound, what
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
by the bound with the memory under it; the megabyte proof file of item 7
refused or checked within the bound, and `TokenRing-50-unfolded_1_1`
with `--derivation-limit none` ending with its verdict; `qbf/48#0` for 120 s on one
pinned core, its memory flat; the four Philosophers-10000 nets of
`lltp-recursion.csv` under `--recursion-limit 16384`; D5's file refused
in milliseconds; R11 an error.

## Deliverables

- Thematic jj commits.
- `plan/reports/20-memory-and-boundaries.md`: the bound's design and
  what it counts, the options and how each front end sets them (D15),
  measurements, decisions, deviations, open questions, what step 21 must
  know.

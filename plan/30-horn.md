# Step 30: Horn programs: an engine, coverability, and problems from practice

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 29. Read before
you start:

- `plan/reports/17-assessment.md`: 2.1, 2.5, 3.7, 3.15, 5.4.
- `plan/later.md`: "The !-Horn fragment through Petri-net reachability",
  "Problems from practice".
- `plan/reports/15-performance.md` ("The default bias"), `16-baseline.md`,
  `29-focused-engine.md`.
- `plan/README.md`: D8, D19.

## Goal

A sequent that is a Horn program (clauses under `!`, a marking, a goal:
a Petri net) is decided by an engine made for it, chosen by the dispatch
from the sequent's shape (D19): reachability by explicit-state search
with the proof built from the firing sequence, and in affine mode
coverability by the backward algorithm, which decides. The library's
1 594 nets that end at the time limit and the coverability suites of
software verification are what it is measured on.

## What is fixed now

1. **First session, reachability**: markings as count vectors,
   transitions indexed by their input places, a visited set, a proof
   term the checker accepts. It is the default for Horn programs only
   where it beats the forward focused search on the library's nets; if
   it does not, the step ends with its first session and says why.
2. **Second session, coverability**: the backward algorithm on upward-
   closed sets for affine mode, with its termination argument and a
   fresh-context review; `Unprovable` from it is the suite's first
   refutation of an affine sequent with exponentials.
3. **The suite**: the 176 coverability instances of `blondimi/qcover`
   (Mist's `.spec` format; five suites; expected results in the files),
   fetched by the flake at a pinned commit as LLTP is, not committed,
   since the files carry no licence statement of their own; a reader for
   the format; parametric initial markings as `!p`.
4. **What it retires**: the focused follow-ups that only served nets
   (the unit of work, the restart from the frontier), if the engine
   takes the nets.

## Measurement

Named when the prompt is finished; by day on pinned cores, the library's
nets at 5 s against `lltp-forward.csv`.

## Deliverables

Thematic jj commits; `plan/reports/30-horn.md`.

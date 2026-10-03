# Step 29: the focused engine in order

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. This prompt is finished
at the reviews of steps 20 and 23. Read before you start:

- `plan/reports/17-assessment.md`: section 1.2 (the refactoring's items
  1 to 4, 9, 13 and 14, D8), 1.3, 1.4, and the author's answers 4, 5
  and 7.
- `plan/reports/15-performance.md` ("Constant factors", "The reviews"),
  `19`, `20`, `23`; `plan/notes/api.md`.
- `plan/later.md`: "Follow-ups: the focused engine".
- `plan/README.md`: D7, D8, D17, D18, D19.
- The search's rules file and `core/src/search/**` in full.

## Goal

The focused engine as one reader can hold it, with nothing that a
refactoring can break in silence; and a dispatch that is a table: for
each fragment, mode and feature of a sequent the engine that a
measurement shows fastest (D19), with one interface every engine
implements, so that steps 30, 34 and 36 add a row and not a special
case.

## What is fixed now

1. **The cut and dependency flags become values** a step of the search
   returns; with them each rule is written once for one thread and for
   the pool, and the worker is built in one place.
2. **The file along its seams**: the scheduling of the two searches
   (out of the parallel module), the arena, the split search, the
   scratch pools, the tests.
3. **One engine interface** and one place that builds a `Verdict`;
   options an engine does not honour are refused or documented, not
   ignored; `prove_goal` takes its goal in any order, as it says.
4. **The dispatch as data**, with the feature that picks each row named
   and the measurement behind it cited; the atom bias out of the forest.
5. **The hot spots, from a profile taken anew.** Step 15's profile
   named three (the memo key hashed once, an insert without an
   allocation, the canonical key only where a member is renamed), but
   steps 19 and 20 changed what it measured: the ranking of the copies
   is linear now, and the memo's keys were laid out again. So the step
   begins with a sampling profile (perf from the flake's nixpkgs, a
   release build with debug symbols set through the environment, pinned
   to cores of one speed and capped, as step 15 took it; read as text,
   by function and by call path: no flame graph or other drawing is
   wanted, the author does not need one) of every row of the target set
   that takes over a second, and works from its ranking; it ends with the same profile, and the report shows the
   two side by side. For a change worth a few percent, times by day are
   too noisy on this machine (one-thread rows whose code did not change
   moved by −6 to +4 % between two runs of step 19): compare instruction
   counts instead (valgrind's callgrind, through the `new-tool` skill)
   on a row small enough to run under it, about fifty times slower.
6. **From the follow-ups**: the Mix prune (`n·2^n` for `3^n`), a loop
   for chains of free splits, the Horn test on the goal's members. And
   from step 19's review of the pool, which item 1 is the place for: a
   premise's failure at a `&` cancels the other only once its worker has
   left its nested scopes, so a stolen task of the sibling runs on
   uncancelled (seen once as an answer that came only with the caller's
   stop); an error of one premise cancels the other, so a pool answers
   "recursion limit" where the other premise would have failed and one
   thread answers "unprovable"; and the differential run of the pool
   against one thread is repeated with recursion limits of 4 to 16,
   since limits of 24 to 63 were reached by 10 of 369 057 random
   sequents.
7. **Ready for quantifiers** (D17) as `plan/notes/api.md` says: where a
   trail of bindings would go, which prunes assume ground atoms.

## The oracle

Every commit that claims no change of the search leaves `nodes`,
`splits`, `memo_hits` and `memo_entries` of every decided row of
`bench/targets.sh` identical; a commit that changes the search says so,
is measured, and is reviewed differentially by a fresh-context reviewer
as step 15's were. Pinned CPU time does not rise.

The counters say that a search is the same search; they say nothing of
a verdict that was wrong before and after. So this step also leaves a
**reference prover in the repository** (the author, 2026-10-03, on the
planning session's recommendation), before it changes anything: a
test-only module, as `core/src/proofs/oracle.rs` is for the checker,
that decides a small sequent by the plain rules of the unfocused
calculus, exhaustively, with no polarity, no bias, no count prune and
no memo keyed as the engines key theirs, classical and two-sided,
linear and affine, with and without Mix, and with a copy bound of its
own for the exponentials. It shares no code with any engine. A
committed test compares it with every engine on the sequents
`search/generate.rs` makes, provable ones and mutants, at sizes it
finishes in the time the neighbouring tests take, and asserts what the
contract allows: never "proved" against "unprovable", in either
direction, and no "unprovable" from an engine where the reference finds
a proof within its bound. Until now such a reference was written anew
by a step's reviewer and thrown away (the unfocused two-sided prover
that step 8's engine was compared with on 60 000 sequents is the one
the rules file names), so none of those runs can be repeated on the
engine as it is today; the checker guards a
wrong "proved" in every build, and only the engines themselves guard a
wrong "unprovable". Steps 30, 34 and 36 add their engines to this test.
A fresh-context reviewer still writes a reference of their own for a
change of the search: the committed one is for repeating, theirs for
independence.

## What waits

Whether the engine's recursion becomes an explicit stack: only if step
27 has shown the web front end needs a search that can be suspended.

## Deliverables

Thematic jj commits, each building and passing alone;
`plan/reports/29-focused-engine.md`.

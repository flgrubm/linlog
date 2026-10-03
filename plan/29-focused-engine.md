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
5. **The three hot spots** (the memo key hashed once, an insert without
   an allocation, the canonical key only where a member is renamed).
6. **From the follow-ups**: the Mix prune (`n·2^n` for `3^n`), a loop
   for chains of free splits, the Horn test on the goal's members.
7. **Ready for quantifiers** (D17) as `plan/notes/api.md` says: where a
   trail of bindings would go, which prunes assume ground atoms.

## The oracle

Every commit that claims no change of the search leaves `nodes`,
`splits`, `memo_hits` and `memo_entries` of every decided row of
`bench/targets.sh` identical; a commit that changes the search says so,
is measured, and is reviewed differentially by a fresh-context reviewer
as step 15's were. Pinned CPU time does not rise.

## What waits

Whether the engine's recursion becomes an explicit stack: only if step
27 has shown the web front end needs a search that can be suspended.

## Deliverables

Thematic jj commits, each building and passing alone;
`plan/reports/29-focused-engine.md`.

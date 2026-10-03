# Step 35: cyclic MLL and the Lambek calculus

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 34. It says what
is fixed.

Read `plan/later.md` ("Cyclic MLL and the Lambek calculus"),
`proof-search-specifications.md` ("MLL variants: units, Mix, cyclic MLL
and Lambek"), `plan/reports/17-assessment.md` (3.8), `34-mll-engines.md`,
and `plan/README.md` (D2, D8, D19).

## Goal

A non-commutative mode: cyclic MLL by planar axiom linkings in the net
engine, no exchange in the derivation view, the interactive rules and
the outputs, and the Lambek calculus with its two divisions and without
empty antecedents, read two-sided. The mode is a value of `Mode` (D2),
and the dispatch gives it its engine (D19).

## Fixed now

The order of a sequent's formulas, which `Sequent::optimize` sorts away
today, has to be kept for this mode, a change of the arena's canonical
form that `plan/notes/api.md` should have left room for; say in the
report what it cost. The embedding of the Lambek calculus into
first-order MILL (Moot and Piazza, 2001) is the other route and is
weighed against the planar search once step 37 exists; this step builds
the planar one. Fable 5.1 at `xhigh`.

Deliverable: thematic jj commits; `plan/reports/35-lambek.md`.

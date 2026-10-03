# Step 33: cut, and cut elimination

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes three
sessions; this prompt is finished at the review of step 32. It says what
is fixed.

Read `plan/reports/17-assessment.md` (5.5), `plan/README.md` (D5, D6,
D6a, D13, D17), `plan/notes/api.md`, and the reports of steps 2, 5, 9
and 32.

## Goal

What a course on linear logic shows and the suite cannot yet: a proof
with cuts, and its cut-free form reached step by step. A cut rule in
interactive proofs (the user names the cut formula), proof terms with
cuts and the checker's rule for them, cut elimination on terms one step
at a time and to the end, and on MLL nets (and on the nets of step 32),
where it is the reason nets exist, each step drawn.

## Fixed now

The forest holds the sequent's subformulas only, so a cut formula and
its dual need roots of their own beside the sequent's: the first session
is the data model, the checker and the interactive rule, with the
propositional search untouched (every proof it returns stays cut-free)
and `plan/notes/api.md` followed for where quantifiers will go (D17).
The second is elimination on terms, the third on nets with the drawing.
Termination and the preservation of the conclusion are tested on
generated proofs with cuts, every result through the checker. Fable 5.1
at `xhigh`.

Deliverable: thematic jj commits; `plan/reports/33-cut.md`.

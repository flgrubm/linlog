# Step 37: first-order linear logic

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step is several
sessions with a plan of its own, which its first session writes; this
prompt is finished at the review of step 36. It says what is fixed.

Read `plan/later.md` ("First-order linear logic"),
`proof-search-specifications.md` ("First-order fragments"),
`plan/notes/api.md` (where terms and binders go, decided in step 23),
`plan/reports/17-assessment.md` (3.9, the author's answer 4), the
reports of steps 23, 28 and 29, and `plan/README.md` (D1, D5, D6, D17).

## Goal

Quantifiers and predicates over terms through every layer: the data
model, the parser, the printer and the JSON; unification with a trail in
the engines; witnesses in proofs and the checker's eigenvariable
condition; the views, the interactive rules (a witness given or left
open), the outputs; certificates in the Rocq library's first-order
development.

## Fixed now (the author, 2026-10-03; D17)

The propositional case must not become meaningfully slower: the target
set's counters stay identical and its pinned CPU time within a few
percent at every stage, by generics or by a duplicated fast path if that
is what it takes. The data model comes first and by itself. First-order
MLL stays in NP and first-order MALL is NEXPTIME-complete (Lincoln and
Shankar, LICS 1994; Lincoln and Scedrov, TCS 1994); with exponentials
the copy bound and its three-valued answer carry over. No problem
library exists: the families gain first-order generators, and Moot's
LinearOne is the prover to compare with. Fable 5.1 at `xhigh`
throughout.

Deliverable: `plan/first-order/README.md` (the step's own plan), thematic
jj commits, `plan/reports/37-first-order.md`.

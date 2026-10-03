# Step 34: engines for MLL and IMLL: net pruning, routing, essential nets

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 31, whose
baseline it is measured against. It says what is fixed.

Read `plan/later.md` ("Net-engine pruning and routing for repeated
literals", "Essential nets for IMLL"), `plan/reports/06-net-search.md`,
`08-intuitionistic.md`, `17-assessment.md` (2.4, 3.3, 3.5, the author's
answer 7), `29-focused-engine.md`, and `plan/README.md` (D8, D19).

## Goal

The author's aim from the start (D19): each fragment has the engine made
for it where that engine is the fastest, picked automatically. For
unit-free MLL that is the net engine wherever it wins, and for IMLL an
essential-net engine if it wins over the embedding.

## Fixed now

1. **The net engine's pruning**: the leaf symmetry break for pure `⊗`
   and `⅋` trees of equal literals, a per-atom balance over the
   skeleton's components, the sound symmetry break for equal compound
   conclusions; each with its argument and a differential review against
   brute-force enumeration, as step 6 had.
2. **The routing feature**: "no two equal literals under one pure tree"
   or whatever the harness shows separates the two engines, in the
   dispatch table of step 29, with the measurement beside the row. The
   honest starting point (step 17): the focused engine is within a
   factor of three of the net engine on its own case and ahead from 256
   literals on, so the net engine has to earn every row, and a row it
   does not earn stays the focused engine's.
3. **Essential nets for IMLL**: Lamarche's polarized structures over the
   reading, correctness by directed acyclicity and the dominator
   condition (Murawski and Ong), the incremental search with a
   transitive closure and an undo log (Moot, 2004), compared with the
   embedding on the baseline's IMLL rows before it becomes a default;
   and the drawing of an essential net, which teaching wants whatever
   the measurement says.

Fable 5.1 at `xhigh`. Deliverable: thematic jj commits;
`plan/reports/34-mll-engines.md`.

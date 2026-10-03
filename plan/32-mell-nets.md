# Step 32: MELL proof nets with exponential boxes

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 31, from what the
released API and the web front end look like by then. It says what is
fixed.

Read `plan/later.md` ("MELL proof nets with exponential boxes"),
`plan/reports/05-proof-nets.md`, `11-svg.md`, `17-assessment.md` (3.4),
and `plan/README.md` (D6a, D12, D15).

## Goal

Proof nets for MELL as a representation: `!`-boxes and the `?` nodes
over the occurrence forest, a correctness criterion independent of any
engine (Danos–Regnier at each box depth with boxes contracted; Guerrini
and Masini, TCS 254, 2001, for the parsing view), sequentialization
through boxes, desequentialization of a proof, a JSON form, and the
drawing with boxes as rectangles, in the command and the web front end.
Search stays with the focused engine.

## Fixed now

First session the structure, the criterion and both conversions, with a
fresh-context reviewer comparing the criterion with switching
enumeration on small structures, as step 5 had; second session the
drawing and the front ends. The choice of `?` nodes (dereliction,
contraction and weakening as nodes, or one generalised node) is the
session's, argued from what makes desequentialization canonical and
weakening checkable. Fable 5.1 at `xhigh`.

Deliverable: thematic jj commits; `plan/reports/32-mell-nets.md`.

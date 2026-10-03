# Step 26: ordinary logic through its embeddings

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. This prompt is finished
at the review of step 25. Read before you start:

- `plan/later.md`: "Ordinary logic through its embeddings" (its layer,
  items 1 to 5, is the requirement; its termination work is not this
  step's).
- `plan/reports/17-assessment.md`: 2.2, 3.13, and the author's answer 7.
- `plan/reports/21-defaults.md`, `23-api.md`, `25-batch.md`.
- `plan/README.md`: D15, D16, D19.

## Goal

A user with a formula of classical, intuitionistic or minimal
propositional logic gets it decided by the linear engines and sees how:
the translation by its name, the image as a linear sequent, the linear
proof, and the proof read back with the rules of LK or LJ. For teaching,
the embeddings are worth seeing by themselves.

## What is fixed now

1. A type and a syntax for ordinary formulas and sequents (`->`, `/\`,
   `\/`, `~`, `<->`, true, false), apart from the linear ones, and
   names that do not collide with `--intuitionistic`, which means ILL.
2. The translations as public functions, each by its name: classical
   logic into affine MALL without exponentials; Girard's translation,
   the call-by-value one and Liang and Miller's 0/1 translation into ILL;
   minimal logic with false as an atom. The image is printable from the
   command.
3. Deciding: classical through affine mode, which terminates without a
   copy bound; intuitionistic and minimal through ILL under step 21's
   deepening default, answering "unknown" where it is unknown. Which
   translation is the default is decided by a run of the ILTP
   propositional problems (274 problems, fetched by the flake at a pinned
   version as LLTP is, not committed: no licence is stated), read in
   their own syntax.
4. The proof read back as LK or LJ with their rule names, drawn by the
   outputs that take a derivation, and what checks the read-back.
5. A certificate over `Prop` for the ordinary statement, which needs no
   library.

## What waits for the review of step 25

The batch mode's interface, through which the ILTP run goes; the names
of step 23's API. Termination on dyadic sequents (a loop check, or a
bound proved enough for an image) is engine research: it is assessed in
this step's report from what the ILTP run leaves undecided, and built
only as a step of its own.

## Deliverables

Thematic jj commits; `plan/reports/26-ordinary-logic.md` with the ILTP
table per translation.

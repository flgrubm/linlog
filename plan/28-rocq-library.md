# Step 28: a Rocq library of linlog's own

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes three
to four sessions, each leaving something usable; this prompt is finished
at the reviews of steps 18 and 23. Read before you start:

- `plan/later.md`: "Second certificate kernels: a Rocq library of
  linlog's own, NanoYalla kept for compatibility" (the shape is the
  author's and is the requirement), "Follow-ups: intuitionistic mode".
- `plan/reports/12-certificates.md`, `17-assessment.md` (3.10 with its
  three corrections, the author's answers 4 and 6), `18-bounded-proofs.md`.
- `plan/README.md`: D6, D15, D17, D20; `plan/notes/distribution.md`.
- `core/src/proofs/check.rs` as step 18 left it: the Rocq checker
  verifies this algorithm, once.

## Goal

Every mode has a statement and a certificate: classical, with Mix,
affine, and two-sided intuitionistic, as a lemma over plain inductives
that a reader checks against a textbook, proved by computation from the
proof term. The NanoYalla export stays exactly as it is.

## What is fixed now

1. **Name and place** (the author, 2026-10-03): the library is `linlog`,
   under `rocq/` in this repository, logical path `Linlog`, and the
   flake builds it the way nixpkgs builds a Rocq library, as a package
   and as a check that compiles every certificate of both kernels and
   prints the assumptions of the main theorem (none).
2. **The standard library only**, by the conventions of the day read
   from the reference manual (`From Stdlib`, a `_RocqProject`, an opam
   file named for the archive's convention), on the Rocq nixpkgs ships.
3. **Certificates as data**: the proof term as a Rocq datatype, a checker
   as a function, one soundness theorem per calculus.
4. **Stages**: the definitions and the checker with its soundness for
   classical LL; Mix, affine and the two-sided statement; the bridges to
   NanoYalla's `ll` and to Yalla's standalone `nanoill.v`, and for Mix
   the reduction to `?(⊥⊗⊥), Γ`, which Yalla proves without cut
   (`mix2_to_ll` in `ll_fragments.v`) and can be followed.
5. **Quantifiers are coming** (D17): the formula type and the checker
   are written so that a first-order extension is a second development
   beside this one, and the report says how.
6. **In the exporter** a second kernel behind `rocq::Options`, chosen by
   the mode where the user did not choose.

## What waits

The checker's final shape (step 18) and the proof term's API (step 23).
No `Admitted`, no axiom.

## Deliverables

Thematic jj commits per stage; `plan/reports/28-rocq-library.md`.

# Step 11: proof certificates for Rocq

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D6, D9) and every report in `plan/reports/`,
  especially `02-proofs.md`, `07-exponentials.md` (structural rules in the
  derivation view) and `08-intuitionistic.md` (two-sided derivations).
- `plan/notes/export-targets.md` §§ "Click & coLLecT" (its Coq export) and
  "Yalla". Then read the live sources: `export_as_coq.ml` and
  `nanoyalla/macroll.v` in github.com/ComputerAidedLL/click-and-collect,
  and `yalla/ll_def.v`, `microyalla/`, the ILL development and the release
  notes in github.com/olaure01/yalla. Facts from the notes may be stale;
  what the sources say wins.
- `proof-search-specifications.md` § "Cross-cutting engineering notes" §
  "Certificates first".
- `.claude/rules/ci.md` and `modules/*.nix` if you add a nix check.

## Goal

`export::rocq`: from a derivation, a Rocq script that certifies the sequent
in a Yalla-based kernel, the way Click & coLLecT does with NanoYalla, for
classical derivations (MLL, MALL, MELL, LL, with Mix where the kernel
supports it) and, if Yalla's ILL development makes it reasonable, for
intuitionistic ones. `linlog prove --format rocq`. A way to actually check
the certificates, in nix if feasible.

## What to build

1. **Target choice.** Decide between NanoYalla (Click & coLLecT's kernel:
   small, `apply (<rule>_ext …); cbn_sequent.` per rule, builds with or
   without Yalla) and Yalla's own `microyalla`/`nanoll.v` kernels, on the
   basis of what builds with the current Rocq and what the generated script
   needs. State the choice, the version pinned and how a user installs it,
   in the report and in the CLI's `--help` for `--format rocq`.
2. **The exporter** (`--format rocq` is a `Format` variant and a
   `prove.rs` arm, `plan/reports/04-api-and-cli.md`): atoms as variables of `formula`, formulas in the
   kernel's constructors and notations, the goal `ll [conclusion]` (or the
   kernel's equivalent), one tactic per derivation node in the order the
   derivation tree gives, with the exchange steps the kernel's list-based
   sequents require (our derivation's sequents are sets of occurrences, so
   the exporter fixes an order and inserts permutations where a rule needs
   its principal formula at the head; Click & coLLecT's `ex_perm_r` is the
   model). Weakening, contraction, dereliction and promotion map to the
   kernel's rules; Mix to `mix_r` if the kernel's `pfrag` allows it, else
   the exporter refuses with a clear message.
3. **Checking**: a flake check that runs Rocq on one generated certificate
   per fragment with the kernel available offline (Rocq from nixpkgs; the
   kernel as a flake input pinned to a commit, built in the check, or
   vendored if it is a handful of files with a compatible license, which
   `deny.toml`'s allow list does not cover for LGPL: vendoring must not
   change linlog's license story, so prefer the flake input). If the
   closure or the build time is unreasonable, keep the check out of `nix
   flake check`, make it a separate `nix build .#checks.<system>.rocq` that
   CI runs in its own job, or document a manual procedure; explain in the
   report.
4. **Lean and Agda**: assess only. Say in the report what a Lean 4 target
   would need (the FormalizedFormalLogic/LinearLogic project's calculus,
   its gaps such as units) and whether an Agda kernel exists that is worth
   targeting; do not implement unless it is nearly free.
5. **Tests**: snapshot tests of the generated scripts; the nix check;
   round-trip on the standard examples in every fragment, including a proof
   with contraction and one with weakening.
6. **Documentation**: README (how to check a certificate), CLAUDE.md
   commands, `.claude/rules/ci.md` if a CI job was added.

## Constraints

- No `unsafe`; the exporter is a pure function in `core`.
- The certificate must be checked by the kernel, not merely parsed: a
  script that Rocq accepts with `Admitted` is not a certificate. Use `Qed`.
- Do not fake results: if the nix check cannot be made to work, say so.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `nix flake check` at the end (`jj st` first), and
the Rocq check itself with its output pasted into the report.

## Deliverables

- Thematic jj commits ("Export derivations as Rocq scripts for NanoYalla",
  "Check the Rocq certificates in nix", …).
- `plan/reports/11-certificates.md`: the kernel chosen and why, the
  permutation strategy, what is certified for which fragments and modes,
  the Lean/Agda assessment, decisions, deviations, open questions.

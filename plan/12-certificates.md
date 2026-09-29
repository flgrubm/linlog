# Step 12: proof certificates for Rocq

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

## What step 9 left you

`plan/reports/09-interactive.md`, "For steps 10 to 12 and the web front
end". A derivation may now contain open goals (inferences with `rule ==
Rule::Open`): a certificate of one does not exist, so the exporter refuses
such a derivation with a clear error rather than emitting `Admitted`. A
finished interactive proof is certified through its term: `Interactive::proof()`
gives the checked `Proof`, and `proof.derivation()` (or
`two_sided_derivation()`) is the tree to export, not
`Interactive::derivation()`, since the checker's placement of `?c` and
`?w` is the term's and the two can differ; only the root sequent agrees.
`Rule::classical()` maps every two-sided name to its one-sided rule, which
the Yalla mapping wants for the classical kernel.

Steps 10 and 11 (`plan/reports/10-latex-typst.md`, "The API" and "For
step 11 and step 12"; `plan/reports/11-svg.md`, "Decisions") built the
export scaffolding to reuse: `export::notation::walk`
walks a derivation with an explicit stack and hands out enter and exit
events, the exits in postfix order, which is the order of tactics for a
script that proves premises before conclusions; `export::Form`
(`Fragment` or `Standalone`) is the type for "a proof script to paste"
versus "a whole `.v` file with its `Require`s", and the CLI's `form`
decides which formats take `--standalone`, `note` writes the verdict as
the format's comment (`(* … *)`) and `derivation(proof, mode, format,
form)` in `prove.rs` has one arm per format. The symbol table `Notation`
is for math notation and does not fit Rocq's constructors; write the
formula printer of the kernel's syntax in `export::rocq`. `Form` is gated
on `latex` or `typst` only (step 11 left `svg` off it, since an SVG has
one form); a Rocq export has two natural forms, the lemma with its proof
script to paste into a development and a whole `.v` file with its
`Require`s, so take `Form`, add `rocq` to its `cfg(any(…))`, and let
`form` in `cli/src/prove.rs` accept `--standalone` for `rocq` (it refuses
the flag for one-form formats). Snapshots live in `core/tests/snapshots/`
through `core/tests/export.rs` (`snapshot`/`pin`, `BLESS=1` rewrites
them), and `modules/export.nix` is the flake check that compiles and
renders exactly those files plus a few CLI outputs, offline, with its
tools from nixpkgs: the Rocq check follows that shape (the snapshots
copied in, `linlog prove --format rocq` for one more, the kernel and Rocq
as `nativeBuildInputs`, output that must be empty), in `export.nix` if it
is cheap or as its own `modules/rocq.nix` check if the closure is large.

## Goal

`export::rocq`, behind the cargo feature `rocq` (decision D14; on by
default, enabled by the CLI): from a derivation, a Rocq script that certifies the sequent
in a Yalla-based kernel, the way Click & coLLecT does with NanoYalla, for
classical derivations (MLL, MALL, MELL, LL, with Mix where the kernel
supports it) and, if Yalla's ILL development makes it reasonable, for
intuitionistic ones: Yalla's `ill` takes two-sided sequents with the ILL
rules, `Derivation::two_sided` is that view (the same inferences as the
classical one, named through `Rule::intuitionistic` by the principal
formula's position; a `⊸L` node's two premises are its split), and the
sequent certified is the one the reading prints, which for formulas built
from `⊤` and `0` alone can differ from the written succedent
(`plan/reports/08-intuitionistic.md`, "Ambiguity"; say so in the report
if it matters to the kernel). `linlog prove --format rocq`. A way to actually check
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
- `plan/reports/12-certificates.md`: the kernel chosen and why, the
  permutation strategy, what is certified for which fragments and modes,
  the Lean/Agda assessment, decisions, deviations, open questions.

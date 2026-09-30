# Step 10: LaTeX and Typst export

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D6, D12) and every report in `plan/reports/`,
  especially `02-proofs.md` (the derivation view) and
  `08-intuitionistic.md` (two-sided derivations).
- `plan/notes/export-targets.md` §§ "Click & coLLecT" (its LaTeX export),
  "LaTeX proof trees", "Typst proof trees". Verify the package syntax
  against the current manuals (WebFetch CTAN for ebproof, the curryst
  README on GitHub) before emitting it.
- `core/src/export/**` (placeholder), `core/src/proofs/**`, `cli/src/**`.

## What step 9 left you

`plan/reports/09-interactive.md`, "For steps 10 to 12 and the web front
end". A proof in progress is `Interactive::derivation()`, an ordinary
`Derivation` (premises before conclusions, the root last, two-sided in
intuitionistic mode) whose open goals are inferences with `rule ==
Rule::Open`, no principal and no premises; `Rule::name` gives `open` for
it, which is never a label. The text renderer draws such a leaf as its
sequent alone, with no bar. `Rule::classical()` maps a two-sided name to
the one-sided rule and `Rule: FromStr` reads a name back, in case the
label table wants either. The `interact` command's `show` prints the text
tree; giving `show` and `proof` a format argument is welcome if it is one
arm each, not a second output layer, since the web client (`plan/later.md`) is
the front end for partial derivations.

## Goal

`export::latex` and `export::typst`: sequents and derivations (classical
one-sided and intuitionistic two-sided) as LaTeX with ebproof and as Typst
with curryst, as fragments to paste and as standalone documents that
compile. `linlog prove --format latex|typst` and `linlog seq print --format
latex|typst`. Proof nets are drawn in step 11 (SVG), not here. Both
emitters live behind cargo features `latex` and `typst` (decision D14), on
by default and enabled by the CLI; step 9's partial derivations (open goals
as leaves of `Rule::Open`) render too, an open goal drawn as a leaf with a
dotted bar and a mark (ebproof's `\hypo` with a marker, curryst's leaf),
one shape for both targets. With `latex` and `typst` the crate has five features, so this step
also replaces the full feature powerset by `cargo hack check
--each-feature -p linlog` plus `--feature-powerset --depth 2 -p linlog` in
`modules/checks.nix` and in CLAUDE.md's command list and verification
table (decision D14).

## What to build

1. **Formula and sequent printers** for both targets, with the symbols the
   packages expect (`\otimes`, `\parr` from `cmll`, `\with`, `\oplus`,
   `\multimap`, `\oc`, `\wn`, `\mathbf{1}`, `\bot`, `\top`, `0`, `^\bot`
   for duals, `\vdash`; Typst: `times.circle`, `⅋` or the `parr` symbol
   available in Typst's math, `\&`, `plus.circle`, `multimap`, `!`, `?`)
   with the same bracketing rules as `Display`. Variable names escaped for
   each target (underscores, non-ASCII identifiers: decide and document).
2. **Derivations**: ebproof's postfix `\hypo`/`\infer<n>[label]` and
   curryst's nested `rule(name: …, premises…, conclusion)`, rule labels
   from the derivation's rule enum, two-sided sequents with the turnstile
   aligned where the package supports it. A two-sided derivation
   (`Proof::two_sided_derivation`, `Derivation::reading()` is `Some`) is
   printed as `proofs/fmt.rs` does: the hypotheses (occurrences in input
   position, in id order), `⊢`, the goal, each formula spelled through
   `Reading::formula` (which recovers `⊸`, `1` and `0` from the one-sided
   arena; the arena's `⅋` never appears two-sided). `Rule::name` gives the
   ILL names with Unicode subscripts (`&L₁`, `⊕R₂`); map them to the
   target's math (`\&L_1`, `plus.circle R_2`) in one table. An option for the standalone
   document (`\documentclass{standalone}` or `article` with the preamble;
   Typst with `#import "@preview/curryst:<version>"`, version pinned in one
   constant) versus a bare fragment.
3. **CLI**: `--format latex|typst` on `prove` and on `seq print`,
   `--standalone` (or the reverse), `--output PATH`. A format is a variant
   of `Format` in `cli/src/argument_parsing.rs` and an arm in `prove.rs`;
   build the derivation inside the `on_large_stack` closure as the text
   format does (`plan/reports/04-api-and-cli.md`). That report also notes
   that the text renderer is slow on huge derivations because every line
   is as wide as the tree; an emitter that writes one inference at a time
   does not have that problem, so do not copy the renderer's structure.
4. **Compile check in nix**, if it is reasonable: a flake check that
   compiles one LaTeX sample (a small texlive set with ebproof and cmll) and
   one Typst sample (nixpkgs' `typst`; curryst must be available offline,
   which may need the package vendored or fetched as a flake input) from the
   CLI's output. If either would add a heavy closure or need the network,
   leave it out, keep the samples as snapshot tests, and document the manual
   check in the report. Follow `.claude/rules/ci.md` for anything under
   `.github/` and the dendritic layout for `modules/`.
5. **Tests**: snapshot tests of the emitted text for a set of derivations
   (MLL, MALL, MELL with structural rules, an ILL derivation, and one
   partial derivation from `Interactive` with an open goal), escaping of
   names, and the compile checks above if they exist.
6. **Documentation**: README usage section; CLAUDE.md commands if a check
   was added; `.claude/rules/core.md` only if the derivation view changed.

## Constraints

- No `unsafe`; no dependency unless it earns its place (string building
  with `std::fmt` is enough).
- Deterministic output; stable across runs so snapshot tests hold.
- Keep the printers in `core` (pure functions of the derivation) and the
  file handling in the CLI.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `nix flake check` at the end (`jj st` first),
plus a manual compile of one LaTeX and one Typst output if the nix check
does not exist, pasted into the report.

## Deliverables

- Thematic jj commits ("Print formulas for LaTeX and Typst", "Export
  derivations with ebproof", "Export derivations with curryst", "Compile
  the export samples in nix", …).
- `plan/reports/10-latex-typst.md`: API, the escaping rules, what the
  standalone documents need installed, decisions, deviations, open
  questions, and what step 11 can reuse (the symbol tables, the bracketing).

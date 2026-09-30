# Step 11: SVG export of sequents, derivations and proof nets

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D6a, D12) and every report in `plan/reports/`,
  especially `05-proof-nets.md` (the net structures and what they expose
  for drawing), `02-proofs.md` (the derivation view) and
  `10-latex-typst.md` (symbol tables, bracketing; reuse them).
- `plan/notes/export-targets.md` § Crates for the `svg` crate.
- `core/src/export/**`, `core/src/nets/**`, `core/src/proofs/**`.

## What steps 9 and 10 left you

`plan/reports/09-interactive.md`, "For steps 10 to 12 and the web front
end": a proof in progress is `Interactive::derivation()`, a `Derivation`
whose open goals are inferences with `rule == Rule::Open`, no principal
and no premises; the text renderer draws such a leaf as its sequent alone
with no bar, and step 10 chose one shape for an open leaf in LaTeX and
Typst (its report, "Decisions"): the sequent under vertical dots, with no
inference line, because neither package draws a dotted bar per inference.
Draw the same thing in SVG. `plan/reports/10-latex-typst.md`, "For step
11 (SVG) and step 12": `export::notation::Notation` is the symbol table
with the printers (`term`, `ill`, `one_sided`, `sequent`) that keep the
bracketing of `Display`; a Unicode table with `&` and `<` escaped for XML
is the SVG text of formulas, and `Rule::name` the label. `notation::walk`
is the explicit-stack traversal with enter and exit events; subtree
widths for the layout come from a post-order pass over the exits. Add
`feature = "svg"` to the `cfg(any(…))` on `notation` and `Form`. In the
CLI, `derivation(proof, mode, format, form)` in `prove.rs` has one arm
per format, `note` writes the verdict as the target's comment (`<!-- …
-->` for SVG) and `form` says which formats take `--standalone`; an SVG
is always a whole document, so decide whether `--standalone` is accepted
as a no-op or refused, and say why. `interact show` takes a format name
(`show latex|typst`): add `svg`. The SVG of a partial
derivation is what the web front end (`plan/later.md`) will draw after every
`apply`, so the derivation function must be cheap to call repeatedly and
pure (data in, string out); the report's list of what the front end will
call is the client's side of it.

## Goal

`export::svg`, behind the cargo feature `svg` (decision D14; on by default,
enabled by the CLI), partial derivations with open goals (`Rule::Open`
leaves) included:
deterministic, dependency-light SVG for a sequent (one line of
text), a derivation (a proof tree) and a proof net (formula trees with axiom
links), usable from the CLI (`--format svg` on `prove` and `seq print`, and
for nets) and later from the web front end (a pure function from data to a
string). Readable at the default size, scalable, with the text selectable.

## What to build

1. **Layout with a declared font and a committed advance table (D12).**
   The font is Euler Math (the author's choice for every drawing of
   sequents, derivations and nets): `font-family: "Euler Math", "Neo
   Euler", serif` in the SVG, with the letters in italic as math sets
   them and rule labels upright. Widths are the sum of per-character
   advances from a table of Euler Math's advances (em units) for every
   character the printers emit (Latin letters, digits, the connectives and
   units, brackets, comma, space, `⊢`, `⊥` as a superscript, the dots of
   an open goal), measured once from the font file and committed as a
   constant with the measuring command in the report; a character outside
   the table gets a fixed fallback advance. Every `<text>` run also
   carries `textLength` (its computed width) with
   `lengthAdjust="spacing"`, so a viewer without the font still fits the
   layout, and the layout stays deterministic and metric-free at run time.
   Get the font from nixpkgs if it is packaged (search for Euler Math,
   Neo Euler; the OTF is Khaled Hosny's `euler-otf`, OFL) or as a flake
   input pinned to a release, for the measurement and for a rendering
   check; the SVG does not embed it. Expose the line height, gaps, margins
   and colours as a `Style` struct with defaults.
1a. **Euler in the step 10 documents.** The standalone documents of the
   LaTeX and Typst exports set the same font: `\usepackage{eulervm}`
   after `amssymb` in the LaTeX preamble (the texlive package `eulervm`
   added to `modules/export.nix`; cmll's own connectives are unaffected)
   and `#show math.equation: set text(font: "Euler Math")` in the Typst
   page setup, with the font on `typst compile`'s `--font-path` in the
   check so that no fallback warning hides a missing font. Re-bless the
   snapshots (`BLESS=1 cargo test -p linlog --test export`) and keep the
   fragments unchanged: a fragment takes the fonts of the document it is
   pasted into.
2. **Sequent**: text on one line, with the turnstile and connectives; a
   `<title>` with the plain text.
3. **Derivation**: the classic bottom-up layout: each node's width is the
   maximum of its sequent's width and the sum of its premises' widths plus
   gaps; premises centred above the conclusion; a horizontal bar with the
   rule name to its right (as ebproof draws it); the tree grows upwards
   with the root at the bottom. Two-sided sequents for intuitionistic mode
   (`Derivation::reading()`, printed as step 10's emitters print them:
   hypotheses in id order, `⊢`, the goal, formulas through
   `Reading::formula`). Long trees are wide; do not try to wrap.
4. **Proof net**: conclusions in a row at the bottom, each formula tree
   drawn upwards from its root with binary nodes as small labelled circles
   (`⊗`, `⅋`, `&`… only MLL nets exist, so `⊗` and `⅋`, plus Mix as
   nothing) and literals as labels at the top layer; each axiom link as an
   arc above the literals it connects, nesting arcs so they do not cross
   where the linking is planar and letting them cross otherwise (a simple
   height rule such as height proportional to the horizontal distance keeps
   it readable); ⅋ premise edges drawn distinguishably (the two edges of a
   ⅋ share a colour or a style so that a reader can see the switchings).
   Optional: highlight a switching cycle or the disconnection when the net
   is incorrect (the criterion returns the witness).
5. **CLI** (a `Format` variant and a `prove.rs` arm, built on the search
   thread; `plan/reports/04-api-and-cli.md`): `--format svg` on `prove` (derivation, or the net with
   `--net`/`--format net-svg`: choose a shape consistent with step 4's
   design and say why), on `seq print`, and a `linlog net` subcommand if
   nets deserve their own entry point (draw a net from a serialized net or
   from a proved MLL sequent; check a net's correctness). `--output`
   writes the file; stdout otherwise.
6. **Tests**: snapshot tests of the SVG text for small inputs (one of them
   a partial derivation from `Interactive`); structural tests (well-formed XML, the expected number of `<text>`/`<path>`
   elements, no element outside the viewBox); a visual check by you, by
   rendering a few samples with `rsvg-convert` or `resvg` from nixpkgs if
   available and looking at the PNGs (the `Read` tool shows images), and
   attaching what you saw to the report.
7. **Documentation**: README (with one SVG committed as an example if
   small), CLAUDE.md commands, `.claude/rules/core.md` only if data types
   changed.

## Constraints

- No `unsafe`. The `svg` crate only if it saves real code over `write!`
  with escaping; either way, escape text for XML.
- Deterministic output; no random ids; ids derived from occurrence ids.
- No general graph layout (D12), no JavaScript inside the SVG.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo deny check` if a dependency changed,
`nix flake check` at the end (`jj st` first).

## Deliverables

- Thematic jj commits ("Add an SVG writer with a fixed-advance layout",
  "Draw derivations as SVG", "Draw proof nets as SVG", "Add net commands
  to the CLI", …).
- `plan/reports/11-svg.md`: API, the layout rules, the visual check's
  findings, decisions, deviations, open questions, and what the web front
  end will need (a pure function signature, the `Style` knobs).

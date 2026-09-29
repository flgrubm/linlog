# Step 10: SVG export of sequents, derivations and proof nets

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D6a, D12) and every report in `plan/reports/`,
  especially `05-proof-nets.md` (the net structures and what they expose
  for drawing), `02-proofs.md` (the derivation view) and
  `09-latex-typst.md` (symbol tables, bracketing; reuse them).
- `plan/notes/export-targets.md` § Crates for the `svg` crate.
- `core/src/export/**`, `core/src/nets/**`, `core/src/proofs/**`.

## Goal

`export::svg`: deterministic, dependency-light SVG for a sequent (one line of
text), a derivation (a proof tree) and a proof net (formula trees with axiom
links), usable from the CLI (`--format svg` on `prove` and `seq print`, and
for nets) and later from the web front end (a pure function from data to a
string). Readable at the default size, scalable, with the text selectable.

## What to build

1. **Layout without font metrics (D12).** A monospace font stack declared
   in the SVG (`font-family: ui-monospace, "JetBrains Mono", Menlo, Consolas,
   monospace`) with a fixed advance per character and a fixed line height,
   so widths are `chars × advance`; Unicode connectives count as one
   character. Expose the advance, line height, margins and colours as a
   `Style` struct with defaults.
2. **Sequent**: text on one line, with the turnstile and connectives; a
   `<title>` with the plain text.
3. **Derivation**: the classic bottom-up layout: each node's width is the
   maximum of its sequent's width and the sum of its premises' widths plus
   gaps; premises centred above the conclusion; a horizontal bar with the
   rule name to its right (as ebproof draws it); the tree grows upwards
   with the root at the bottom. Two-sided sequents for intuitionistic mode.
   Long trees are wide; do not try to wrap.
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
5. **CLI**: `--format svg` on `prove` (derivation, or the net with
   `--net`/`--format net-svg`: choose a shape consistent with step 4's
   design and say why), on `seq print`, and a `linlog net` subcommand if
   nets deserve their own entry point (draw a net from a serialized net or
   from a proved MLL sequent; check a net's correctness). `--output`
   writes the file; stdout otherwise.
6. **Tests**: snapshot tests of the SVG text for small inputs; structural
   tests (well-formed XML, the expected number of `<text>`/`<path>`
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
- `plan/reports/10-svg.md`: API, the layout rules, the visual check's
  findings, decisions, deviations, open questions, and what the web front
  end will need (a pure function signature, the `Style` knobs).

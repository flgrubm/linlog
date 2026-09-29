# Step 11 report: SVG export of sequents, derivations and proof nets

Session of 2026-09-29, from `plan/11-svg.md`. Realises D12 (generated
layouts, Euler Math advances, `textLength`) and the `svg` feature of D14,
and switches the step 10 standalone documents to Euler.

## Outcome

`linlog::export::svg`, behind the default cargo feature `svg`, draws a
sequent as one line of text, a derivation (finished or in progress,
one-sided or two-sided) as a proof tree, and a proof structure (a net or
not, complete or not) as formula trees under arcs for the axiom links.
Every function is pure, from data and a `Style` to a `String`, and the
output is byte-stable: all coordinates are integers computed from a
committed table of Euler Math's advances. The CLI gained `--format svg`
(the derivation) and `--format net-svg` (the proof net) on `prove` and
`check`, `--format svg` on `seq print`, and `show svg` in `interact`. The
standalone LaTeX documents now load `eulervm` and the Typst documents set
math in Euler Math. The flake's `export` check additionally renders every
SVG snapshot and two CLI drawings with resvg, with Euler Math as the only
font, and fails on any warning from Typst or resvg.

Seven commits:
1. "Add an SVG writer with a fixed-advance layout"
2. "Draw derivations as SVG"
3. "Draw proof nets as SVG" (with the dev-dependency roxmltree for the
   structural tests)
4. "Add the svg and net-svg formats to the CLI"
5. "Set the standalone documents in Euler and render the SVGs in the
   export check"
6. "Document the SVG export"
7. This report.

All checks pass at the last code change (see "Verification").

## The API

- `svg::Style`: the knobs, with `Default`. Lengths are `u32` thousandths
  of an em of formula text; colours are CSS strings, escaped on output.
  - `font_size` (16 px): the scale of the whole drawing.
  - `label_size` (800): rule names and the connectives in net nodes.
  - `line_height` (1500): baseline to baseline between a conclusion and
    its premises, and between two layers of a net.
  - `premise_gap` (1500), `literal_gap` (1000), `label_gap` (200),
    `margin` (300), `stroke_width` (40).
  - `link_height` (600): an arc's height per thousand of half its width.
  - `node_radius` (380).
  - `text`, `line`, `par` (`#0072b2`), `link`, `highlight` (`#d55e00`),
    and `background` (`None`, transparent).
- `svg::sequent(&Sequent, &Style)` and `svg::two_sided(&Reading,
  &Style)`: one line.
- `svg::derivation(&Derivation, &Style)`: the tree, two-sided when the
  derivation has a reading, open goals included.
- `svg::net(&ProofStructure, &Style)`: any structure; a switching cycle
  is highlighted when `is_correct` returns one.
- Every document has a `<title>` with the plain text of its sequent
  (`⊢ A⊥, A`, `A, A ⊸ B ⊢ B`), the root sequent for a derivation.
- Ids, all derived from the data: `i<n>` for the conclusion of inference
  `n`, `o<n>` for the literal or connective at occurrence `n`, `l<m>-<n>`
  for a link.

## The layout rules

- **Text.** Formulas are written by `notation::Notation` with a third
  table: Unicode connectives, the atom letters as mathematical italic
  code points (U+1D434…, `h` as U+210E), and `\u{1}` for the `⊥` of a
  negated atom. A line is cut into pieces of one baseline and one size;
  each piece is its own `<text>` with `x`, `y`, `textLength` and
  `lengthAdjust="spacing"`. The raised `⊥` is 70 % high and 0.4 em up;
  the `₁`/`₂` of a rule name are 70 % digits 0.15 em down, since the font
  has no subscript characters. Spaces separate pieces rather than start
  or end one.
- **Widths** are the sum of the table's advances, a script character at
  70 %, and 650 thousandths for a character outside the table. Every line
  reserves 890 above its baseline (a raised `⊥`) and 210 below (the
  comma).
- **Derivations.** One pass up the tree over `walk`'s exits and one pass
  down over its enters, both with the walk's own stack.
  - A leaf's box is its conclusion, plus its label for a rule.
  - Premise boxes stand side by side, `premise_gap` apart.
  - The conclusion is centred under the span from the first premise's
    conclusion to the last one's.
  - The inference line spans that span and the conclusion. It lies
    halfway between the premises' lowest reach and the conclusion's
    highest.
  - The rule name follows the line after `label_gap`, with its math axis
    on the line, and counts towards the box's width.
  - Rows are uniform, `line_height` apart, with the root at the bottom.
  - An open goal is its sequent under `⋮` at the premises' height, with
    no line.
- **Nets.**
  - The literals stand in occurrence order along the top, which is left
    to right in every tree, `literal_gap` apart.
  - A connective sits halfway between its premises, one layer below the
    lower one; a binary root sits on the bottom layer. Every root has an
    edge hanging to half a layer below the bottom layer, from its literal
    when the root is one.
  - Nodes are circles with `⊗` or `⅋` inside. The edges of a `⅋` to its
    premises are dashed in `par`, and an edge ends at the circle.
  - A link is a half-ellipse from the tops of its two literals, with
    `rx` half the distance and `ry = rx × link_height / 1000`. All arcs
    have one shape, so nested pairs give nested, non-crossing arcs, and
    interleaved pairs cross.
  - The edges and links of a switching cycle are drawn in `highlight`.

## The advance table

`core/src/export/svg/font.rs` holds 202 entries: printable ASCII, the 52
mathematical italic letters, Greek (without final sigma, which the font
lacks), and `⊗ ⅋ ⊕ ⊸ ⊥ ⊤ ⊢ ⋮`. The font is Euler Math 0.75 (1000 units per
em) from nixpkgs' `texlivePackages.euler-math` (CTAN `euler-math`, Khaled
Hosny's OTF, OFL). The vertical constants (math axis 250, superscript
shift, script size 70 %) are from its `MATH` table, and the reaches from
the glyph bounds. The table was printed by this script:

```python
# advances.py: prints the advance table of Euler Math for the SVG export.
import sys
from fontTools.ttLib import TTFont
font = TTFont(sys.argv[1])
assert font['head'].unitsPerEm == 1000
cmap, hmtx = font.getBestCmap(), font['hmtx']
chars = [chr(c) for c in range(0x20, 0x7F)]
chars += [chr(0x1D434 + i) for i in range(26)]
chars += ['ℎ' if i == 7 else chr(0x1D44E + i) for i in range(26)]
chars += [chr(c) for c in range(0x391, 0x3AA) if c != 0x3A2]
chars += [chr(c) for c in range(0x3B1, 0x3CA)]
chars += list('⊗⅋⊕⊸⊥⊤⊢⋮')
for c in sorted(set(chars)):
    if ord(c) in cmap:
        esc = {"'": "\\'", "\\": "\\\\"}.get(c, c)
        print(f"    ('{esc}', {hmtx[cmap[ord(c)]][0]}),")
    else:
        print(f"missing {c!r}", file=sys.stderr)
```

It was run as follows:

```sh
font=$(nix build --no-link --print-out-paths --inputs-from . \
  nixpkgs#texlivePackages.euler-math.tex)/fonts/opentype/public/euler-math/Euler-Math.otf
nix shell --impure --expr 'let pkgs = (builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux;
  in pkgs.python3.withPackages (ps: [ ps.fonttools ])' -c python3 advances.py "$font"
```

## The visual check

Renders were made with resvg 0.48.1 from the locked nixpkgs, with Euler
Math as the only font, and compared with librsvg 2.62.3 through a
fontconfig file that lists only the Euler directory. What I saw:
- **resvg spread text with a `dy` tspan.** The first version drew a
  superscript as a `<tspan dy>` inside one `<text>` with one
  `textLength`. librsvg drew it correctly. resvg spread the glyphs of
  every line holding a `⊥` across wide gaps and cut the `₂` of `&L₂`.
  resvg is also what Typst uses to draw SVG images, so pieces became
  separate `<text>` elements (commit 2). After that, both renderers
  agree.
- **Derivation snapshots** (`mll`, `mall`, `mell`, `ill`, `open`) and
  their larger siblings look as intended.
  - Checked: `!A, A -o B, B -o C |- C * (A & !A)`, twelve inferences
    over five rows.
  - Checked: a partial derivation from `interact` with two open goals.
  - Premises are centred, lines span the wider of premises and
    conclusion, labels sit on the lines, and `⋮` stands over open goals.
  - Superscript `⊥`, the subscript `₂` and the italic atoms read as in
    the Typst output, which was also rendered with Euler Math.
- **Nets.**
  - `⊢ A⊥ ⅋ B⊥, B ⊗ A` has nested arcs, and `A ⊸ B, B ⊸ C ⊢ A ⊸ C` has
    one crossing, as the linkings are.
  - The dashed blue `⅋` edges stand out.
  - The cycle of `⊢ A ⊗ A⊥` with its link is drawn orange.
- **Without Euler Math** (resvg with DejaVu only), the layout holds.
  `textLength` fits every piece, the italic letters come from DejaVu's
  math italic glyphs, and nothing overlaps.
- **Cost.** A 300-atom `⊗` chain (599 inferences, 8.9 MB of SVG, since
  every inference prints its whole sequent) took 0.18 s in a debug build
  of the CLI, search included. The text tree took 0.26 s.
- **Minor:** an edge or arc meets a negated literal at the middle of
  `A⊥`, not of `A`.

## Decisions where the prompt left room

- **No `svg` crate.** The text had to be measured and cut into pieces
  anyway, so a builder would have saved only the tag syntax. The writer
  is `write!` with one escaper (`& < > "`, and control characters in atom
  names as U+FFFD, which XML cannot carry).
- **Letters as mathematical italic code points, not `font-style:
  italic`.** Euler Math has no italic face, so a browser would slant the
  upright Euler shapes. The math italic characters are what a math
  renderer (Typst, unicode-math) sets. Euler maps them to its own
  letters with the same advances as ASCII, and a fallback font takes
  them from its math italic. Copied text carries those characters; the
  `<title>` holds plain text. Greek stays at its own code points. Rule
  names use ASCII letters and are upright.
- **Integer coordinates in thousandths of an em** with `font-size="1000"`
  in the view box, and `width`/`height` in pixels from `font_size`. That
  keeps the snapshots stable and the output free of float formatting.
- **`--standalone` is refused with `--format svg|net-svg`**, exit 2, with
  the same message as for text. The flag chooses between a fragment and
  a document, an SVG has only the document, and every other one-form
  format already refuses it. Accepting it as a no-op would make it mean
  two things.
- **`--format net-svg`, not `--net` or a `linlog net` command.** It keeps
  step 4's shape of one `--format` that picks both what and how. `net`
  prints the net as text, `svg` the derivation, `net-svg` the net as a
  drawing; `nets_exist` refuses the same sequents for both net formats.
  No `net` subcommand exists, because the CLI's nets are all nets of
  proofs, which `prove` and `check` already reach. A serialized
  `ProofStructure` has no producer outside the library (the outcome JSON
  carries the proof, not the net). Drawing structures built by hand is
  the web front end's job, and the API already draws any structure.
- **Comments.** The verdict and statistics are `<!-- … -->` lines before
  and after the document. `note` turns every `-` into `‐` (U+2010),
  since an XML comment cannot hold `--` and the advice names flags (`raise
  it with ‐‐copies`). An unprovable sequent prints the comment alone,
  which is not an XML document; the exit status says why, as with the
  other formats.
- **The switching cycle is highlighted** (the prompt's optional item),
  since it costs a lookup per edge. A disconnection is not coloured.
- **The export check renders the SVGs** with resvg (11.5 MiB). Output is
  fatal for both Typst and resvg, and both run without system fonts, so
  a missing Euler Math fails the check instead of falling back.

## Deviations from the prompt, with reasons

- `Form` is not gated on `svg`, although step 10's report suggested it.
  The SVG functions take no `Form`, since an SVG is always a document,
  and gating a type on a feature that does not use it adds nothing.
  `notation` is gated on `svg`, as asked.
- The prompt suggests escaping `&` and `<` in the table's strings. They
  are escaped where the line is laid out instead, since the layout reads
  every character for its advance anyway.
- Commit 2 also rewrote commit 1's text runs from tspans to pieces (the
  resvg finding above).

## Open questions and follow-ups

- **Per-formula ids** in derivation sequents (`i<n>` names a whole
  conclusion). The web front end will need them to click a formula of a
  goal; that is a `<g>` per formula with the position in its id.
- **Anchors at the atom.** An edge or link could meet a negated literal
  at its atom rather than at the middle of `A⊥`.
- **Wide nets have tall arcs.** Height grows linearly with width, as the
  prompt suggested. A cap would break the nesting guarantee (by
  reasoning, not tried: an inner arc as high as the outer one pokes
  through it at its apex), so any cap needs a different nesting-safe
  rule.
- **Disconnection** could colour the parts of `NetError::Disconnected`.
- **Greek in math italic** (U+1D6FC…) to match Typst's italic lowercase
  Greek, if users name atoms that way.
- **Rows are uniform.** A two-sided tree with no raised `⊥` could use a
  smaller `HEIGHT`; `line_height` is the knob for now.

## What the web front end will call

Every call is pure, allocation-only, with no clock or I/O, and works
on wasm:
- after every `Interactive::apply`, `undo` or `close`:
  `svg::derivation(&state.derivation(), &style)`. It is linear in the
  inferences times their sequents' length, and every conclusion is
  `<text id="i<n>">` or a `<g id="i<n>">` of pieces, so a click maps back
  to the `InfId` of a goal;
- while a student links a net with `ProofStructure::link`/`unlink`:
  `svg::net(&structure, &style)`. A partial structure draws with its
  unlinked literals bare. A complete one with a switching cycle shows
  it; that runs `is_correct`, whose witness isolation is quadratic on
  the error path (step 5's report). Literals and nodes are `o<n>`,
  links `l<m>-<n>`;
- `svg::sequent`/`svg::two_sided` for a goal list or a title;
- `Style` for sizes, gaps and colours (a dark theme sets `text`, `line`,
  `link` and `background`).

The front end should serve Euler Math (OFL) as a web font under the
family name `Euler Math`. Without it the layout still holds, through
`textLength`.

## Verification

At the last code change:
- `cargo clippy --workspace --all-targets -- --deny warnings` is clean.
- `cargo test --workspace` passes: core 105 unit tests (6 ignored), 5
  export, 9 parse and 10 serialize integration tests and 16 doc tests;
  CLI 2 unit and 9 integration tests.
- `cargo hack check --each-feature -p linlog` and `cargo hack check
  --feature-powerset --depth 2 -p linlog` pass (32 feature sets).
- `cargo deny check` passes with roxmltree added.
- `nix build .#checks.x86_64-linux.export` passes: pdfLaTeX with
  eulervm, Typst with Euler Math and no output, resvg on every SVG.
- `nix flake check` passes (run after the documentation commit; this
  report is prose that no check reads).
- The CLI was run by hand:
  - every new format on `prove` and `check`;
  - `seq print --format svg`, `interact show svg` and the error for
    `show pdf`;
  - the `--standalone` refusal (exit 2);
  - an unprovable sequent (the comment alone, exit 1);
  - an unknown one (`‐‐copies` in the comment, exit 3);
  - `net-svg` outside MLL (exit 2).
- No fresh-context reviewer ran: nothing trusted reads the drawings (the
  checker, the search and the certificates never do), so this is not a
  soundness-critical piece in the plan's sense. The structural test
  checks the geometry of every element against the view box instead.

# Step 10 report: LaTeX and Typst export

Session of 2026-09-29, from `plan/10-latex-typst.md`. Realises the export
half of plan decision D6 (LaTeX and Typst consume the derivation view) and
the `latex` and `typst` features of D14.

## Outcome

Sequents and derivations export to LaTeX and to Typst, and both outputs
compile. The export covers one-sided classical and two-sided
intuitionistic derivations, finished or in progress. `linlog::export::latex`
writes ebproof proof trees with the connectives of cmll and amssymb, and
`linlog::export::typst` writes curryst proof trees. Each target gives a
fragment to paste or a standalone document. Both modules sit behind cargo
features of their own names, on by default and enabled by the CLI. An open
goal of a proof in progress is drawn the same way in both targets: its
sequent under vertical dots, with no inference line. The CLI gained
`--format latex|typst` and `--standalone` on `prove` and `check`, `--format
text|latex|typst` and `--standalone` on `seq print`, and `show latex|typst`
in `interact`. `--output PATH` existed already.

A new flake check, `export`, compiles every pinned snapshot plus one proof
per format as the CLI writes it, with pdfLaTeX from a minimal TeX Live and
Typst with curryst from nixpkgs. It runs offline. The feature check
now runs `cargo hack check --each-feature` plus `--feature-powerset --depth
2` instead of the full powerset, as D14 asks once a fifth feature exists.

Eight commits:
1. "Print formulas and sequents for LaTeX and Typst"
2. "Export derivations with ebproof"
3. "Export derivations with curryst"
4. "Check the core crate's features one at a time and in pairs"
5. "Add the latex and typst formats to the CLI"
6. "Compile the export samples in nix"
7. "Document the LaTeX and Typst exports"
8. This report.

The first three were staged so that each builds and passes its tests alone.
All checks pass at the last code change (see "Verification"). No dependency
was added: the output is built with `String` pushes.

## The API

- `export::Form`: `Fragment` (the default) or `Standalone`. It is gated on
  either feature.
- `export::latex` (feature `latex`), with one function per kind of input:
  - `sequent(&Sequent, Form)` writes a sequent one-sided, as
    `$\vdash A^\bot, A$`.
  - `two_sided(&Reading, Form)` writes the reading's sequent as `$A,
    A \multimap B \vdash B$`.
  - `derivation(&Derivation, Form)` writes a `prooftree` environment, one
    `\infer<n>[$label$]{…}` per inference in ebproof's postfix order. It is
    two-sided when `Derivation::reading()` is `Some`, with the turnstiles
    aligned by `&\vdash`. An open goal is `\hypo{\vdots}` followed by
    `\infer[no rule]1{…}`.
- `export::typst` (feature `typst`) has the same three functions.
  `derivation` writes `#prooftree(rule(name: $…$, premises…,
  $conclusion$),)`. A rule with premises spans several lines, indented by
  its depth, and a leaf takes one line. An open goal is `grid(align:
  center, row-gutter: 0.4em, $dots.v$, $…$)`, a curryst leaf with no bar.
  `CURRYST` is the version a standalone document imports, `"0.6.0"`, and
  the flake check compiles against the same version.
- Standalone documents:
  - LaTeX is `\documentclass[border=5pt]{standalone}` with `amssymb` and
    `cmll`, plus `ebproof` for a tree.
  - Typst is `#set page(width: auto, height: auto, margin: 5pt)`, plus the
    curryst import for a tree. A sequent document imports nothing, so it
    compiles offline without the package.
- `export/notation.rs` is private and shared by both targets:
  - `Notation`, the symbol table: connectives, units, the dual mark, the
    turnstile, the alignment mark and the atom escaper.
  - `Notation::term` and `Notation::ill` print a formula with the same
    bracketing as `Sequent`'s and `Reading`'s `Display`.
  - `Notation::one_sided` and `Notation::sequent` print sequents.
  - `walk` visits a derivation's inferences with an explicit stack and
    emits `Step::Enter` and `Step::Exit` events. The exits come in postfix
    order.
- The rule labels are one table per target, `latex::label` and
  `typst::label`, each with one arm per `Rule`.

## Escaping rules

- **LaTeX.** An atom named by one ASCII letter is written as it is.
  - Every other name goes into `\mathit{…}`. Inside it, `{ } $ # % & _`
    get a backslash; `\`, `^` and `~` become `\mbox{\textbackslash}`,
    `\mbox{\textasciicircum}` and `\mbox{\textasciitilde}`; and a space
    becomes `\ `.
  - Other characters pass through. A name beyond ASCII (such as `α`)
    therefore needs LuaLaTeX or XeLaTeX with `unicode-math`, or a
    character declaration under pdfLaTeX. The parser only produces
    Unicode identifiers, so the ASCII specials can arrive only through
    JSON input or the API.
- **Typst.** An atom named by one letter (`char::is_alphabetic`, which
  includes `α`) is written as it is. Every other name becomes `italic("…")`
  with `"` and `\` escaped, because Typst would read a longer name as a
  variable. Typst is Unicode throughout, so nothing else is needed.
- Unit tests in each module pin these rules. `core/tests/export.rs`
  (`sequents`) pins `x_1` and `foo` in a whole sequent.

## What the standalone documents need

- **LaTeX:** pdfLaTeX with the packages `standalone`, `amsfonts` (for
  `amssymb`), `cmll` and `ebproof`. The check uses TeX Live 2025 through
  nixpkgs' `texliveBasic.withPackages`, with ebproof 2.1.1. Its closure
  is 435 MiB, much of it perl and ghostscript.
- **Typst:** `typst` with curryst 0.6.0. `typst compile` downloads it from
  Typst Universe on first use, or it comes from nixpkgs'
  `typst.withPackages (ps: [ps.curryst_0_6_0])`, whose closure is
  103 MiB. The output was tested with Typst 0.15.1 only. curryst 0.6.0
  declares Typst 0.12 as its minimum, and the output uses nothing newer
  than that as far as I know, but no older Typst was tried.

## Decisions where the prompt left room

- **One shape for an open goal: the sequent under vertical dots, with no
  inference line.**
  - The prompt asked for a leaf with a dotted bar and a mark.
  - ebproof 2.1.1 offers the rule styles simple, no rule, double and
    dashed, with no dotted style.
  - curryst 0.6.0 takes one stroke for the whole tree (a `prooftree`
    argument), so no single inference can be drawn differently.
  - Vertical dots over the sequent is the textbook mark for a subproof
    still to be found, and both packages draw it natively. In ebproof it is
    `\hypo{\vdots}` and `\infer[no rule]1`; in curryst a content leaf is
    never barred.
  - It also stays close to the text renderer's bare sequent.
- **Typst connectives are Unicode characters, not symbol names.**
  - The prompt named `times.circle` and `plus.circle`, but Typst 0.15.1
    rejects both ("unknown symbol modifier"): they were renamed to
    `times.o` and `plus.o`.
  - Characters survive such renames, and Typst gives them their Unicode
    math class, so `A ⊗ B` spaces as `A times.o B` does.
  - Three spellings needed care: `&` is `class("binary", \&)`, since a
    bare `\&` sets no spacing; `?` is `class("normal", ?)`, since Typst
    treats `?` as punctuation and puts a space after it; and `1` is
    `bold(1)`, to match LaTeX's `\mathbf{1}`.
- **Rule labels are math with upright letters.**
  - LaTeX examples: `$\multimap\mathrm{L}$`, `$\with\mathrm{L}_1$`,
    `$\wn\mathrm{d}$`, `$\mathrm{ax}$`.
  - Typst examples: `$⊸ upright(L)$`, `$\& upright(L)_1$`,
    `$class("normal", ?) upright(d)$`, `$"ax"$`.
  - Typst keeps the space before a string in math, so letters are
    `upright(L)`; `"L"` rendered as `1 L` and `? d`.
  - All 33 labels were compiled in both targets in a scratch document and
    compared by eye.
  - Labels sit to the right of the bar (ebproof's `[label]`, curryst's
    `name:`).
- **Turnstiles are aligned only in two-sided LaTeX trees.** ebproof aligns
  a unary inference at the `&`. For one-sided `⊢ Γ` that is the left edge,
  which would stack the sequents flush left, so one-sided trees stay
  centred. curryst cannot align, so every Typst sequent is centred.
- **`0` is plain and `1` is bold**, as the prompt's symbol list says.
- **`Form` is an enum parameter** of every function, not a `bool` and not
  separate document functions. Each function knows which packages its
  document needs.
- **CLI output.**
  - With `--format latex|typst`, the verdict line and the `--stats` lines
    are written as comments of the target (`% …`, `// …`), so the whole
    output still compiles.
  - An unprovable or unknown sequent prints only that comment, with exit
    status 1 or 3 as before. `--quiet` prints only the comment.
  - `--standalone` with another format is an error, exit 2.
  - `check` gets the formats too, since it shares `Format` with `prove`.
  - `seq print` has its own `SequentFormat` (`text`, `latex`, `typst`),
    because JSON and nets mean nothing for a bare sequent and `seq json`
    exists.
  - `interact` gives `show` an optional format, printed as a fragment.
    `proof` got none: its argument is already a file name, and `proof
    latex` would be ambiguous.
- **The snapshots are standalone documents in files**
  (`core/tests/snapshots/{mll,mall,mell,ill,open}.{tex,typ}`), not
  strings in the test. That way the nix check compiles exactly what the
  tests pin. `BLESS=1` rewrites them.
  - crane's `cleanCargoSource` keeps only Rust sources, so
    `modules/workspace.nix` adds that directory to the source through
    `craneLib.fileset.commonCargoSources`.
  - The dependency build is unaffected, since crane's dummy source keeps
    only the manifests.
- **The compile check exists, with both halves.**
  - The Typst half (103 MiB) is clearly reasonable.
  - The TeX half (435 MiB) was the judgement call. I kept it because it is
    the only thing that proves the LaTeX output compiles, it is fetched
    from the binary cache rather than built, and it is small next to the
    Rust toolchain the other checks already pull.
  - If CI time or cache traffic matters more, drop `latex` from
    `modules/export.nix` and keep the `.tex` snapshots as plain snapshot
    tests.
- **Both cargo-hack commands, as D14 says.**
  - `--feature-powerset --depth 2` already covers every single feature,
    the empty set and the defaults (17 sets).
  - `--each-feature` adds only `--all-features`, which equals the defaults
    today but will differ once the non-default `parallel` feature arrives.

## Deviations from the prompt, with reasons

- The open goal has no dotted bar (see above: neither package can draw
  one per inference).
- Typst symbols are characters rather than the names the prompt lists,
  since two of those names no longer exist.
- The commits are grouped as listed above rather than exactly as the
  prompt's examples. Formulas and sequents came first, for both targets,
  because they share one printer; the prompt's names for the two
  derivation commits are kept.
- `.claude/rules/core.md` gained an "Export" section although the
  derivation view did not change. The step's general instructions ask for
  the invariants a later session cannot see in the code: the shared table,
  the shape of open goals, the snapshot and curryst coupling, and the
  package limits.

## Review

No fresh-context reviewer was run. Nothing trusted reads the export: the
checker, the search and the certificates never consume it, so it is not a
soundness-critical piece in the plan's sense. Instead:
- **Bracketing parity with `Display`.** A throwaway test mapped the Typst
  output of 400 random one-sided sequents (the classical generator) and
  400 random two-sided ones (the ILL generator) back to Unicode, and all
  800 equalled `Display`. The test was not committed. LaTeX shares the
  printer, so it has the same bracketing.
- **Rendering.** All ten snapshots were compiled with pdfLaTeX and Typst,
  rendered to PNG and inspected side by side: the same trees, labels and
  open goal in both targets. All 33 rule labels were compiled in both
  targets.
- **Limits.** Taller and wider trees were tried (see "Open questions").

## Open questions and follow-ups

- **curryst depth limit.** Under Typst 0.15.1, curryst 0.6.0 refuses a
  tree more than about eleven inferences high, with "maximum show rule
  depth exceeded". A par/tensor chain of height 11 compiled and one of
  height 13 did not. curryst nests several layout elements per level, and
  Typst caps show-rule nesting.
  - Many real derivations are taller, so Typst export is for small proofs
    until curryst changes.
  - This could be reported upstream.
  - The README, the module doc and `core.md` state the limit.
- **Wide LaTeX trees.** ebproof compiled a 120-high tree without trouble.
  A tree whose sequent lines are wider than TeX's largest dimension
  (about 5.7 m; here a sequent of 300 formulas) fails with "Arithmetic
  overflow". That is inherent to the content.
- **Greek atom names under pdfLaTeX.** Mapping `α` to `\alpha` and so on
  would make them compile without a Unicode engine. It was left out as
  scope. Proposed for step 15 if users name atoms that way.
- **`proof` in `interact`** could take a format with a syntax that does
  not collide with its file argument (such as `proof --latex`), if the web
  client does not make it moot.
- **Rule-label conventions.** Upright `L`/`R`, a subscript `1`/`2`, and
  `?d` for dereliction are one common convention among several. A user
  who prefers `\multimap_L` edits one table per target.

## For step 11 (SVG) and step 12 (certificates)

- **Reuse the symbol table and the printer.**
  - `export::notation::Notation` with a Unicode table is the SVG text of
    formulas and sequents: the same bracketing, with `&` and `<` escaped
    for XML by the table's strings and atom escaper.
  - Add `feature = "svg"` to the `cfg(any(…))` on `notation` and `Form`.
  - The two-sided layout (`Notation::sequent` with a reading: hypotheses
    in id order, turnstile, goal) is already the one the text renderer
    and these exports share.
- **Reuse the traversal.** `notation::walk` gives the explicit-stack
  traversal. SVG layout needs subtree widths (D12), which a post-order
  pass over the exits computes without recursion.
- **Keep one open-goal shape**: vertical dots over the sequent, no bar,
  in SVG too.
- **Labels.** `Rule::name` is the Unicode label for SVG. The LaTeX and
  Typst label tables show the upright-letter convention if SVG wants
  italic formulas with upright rule names.
- **Certificates.** Step 12 exports `proof.derivation()` (the term's view)
  and refuses `Rule::Open`, as step 9's report says; nothing here changes
  that.

## Verification

At the last code change:
- `cargo clippy --workspace --all-targets -- --deny warnings` is clean.
- `cargo test --workspace` passes: 102 unit tests in core with 6 ignored,
  3 export, 9 parse and 10 serialize integration tests, 15 doc tests, and
  2 unit and 8 integration tests in the CLI.
- `cargo hack check --each-feature -p linlog` and `cargo hack check
  --feature-powerset --depth 2 -p linlog` pass.
- `cargo doc -p linlog` gives no warnings.
- `nix flake check` passes, including the new `export` check and the
  `test` check with the snapshots in its source.
- Each of the three core commits was built, linted and tested on its own
  while staging. The first two were amended for a rustdoc link, then
  rebuilt and their docs checked.
- The CLI was exercised by hand:
  - every format on `prove`, `check`, `seq print` and `interact show`;
  - the `--standalone` refusal (exit 2);
  - an unprovable sequent in Typst (the comment alone, exit 1).
- No dependency changed, so `cargo deny check` was not run.

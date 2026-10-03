# Step 22: configurable output, and no font in LaTeX and Typst

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; the second starts from the first's report. Read before you
start:

- `plan/later.md`: "Configurable output, and no font in the LaTeX and
  Typst output" (the list of what a user might vary is the requirement),
  "Follow-ups: the exports", "Follow-ups: the command's output" (the
  compact view).
- `plan/reports/17-assessment.md`: 1.1, 1.2 (items 5 to 7 of the
  refactoring's list, D9), 3.2, and the author's answers.
- `plan/reports/10-latex-typst.md`, `11-svg.md`, `12-certificates.md`,
  `18-bounded-proofs.md`.
- `plan/README.md`: D12, D15, D16, D18.
- `.claude/rules/core.md`, "Export"; `.claude/rules/cli.md`.
- `core/src/export/**`, `core/src/proofs/derivation.rs` (`Rule`),
  `proofs/fmt.rs`, `core/tests/export.rs` with its snapshots,
  `cli/src/prove.rs`, `cli/src/interact.rs`, `modules/export.nix`.

## What the earlier steps left you

D15 holds for one export of four: `rocq::Options` has serde; `svg::Style`
has none; LaTeX, Typst and the text tree take no options value, and
their choices are constants. The standalone LaTeX and Typst documents
still set a font, against D12. The command passes every default and has
no flag for any of it. Typst refuses a curryst tree more than about
eleven inferences high, and that will not change upstream: the report is
curryst's issue 19, closed as not planned, and Typst 0.15.1 keeps its
depth constants. A drawing's inference ids are the derivation's and the
interactive session numbers its goals otherwise, with no map between
them. Of the 33 rule labels per target, 19 are in no snapshot.

Step 18 put one options value in front of every derivation
(`ViewOptions`, with the bound on its estimated size) and left these to
this step: the four exports return a `String` and cannot be stopped
inside; the command assembles its whole output as one string before it
writes it, twice the text tree at the peak, and the verdict with it; the
SVG layout takes 50 bytes of memory per character of sequent, six times
the other formats; `SCREENS` in the command and `GAP` in the text
renderer are constants; `linlog check` writes the whole sequent in its
verdict line, 8 MB for the largest net, with `--quiet` too; `--tree
never` leaves the derivation out of every format where its help speaks
of the text tree; and `ViewError`'s `Display` writes a saturated count
as `18446744073709551615` where the command writes `more than 10¹⁹`.

## Goal

Everything a user might want to vary in an output is a field of one
plain-data options value per output, with a sensible default, serde, and
a way to set it from the command, from JSON and from any other front
end; linlog's LaTeX and Typst never choose a font; and a proof of any
height can be set in Typst.

## What to build

First session:

1. **No font** in the LaTeX and Typst output, standalone or not; the
   `export` check's closure loses what only the font needed.
2. **One options value per output** (text tree, LaTeX, Typst, SVG, Rocq;
   the interactive session's messages if they are to be localised),
   each with `Default`, `Clone`, `PartialEq` and serde behind
   `serialize`, holding what `plan/later.md` lists; presets as named
   values. One signature for "write this derivation" across the
   outputs: into a writer, with a stop condition, so that the command
   writes the verdict first and the derivation after it as it is made,
   and holds no second copy of it.
3. **The rule labels as one table** per convention, not one match per
   target, with a user table as an option; and the round trip
   `Rule::from_str(rule.name())` as a test over every rule.
4. **The command's surface**: `--style KEY=VALUE` and `--style-file`
   (the options' JSON), `--lemma` and `--prelude`, whether the verdict
   comment is written; `interact` certifies a finished session, and its
   `show` and `proof` take a format without colliding with a file name.
   What step 18 left as constants or as one switch gets its place here:
   the screens a tree may fill, the gap between premises, how a sequent
   is abbreviated in a verdict line, and whether `--tree` speaks for
   the text tree alone.
5. **Ids a client can click**: a group per formula of a goal's sequent
   with its position, and a map from a drawing's inferences to the
   session's goals.
6. **Snapshots that cover every rule label**, compiled by the `export`
   check, fragments included.

Second session:

7. **A Typst tree from linlog's own layout** (the subtree widths of the
   SVG layout as a Typst grid or stack), as an option beside curryst and
   the default above the height curryst can set.
8. **A compact view of a derivation**: a run of one structural rule
   drawn as one inference (the 65 000 `?` steps of a net as one line),
   as an option of every output that draws a tree, and the default where
   step 18's bound would otherwise leave the tree out.
9. **The smaller follow-ups of the exports**: Greek atom names under
   pdfLaTeX, the names `lltp::read` makes, anchors at the atom, the
   nesting-safe arc cap, a disconnection coloured as a cycle is.

## Constraints

- D12: Euler stays where linlog draws (SVG), with the font and its
  advance table as options.
- Defaults reproduce today's output except for the font; the snapshots
  change by that and by what is added, and the report lists every other
  difference.
- No new export target. The surface of the library around the exports
  (errors, ownership, names) is step 23's; change here only what an
  option needs.

## Verification

The checks of CLAUDE.md's table, both `cargo hack` runs (the options
compile with and without `serialize`), `BLESS=1` once and the diff read,
`nix build .#checks.x86_64-linux.export`, `nix flake check` at the end;
every option set once from the command and once from JSON in a test; a
tree forty inferences high compiling under Typst.

## Deliverables

- Thematic jj commits.
- `plan/reports/22-configurable-output.md`: every option, its default,
  and how the command, the web front end and a third wrapper set it;
  decisions, deviations, open questions, what steps 23 and 27 must know.

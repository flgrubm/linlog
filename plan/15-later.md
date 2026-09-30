# Step 15 and beyond: later work

Sketches, not prompts. Each becomes a prompt in the style of the others
when its turn comes, after the reviews of steps 1 to 14 have settled the
APIs. The order is a suggestion.

## 15a. MELL proof nets with exponential boxes

Extend `nets` (step 5) with `!`-boxes and the `?` nodes (dereliction,
contraction, weakening as net nodes; or the "generalized ?" node with
auxiliary doors), correctness as the Danos–Regnier criterion applied at
each box depth with boxes contracted to single nodes (Guerrini–Masini 2001
for the parsing view), sequentialization through boxes, and the SVG drawing
with boxes as rectangles. Search stays with the focused engine; the value
is the representation (display, conversion, correctness). Fable 5.1, xhigh.

## 15b. Essential nets for IMLL

The spec's IMLL-Net: Lamarche's polarized structures using the D1
polarization from step 8, correctness by directed acyclicity plus the
dominator condition (Murawski–Ong), incremental search with a transitive
closure bit-matrix and an undo log (Moot 2008). Compare against the
embedding route of step 8 on the benchmarks of step 14 before making it the
default for IMLL: step 8 showed the verdict needs no essential-net
condition (every sequentialization of a classical net of an IMLL sequent
is intuitionistic), so this is a performance alternative, and its report
sketches the engine as the dominator condition added to the net engine's
`complete` branch. Fable 5.1, xhigh.

## 15a'. Focused-engine follow-ups from the exponentials

Taken up by step 14b (`plan/14b-performance.md`); what its report leaves
open stays here. For the performance pass (14b), with step 14's numbers: a canonical choice
among identical members of a stable sequent (hypotheses that are the same
formula are distinct occurrences today, so the memo sees `C(n, k)` sequents
where there is one up to renaming), a nested forced rule for a `⊗` factor
that is itself a tensor of positive literals, a per-level restart from the
frontier of exhausted sequents rather than re-exploring the levels below,
and a hash per branch-stack entry for the loop check. Separately, whether
a sound and useful affine prune exists (Kopylov's decidability argument
does not give one directly; the spec's was unsound) is a research question
to keep open; until then affine mode stays bounded.

## 15a''. Intuitionistic follow-ups

Left open by step 8, none of them a correctness issue. The written
succedent: for formulas built from `⊤` and `0` alone the reading's goal is
the last root by id, not the written one (`0, ⊤ ⊢ ⊤` prints as `0, 0 ⊢ 0`;
provability never differs, 607 464 cases brute-forced), and recovering it
means the arena keeps the parser's root order or the count of right-hand
roots, a change to `Sequent`'s canonical form and JSON; do it only if a
user of the two-sided print or the certificates asks. The additive path on
more than two roots, and on a `!` of an additive formula, is decided by
the focused engine today. The identical-hypotheses follow-up of 15a'
applies two-sided as well.

## 15b'. Net-engine pruning for repeated literals

Part of the performance pass (14b) if step 14's numbers ask for it: leaf
symmetry breaking for pure `⊗` and `⅋` trees of equal literals, a per-atom
balance over the `⊗`-skeleton components of a partial structure (the net
engine's analogue of the focused engine's split counts), and the sound
variant of symmetry breaking for equal compound conclusions (keys under
roots no symmetry moves; the spec's first-literal key is unsound across
groups, as step 6's report shows). Step 14's numbers
(`plan/reports/14-benchmarks.md`, "Focus against net") give the targets:
the Partition table (4.5 s and 3.7 s against 6 ms on the focused engine)
and the MLL 3-Partition at bins of five (57 s against 17 µs). They also
show that literal multiplicity is the wrong routing feature: the net
engine wins by four orders of magnitude on literals repeated three or
four times across conclusions (`wide-m3`, `wide-m4`) and loses as badly
on equal literals inside one pure `⊗` or `⅋` tree, and the reviewer's
counterexample at multiplicity four kept `NET_MULTIPLICITY` at two. So
the dispatch should route on "no two equal literals under one pure tree",
or the leaf symmetry break should remove that weakness and the threshold
rise; decide by the harness. Fable 5.1, xhigh.

## 15c. The focused inverse method

The spec's second engine for MALL and the semi-decision alternative for
MELL/ILL when Θ is large: forward saturation from initial sequents in the
subformula closure with subsumption indexing (feature vectors as in Schulz
2013). A new `search::inverse` engine with its own dispatch row driven by a
heuristic (many hypotheses, small goal) or `--engine inverse`. Fable 5.1,
xhigh.

## 15d. The !-Horn fragment through Petri-net reachability

Detect the fragment, build the net, and either call an external reachability
tool (KReach) through the CLI or implement coverability for the affine case.
Only worth it with the ILLTP Petri-net problems from step 14 as the
benchmark: of the 2 664 Petri nets its partial pass reached, 184 were
decided in 5 s, 895 stopped at the recursion limit and 651 at the
63-member split limit, both of which step 14b addresses, so assess after
14b what is left for a reachability route. Opus 5.5, xhigh.

## 15e. Cyclic MLL and the Lambek calculus

A non-commutative mode: planar axiom linkings in the net engine (links may
not cross in the cyclic order of literals), no exchange in the derivation
view and exports, and the Lambek restrictions (no empty antecedent, the two
divisions). Fable 5.1, xhigh.

## 15f. MALL proof nets

Only if a use case appears: Hughes–van Glabbeek nets or conflict nets are
non-canonical or exponentially large, so they are a display feature, not a
search vehicle. Assess first.

## 15g. The web front end

`linlog-web`: the `core` crate compiled to wasm without the `parallel`
feature (and without whichever optional features of D14 the client does not
ship), the interactive state of step 9 as the client's state with its JSON
as the wire form, the SVG of partial derivations from step 11 as the
picture (Euler Math served through `@font-face`, the same advance table
as the SVG), and the JSON formats for import and export. Its own plan, which
starts from `plan/reports/09-interactive.md`, "For steps 10 to 12 and the
web front end": the calls the client makes (`Interactive::new` or the JSON,
`goals`/`goal`/`reading` to draw the goals with positions as click
targets, `rules` for the menu, `split_passes` to grey out a split,
`apply`, `undo`, `close` with a node-counting stop closure since wasm has
no clock in core, `derivation` for the picture, `proof` at the end), and
the `linlog interact` command of step 9 as the reference behaviour; and
`plan/reports/11-svg.md`, "What the web front end will call": pure,
clock-free `svg::derivation`, `svg::net`, `svg::sequent` and
`svg::two_sided` with a `Style` (a dark theme sets its colours), the ids
`i<n>` (conclusion of inference `n`), `o<n>` (literal or node of
occurrence `n`) and `l<m>-<n>` (a link) as click targets, and Euler Math
(OFL) served as a web font under the family name `Euler Math`, the layout
holding without it through `textLength`.

## 15g''. Export follow-ups

Left open by step 10 (`plan/reports/10-latex-typst.md`, "Open questions").
Typst refuses a curryst 0.6.0 tree more than about eleven inferences
high ("maximum show rule depth exceeded", curryst nesting several layout
elements per level), so the Typst export is for small proofs: report it
upstream, and if it stays, write the tree with linlog's own layout (the
subtree widths of step 11's SVG, emitted as a Typst `grid`/`stack` with
explicit widths) instead of curryst, which also removes the package
import. Greek atom names under pdfLaTeX (`α` to `\alpha` in one table)
if users name atoms that way. `interact`'s `proof` could take a format
with a spelling that does not collide with its file argument (`proof
--latex`). The rule-label convention (upright `L`/`R`, subscript `1`/`2`,
`?d`) is one table per target for a user who wants another.

From step 11 (`plan/reports/11-svg.md`, "Open questions"), for the web
front end above all: a `<g>` per formula of a goal's sequent with the
position in its id, so that a click on a formula maps to `(InfId,
position)` for `Interactive::apply` (today `i<n>` names a whole
conclusion); edges and links meeting a negated literal at its atom rather
than at the middle of `A⊥`; a nesting-safe cap on the height of wide
axiom links (the height grows with the width, and a plain cap makes an
inner arc poke through its outer one); colouring a disconnection
(`NetError::Disconnected`) as the switching cycle is coloured; Greek
letters as mathematical italic code points to match Typst; and a smaller
row height for trees without a raised `⊥` (`Style::line_height` is the
knob).

## 15h. Configurable output, and no font in the LaTeX and Typst output

The author's requests of 2026-09-29, which D15 records as a decision for
every later step; this item applies it to what steps 10 and 11 built.

First the fix: the standalone LaTeX and Typst documents set a font
(`eulervm` in `export::latex`'s preamble, `#show math.equation: set
text(font: "Euler Math")` in `export::typst`'s page setup, both from step
11's item 1a). Remove both: a user pastes linlog's output into a document
and wants it to look like the rest of that document, so the program never
chooses a font there, standalone or not. Re-bless the snapshots, drop
`eulervm` from `modules/export.nix`'s TeX Live and keep Typst's
`--font-path` only for what still needs it (nothing, once no document
names a font; Typst's bundled fonts then serve the check). Euler stays
where linlog draws (SVG, the web).

Then the options. Each export gets one plain-data options value with
`Default`, `Clone`, `PartialEq` and serde behind `serialize` (the SVG
`Style` is the model; the LaTeX and Typst emitters take a `Form` only
today, which becomes a field). What a user might vary:
- the shape of an open goal: vertical dots over the sequent (today),
  the bare sequent, a marked leaf (`?`, a name), a dotted or dashed line
  where the target can draw one;
- the rule-label convention: upright `L`/`R` with subscripts (today),
  `\multimap_L`-style, no labels, or a user table;
- turnstile alignment in two-sided LaTeX trees on or off;
- whether the CLI's verdict and statistics comments are emitted in a
  source-file format, and whether the LaTeX standalone class is
  `standalone` or `article` with a preamble the user supplies;
- the curryst import (version) and the ebproof options;
- for SVG: the font (family name and its advance table, Euler Math the
  default, a monospace preset with one advance for a viewer that has no
  math font), the sizes, gaps and colours `Style` already has, a dark
  preset, and per-formula ids on or off (15g'');
- for the text renderer: the bar character and the gap between premises;
- for the certificates: `rocq::Options` exists (`lemma`, `prelude`); the
  CLI gets `--lemma` and `--prelude` through the same `--style` surface,
  and `interact` a way to certify a finished session (`show rocq` cannot,
  since `show` draws the user's partial derivation; the certificate is
  `Interactive::proof()` then `proof.derivation()`, so a `proof rocq`
  spelling or a `certify` command);
- for the interactive session: the message language of `Refusal` if the
  web front end localises.
Presets are named values of the options type (`Style::dark()`), not code
paths. The CLI maps `--style KEY=VALUE` flags or a `--style-file` (JSON,
the options' serde form) onto the options; the web front end holds the
JSON in its settings and sends it back with each request; an editor
plugin or a notebook reuses the same JSON. The step that does this
records in `.claude/rules/core.md` that a new export option is a field,
never a constant, and its report says how each front end sets each
option. Opus 5.5, xhigh (it wrote these exports in steps 10 and 11).

## 15i. Second certificate kernels

Left open by step 12 (`plan/reports/12-certificates.md`, "Open questions").
Yalla's `ill` for intuitionistic certificates: a `kernel` field of
`rocq::Options` (D15), full Yalla and OLlibs as flake inputs built at
their Rocq version (nixpkgs packages neither), the two-sided statement
from the reading (which carries step 8's `⊤`/`0` ambiguity, 15a'', into
the statement), and `Permutation_Type` witnesses for every exchange
since `ill` has no `_ext` layer; the same kernel would take Mix through
`mix2_r`. A Lean 4 target once FormalizedFormalLogic/LinearLogic has
units and a release (its multiset sequents need no exchange at all). The
exchange `ex_perm_r` makes Rocq compute `permL_of_perm`, whose cost grows
with the sequent's width; a chain of `ex_t_r` swaps if a wide sequent
turns out slow. Identifier escaping writes non-ASCII as code points where
Rocq would accept many Unicode letters. Fable 5.1, high.

## 15j. Parallel follow-ups

Left open by step 13 (`plan/reports/13-parallel.md`, "Open questions").
A per-worker proof arena with a relocation pass at the merge
(`Proof::new` renumbers already, so `(worker, index)` ids are a bounded
change) if a profile ever shows the shared arena's lock, which the
report's table does not. A `Runtime` kept across calls for a caller that
closes many small goals (a web server, an `interact` session), which
needs a runtime value in the public API next to D9's plain-data options.
The duplicated exploration of and-parallel `&` premises and of copies as
alternatives on memo-bound families, which is where the parallel focused
engine gains little (14b measures it first). A thread-sanitizer run needs
nightly and a rebuilt standard library; the code has no `unsafe` and
every shared value is behind a lock or an atomic, so it stays a wish.
The parallel tests take about a minute in debug builds; trim the samples
if the suite's time matters more than the coverage.

## 15k. Benchmark follow-ups

Left open by step 14 (`plan/reports/14-benchmarks.md`). The baseline is
step 14c in the plan's table: `bench/baseline.sh --detach --fresh` on a
free night, after 14b, by resuming step 14's session, which then commits
`bench/results/*.csv` and `bench/RESULTS.md` and replaces the report's
preliminary numbers; the script's own header and
`.claude/rules/bench.md` say what the administrator does that night.
Fourteen LLTP headers contradict their problems (KLE065, SYJ212+1.001,
SYN001, KLE013, SYN041, SYN915 in the translations the report lists) and
SYJ206+1.018 is malformed: worth reporting upstream with linlog's checked
proofs attached. The CLI does not read LLTP files, a one-flag addition
over `linlog::lltp::read`. The net engine's exact test could run less
often on large structures (a period of 8 or 16 was 1.3× faster at 14 000
occurrences). Matsuoka's 3D-Matching encoding is not among the families.
The largest SYJ files (up to 103 MB) take longer to parse than a run
gets.

## 15g'. Interactive follow-ups

Left open by step 9, none a correctness issue. The net engine works on a
proof structure over the whole forest, so an MLL goal off the roots goes
to the focused engine and `Engine::Net` forced on it is `Error::NetGoal`;
a structure over a sub-forest (the goal's subtrees as conclusions) would
let the net engine close such goals, worth it only if the front end's
profiles show `close` on wide MLL goals. The intuitionistic reading is
recomputed per operation (O(n) in the forest), since `Reading` borrows the
forest the state owns; positions stored in the state fix that if a client
with thousands of occurrences asks. `rules` lists by connective and mode
only, and `apply` says what the context lacks; a fully filtered list means
trying each rule on a clone. A Mix that sends every formula to one side
opens an empty goal that nothing closes (nullary Mix is not a rule);
refusing that split is a one-line policy decision. `close_all` runs the
goals in order under one stop closure; a per-goal budget is the client's
wrapping. Reading a state back requires the two-sided rule names that the
library writes, while `apply` also accepts the classical name in
intuitionistic mode; harmless for the library's own JSON.

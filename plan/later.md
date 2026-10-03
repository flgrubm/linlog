# Later work: candidates

Sketches, not steps: unnumbered and in no order. Step 17
(`17-assessment.md`) assesses them against the state of the repository and
the two baselines, says which are worth doing, how they depend on each
other and in which order, puts the decisions that are the author's to the
author, and then plans the steps from 18 accordingly, with their prompts.
Until then nothing here is planned. The lists of follow-ups
at the end are smaller still: each entry is folded into whichever step
touches its code.

## Code audit and refactoring

Requested by the author on 2026-09-30. Sixteen steps by separate sessions
built the code, each reviewed for its own correctness and none for the
whole: read the workspace as one maintainer would and put it in order,
changing no behaviour. The audit first, as a ranked list of findings, then
the refactorings the list justifies, each a commit of its own. Step 15
leaves it a second list to start from: the hot spots its profile showed
and it did not take (`plan/reports/15-performance.md`), which are
changes of representation with a measured share of the time behind them.

What to look for: modules grown past what a reader holds (the focused
engine's `mod.rs` is over 2 000 lines, the interactive state 1 500, the
checker and the derivation view over 1 100 each); things done twice (lock
helpers, the split and Mix enumerations, the derivation functions of the
CLI, the symbol and label tables of the exports); the public surface
(what is `pub` and need not be, names and argument orders that differ
between neighbours, `#[non_exhaustive]` and error types applied unevenly,
options types against D15); the crate-wide `#![allow(dead_code)]` and
`#![allow(unused_variables)]` in `core/src/lib.rs`, kept "while things
are scaffolded", and what they hide once removed; invariants that live
only in `.claude/rules/*.md` and could be a type, a debug assertion or a
test; tests in excess of what they pin or missing for a stated behaviour,
and the suite's running time; feature gates and what each combination
really compiles; dependencies against their use; documentation that has
drifted from the code (doc comments, the rules files, CLAUDE.md, README).

What must hold: every test and every flake check passes after every
commit; the JSON formats and the pinned snapshots do not change; on one
thread the engines' counters on the benchmark target set are identical
before and after, which is the regression oracle a refactoring of a
search engine needs, since the search is deterministic; the two baselines
stay comparable. A change of behaviour that the audit finds necessary is
reported, not slipped in. Where it belongs in the order, and what it
should settle before new engines are written on top, is for step 17 to
say.

## Configurable output, and no font in the LaTeX and Typst output

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
  preset, and per-formula ids on or off (the export follow-ups);
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

## Net-engine pruning and routing for repeated literals

After the performance pass on the focused engine (step 15), the net
engine's turn: leaf symmetry breaking for pure `⊗` and `⅋` trees of equal literals, a per-atom
balance over the `⊗`-skeleton components of a partial structure (the net
engine's analogue of the focused engine's split counts), and the sound
variant of symmetry breaking for equal compound conclusions (keys under
roots no symmetry moves; the spec's first-literal key is unsound across
groups, as step 6's report shows). Step 14's numbers
(`plan/reports/14-benchmarks.md`, "Focus against net") give the targets:
the Partition table (4.6 s and 3.7 s against 6 ms on the focused engine)
and the MLL 3-Partition at bins of five (56 s against 18 µs; six bins
stay over 60 s at every thread count, though the cubes scale 13× at
sixteen threads). They also
show that literal multiplicity is the wrong routing feature: the net
engine wins by four orders of magnitude on literals repeated three or
four times across conclusions (`wide-m3`, `wide-m4`) and loses as badly
on equal literals inside one pure `⊗` or `⅋` tree, and the reviewer's
counterexample at multiplicity four kept `NET_MULTIPLICITY` at two. So
the dispatch should route on "no two equal literals under one pure tree",
or the leaf symmetry break should remove that weakness and the threshold
rise; decide by the harness. Fable 5.1, xhigh.

Step 15 changed the premise (its report, "What is left for the net
engine and the inverse method", and the planning session's check of
2026-10-02): the focused engine now refutes the Horn encodings in
microseconds and proves `wide-m3` at 30 and `wide-m4` at 28 in 0.15 ms,
as fast as the net engine, so these are no longer cases where the suite
is slow, only where that engine is. What the net engine alone still does
is width at the default recursion limit (`wide-m1` at 2 048: a free
split costs the focused engine a level per link) and, with it, the net
itself as a result. So the candidate is now worth what nets are worth as
a search vehicle in their own right (teaching, the canonical proof
object, the cubes' near-linear speedup), not what it gains the suite in
verdicts; the second baseline's `engines` runs say whether `net` should
stay a default route at all, and a loop for chains of free splits in the
focused engine would take the last case.

## MELL proof nets with exponential boxes

Extend `nets` (step 5) with `!`-boxes and the `?` nodes (dereliction,
contraction, weakening as net nodes; or the "generalized ?" node with
auxiliary doors), correctness as the Danos–Regnier criterion applied at
each box depth with boxes contracted to single nodes (Guerrini–Masini 2001
for the parsing view), sequentialization through boxes, and the SVG drawing
with boxes as rectangles. Search stays with the focused engine; the value
is the representation (display, conversion, correctness). Fable 5.1, xhigh.

## Essential nets for IMLL

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

## The focused inverse method

The spec's second engine for MALL and the semi-decision alternative for
MELL/ILL when Θ is large: forward saturation from initial sequents in the
subformula closure with subsumption indexing (feature vectors as in Schulz
2013). A new `search::inverse` engine with its own dispatch row driven by a
heuristic (many hypotheses, small goal) or `--engine inverse`. Fable 5.1,
xhigh.

## The !-Horn fragment through Petri-net reachability

Detect the fragment, build the net, and either call an external reachability
tool (KReach) through the CLI or implement coverability for the affine case.
Only worth it with the ILLTP Petri-net problems from step 14 as the
benchmark: of the library's 3 137 Petri nets the first baseline decides
210 in 5 s; 982 time out, 171 stop at the copy bound and 1 774 at the
recursion limit, the 63-member split limit or the harness's kill (a
search that misses its stop), all of which step 15 addresses, so assess
after step 16 what is left for a reachability route. Opus 5.5, xhigh.
After step 15 the nets no longer stop at a limit but run into the time
limit (69 of its 113 sampled problems), and forward chaining by
`--bias factors` with a raised copy bound proves 48 of 109 where the
default proves 11: the focused engine under that bias is most of what a
reachability route would add for the provable nets, so what is left for
this candidate is refutation (a net whose goal is unreachable ends at
the copy bound, not at `Unprovable`) and the nets beyond the bound.

## Cyclic MLL and the Lambek calculus

A non-commutative mode: planar axiom linkings in the net engine (links may
not cross in the cyclic order of literals), no exchange in the derivation
view and exports, and the Lambek restrictions (no empty antecedent, the two
divisions). Fable 5.1, xhigh.

## Second certificate kernels: a Rocq library of linlog's own, NanoYalla kept for compatibility

Left open by step 12 (`plan/reports/12-certificates.md`, "Open questions").
The shape is the author's, of 2026-10-02: keep the NanoYalla export as it
is, for compatibility, with its refusals, and write everything new as a
library of linlog's own, idiomatic and by current practice, rather than
growing a development of 2021. (The planning session had first
recommended a fork of NanoYalla that adds kernels beside the unmodified
one; it stays below as the smaller alternative.) Today a certificate
exists for classical proofs only:
Mix and affine weakening are refused, and an intuitionistic proof is
certified as the classical proof of its one-sided sequent, not as
`Γ ⊢ A`.

The facts, read from the sources on 2026-10-02 (they correct step 12's
report in one point). NanoYalla is Click & coLLecT's `nanoyalla/`
(LGPL-2.1, unchanged since 2021): `nanoll.v`, 48 lines, the trusted
definition of cut-free one-sided LL, and `macroll.v`, the proved
positional layer (`_ext` rules, `ex_perm_r`). Yalla (olaure01/yalla,
LGPL-3.0, active, on opam as `rocq-yalla`, not in nixpkgs, needs OLlibs)
is the library of meta-theory behind it; its `microyalla/nanoll.v` is the
same definition up to binder syntax, and its `microyalla/nanoill.v`, there
since 2019, is a two-sided ILL kernel with exactly linlog's connectives
and rules, exchange by adjacent transposition, and no imports: it needs
neither full Yalla nor OLlibs, which step 12's report assumed. What it
lacks is a positional layer. Full Yalla has a general Mix rule
(`pmix`) and proves Mix-provability of `Γ` equivalent, with cut, to
provability of `?(⊥⊗⊥), Γ` (`ll_fragments.v`); no nano kernel has Mix.
Nothing in Yalla has general weakening.

The candidate has two parts.

**The NanoYalla export stays as it is** (`--format rocq`): classical
proofs against the unmodified kernel that a Click & coLLecT user has
installed, with today's trusted base and today's three refusals. It is
the compatibility target and is not grown. The one thing worth adding
there, if wanted, is an unfinished proof with its open goals as
hypotheses of the lemma, which is how Click & coLLecT's own exporter
writes one (`Goal H1 -> H2 -> conclusion`).

**A Rocq library of linlog's own**, written from nothing, in which every
mode has a statement and a certificate:
- formulas in negation normal form over a type of atoms with decidable
  equality, and the calculi as plain inductives that a reader checks
  against a textbook: one-sided LL with Mix and general weakening as
  parameters, and two-sided ILL with its affine variant over the
  connectives of the reading (the two-sided statement then carries
  step 8's `⊤`/`0` ambiguity, see the intuitionistic follow-ups below);
- the certificate as data rather than a tactic script (the planning
  session's proposal for what "idiomatic" should mean for proofs a
  program emits, which the author accepted on 2026-10-02): linlog's proof
  term (D6) as a Rocq datatype, a checker written as a function, and one
  theorem that a term the checker accepts yields a derivation. A
  certificate is then the sequent, the term and `check … = true` by
  computation: no exchange bookkeeping, no dependence on how tactics
  unify lists across Rocq versions, size and checking time linear in the
  proof, and the algorithm of the Rust checker verified once. The
  plainer alternative keeps tactic scripts with positional lemmas, as
  the NanoYalla export has, against the new inductives; it is less work
  and scales worse;
- bridge modules, optional and not needed to check a certificate, that
  import the pinned NanoYalla and Yalla's standalone `nanoill.v` and
  prove that our classical calculus and theirs derive the same sequents,
  and likewise for ILL, so that the new kernel is ours and provably the
  standard one; for Mix the theorem that a derivation of `Γ` with Mix
  gives one of `?(⊥⊗⊥), Γ` without (sketched on paper without cut: `⊥`
  on both premises, `⊗`, dereliction, two contractions; not
  machine-checked); for affine no such reduction is known, and the
  weakening rule is the definition;
- the standard library only, so that nixpkgs' Rocq builds it, and the
  conventions of the day, which the step reads from the Rocq reference
  manual and packaging documentation rather than from memory (`From
  Stdlib`, a `_RocqProject` or a dune theory, explicit locality
  attributes, `rocqdoc` comments, an opam file);
- in the exporter a second kernel behind `rocq::Options` (D15), chosen
  by the mode where the user did not choose, and in the flake a check
  that builds the library and compiles the certificates of both kernels;
- in this repository under linlog's licence, since nothing in it derives
  from LGPL text: the bridges only import the pinned kernels when they
  are checked.

What must hold: NanoYalla certificates exactly as today; no `Admitted`
and no axiom anywhere; the lemma a user reads states the sequent over
the plain inductive, with the checker only in its proof.

Size and risk: the checker's soundness over the dyadic exponentials (the
least unrestricted zone, the absorbing `⊤`), the one-succedent condition
and Mix is real proof engineering. Three to four sessions, in stages that
each leave something usable: the definitions and the checker with its
soundness for classical LL; Mix, affine and the two-sided statement; the
bridges. Fable 5.1, `xhigh` for the checker and its proof, `high` for
the rest. With tactic scripts instead of the checker, about two.

The smaller alternative, should step 17 find this too much: a fork of
NanoYalla that adds and does not edit (upstream's `nanoll.v`,
`macroll.v` and `nanoill.v` verbatim and pinned; beside them a
positional layer for `nanoill`, a Mix and an affine kernel with one more
rule each, in a namespace of our own over upstream's `formula` and `ll`,
so that Click & coLLecT's exports compile next to it unchanged). About
two sessions; it derives from LGPL files and would live in a repository
of its own, pinned as the kernel is today.

For the author to decide: the library's name, and whether it is
published on its own (opam) or only built by the flake.

Not part of it, and why: anchoring the Mix calculus to full Yalla's Mix
fragment (it needs Yalla and OLlibs built from source at a pinned Rocq
minor version, about a session, for a link the theorem above gives more
cheaply); affine logic in Yalla proper (a new parameter of its central
inductive and the meta-theory over it, the maintainer's project); a
direct certificate for nets (a Rocq development of unit-free MLL nets
with sequentialization exists, RemiDiG/proofnet_mll, and nets are
certified through sequentialization already); a Lean 4 target once
FormalizedFormalLogic/LinearLogic has units and a release.

Small things in the NanoYalla export, unchanged by this: `ex_perm_r`
makes Rocq compute `permL_of_perm`, whose cost grows with the sequent's
width (a chain of `ex_t_r` swaps if a wide sequent turns out slow), and
identifier escaping writes non-ASCII as code points where Rocq would
accept many Unicode letters.

## MALL proof nets

Only if a use case appears: Hughes–van Glabbeek nets or conflict nets are
non-canonical or exponentially large, so they are a display feature, not a
search vehicle. Assess first.

## A batch mode for the CLI

The CLI decides one sequent per call (the author's question,
2026-10-03): `prove` takes it as an argument, from `--file` or from
standard input, and `check`, `interact` and `seq` likewise take one. The
only thing that runs many is `linlog-bench run`, which is not published,
measures rather than answers, and pays a child process per run. Someone
with a file of sequents, a directory of problems or a program that asks
many questions writes a shell loop and pays the process start, the
thread sized from `--recursion-limit` and, with `--jobs` above one, a
pool per sequent, which on the small sequents that are the common case
is most of the time.

What is wanted is `prove` over many sequents in one call, in the form
practical use takes:

- **Input.** A file or standard input with one sequent per line (blank
  lines and comments skipped, an optional name per line), several
  `--file` arguments, a directory; and the formats that exist already:
  the harness's problem files (`name; mode; expected; copies; sequent`)
  and LLTP problems, which `lltp::read` reads in the core crate but the
  CLI cannot take today. Whether a line may carry its own mode and copy
  bound, as the problem files do, or the flags hold for the whole batch.
- **Output.** One result per sequent, in input order, as it is decided:
  a line of text (name, verdict, reason, time) or a JSON Lines record
  with what `--format json` and `--stats` carry now, the proof included
  on request; for the drawing formats a directory with one file per
  sequent. A malformed line is that line's error, not the batch's end.
  The exit status needs a rule for many verdicts (the worst, by the
  order error, unknown, unprovable, proved, is the obvious one). By D15
  the options are one value that the web front end and other wrappers
  use too, so the batch is a library notion (an iterator of problems to
  an iterator of results) with the CLI as its first caller.
- **Limits.** `--timeout` per sequent, and one for the whole batch.
- **Cores.** With many sequents the cores belong across them (one
  sequent per worker on the sequential engines, deterministic and
  without a pool's set-up), and within one only when the batch is short
  or a sequent is hard; which of the two a default takes is the same
  question as the CLI's default `--jobs` ("Follow-ups: parallel search")
  and should be decided with it.
- **Isolation.** In one process a sequent that exhausts memory or the
  stack takes the batch with it, which is why the harness starts a child
  per run. The search thread per sequent stays; whether a memory bound
  per sequent is needed (the memo and the arena are the growing parts,
  and the additive memo has a cap already) or `--isolate` falls back to
  children is the design's to say.
- **A stream.** Reading standard input line by line and flushing each
  answer makes the same command a server for an editor plug-in or a
  script that asks, waits and asks again, without a start per question;
  it costs nothing if the batch is built as a stream from the start.

One session. It touches `cli/` and a small entry point in the core
crate, no engine. It is checked by the batch's results being those of
the single calls on the problem file and on a sample of the LLTP
library, in both orders of cores, and by a timing that shows what the
loop paid.

## The web front end

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

## Follow-ups: the focused engine

Left open by step 15 (`plan/reports/15-performance.md`, which has the
numbers behind each). Step 15 took the canonical choice among identical
members, the forced rule for a tensor of positive literals and the hash
per branch-stack entry, and measured and dropped the restart of a level
from the frontier.

- **The default bias with exponentials**, built by the second session of
  step 15 (`Bias::Auto` runs the backward and the forward search and
  answers with the first that decides; `.claude/rules/core.md` has the
  scheme). What it leaves open:
  - *The unit of work is rough.* The two searches share one core by
    work the engine counts (a split step, a stable sequent weighted by
    its size), and the time a unit takes varies by two orders of
    magnitude between problems, so one search can get several times the
    other's time. The costs not counted are known (the lookups of duals
    in a forced chain, the sort of the copies); a unit calibrated on
    more nets, or the removal of those costs (`meets` compares every
    literal of every copy with every member of `Γ`, which is most of a
    stable sequent's cost on a net of thousands of transitions), would
    bring the measured cost nearer the scheme's 1.5 and 3 times.
  - *The forward search's own bound applies only to a Horn program*
    (clauses, a marking, a goal). A sequent with one formula of another
    shape gets the forward search within `--copies` only, and the
    price of the bound where it applies is a slower "unknown" on
    programs whose markings grow (45 of a review's 2 000 random ones
    over a second). A
    count per copy (a clause's copy as a step, any other as a copy)
    would lift that, at the price of a budget with two parts in the
    memo's `Exhausted` entries.
  - *The deepening restarts every level from the root*, which on a
    forward chain of `n` steps costs `n²/2`; with a bound of 30 that
    is nothing, with nets that need 100 steps it is what the restart
    from the frontier (below) would save.
  - *Five sampled LLTP problems that `--copies 10` decides under either
    bias stay at the default bound of 3* (translations of intuitionistic
    problems, not Horn): whether the default of `--copies` should rise
    is a question for the second baseline's `lltp-copies-10` pass.
  - *`Options::portfolio` has no use left that a measurement supports*:
    the two searches side by side are the portfolio that pays. Remove
    it if the second baseline shows no gain once more.
  - *On a pool the threads are split evenly* between the two searches;
    not measured (it needs the machine), and the split is a candidate
    for a measurement in the second baseline's all-core stage.
  - *The two searches alternate on two threads, and only where threads
    exist.* The engine recurses on its thread's stack, so a search
    cannot be suspended and resumed in place; to alternate without
    starting again, the backward search gets a thread of its own and a
    baton lets one of the two run at a time. What that costs and where
    it leaks (the author asked, 2026-10-02): a thread and its stack per
    call even under `--jobs 1` and `--deterministic` (37 µs against
    7 µs on a small sequent, which a caller that closes many small
    goals pays every time); a library user who builds with the
    `parallel` feature gets a thread started inside `prove` without
    asking for one; and a build without the feature, wasm above all,
    runs the other scheme, turns that start again on growing budgets,
    which costs up to five times the better search where the threaded
    one costs one and a half to three, and gives other counters for
    the same input, so a stop that counts work (the web front end's)
    behaves differently from the command. The fix is a search that can
    be suspended: the engine's recursion turned into an explicit stack
    (the net engine has one; for the focused engine it is a rewrite of
    its control flow, with the counters as the oracle), after which the
    two alternate on one thread everywhere, the baton and the restarts
    go, and the same input gives the same counters in every build. It
    belongs with the code audit's refactoring or with the web front
    end, whichever comes first. Short of that: an option to choose the
    scheme, so that a caller can refuse the thread.
  - *The Horn test reads the forest's roots, not the goal* (`chains`,
    found by the planning session's review): for a goal off the roots,
    as `prove_goal` and the interactive `close` hand one over, the
    forward bound follows the shape of the whole sequent. It decides a
    bound and no verdict, and the closes tried by hand behaved; the
    test belongs on the goal's members.
  - *The rows the backward search decides slowly pay most*: the
    `NeighborGrid_z_2d_3n_1m_t_1_2_*` nets take 2.2 s under `--bias
    rarer` and 3.9 s under the default (1.4 s in the first baseline),
    1.8 times, where the scheme's own figure is 1.5; under a 5 s limit
    on a loaded machine one of them was lost. They are what a better
    unit of work is measured on.
- **Mix costs `3^n` memo lookups** for `n` members that no prune
  separates (the `mix` family: 14.3 million stable sequents at eight
  pairs, eleven pairs not within 300 s), since every part enumerates its
  own partitions. A fact "no subset of this part is provable", which
  holds for a part when it fails without Mix and holds for each of its
  subsets with one member less, would make that `n·2^n`.
- **Free splits where the counts have no rows**: every atom under a `!`
  or `?` has no row, so on a Petri net with a clause body that is not a
  tensor of positive literals (the rarer-literal bias makes some body
  atoms negative) the split search cuts nothing, and such nets still
  spend their time limit at one stable sequent (69 of the 113 sampled
  LLTP problems time out). The forward bias removes those splits; a
  count for exponential atoms that is sound under copies does not exist.
- **The restart of a copy-bound level from the frontier** was not built:
  on the counter and the sampled nets the levels below the last are 24
  to 37 % of the stable sequents, so that is the most it could save;
  on `chain` and `growing`, whose bound is in the hundreds, the levels
  are quadratic in all and a restart would make them linear.
- **Constant factors the profile showed and step 15 did not take**, in
  the order of their share: hashing and comparing memo keys (29 % of the
  samples on a Petri net with 12 926 occurrences, where both zones are
  hashed in full for every stable sequent though `Θ` rarely changes;
  25 % on Mix), the allocations of a memo insert (23 % on that net: two
  boxes and a vector per key; keys in an arena), the canonical key built
  for every stable sequent (6 to 11 %; a bitset of the occurrences that
  have an earlier equal would skip it where no member is renamed), the
  member list and tally built per stable sequent (35 % on `growing`),
  `OccSet` as a `Box<[u64]>` at every size (no target over a second has
  a forest of at most 64 occurrences, so the profile does not point at
  it), link-time optimisation (1 to 6 % for twice the build time).
- **Sharing more among interchangeable sequents.** Only complete
  failures are shared. Failures cut by the copy budget, shared the same
  way, halved the stable sequents of `chain` and saved a fifth on the
  counter, and kept sequents that the search used to refute at the copy
  bound for good (a relative answered by the sequent's own entry of the
  level before); a deepening that can tell "cut" from "cut because a
  relative was cut" would get the saving back. Keying `Θ` up to
  interchangeable members would merge more as well; it is sound by the
  lemma in `.claude/rules/core.md` and costs a pass over `Θ`.
- **A free split still costs a level of recursion per link** of a chain
  of `⊗`; only forced chains and `?` rules run in a loop. Two sampled
  ILLTP-SYJ problems still end at the limit of 2 048.
- **The order in which a split search tries the members** changes which
  proof is found first, and on some generated sequents with many `⊤` the
  new order visits more stable sequents than the enumeration it
  replaced did (the report has the cases); an order informed by which
  side needs a member is untried.
- Whether a sound and useful affine prune exists (Kopylov's decidability
  argument does not give one directly; the spec's was unsound) is a
  research question to keep open; until then affine mode stays bounded.
- The interval of `&` could be the intersection instead of the hull.

## Follow-ups: intuitionistic mode

Left open by step 8, none of them a correctness issue. The written
succedent: for formulas built from `⊤` and `0` alone the reading's goal is
the last root by id, not the written one (`0, ⊤ ⊢ ⊤` prints as `0, 0 ⊢ 0`;
provability never differs, 607 464 cases brute-forced), and recovering it
means the arena keeps the parser's root order or the count of right-hand
roots, a change to `Sequent`'s canonical form and JSON; do it only if a
user of the two-sided print or the certificates asks. The additive path on
more than two roots, and on a `!` of an additive formula, is decided by
the focused engine today. The canonical choice among identical hypotheses
(step 15) applies two-sided as well.

## Follow-ups: interactive proving

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

## Follow-ups: the exports

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

## Follow-ups: parallel search

Found by the review of step 15's second session, and older than it: the
doc comment and the second assertion of `focus::parallel::tests::agree`
claim that a pool proves exactly where one thread proves. That is false
(`b, ((a * 1) -o !a), !(b -o 1), !(1 -o ((1 * 1) -o a)), b |- (!!a * b)`
with a copy bound of 2 is at its bound on one thread and proved on
four, every time); the test passes on its samples only. The contract in
`.claude/rules/core.md` is the true one (decisiveness within the bound
may differ either way); the helper should assert that and no more.

Left open by step 13 (`plan/reports/13-parallel.md`, "Open questions").
A per-worker proof arena with a relocation pass at the merge
(`Proof::new` renumbers already, so `(worker, index)` ids are a bounded
change) if a profile ever shows the shared arena's lock, which the
report's table does not. A `Runtime` kept across calls for a caller that
closes many small goals (a web server, an `interact` session), which
needs a runtime value in the public API next to D9's plain-data options.
The duplicated exploration of and-parallel `&` premises and of copies as
alternatives on memo-bound families, which is where the parallel focused
engine gains little (the baselines of steps 14 and 16 measure it). What
the first baseline adds, for step 17 to weigh with the second: on the
LLTP problems one thread does not decide at once, sixteen threads are
2.2× slower in the median and decide 32 against 28 (11 gained, 7 lost),
so the CLI's default of every core is in question for small problems (a
smaller default, or a sequential first attempt of a few milliseconds);
the portfolio gains nothing on the families nor on LLTP and is a
candidate for removal; QBF gains 1.8× from the second thread and nothing
from more; and a pool on the largest SYJ problems (8 to 15 million
occurrences) needs more than 12 GiB where one thread stays within it,
presumably the workers' per-forest state (not looked into). A
thread-sanitizer run needs
nightly and a rebuilt standard library; the code has no `unsafe` and
every shared value is behind a lock or an atomic, so it stays a wish.
The parallel tests take about a minute in debug builds; trim the samples
if the suite's time matters more than the coverage.

## Follow-ups: the benchmarks

Left open by step 14 (`plan/reports/14-benchmarks.md`), beyond the two
baselines, which are steps 14 and 16. Twenty-five LLTP files have
headers that contradict them (23 headers: KLE065, SYJ212+1.001, SYN001,
KLE013, SYN041, SYN915, and, found with a copy bound of 10 and confirmed
by classical countermodels, KLE017, KLE069, KLE078, KLE088, SYJ103,
SYJ105+1.003 and +1.004, in the translations the report lists) and
SYJ206+1.018 in its 01 translation has a tab for a closing parenthesis
(repaired in the flake's fetch): worth reporting upstream with linlog's
checked proofs and the countermodels attached. The CLI does not read
LLTP files, a one-flag addition over `linlog::lltp::read`. The net
engine's exact test could run less often on large structures (a period
of 16 was 1.39× faster at 14 000 occurrences). Matsuoka's 3D-Matching
encoding is not among the families. `summary` counts a verdict found after the time limit as
solved, which a long grace makes possible (step 16 keeps those apart in
its comparison). The largest SYJ files (up to 103 MB) load in up to 16 s
with 2 GB, past the default kill, so only the reruns of
`bench/reruns.txt` reach their search; a kill that counts from the end
of the load would make the list unnecessary for them. The proof
checker's `derive` keeps a bitset of the forest's width for every node
of the proof, 7.6 GB for the additive identity of depth 16 (262 141
nodes of 32 KB), which is what the first baseline took for the additive
memo; a `Θ` shared along a branch, or a set of ids, would fix it, and
the caps of `bench/baseline.sh` are sized for it until then. `summary`
prints times, not the counters that step 15's comparisons rest on; a
table of `nodes` and `splits` per configuration would serve the code
audit's oracle.

**Problems from practice** (the author's question during the second
session of step 15, 2026-10-02: the point of making the tool efficient
is that it can be used). The benchmark's one real-world set is the LLTP
library's 3 137 reachability problems over 76 Petri nets of the Model
Checking Contest, and they are theorems by construction: each goal is
the marking a replayed firing sequence of 1 to 150 steps ends in. What
is missing, as candidates whose sources, formats and licences are not
yet checked:

- real non-theorems on nets: coverability suites from software
  verification (the nets of concurrent C and Erlang programs that Mist,
  BFC and Petrinizer are measured on) have unreachable targets, and
  coverability is affine mode, which no set from practice exercises;
- planning domains (blocks world, logistics) as `!` Horn clauses, the
  nets' shape with goals that fail;
- program synthesis from linear types (the Granule synthesis
  benchmarks, for one): small ILL sequents with additives where the
  proof is a program;
- llprover's example collection: classical LL with additives and
  exponentials together, which the generated families barely cover.

Each enters as a problem file under `bench/problems/` or as a source in
`bench/src/problems.rs`. The nets among them are also what the forward
search's own bound and the unit of work of the default bias should be
tuned on, beyond the 109 sampled LLTP problems they were set on.

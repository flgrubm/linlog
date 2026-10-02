# Proof search: the implementation plan

The steps that take linlog from "parse, print, serialize" to a proof-search
suite: the algorithms of `proof-search-specifications.md` (the spec) for every
propositional fragment, automatic fragment detection with manual override,
intuitionistic and affine modes, proofs kept in a checkable format, exported
as LaTeX and Typst trees and as Rocq certificates, and a CLI that a human
understands. `linlog-web` is out of scope for now.

Each step is one Claude Code session started from one prompt file in this
directory. After each step the plan is reviewed and the later prompts are
amended (see "Review protocol"). `notes/` holds research the prompts rely on.
`reports/` is written by the sessions, one report per step.

## The steps

| # | Step | Prompt | Model | Effort | Depends on |
|---|---|---|---|---|---|
| 1 | Core refactor: fragments, modes, occurrence forest | `01-core-refactor.md` | Fable 5.1 | xhigh | – |
| 2 | Proof terms, the checker, derivations, serialization | `02-proofs.md` | Fable 5.1 | xhigh | 1 |
| 3 | The focused engine for MLL, MLL+units and MALL | `03-focused-engine.md` | Fable 5.1 | xhigh | 2 |
| 4 | Library API and CLI: `prove`, detection, overrides, output | `04-api-and-cli.md` | Opus 5.5 | high | 3 |
| 5 | Proof nets as a representation: structures, correctness, both conversions | `05-proof-nets.md` | Fable 5.1 | xhigh | 4 |
| 6 | Proof-net search for MLL | `06-net-search.md` | Fable 5.1 | xhigh | 5 |
| 7 | Exponentials: MELL and LL, affine mode | `07-exponentials.md` | Fable 5.1 | xhigh | 4 |
| 8 | Intuitionistic mode: ILL fragments, additive fast path | `08-intuitionistic.md` | Fable 5.1 | xhigh | 6, 7 |
| 9 | Interactive proving: partial derivations, rule application, search from a goal | `09-interactive.md` | Fable 5.1 | xhigh | 8 |
| 10 | LaTeX and Typst export of sequents and derivations | `10-latex-typst.md` | Opus 5.5 | xhigh | 9 |
| 11 | SVG export of sequents, derivations and proof nets | `11-svg.md` | Opus 5.5 | xhigh | 9 |
| 12 | Rocq certificates | `12-certificates.md` | Fable 5.1 | high | 10 |
| 13 | Parallel search | `13-parallel.md` | Fable 5.1 | xhigh | 9 |
| 14 | Benchmarks, LLTP input, hard families, and the baseline (taken on the night of 2026-09-30, completed by a supplement on the night of 2026-10-01) | `14-benchmarks.md` | Opus 5.5 | xhigh | 13 |
| 15 | Performance pass on the focused engine, driven by 14's baseline, measured by day through the engines' counters | `15-performance.md` | Fable 5.1 | xhigh | 14 |
| 16 | The baseline again, after the pass, and the comparison of the two | `16-baseline.md` | Opus 5.5 | xhigh | 15 |
| 17 | Assessment and planning: the state of the repository, the two baselines, every candidate in `later.md`; the author's decisions; then the prompts for the steps from 18 | `17-assessment.md` | Fable 5.1 | xhigh | 16 |
| 18– | Planned by step 17 from the candidates in `later.md` (configurable output, a code audit and refactoring, net-engine pruning and routing, MELL nets with boxes, essential nets, the inverse method, Petri nets, Lambek, second certificate kernels, MALL nets, the web front end) | written by step 17 | – | – | 17 |

A step has a whole number, one prompt file named after it, one report
under `reports/` with the same name, and as many sessions as it takes to
finish (step 14 took two). Work that is not yet planned is an unnumbered
candidate in `later.md`, which step 17 assesses and turns into steps;
small follow-ups are lists there, by area, and are folded into the step
that touches their code. Until 2026-09-30 some
labels carried letters and primes; the Status log and the reports keep
the labels they were written with, which map as follows: 14b is step 15
and 14c step 16; 15a to 15i are the candidates of `later.md` by name
(15a MELL nets with boxes, 15b essential nets, 15b' net-engine pruning,
15c the inverse method, 15d Petri nets, 15e Lambek, 15f MALL nets, 15g
the web front end, 15h configurable output, 15i second certificate
kernels); and 15a', 15a'', 15g', 15g'', 15j and 15k are its follow-up
lists (the focused engine, intuitionistic mode, interactive proving, the
exports, parallel search, the benchmarks).

Steps 5–6 and 7 are independent of each other; 10, 11 and 13 are
independent of each other. Everything else is in order.

### Why these models and efforts

Checked against the model docs on 2026-09-29
(platform.claude.com/docs/en/about-claude/models/overview and
…/choosing-a-model). The current lineup: Claude Fable 5.1 (`claude-fable-5-1`,
$10/$50 per MTok, "for demanding reasoning and long-horizon agentic work",
default effort `high`), Claude Opus 5.5 (`claude-opus-5-5`, $4/$20, "for
long-running agentic coding", recommended starting point for most workloads,
default effort `medium`), Claude Sonnet 5.5 (`claude-sonnet-5-5`, $2/$10) and
Haiku 4.5. Effort levels are `low`, `medium`, `high`, `xhigh`, `max`; the docs
say `xhigh` is the best setting for most coding and agentic work, and to move
from Opus 5.5 to Fable 5.1 when a task's demands on reasoning or horizon exceed
what Opus at `xhigh`/`max` delivers.

- **Fable 5.1 at `xhigh`** for every step whose correctness is subtle and
  whose mistakes propagate: the data model (1), the checker that anchors
  soundness (2), proof nets and their criterion (5), each search engine
  (3, 6, 7, 8), the interactive state with its translation back to terms
  (9) and the parallel runtime (13). `max` is the knob to turn if a
  step's review finds reasoning gaps; it was not chosen up front because it
  costs more on every turn.
- **Fable 5.1 at `high`** for the certificate step (12): a long,
  research-heavy session (Rocq, NanoYalla, nix) rather than a deep
  algorithmic one.
- **Opus 5.5** for plumbing and user-facing work (4, 10, 11, 14): clap,
  output formats, emitters, SVG layout and a benchmark harness. Step 4 ran
  at `high`; 10, 11 and 14 run at `xhigh` (see the re-evaluation below).
  Sonnet 5.5 at `xhigh` is the cheaper alternative for 10, 11 and 14 if
  cost matters more than a first-pass finish.

Re-evaluated on 2026-09-29 after step 8, against the same docs (the lineup
and prices are unchanged). What eight steps showed: every Fable 5.1
`xhigh` step delivered its engine or representation with a fresh-context
reviewer and differential tests in the tens of thousands, and the reviews
found nothing that would call for `max`; the one failure was step 7's
first attempt exceeding the output limit, which the settings and
`conduct.md` now prevent; step 4 on Opus 5.5 at `high` delivered the
plumbing as asked. The remaining Fable steps keep their settings: 9's
translation from the standard calculus back to terms and its rule
validation are checker-grade, 13's shared memo under concurrency and step 15's
prunes are soundness work, and 12 stays at `high` because its difficulty
is research and packaging, which Rocq itself verifies. The Opus steps move
from `high` to `xhigh`: the docs name `xhigh` the best setting for coding
and agentic work, the price difference on Opus is small next to a Fable
turn, and 10 and 11 have exactly the kind of detail (package syntax
checked against manuals, XML escaping, arc layout) where more effort on a
cheaper model pays. Nothing moves to Sonnet: the savings are minor against
the cost of a step that has to be redone.

### How the prompts are written

Following Anthropic's prompting guidance for Fable 5.1 and Opus 5.5 (the
`claude-api` skill's migration guide, read 2026-09-29): both models do
better with the goal, the constraints and the reason than with enumerated
steps, so each prompt states what the step must achieve and points at the
spec sections and reports to read, and its numbered items are requirements,
not an order of work. The behaviours the guidance singles out for these
models (over-planning at high effort, unrequested tidying and abstraction,
test sprawl, ungrounded progress claims, whole-file rewrites, terse final
summaries) are addressed once, in `conduct.md`, which the command below
appends to every prompt: placement at the end of the first user message is
where the guidance found such instructions most effective. `conduct.md`
also asks for tests only where a behaviour needs pinning, and for sub-agents
(the repository's `crate-source-explorer`, a fresh-context reviewer for
soundness-critical code), which the guidance says Fable 5.1 uses well.

Re-evaluated on 2026-09-30 after steps 9 to 14, against the same docs
(lineup, prices and default efforts unchanged: Fable 5.1 $10/$50 default
`high`, Opus 5.5 $4/$20 default `medium`, Sonnet 5.5 $2/$10 default
`high`, Haiku 4.5; the docs still say to start with Opus 5.5 and move to
Fable 5.1 when `xhigh` or `max` falls short on demanding reasoning). What
the six steps showed: Fable 5.1 delivered 9 and 13 at `xhigh` and 12 at
`high`, and in each the defects that mattered were found by its
fresh-context reviewer's differential fuzzing, not missed by reasoning
that more effort would have supplied, so `max` stays unused; Opus 5.5 at
`xhigh` delivered 10, 11 and 14 with clean first passes on emitters,
layout and the harness, and in 14 found the engine's missing split poll
and fourteen wrong LLTP headers on its own. The one failure, the machine
running out of memory under a reviewer's scratch program, was a matter of
process, now in `conduct.md`, not of model. The choices from here:

- **Step 14, run again, is Opus 5.5 at `xhigh`** in a fresh session with
  the rewritten prompt: the harness is its own work, and what remains is
  checking the machine, a small change to the script, a night's run and
  its tables.
- **Step 15 (the performance pass) stays Fable 5.1 at `xhigh`**: four
  soundness-critical changes to the focused engine (the split search, two
  canonical choices, the atom bias), each needing an argument and a
  differential review.
- **Step 16 (the second baseline and the comparison) is Opus 5.5 at
  `xhigh`**, as step 14 is.
- **Step 17 (the assessment and the planning) is Fable 5.1 at `xhigh`**:
  reading the whole repository and two baselines, checking the outside
  world's state for every candidate and carrying the analysis through to
  prompts is the deep research and long horizon the docs name Fable for.
- **The candidates of `later.md`** get their models from step 17, which
  checks the docs on its day. The planning session's view, for it to
  weigh: new engines, criteria and prunes (net-engine pruning, MELL nets
  with boxes, essential nets, the inverse method, Lambek) are Fable 5.1
  at `xhigh`; a second certificate kernel is Fable 5.1 at `high`, as step
  12 was; configurable output, the Petri-net route, the web front end's
  bindings and interface, and the code audit with its refactoring are
  Opus 5.5 at `xhigh` (breadth over the code Opus wrote or the large
  refactoring the docs name it for), the audit's engine parts under the
  counter oracle.
- **The follow-up lists of `later.md`** are not sessions of their own:
  each entry is folded into the step that touches its code. Run alone,
  the purely mechanical ones (a flag, a label table, ids per formula) are
  Sonnet 5.5 at `high`.

### The commands (nushell)

Run from the repository root, one at a time, in order. `open --raw` reads a
file as one string; the step's prompt comes first and `conduct.md` is
appended; `--name` labels the session in `claude --resume`.

```nu
claude --model claude-fable-5-1 --effort xhigh --name step-01 ((open --raw plan/01-core-refactor.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-02 ((open --raw plan/02-proofs.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-03 ((open --raw plan/03-focused-engine.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-opus-5-5 --effort high --name step-04 ((open --raw plan/04-api-and-cli.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-05 ((open --raw plan/05-proof-nets.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-06 ((open --raw plan/06-net-search.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-07 ((open --raw plan/07-exponentials.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-08 ((open --raw plan/08-intuitionistic.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-09 ((open --raw plan/09-interactive.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-opus-5-5 --effort xhigh --name step-10 ((open --raw plan/10-latex-typst.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-opus-5-5 --effort xhigh --name step-11 ((open --raw plan/11-svg.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort high --name step-12 ((open --raw plan/12-certificates.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-13 ((open --raw plan/13-parallel.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-opus-5-5 --effort xhigh --name step-14 ((open --raw plan/14-benchmarks.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-15 ((open --raw plan/15-performance.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-opus-5-5 --effort xhigh --name step-16 ((open --raw plan/16-baseline.md) + "\n" + (open --raw plan/conduct.md))
claude --model claude-fable-5-1 --effort xhigh --name step-17 ((open --raw plan/17-assessment.md) + "\n" + (open --raw plan/conduct.md))
```

The aliases `fable` and `opus` also work for `--model`. The flags are
documented at code.claude.com/docs/en/cli-reference.

## Review protocol

1. A step's session commits its work thematically with jj and ends by writing
   `plan/reports/NN-<name>.md` (what was done, decisions, deviations from the
   spec or this plan, open questions, what later steps must know). It does
   not push and does not edit the other prompt files.
2. The planning session (the one that wrote this file; `claude --resume` and
   pick it, or start a fresh one from `plan/handoff.md`, which says what
   that session does and what it must know)
   reads the report and the changes (`jj log`, `jj diff -r`), runs the checks,
   judges whether the result fits the plan, and amends the later prompts and
   the decisions below. It records the outcome in "Status" and commits the plan
   changes as "Plan: review step NN".
3. Only then the next command is run.

## Design decisions

These bind every step. A step that finds one of them wrong says so in its
report and, if the fix is local, makes it; otherwise it stops and asks.

**D1. One formula representation.** Formulas stay in the hash-consed arena
of `Sequent` (`core/src/sequents/`), in negation normal form, one-sided:
two-sided input `Γ ⊢ Δ` is lowered to `⊢ Γ^⊥, Δ`. Intuitionistic sequents
need no second data model: an ILL formula in NNF has a *polarized shape*
(output position: `⊗ ⊕ & ! 1 ⊤ 0`, atoms `Var`, and `A^⊥ ⅋ B` for `A ⊸ B`;
input position: the duals, `DualVar` atoms, `⊥ ⊤ 0`, and `A ⊗ B^⊥` for a
hypothesis `A ⊸ B`), which is read back losslessly. An ILL sequent is a
one-sided sequent with exactly one output-shaped root and only input-shaped
others. This is Lamarche's polarization and what the spec's two-sided engine
computes its counts on. Two-sided printing and export recover `⊸` from it.

**D2. Fragments and modes are runtime values.** Automatic detection needs a
value, not a type. `Fragment` records which connective classes an input uses
(multiplicatives, their units, additives, their units, exponentials) and
`Mode` records what the user asked for: classical or intuitionistic, linear
or affine, with or without Mix. The `Logic` type parameter, the marker types
`LL`/`MLL` and the `subenum` dependency go away; fragment-typed sequents have
no consumer once engines run on occurrence forests.

**D3. One index type.** Arena and occurrence indices are `u32`. The `Index`
trait and the type parameter `I` on `Sequent` are removed: their flexibility
has no user, they make every signature and every `impl` harder to read, and
the spec's engines use `u32` throughout. Newtypes (`TermId`, `OccId`, `Atom`)
keep the indices apart at the type level. Narrow indices were considered for
cache footprint and rejected: a forest small enough for `u8` or `u16` fits in
L1 at `u32` anyway, the memory that matters in search is bitsets and memo
keys, which do not depend on index width, and the spec's `u64` bitset
specialisation for `n ≤ 64` is the optimisation that pays. Two hedges: the
occurrence index stays behind its single newtype so narrowing the forest
later is a local change, and step 14's benchmarks decide with numbers
whether to revisit.

**D4. Module layout inside the `linlog` crate**, not eight crates as the
spec proposes: the workspace stays `core/` and `cli/` (a `bench/` crate may
come in step 14). Suggested modules, adjusted by the steps as they see fit:
`sequents` (arena, printing), `parse`, `serialize`, `fragment` (D2 and
detection), `occurrences` (the forest of D5), `proofs` (terms, checker,
derivations, rendering), `nets` (proof structures, the criterion, the two
conversions), `search` (`prove`, options, outcome, dispatch, and one
submodule per engine: `focus`, `net`, `additive`), `export` (`latex`,
`typst`, `svg`, `rocq`). `core/src/linear/` (empty placeholders) is
replaced.

**D5. Occurrence forest per problem.** Every engine works on the spec's
occurrence forest (section "Common infrastructure"): subformula occurrences
numbered in DFS preorder so a subtree is a contiguous range, with per
occurrence kind, parent, children, size, polarity, atom and sign, plus per
atom the lists of positive and negative literal occurrences. Sequents inside
a search are sets (bitsets) or count vectors over occurrence ids. The forest
is built deterministically from a `Sequent` (root order = `term_ids` order),
so occurrence ids are stable across runs and serializations.

**D6. Proofs are terms over occurrence ids, checked independently.** The
canonical proof is the spec's term (`Ax`, `Tensor`, `Par`, `With`, `Plus`,
`One`, `Bot`, `Top`, `Bang`, `Quest`, `Copy`, plus what weakening, Mix and
the intuitionistic rules need), stored compactly. A checker that shares no
code with any engine re-derives the sequent of every node and rejects
anything else; every engine's output passes it in tests. From a term and its
forest a `Derivation` view is produced: the tree of explicit sequents and
rule names of the standard (unfocused, non-dyadic) sequent calculus, one-sided
for classical and two-sided for intuitionistic mode, with structural rules
explicit. Rendering, LaTeX, Typst and Rocq export consume the view.

**D6a. Proof nets are a second first-class representation.** A `ProofNet`
is a proof structure over the occurrence forest (the formula trees plus
axiom links, and Mix where enabled), with a correctness checker that is
independent of any search engine (the Danos–Regnier criterion, implemented
through the Yeo-theorem test the spec describes, with the connectedness
equation unless Mix is on), sequentialization (net → derivation, by the
splitting-tensor lemma) and desequentialization (derivation → net, by
reading the axiom links off the term). The checker's verdict together with a
successful sequentialization is the net's correctness certificate: the
resulting term passes the proof checker of D6. Nets exist for MLL with or
without Mix, where they are canonical; MELL nets with boxes come later
(step 15) and MALL nets are out of scope (non-canonical, exponentially
large). Search engines use whichever representation suits them; the
conversions make every proof available in both.

**D7. One engine per algorithm, not per fragment.** The spec's MLL-Seq,
MALL-Seq, MELL-Seq and the LL engine are one focused engine with rule sets
switched on by fragment and mode (`search::focus`). The two-sided
intuitionistic engine is the same skeleton with a goal side (step 7 decides
whether it is a parameter or a sibling module). Proof-net search is
`search::net`. The additive-only fast path is `search::additive`.

**D8. Dispatch table** (auto-detection picks the row; `--engine` forces one,
`--fragment` asserts one):

| Mode | Detected fragment | Engine |
|---|---|---|
| classical | additives only, two roots | additive (step 8): a memoized recursion on subformula pairs |
| classical | MLL without units, with or without Mix, no literal more than twice | net search (step 6) |
| classical | MLL without units, some literal three or more times | focus (step 3); threshold tuned in step 14 |
| classical | MLL with units, MALL | focus (step 3) |
| classical | MELL, LL | focus with exponentials and copy bound (step 7) |
| affine | anything | focus with weakening at the leaves, bounded like linear mode (step 7); the spec's supermultiset prune is unsound, so affine mode is not a decision procedure |
| intuitionistic | additives only, two roots | additive (step 8): the same recursion, in every mode |
| intuitionistic | IMLL without `1`, no literal more than twice | embedding into net search (step 8): every sequentialization of the classical net is intuitionistic; essential nets later (step 15) |
| intuitionistic | IMLL with `1` or repeated literals, IMALL, IMELL, ILL, affine variants | two-sided focus (step 8): the focused engine given the reading, keeping the goal on the consequent's side of every `⊸L` split |

**D9. Outcomes are three-valued.** `Proved(proof)`, `Unprovable` (only when
the search was exhaustive) and `Unknown` (bound or time limit hit, with the
reason). Search takes options (copy bound, time limit, memo cap, thread
count, determinism) and returns statistics (nodes, memo size, time).

**D10. Dependencies.** Adopt through the `new-tool` skill, scoped to the
crate and feature that uses them. Candidates checked on 2026-09-29 (all
licenses allowed by `deny.toml`; see `notes/export-targets.md` § Crates):
`fixedbitset` (bitsets), `foldhash` or `rustc-hash` (hashing; `ahash` needs
`getrandom` care on wasm), `smallvec`, `ena` (union-find with snapshots and
rollback), `rayon` and `dashmap` behind a `parallel` feature (off on wasm),
`postcard` if a binary serialization format is wanted next to JSON. No
`unsafe`.

**D11. Wasm stays possible.** Nothing in `core` may need threads or the OS
unconditionally: parallelism and timing live behind features or in the CLI.

**D12. Graphics are generated, not laid out by a general graph tool.** SVG
comes from deterministic layouts that the objects themselves suggest: a
derivation is a tree laid out bottom-up by subtree width (as ebproof does),
a proof net is its formula trees drawn downwards from the conclusions with
the axiom links as arcs above the literals, a sequent is a line of text.
Where linlog itself draws (SVG, the web front end), the font is Euler
math (the author's choice, 2026-09-29: the Euler Math OpenType font), so
widths cannot come from a fixed advance: the SVG layout uses a table of
per-character advances of Euler Math measured once and committed, a fixed
fallback advance for a character outside it, and `textLength` on every
text run so that a viewer without the font still fits the layout. Where a
document system sets the text (LaTeX, Typst), linlog's output never
chooses a font, not even in a standalone document: the user pastes the
output into a document and wants it to look like the rest of that
document (the author, 2026-09-29; step 11 had set `eulervm` and Euler
Math there, which the configurable-output candidate removes). No graphviz, no browser-side layout. The same layouts feed the LaTeX and Typst exports where
a package needs coordinates (it does not for ebproof and curryst trees).

**D13. Interactive proving is a partial derivation.** The state a client
holds for step-by-step proving is a derivation of the standard calculus
over the same occurrence forest, with open goals as leaves: the same
sequent representation and rule names as the derivation view, applied by
naming a goal, a formula position and a rule (with the choices some rules
need, such as the split of a `⊗`). Search runs from any goal, not only the
roots, and its result is grafted as a derivation. A completed state
translates back into a proof term that the checker of D6 validates, so the
interactive layer is trusted no more than an engine. The state has a
stable JSON form, which is what a web client keeps between requests.

**D14. Optional features.** Next to `parse` and `serialize`, the `linlog`
crate gates the optional layers behind cargo features so that a client
takes only what it ships: `interactive` (D13), `latex`, `typst`, `svg`,
`rocq` and `parallel`. All are default features except `parallel`, which
the CLI enables. Search, the checker, the derivation view and proof nets
are unconditional: they are the crate. Once there are more than four
features, the `features` check and the documented command become
`cargo hack check --each-feature -p linlog` plus
`cargo hack check --feature-powerset --depth 2 -p linlog` rather than the
full powerset; the step that adds the fifth feature makes that change in
`modules/checks.nix` and CLAUDE.md.

**D15. Whatever a user might want to vary in an output is configured
through the library, for every front end.** A library feature whose output
people see (text, LaTeX, Typst, SVG, certificates, the interactive
session's messages) takes one plain-data options value with `Default`,
`Clone` and serde behind `serialize`, and never hides a choice in a
constant: the shape of an open goal, a rule-label convention, colours,
sizes, the SVG font (with its advance table), whether a verdict comment is
emitted, a lemma's name, and so on. The options are designed for the
wrappers at once: the CLI maps flags or a config file onto them, the web
front end holds them as JSON in its settings and sends them back, and a
third wrapper (an editor plugin, a notebook) gets the same surface without
new library code. Presets are named values of the options type, not
alternative code paths. A step that adds an output feature says in its
report which options it exposes and how each front end would set them; a
step that finds a hidden constant a user might want to change turns it
into an option or names it as a follow-up.

## Status

- 2026-09-29: plan written; spec committed as "Add the proof search
  specification". D2 and D3 (runtime fragments, fixed `u32` indices)
  confirmed by the author after discussion.
- 2026-09-29: step 1 reviewed and accepted. Seven commits, from "Drop the
  Logic and Index type parameters" to "Document the new core model"; all
  checks pass including `nix flake check`. The report
  (`reports/01-core-refactor.md`) is the reference for the type names
  (`Sequent`, `Term`, `TermId`, `Atom`, `Kind`, `Fragment`, `Mode`,
  `Forest`, `OccId`, `OccSet`, `Sign`, `Polarity`). Decisions taken there
  and accepted: a hand-written `OccSet` over `Box<[u64]>` instead of
  fixedbitset; foldhash with a fixed seed; the forest owns a clone of its
  sequent; children derived from the preorder, not stored; `submasks` for
  up to 63 members. Open questions the report raised are assigned: proof
  ownership of the forest (step 2), serde for `Fragment`/`Mode` (step 4),
  intuitionistic fragment names (step 8), a public `Sequent` builder (when
  a step needs it). Prompts 2, 3, 4 and 8 amended accordingly, and
  `conduct.md` added to every command.
- 2026-09-29: step 2 reviewed and accepted. Five commits, "Add proof
  terms" to "Document the proof model"; all checks pass including
  `nix flake check`. `Proof` owns its `Forest`; `Node` is a 16-byte enum of
  the dyadic rules plus `Weaken` and `Mix`; the checker runs bottom-up with
  the least unrestricted zone and an absorbing-`⊤` flag, and was
  differentially tested against an independent top-down checker by a
  fresh-context reviewer (no disagreement on 1255 proofs, 75 300 mutants
  and 600 000 random terms); the derivation view inserts `?d`/`?c`/`?w`
  only where needed; JSON tags are frozen. Accepted deviations: multisets
  as sorted vectors, no ILL rule tags (every ILL rule is a classical node
  on the lowered sequent), intuitionistic checking refused until step 8
  supplies the side reading, no postcard. Open items assigned: the ILL
  one-succedent check and two-sided view (8), `CheckError` with formulas
  and the `check` command's mode (4), affine `Weaken` placement and the
  dyadic term shapes (7), axiom links from `Node::Ax` (5). Nullary Mix and
  a canonical node order stay open until something needs them.
- 2026-09-29: step 3 reviewed and accepted. Seven commits, "Add interval
  counts per occurrence" to "Document the focused engine"; all checks pass
  including `nix flake check`. The engine is the spec's MALL-Seq with
  units and Mix as switches, a memo of stable sequents (cleared when
  full), a counted recursion depth with `Reason::RecursionLimit`,
  `Reason::ContextTooWide` above 63 members, and pools that keep the hot
  path allocation-free; `prove`/`prove_until`, `Options`, `Outcome`,
  `Verdict`, `Reason`, `Statistics`, `Engine` are the front door. A
  fresh-context reviewer compared it with an independent unfocused prover
  on 1.43 million sequents with no disagreement. Two corrections to the
  spec are recorded in the report and in `.claude/rules/core.md`: a `0` in
  a stable sequent is fatal only when no member has a `⊤` below it, and
  the literal-only failure holds only without Mix; also `⊥` and `⊤` factors
  do not force a split. Performance follow-ups (branch-and-bound splits,
  atom bias, tighter counts, memo key arena, the `3^k` cost of Mix) become
  step 13b, driven by step 13's numbers. Prompts 4, 6, 7, 8, 12 and 13
  amended.
- 2026-09-29: step 4 reviewed and accepted. Ten commits, "Give fragments,
  modes and search outcomes a JSON form" to "Refuse to check
  intuitionistic proofs instead of calling them invalid"; all checks pass
  including `nix flake check`; the CLI was exercised by hand. The binary
  is `linlog` with `prove`, `check` and `seq print|json|fragment`; exit
  statuses 0/1/2/3 for provable/unprovable/error/unknown; `--format json`
  writes an outcome that `check` reads back; `ctrlc` is the one new
  dependency; the search always runs on a spawned thread sized from
  `--recursion-limit`. Extension points for engines and formats are in
  the report and now cited by prompts 5, 7, 8, 9, 10 and 11. The README
  was rewritten to describe what exists ("What exists and what is
  planned") and to link the API documentation; `conduct.md` now asks
  every step to keep it current. The repository's description and
  homepage on GitHub point at the Pages site. Follow-ups left open: the
  text renderer is slow on huge derivations (the export steps emit per
  inference instead), `check` could fall back to the file's mode.
- 2026-09-29: step 5 reviewed and accepted. Seven commits, "Add proof
  structures over the occurrence forest" to "Document proof nets"; all
  checks pass including `nix flake check`; `--format net` tried by hand.
  `ProofStructure` with O(1) `link`/`unlink`, the Danos–Regnier criterion
  through Yeo's deletion test reduced to bridges (colours implicit),
  witnesses for cycles and disconnections, sequentialization by the
  splitting-tensor lemma, `from_proof`, text and JSON forms. A
  fresh-context reviewer compared it with switching enumeration and Danos
  contractibility on 413 745 structures with no disagreement. Accepted
  decisions: no `ena` (a sixty-line union-find with an undo log), no stored
  colours, `Scratch` explicit for the hot loop, links as a stack. The spec's
  sequentialization test was wrong (deleting a `⊗` and counting components
  under a switching cannot distinguish); the correction, with step 3's,
  is now an "Errata" section at the top of the spec. Follow-ups left open:
  `sequentialize` recurses (step 6 runs it on the search thread), the
  witness isolation is quadratic on the error path, Guerrini's linear
  criterion and sequentialization if profiles ask.
- 2026-09-29: plan restructured on the author's request. Interactive
  proving (D13) is new step 9 and the later steps moved up by one (10
  LaTeX/Typst, 11 SVG, 12 certificates, 13 parallel, 14 benchmarks, 14b
  performance, 15 later); optional cargo features (D14) gate the
  interactive state and the exporters; prompts 7, 8, 10, 11, 12 and 15
  amended for both. The CLI's code became the library `linlog_cli` with a
  one-line binary so that rustdoc lists both crates in one tree.
- 2026-09-29: step 6 reviewed and accepted. Four commits, "Search axiom
  linkings with incremental pruning" to "Document the net engine"; all
  checks pass including `nix flake check`; the engine was exercised by
  hand. Minimum-remaining-values choice with forward checking, both O(1)
  rejections, the exact test at the spec's cadence (`Options::test_period`),
  symmetry breaking for equal literal conclusions (the spec's key for
  compound copies is unsound across groups and was not implemented), an
  explicit stack, `Engine::Net`, `Outcome::net`, per-engine statistics. A
  fresh-context reviewer compared it with brute-force enumeration of all
  linkings on about 54 000 cases with no disagreement. The timings decide
  a routing question the plan had left to D8: the net engine is linear on
  distinct atoms and wide contexts (13 999 occurrences in half a second
  where the focused engine gives up) and loses by orders of magnitude on
  Horn encodings with repeated literals. The planning session changed the
  dispatch ("Route MLL with repeated literals to the focused engine"):
  unit-free MLL goes to `net` only when no literal occurs more than twice;
  D8 updated, step 14 tunes the threshold and measures the follow-ups
  (leaf symmetry breaking, a per-atom balance over skeleton components, a
  portfolio), now listed as 15b' for the performance pass. Prompts 8, 13
  and 14 amended.
- 2026-09-29: step 7 reviewed and accepted, after a first attempt whose
  reply exceeded the output token limit (the limit is raised to 128 000
  in `.claude/settings.json`, and `conduct.md` warns against composing a
  module in one reply). Thirteen commits, "Add exponentials and affine
  mode to the focused engine" to "Keep the large generated sample to the
  fragments without exponentials"; all checks pass including
  `nix flake check`; the engine was exercised by hand. Dyadic sequents
  with `Θ` as a set and `Γ` as a bitset plus a sorted list of extra
  copies, a per-branch copy budget deepened from 0 to `Options::copies`
  (default 3), memo entries `Proved`/`Failed(Complete)`/
  `Failed(Exhausted(r))` merged so that validity only grows, an
  ancestor-repeat loop check whose failures are not memoized, both
  initial rules, `Reason::CopyBound`, `--copies`. A fresh-context
  reviewer ran an independent unfocused dyadic prover on 5 200 random
  sequents at every bound with no disagreement in linear mode, and
  showed the spec's affine supermultiset-ancestor prune unsound (426
  wrong refutations; `⊢ ?(a ⅋ ~a)` is provable only through a stable
  sequent containing its ancestor). The prune is gone, affine mode is the
  bounded search with weakening, D8 and the spec's errata say so; whether
  a sound and useful affine prune exists stays open (15a'). Further spec
  corrections recorded by the session: a `⊤` below a `?` or `!` disables
  the interval check, a `0` is not fatal when a `Θ` member absorbs, the
  literal-only failure is wrong with a non-empty `Θ`. Follow-ups for
  the performance pass (identical members, nested forced factors,
  per-level restarts, a hashed loop check) are in 14 and 15a'. Prompts 8,
  9, 13 and 14 amended.
- 2026-09-29: step 8 reviewed and accepted. Nine commits, "Recognise
  intuitionistic sequents by shape" to "Add ILLTP-style problems as a slow
  test"; all checks pass including `nix flake check`; intuitionistic mode
  was exercised by hand on the CLI. D1 held: `Reading` reads the one-sided
  arena two-sided by Lamarche's polarization (`Position`, the goal, `⊸`
  recovered, `Γ ⊢ A` printing, `ShapeError` otherwise), the checker adds
  the one-succedent condition as three bottom-up rules, the two-sided
  engine is the focused engine given the reading (one constraint, in the
  `⊸L` split), unit-free IMLL goes to the net engine by the embedding
  (proved sound in the report), and the additive fast path decides two
  additive-only formulas in every mode. A fresh-context reviewer compared
  everything with an independent two-sided prover on about 60 000
  sequents and caught one bug (the additive path's `⊕` choice, fixed). The
  planning session's decisions: the reading's tie-break among `⊤`/`0`-only
  roots (the written succedent is lost; the verdict never differs) is
  parked as 15a'' rather than changed, since fixing it touches the arena's
  canonical form and JSON; D8 rewritten with the three intuitionistic
  rows; the model and effort choices re-evaluated ("Why these models and
  efforts": the Opus steps 10, 11 and 14 move to `xhigh`, nothing else
  changes). Prompts 9 to 15 amended with what the reading, the two-sided
  view and the goal search now offer.
- 2026-09-29: step 9 reviewed and accepted. Seven commits, "Search from a
  goal instead of the roots" to "Document interactive proving"; all checks
  pass including `nix flake check`; `linlog interact` was exercised by hand
  in every mode (root `?w`, `?c` under a `⊗` with `!` above, affine `wk`,
  Mix, `⊤R` and `0L`, the additive path off the roots, `!c` with `!R`
  two-sided, the error paths). D13 held as written: `Interactive` is a
  top-down arena of the view's `Inference`s with `Rule::Open` leaves,
  formulas addressed by position, validation at application time complete
  for the checker (R1 and R3 two-sided, R2 implied), `undo` by truncation,
  search from any goal through the new front door `prove_goal` (the net
  engine stays with the roots), the finished tree translated back to a
  dyadic term with `Quest` at each `?` formula's entry and checked, JSON
  with a replayed history; D14's `interactive` feature is on by default and
  the CLI enables it. A fresh-context reviewer drove about 106 000 random
  rule applications and 2 500 completed derivations against its own rules
  and caught a Mix bug and several over-rejections on reading a state
  back, all fixed and pinned. The planning session fixed two CLI nits
  itself (a refused `close` printed its message twice through the error
  chain; `--state` ignored the mode flags instead of refusing them), one
  commit. Prompts 10 to 13 and 15 amended: `Rule::Open` leaves for the
  exporters (one shape, a partial derivation in each snapshot set), the
  certificates refuse open goals and export the term's derivation, the
  parallel layer serves `prove_goal`, the web front end's plan starts from
  the report's list of calls, and the interactive follow-ups (net engine
  on a sub-forest, stored positions, nullary Mix) are 15g'.
- 2026-09-29: step 10 reviewed and accepted. Eight commits, "Print
  formulas and sequents for LaTeX and Typst" to "Document the LaTeX and
  Typst exports"; all checks pass including `nix flake check` with the
  new `export` check (pdfLaTeX and Typst compile every snapshot and two
  CLI outputs, offline); the formats were exercised by hand on `prove`,
  `check`, `seq print` and `interact show`, and two Typst snapshots were
  rendered to PNG and looked right. D6 and D14 held: one symbol table per
  target over one printer with `Display`'s bracketing, an explicit-stack
  walk so the emitters never recurse over the tree, `Form` for fragment
  versus standalone, the `latex` and `typst` features, and the feature
  check switched to `--each-feature` plus `--feature-powerset --depth 2`
  as D14 asked at the fifth feature. Decisions accepted: an open goal is
  its sequent under vertical dots with no bar in both targets (neither
  package draws a dotted bar per inference), Typst connectives as Unicode
  characters since Typst 0.15 renamed `times.circle`, the TeX Live
  closure (435 MiB, from the cache) kept for the only proof that the LaTeX
  compiles. Known limit: curryst refuses trees above about eleven
  inferences, recorded as 15g''. The author chose Euler math for every
  drawing of sequents and derivations (D12 amended): step 11 lays SVG out
  from a committed advance table of Euler Math with `textLength` as the
  safety net, and switches the step 10 standalone documents to `eulervm`
  and the Euler Math font (item 1a). Prompts 11, 12, 13 (the cargo-hack
  commands) and 15 amended.
- 2026-09-29: step 11 reviewed and accepted. Seven commits, "Add an SVG
  writer with a fixed-advance layout" to "Document the SVG export"; all
  checks pass including `nix flake check`, whose `export` check now also
  renders every SVG snapshot and two CLI drawings with resvg and Euler
  Math as the only font, with any tool output fatal; the planning session
  rendered every snapshot and three CLI drawings (a twelve-inference
  tree, a net with a crossing, a partial derivation) to PNG and they look
  as intended. D12 as amended held: a committed table of Euler Math's
  advances (202 characters, the script that printed it in the report),
  integer coordinates in thousandths of an em, `textLength` on every text
  run, one pass up and one down the tree over the explicit-stack walk,
  nets as formula trees under half-ellipse links of one shape (nested
  pairs nest, interleaved pairs cross), switching cycles highlighted.
  Item 1a done: the standalone LaTeX documents load `eulervm` and the
  Typst documents set Euler Math. Decisions accepted: no `svg` crate;
  atom letters as mathematical italic code points (Euler has no italic
  face); `--standalone` refused for the one-form SVG formats;
  `--format net-svg` rather than a `net` subcommand; verdicts as XML
  comments with `-` written as U+2010 since `--` cannot appear in one;
  roxmltree as a dev-dependency for the structural tests. Prompts 12 and
  15 amended: the certificates take `Form` (two natural forms) and follow
  `modules/export.nix`'s shape for their check; the web front end's plan
  gets the report's calls and ids; the SVG follow-ups (per-formula ids
  for clicks, anchors at the atom, a nesting-safe arc cap, disconnection
  colouring) join 15g''.
- 2026-09-29: after the step 11 review, the author asked for two things,
  recorded before step 12 runs. The LaTeX and Typst output must never set
  a font, standalone documents included, since users paste it into their
  own documents and want visual consistency: D12 now says so, and the
  removal of step 11's `eulervm` and Euler Math lines is 15h (Euler stays
  for SVG and the web). Visual outputs must be configurable through the
  library, designed for the CLI, the web front end and any other wrapper
  at once: new decision D15 (one plain-data options value per output
  feature, with defaults and serde, presets as values, no hidden
  constants), a paragraph in `conduct.md` so every later step designs
  that way, 15h as the item that retrofits it to the exports (open-goal
  shape, label convention, alignment, comments, the SVG font and its
  advance table, presets, the CLI's `--style` flags), and a note in
  prompt 12 applying D15 to the certificates from the start.
- 2026-09-29: step 12 done, three commits ("Export derivations as Rocq
  scripts for NanoYalla", "Check the Rocq certificates in nix", "Document
  the Rocq certificates"); report `reports/12-certificates.md`. The
  kernel is NanoYalla 1.1.3 from Click & coLLecT, as a non-flake input
  pinned to a commit and built by the `rocq` check with nixpkgs' Rocq
  9.1.1, whose closure (1.2 GB from the binary cache) is the cost of
  keeping the check in `nix flake check`. Intuitionistic proofs are
  certified as the classical proofs they are (Yalla's `ill` needs full
  Yalla with OLlibs, which nixpkgs lacks, and has no derived-rule layer);
  a Yalla `ill` target, `show rocq` in `interact`, a `--lemma` flag and
  the Lean target join the follow-ups. D15 held: `rocq::Options` (lemma
  name, prelude) is the whole configuration.
- 2026-09-29: step 12 reviewed and accepted (the entry above is the step
  session's own). All checks pass including `nix flake check` with the
  `rocq` check; the planning session built the kernel locally from the
  pinned input and compiled ten further certificates from the CLI (a
  distribution over `⊕`, three copies of a `!`, a chain of `⊸L`s with
  `⊤`, four-way `⊗` nestings in both orders, two contracted `!`s under a
  four-way `⊗`, `⊤` and `⊥` with contexts, the four units, a `⅋` inside a
  `⊗`), all accepted with no output. D6 and D15 held: the exporter is a
  pure function of the derivation, tracks the list Rocq shows for every
  goal so that one exchange per `⊗` suffices, refuses Mix, affine
  weakening and open goals before writing anything, and `Options` (lemma
  name, prelude) is its whole configuration. Decisions accepted: NanoYalla
  over Yalla's kernels (derived rules at a position, builds with nixpkgs'
  Rocq 9.1.1 and the standard library alone, what Click & coLLecT users
  have), pinned as a non-flake input rather than vendored (LGPL stays out
  of the tree); a `Lemma` with `formula` binders, so the certificate is
  schematic in the atoms; constructors rather than the kernel's
  notations; intuitionistic proofs certified as the classical proofs they
  are, which sidesteps the reading's ambiguity; the check kept in
  `nix flake check` despite the 1.2 GB closure, since it is a cached
  download. Prompts 13 and 15 amended: 13 notes that the exports need no
  parallel work and how to run one check alone; 15i collects the second
  kernels (Yalla `ill` with the ambiguity in its statement, Lean once it
  has units, the `ex_t_r` chain for wide sequents) and 15h the
  `--lemma`/`--prelude` flags and certifying a finished `interact`
  session.
- 2026-09-30: step 13 reviewed and accepted. Six commits, "Add the
  parallel runtime behind a feature" to "Measure parallel speedups"; all
  checks pass with and without the feature, including `nix flake check`;
  the planning session read the runtime, the focused engine's parallel
  layer and the net engine's cubes in full, timed a 3-Partition
  refutation in release (one thread past a 90 s limit, eight threads
  exhaustive in 51 s), checked the parallel time limit and a parallel
  `close` in `interact`, and fixed one pre-existing nit the report named
  (two test imports of the nets module unused without default features,
  now gated on `parse`). D7, D9, D10 and D11 held: rayon behind the
  `parallel` feature, off by default and on in the CLI, one pool per
  call and no global; cube-and-conquer as nested fork-join at the first
  two choices of a branch with and-parallel `&`, every alternative on a
  worker so the choice's cancel flag reaches it; a sharded memo whose
  merge under the shard's lock is the compare-and-swap D10 asked for; one
  shared arena so `Proved` entries mean one node to every worker; the
  net engine's cubes from a counter; the caller's stop closure polled on
  the calling thread once a millisecond, so no `Send` bound and no API
  change; `--jobs` and `--deterministic`; every level of the copy bound
  searched to its end, so a parallel run may return another proof and
  never another decided verdict, which the tests and the step's
  fresh-context fuzzer (about 20 000 parallel runs, no mismatch) pin. The
  fuzzer's one real finding, an alternative run in place that no sibling
  could cancel, was fixed in the step. The numbers: 6× on 8 threads for
  wide or-trees, 7× for the net engine's cubes, little for memo-bound
  families, the portfolio nothing. Decisions accepted: nested fork-join
  over a static cube list, a shared arena over relocation, no `dashmap`,
  no thread sanitizer on stable Rust, a timing test over criterion.
  Prompts 14 and 15 amended: 14 gets the baseline-row rule, the pool's
  per-call cost, the generators to reuse from the ignored `speedups`
  tests and the portfolio's status; 15j collects the parallel follow-ups.
- 2026-09-30: step 14 reviewed on its intermediate state and accepted as
  far as it goes. Ten commits, "Read LLTP problems" to "Report the
  machine's noise and how the baseline guards against it"; all checks
  pass including `nix flake check` with the new `bench` check. Delivered
  and sound: the LLTP reader (`linlog::lltp`), seventeen families with
  verdicts by construction (`linlog::families`, the engines' tests now
  use them and the ignored timing tests are retired), the harness
  (`linlog-bench`: a child process per run, CSV with counters, CPU time
  and run-queue wait, Markdown summaries), a resumable, detached,
  memory-capped baseline script, and one engine fix outside the brief
  that was right to make (the focused engine polled its stop condition
  only at stable sequents, so a 2 s limit ran past five minutes on a
  Petri net; it polls every 4 096 splits now). Not delivered: the
  baseline. A first run died at 02:03 when a reviewer's scratch checker
  took 62 GB and the kernel's OOM killer took the terminal with it; a
  second shared the machine and was stopped; the overnight run was
  cancelled because the machine cannot be used exclusively any more. The
  report's numbers are therefore preliminary upper bounds. What they
  show, machine-independently: the focused engine enumerates splits (the
  unsolvable 3-Partition at bins of four 52.6 s, Mix over ten pairs
  217 s, QBF over twenty variables about 100 s, sixteen counter tokens
  undecided, 651 Petri nets beyond the 63-member limit, 895 at the
  recursion limit); the net engine loses by orders of magnitude on equal
  literals within one tree and wins by as much on wide sequents, and no
  multiplicity threshold separates the two (the step's reviewer produced
  the counterexample that kept `NET_MULTIPLICITY` at two); intuitionistic
  mode beats the classical search 2× to 27× on Horn-like families; 18 %
  of the ILLTP problems reached are decided in 5 s; fourteen LLTP headers
  contradict their problems. The planning session fixed four references
  to a results file that does not exist yet and one plan reference in a
  script comment (one commit). Decisions: the baseline becomes 14c, taken
  once after 14b by resuming step 14's session when a night is free, so
  the machine is needed once and not twice; 14b is written
  (`14b-performance.md`) to measure by the engines' deterministic
  counters and pinned CPU time on a shared machine, with its own
  before-and-after target set; `conduct.md` gains the rules for a shared
  machine (capped scopes for scratch programs, nothing on every core
  unasked); the models and efforts were re-evaluated ("Why these models
  and efforts": 15h to Opus 5.5 at `xhigh`, 15d to `xhigh`, the rest
  unchanged); 15 gains 15k for the benchmark follow-ups and the routing
  feature for the net engine in 15b'.
- 2026-09-30, later: the author pointed out that step 14 did not
  complete, which is right and was understated above: the step owed a
  baseline and recorded none. Three corrections. The rows of the run that
  got furthest existed only in the step session's scratch directory under
  `/tmp`; they are now committed as `bench/preliminary/` with their
  summary, so the numbers the plan argues from have their data. The
  report's header said the baseline was scheduled for a night that was
  cancelled; it now says the step is incomplete in that respect and that
  the baseline is 14c. And 14b no longer assumes a record of the engine
  before its changes: it checks its set-up against the preliminary
  counters and takes a whole sequential pass over the ILL library before
  and after on one pinned core (item 1a), which is the LLTP-wide
  comparison the missing baseline would have given.
- 2026-09-30, numbering and the second run of step 14: the author can
  give the machine to the benchmarks from 20:00 to 07:00 on two nights,
  so the baseline is taken twice after all, before and after the
  performance pass, and step 14 is run again to be completed rather than
  left open. `14-benchmarks.md` is rewritten for that second run: what is
  built, what failed and why, the slot, and what remains (check the
  harness and the machine by day, keep every baseline in a directory of
  its own with the commit it measured, start and stop the run unattended
  within the slot, give the author the two blocks only root can run, do
  not poll overnight, finish the report in the morning); what the step
  first asked for is kept below it. The labels are whole numbers from
  here: the performance pass is step 15 (`15-performance.md`, which now
  starts from the baseline and leaves the whole-library and all-core
  numbers to step 16), the second baseline with the comparison of the two
  is step 16 (`16-baseline.md`), and `15-later.md` is `later.md` with its
  sketches numbered 17 to 26 and its follow-ups as lists by area. The
  mapping from the old labels is under the step table; this log and the
  reports keep the labels they were written with. Step 15 waits for step
  14's baseline, since the baseline measures whatever is checked out at
  20:00.
- 2026-09-30, the later work is not planned yet: on the author's wish the
  sketches in `later.md` lose their numbers and their order and become
  candidates, and a new step 17 (`17-assessment.md`, Fable 5.1 at
  `xhigh`) assesses the state of the repository, the two baselines and
  every candidate (worth, feasibility, cost, interactions, order), puts
  the decisions that are the author's to the author, and then plans the
  steps from 18 as the author decided, prompts included. It is the one
  step that stops to ask. A candidate is added at the author's request: a
  general code audit with the refactoring it justifies, behaviour
  unchanged, with the engines' deterministic counters as the regression
  oracle.
- 2026-10-01: step 14's baseline reviewed and accepted; the step
  continues with a supplement in the night of 2026-10-01 and its report
  is finished the morning after. Ten commits of the second run so far,
  "Keep every baseline in a directory of its own and run it unattended
  in the night slot" to "Report the progress of the reruns"; all checks
  pass including `nix flake check`; `core/`, `cli/`, the Cargo files and
  the toolchain are identical to the measured commit. The baseline is
  taken: 21:23 to 06:45 on commit `b53cb17c6831`, every stage complete
  before the 07:00 stop, no scheduled job in the run, the sequential
  rows waiting for a CPU 0.07 % of their time; `bench/results/2026-09-30/`
  holds the rows and `starts.txt` the commit, `bench/RESULTS.md` the
  tables, `bench/preliminary/` is gone, and the documentation no longer
  says the baseline is pending. The planning session checked the
  report's counts against the rows (638 proved and 99 refuted of 4 495
  intuitionistic problems; 984, 983, 898, 845 and 47 unknown by reason;
  209 kills and 5 aborts at sixteen threads; the preliminary verdicts and
  counters reproduced exactly) and the unattended path against the
  journal (the unit waited one minute for the load to fall, the timers
  it paused, the stop timer firing on an inactive unit). The numbers
  confirm step 13's speedups (6.1× at eight threads on the unsolvable
  3-Partition, the net engine's cubes 6.3–7.0×, 11.8–13.6× at sixteen),
  show sixteen threads a net loss on LLTP and the portfolio worthless,
  and make the two-sided engine's gain library-wide (never fewer
  decisions, 1.36× in the median on the Petri nets). Two findings go to
  step 15 as defects, not tuning: the stop condition is missed inside a
  long split enumeration (a Petri net examines 1.8 billion splits at one
  stable sequent and stops after 242 s under a 5 s limit; 90 one-thread
  and 166 sixteen-thread runs ran past the harness's kill; the CLI's
  `--timeout` has the same hole), and the proof arena grows without
  bound under a failing enumeration (aborts at 16 GiB). Accepted: the
  first night's `parallel-net` filter dropped the MLL 3-Partition
  family, fixed for the supplement; the LLTP library's one malformed
  file is repaired in the flake's fetch; `--grace` and `FAMILY/NAME`
  filters in the harness; stage 4 from an explicit `bench/reruns.txt`,
  since whether a killed run can finish is known only by running it.
  Not accepted as practice: the step ran about three hours of probes by
  day on the efficiency cores to choose that list, against the rule,
  until the author stopped it; `conduct.md` now says a measurement the
  step does not name is asked for first. Nits for the final report: the
  mismatch count is 25 files in 26 rows (23 distinct headers plus two
  further translations of a known one), and the step edited one line of
  `15-performance.md`, which the protocol reserves for this session
  (harmless, a heading's name). Prompts 15 and 16 amended: 15 gets the
  two defects as requirements with the stop-miss instance in its target
  set, 16 the two-night shape of the first baseline, `reruns.txt` run as
  it stands, the evening block before 20:00 and the killed rows as the
  place to look for step 15's fixes.
- 2026-10-02: step 14 reviewed and accepted in full. Four more commits,
  "Add the night of 2026-10-01 to the baseline: the reruns and the net
  engine's cubes on MLL 3-Partition" to "Finish the step 14 report with
  the night of 2026-10-01"; all checks pass including `nix flake check`;
  the engines are still those of `b53cb17c6831` (the supplement's start
  names `5b2d49de6880`, which adds `plan/` and the harness's kill and
  filter). The supplement ran from 20:00 to 21:27 on an idle machine,
  the author's block pasted before the slot this time, after a reboot at
  19:29 had dropped the transient timers and they were armed again. The
  planning session checked the report against the rows: 95 reruns, of
  which 81 end cleanly (75 timeouts, 4 at the copy bound, 2 proofs), 7
  abort and 7 are killed at 605 s; the net engine's cubes on MLL
  3-Partition (59.7 s on one thread at five bins, 13.0× at sixteen);
  26 mismatch rows on 25 files, as the report now says. What the
  supplement settled: the kills on the library's largest files were the
  load, not the search (16 s where the kill came at 10.5 s); the five
  aborts at sixteen threads were the 12 GiB cap on a pool, not the
  arena; the arena defect is real on one thread (seven more aborts);
  and a missed stop is also a late verdict (two Petri nets proved 21.6 s
  and 552 s into a 5 s limit, which `summary` counts as solved). One
  planning concern from the durations: stages 1 to 3 and the supplement
  took 10 h 50 min of the 11-hour slot against the script's estimate of
  10 h 30 min, and step 15 will make 845 problems searchable that were
  refused at once, so step 16's night may not fit; its prompt now asks
  for the estimate by arithmetic, a word to the author before the night,
  and a second night by the resume rather than dropped rows. The
  planning session made one fix of its own in `.claude/rules/bench.md`
  (the timers are transient; what a resumed baseline's starts must
  share). Prompts amended: 15 (the corrected account of the aborts, the
  two TokenRing nets as targets, the crashed child's whole error output
  in the log), 16 (the estimate and the slot, the transient timers, late
  verdicts kept apart in the comparison), and `later.md` (the Petri-net
  and net-engine candidates' numbers, the parallel follow-ups with the
  CLI's default thread count and the portfolio's removal as questions
  for step 17, the benchmark follow-ups brought up to date).

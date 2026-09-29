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
| 14 | Benchmarks, LLTP input, hard families | `14-benchmarks.md` | Opus 5.5 | xhigh | 13 |
| 14b | Performance pass on the focused engine, driven by 14's numbers | written after 14's review | Fable 5.1 | xhigh | 14 |
| 15 | Later: MELL nets with boxes, essential nets, inverse method, Petri nets, Lambek, MALL nets, the web front end | `15-later.md` | – | – | 14b |

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
validation are checker-grade, 13's shared memo under concurrency and 14b's
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
```

The aliases `fable` and `opus` also work for `--model`. The flags are
documented at code.claude.com/docs/en/cli-reference.

## Review protocol

1. A step's session commits its work thematically with jj and ends by writing
   `plan/reports/NN-<name>.md` (what was done, decisions, deviations from the
   spec or this plan, open questions, what later steps must know). It does
   not push and does not edit the other prompt files.
2. The planning session (the one that wrote this file; `claude --resume` and
   pick it, or start any session with "review step NN of plan/README.md")
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
Fonts are declared, not measured: a monospace font with a fixed advance per
character makes the layout exact without font metrics. No graphviz, no
browser-side layout. The same layouts feed the LaTeX and Typst exports where
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

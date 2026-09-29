# Step 8: intuitionistic mode and the additive fast path

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D1 in full, D2, D6, D7, D8) and every report in
  `plan/reports/`.
- `proof-search-specifications.md`: "Intuitionistic fragments" in full
  (verdict, complexity, the two-sided focused engine specification,
  pitfalls, data layout; the IMLL-Net essential-net engine is step 15, but
  read it so nothing you build precludes it), "MLL variants" § "IMLL by
  embedding", "Other fragments" § "Additive-only LL", "Common
  infrastructure" (counts on the translation `⊢ Γ⊥, A`).
- `.claude/rules/core.md`, `core/src/**`.

## What steps 3 and 7 left you

The dispatch in `search::prove_until` refuses intuitionistic mode with
`Error::NoEngine`; `Engine` is `#[non_exhaustive]` and gains the two-sided
engine and `Additive`. The focused engine's report
(`plan/reports/03-focused-engine.md`) describes the phases, the memo, the
counts and the pools the two-sided engine should share rather than copy.

Step 9 (interactive proving, decision D13) presents goals two-sided: the
polarity of every occurrence (input or output) must be a query on the
forest a goal can use, not something computed for the roots alone.

## Goal

`Mode::intuitionistic` works end to end: the polarized-shape test of D1
recognises ILL sequents in the one-sided arena and reports a clear error
otherwise; the dispatch sends IMLL over `⊗ ⊸ 1` through the embedding into
the net engine and everything else intuitionistic to a two-sided focused
engine (IMALL, IMELL, ILL, affine variants, with the bounds and outcomes of
step 7); proofs pass a two-sided-aware checker and render as two-sided
derivations `Γ ⊢ A` with `⊸` and the ILL rule names. Also the additive-only
fast path of D8.

## What to build

1. **ILL shape (D1).** A function on the forest that decides whether the
   sequent is an ILL sequent: exactly one output-shaped root, all others
   input-shaped, with the output/input grammar of D1 applied recursively
   (an output `⅋(a, b)` is `a⊥ ⊸ b` with `a` input-shaped and `b`
   output-shaped; decide and document how the symmetric reading is handled).
   It returns the polarity of every occurrence (input or output), which is
   Lamarche's polarization and what the two-sided engine, the two-sided
   printing and step 15's essential nets use. Extend `Fragment` naming with
   the intuitionistic fragment names (IMLL, IMALL, IMELL, ILL): step 1's
   report leaves the classical `Display` as is and asks for a mode-aware
   name (a method taking `Mode`, or a small wrapper type) that the CLI's
   fragment line and the JSON output use.
2. **Two-sided printing.** `Sequent` (or a view over it plus the
   polarization) prints as `Γ ⊢ A` with ILL formulas and `⊸`; the parser's
   output for `A, A -o B |- B` in intuitionistic mode prints back as
   `A, A ⊸ B ⊢ B`. Property test: parse, print, parse again.
3. **The two-sided focused engine** as the spec states it: sequents
   `Θ ; Δ ⊢ A`, right-negative goals and left-positive hypotheses decompose
   invertibly (`0` on the left succeeds; `⊤` on the left is inert), stable
   sequents memoized on `(Θ, Δ, goal)`, decide right on a positive goal or
   left on a negative hypothesis from Δ (consumed) or Θ (copied, bounded),
   `⊸` left splits Δ, `&` left chooses, initial rule with the two forms;
   counts on the classical translation; bound, loop check, affine prune and
   outcomes as in step 7. Decide (D7) whether this is the step 3/7 engine
   with a goal side or a sibling module sharing its building blocks; avoid
   duplicating the memo, the deepening and the count machinery.
4. **IMLL by embedding**: for `⊗ ⊸ 1` sequents in intuitionistic mode,
   run the net engine on the one-sided sequent (it is the same arena) and
   convert the proof to a two-sided derivation; the spec says this
   embedding is conservative for this fragment only. Test agreement with
   the two-sided engine.
5. **Checker and derivation view for ILL**: step 2 established (its
   report, "No ILL rule tags") that on the lowered sequent every ILL rule is
   a classical node, so the term type needs nothing; what is missing is the
   one-succedent condition, which the checker currently refuses with
   `Problem::Intuitionistic` because it needs the input/output side of every
   occurrence, i.e. this step's reading. Replace that refusal with the test
   (every derived sequent has exactly one output-shaped formula, a count
   over the side map), and extend the derivation view (`Inference`, `Rule`
   are the extension points) so intuitionistic derivations show two-sided
   sequents `Γ ⊢ A` with the ILL rule names (`⊸L`, `⊸R`, `⊗L`, `⊗R`,
   `&L₁`, `&L₂`, `&R`, `⊕L`, `⊕R₁`, `⊕R₂`, `1L`, `1R`, `0L`, `⊤R`, `!L`,
   `!R`, `!c`, `!w`, `ax`), mapped from the classical rules by the report's
   table; rendering, JSON and the exports of steps 10 to 12 see the same view
   type.
6. **Additive fast path** (`search::additive`): the memoized recursive
   procedure on subformula pairs for additive-only sequents with exactly two
   roots, O(|A|·|B|), emitting a proof term; classical and intuitionistic;
   dispatch row.
7. **CLI**: `--intuitionistic` reaches everything; error messages when the
   input is not an ILL sequent name the offending root or subformula.
   `linlog check -i` is refused in the CLI before the checker is asked
   (`plan/reports/04-api-and-cli.md`, "For later steps"): remove that
   refusal once the checker handles intuitionistic mode. The new engines
   are `Engine` variants plus `EngineArg` variants in
   `cli/src/argument_parsing.rs`.
8. **Tests**: the spec's pitfalls as tests (`0` on the left proves
   anything; `⊤` inert; promotion needs empty Δ unless affine; classical
   non-conservativity: a sequent provable classically but not in ILL, from
   Schellinx's counterexamples with `0`, is refused by the two-sided engine),
   generator-based positives for ILL, differential tests IMLL via embedding
   versus two-sided, ILLTP-style problems as ignored slow tests, and the
   additive path against the focused engine.
9. **Documentation**: `.claude/rules/core.md` (the polarized reading, the
   two-sided engine's invariants), CLAUDE.md, README (the modes).

## Constraints

- No `unsafe`; no second formula representation (D1); no duplicated engine
  machinery (D7).
- Determinism with the same options.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`cargo deny check` if a dependency changed, `nix flake check` at the end
(`jj st` first).

## Deliverables

- Thematic jj commits ("Recognise intuitionistic sequents by shape", "Print
  sequents two-sided", "Add the two-sided focused engine", "Embed IMLL into
  proof-net search", "Add the additive fast path", …).
- `plan/reports/08-intuitionistic.md`: API, the shape rules as implemented,
  how the two-sided engine shares the classical one, decisions, deviations,
  open questions, and what steps 9 to 12 (interactive proving, two-sided exports, ILL
  certificates in Yalla's `ill`) and 15 (essential nets) must know.

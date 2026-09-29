# Step 7: exponentials (MELL, full LL) and affine mode

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D6, D7, D8, D9) and every report in `plan/reports/`,
  especially `02-proofs.md` (the dyadic terms and the translation to the
  standard calculus) and `03-focused-engine.md` (the engine you extend).
- `proof-search-specifications.md`: "MELL" in full (decidability status,
  the MELL-Seq specification: dyadic sequents, rules, bound, loop check,
  memo, copy heuristics, count prunes; the affine variant; pitfalls; data
  layout), "Other fragments" § "Full propositional LL" and § "Contractive
  fragments" (read, not implemented), and "Common infrastructure" §
  "Memoization contract".
- `.claude/rules/core.md`, `core/src/search/focus/**`, `core/src/proofs/**`.

## What step 3 left you

`plan/reports/03-focused-engine.md`, "The engine, in the spec's terms" and
"For step 7": `Rules` is where the exponential switches go, `asynchronous`
currently hits `unreachable!` on `Kind::Quest`, `decide` must see `!` as a
focus candidate, `memo::Entry` and `Memo::get` get the bound comparison and
the key both zones, and `Counts::new` must give `!A`/`?A` a row (the spec:
skip atoms below a `?`). The dispatch refuses exponentials and affine mode
with `Error::NoEngine`; replace those refusals. The generator
(`search/generate.rs`, test-only) is the differential-testing tool: extend
its `Rules` with exponentials. Keep the engine's shape: the functions are
the spec's rules, the pools keep the hot path allocation-free, and the
stop closure is polled per stable sequent.

## What step 2 fixed about the terms

Read "How an engine constructs a proof" and "The dyadic-to-standard
translation" in `plan/reports/02-proofs.md` before designing the dyadic
state; the checker is the reference. In short: a `Quest` node where the
asynchronous phase moves `?A` into `Θ`; a `Copy` node on the occurrence of
`A` (the subformula of the `?`) for every D2 step; the initial rule with
`p⊥ ∈ Θ` is `Copy(p⊥)` above `Ax(p, p⊥)`; `Bang` needs an empty linear
zone; a `?` formula that goes unused needs no node (its `Quest` becomes
`?w` in the view). In affine mode the term has no relaxed rules: the engine
emits one `Weaken` node per surplus formula *below* the `ax`, `!` or `1`
whose context it relaxes, never above a promotion. The checker derives the
least unrestricted zone bottom-up, so it accepts any correct dyadic proof
regardless of how the engine tracked `Θ`, and the derivation view puts
`?d`, `?c` and `?w` where the translation table says.

## Goal

The focused engine handles exponentials: dyadic sequents `⊢ Θ ; Γ`, the
D2 copy rule bounded per branch with iterative deepening, memo entries that
carry the remaining bound, the loop check, and the three-valued outcome
(`Proved`, `Unprovable` only after a level completed without hitting the
bound, `Unknown` otherwise). Full LL is MELL plus the MALL rules already
present. Affine mode adds weakening and the supermultiset-ancestor prune,
which makes the engine a decision procedure for affine fragments. Every
proof passes the step 2 checker and expands to a standard derivation with
explicit dereliction, contraction, weakening and promotion.

## What to build

1. **Dyadic sequents** in the engine's state: `Θ` as a bitset, `Γ` as a
   bitset with a lazily promoted count vector when a copy repeats an
   occurrence (spec "Data layout"); memo key over both zones; the branch
   stack of stable sequents for the loop check and the affine prune.
2. **Rules**: `?` asynchronous into Θ; `!` in focus with empty `Γ` (affine:
   any `Γ`), release; D1 and D2 as the spec states with D2 counting against
   the per-branch bound; the two initial rules; units with Θ. The `&` rule
   duplicates `Γ` and keeps `Θ`, each premise inheriting the remaining
   budget.
3. **The bound**: `b = 0, 1, 2, …` up to `Options::copies`, with a level's
   result `Failed` versus `Exhausted` tracked exactly as the spec requires
   for soundness of `Unprovable`. Memo entries survive across levels;
   `Failed{bound_remaining}` hits only when the current remaining bound is
   ≤ the stored one.
4. **Copy heuristics** as the spec lists them (D1 before D2; prefer Θ
   formulas whose literals match unmatched atoms; skip a D2 on a formula
   with an unconsumed identical copy in Γ). Count prunes disabled for atoms
   below any `?` or `!` in the problem.
5. **Affine mode**: weakening in the initial rule, under `!` and `1`; the
   supermultiset-ancestor prune against the branch stack; the affine memo
   contract (say what changes and why it stays sound); termination test.
   `Mode::affine` reaches the CLI's `--affine`.
6. **Full LL**: confirm the additive rules and the dyadic rules compose
   without new cases; test sequents mixing `&`/`⊕` with `!`/`?`.
7. **Dispatch**: MELL and LL rows of D8, the affine row, `--copies`
   (default: a documented small number, e.g. 3 as llprover uses, with
   `Unknown` reported when it binds), `--timeout` honoured inside deepening.
   The CLI's `Unknown` line says which limit bound. In the CLI, `--copies`
   already exists hidden and refused (`plan/reports/04-api-and-cli.md`, "The
   copy bound"): remove `hide = true` and the refusal, pass the value to
   the `Options` setter you add, and give the new `Reason` variant its line
   in `core/src/serialize/search.rs`, which the compiler does not check.
8. **Derivation view**: the dyadic proofs expand into standard derivations
   (dereliction at D2, contraction where a Θ formula is used more than once,
   weakening for unused Θ members at the leaves, promotion for `!`); the
   step 2 checker validates the terms, the rendering shows the standard
   rules. Tests compare against expected renderings for small MELL proofs.
9. **Tests**: the spec's soundness pitfalls as tests (an `Exhausted` level
   never yields `Unprovable`; memo reuse with a larger remaining bound is
   refused), the classic MELL examples (`!a ⊢ a` by dereliction; `!a ⊢ 1`
   by weakening; `!a ⊢ a ⊗ a` and `!a ⊢ !a ⊗ !a` needing two copies;
   `⊢ !(a ⊸ a)`; `!a, !(a ⊸ b) ⊢ !b` with contraction under promotion;
   `a ⊢ !a` and `?a ⊢ a` unprovable), the generator extended with `!`/`?`
   rules (generated proofs
   must be found within their copy count), affine examples (`a ⊢ 1`,
   `a, b ⊢ a` provable only affinely), and a handful of ILLTP-style Horn
   problems as ignored slow tests.
10. **Documentation**: `.claude/rules/core.md` (dyadic invariants, the bound
    and memo contract, the affine prune's soundness argument); CLAUDE.md
    and README for the new flags.

## Constraints

- No `unsafe`; hot-path allocation only when a count vector is promoted.
- Determinism with the same options.
- `Unprovable` is a claim of completeness: make its conditions explicit in
  code and doc comments, and test them.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`cargo deny check` if a dependency changed, `nix flake check` at the end
(`jj st` first). Report release timings on the slow tests and how the memo
size grows with the bound.

## Deliverables

- Thematic jj commits ("Add dyadic sequents to the focused engine",
  "Bound copies with iterative deepening", "Add affine mode", "Expand dyadic
  proofs into standard derivations", …).
- `plan/reports/07-exponentials.md`: API and option changes, the soundness
  arguments in a few sentences each, timings, decisions, deviations, open
  questions, what step 8 (two-sided engine reuses the dyadic machinery) and
  step 12 (memo with bounds under concurrency) must know.

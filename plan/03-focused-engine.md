# Step 3: the focused engine for MLL, MLL with units and MALL

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D5 to D9; this step realises D7 for the classical
  focused engine and the first rows of D8) and the reports
  `plan/reports/01-*.md`, `plan/reports/02-*.md`.
- `proof-search-specifications.md`: "Common infrastructure" in full
  (focused skeleton, count invariants, memoization contract), "MLL" (the
  "Fallback: MLL-Seq" subsection, "Pitfalls", "Data layout"), "MLL variants"
  (units, Mix), and "MALL" in full (MALL-Seq specification, interval counts,
  pitfalls, data layout). The parallelization paragraphs are for step 13;
  design so they remain possible (no global mutable state, memo behind a
  trait or a type that a sharded map can replace).
- `.claude/rules/core.md`, `core/src/**`.

## Goal

`search::focus`: the focused backward engine over occurrence bitsets with a
memo of stable sequents, complete and terminating for MLL (with and without
units), MALL, and both with Mix, returning a checked proof term or
`Unprovable`. Also the `search` front door that step 4 will wire into the
CLI: `prove(&Sequent, &Mode, &Options) -> Outcome` with dispatch that, for
now, sends every classical input without exponentials to this engine and
rejects the rest with a clear error (steps 6 to 8 fill the table).

## What steps 1 and 2 left you

`Forest` and `OccSet` (`plan/reports/01-core-refactor.md` shows the API:
`root_set`, `toggle`, `submasks` in Gray order over a compacted member list,
`hash::HashMap<OccSet, _>` for the memo), and the proof terms of step 2
(`plan/reports/02-proofs.md`, "How an engine constructs a proof"): there is
no builder type; the engine owns a `Vec<Node>` during search, pushes a node
after its premises, keeps the `NodeId` of a proved stable sequent in the
memo so a hit reuses the subproof, and at the end calls
`Proof::new(forest.clone(), nodes, root)`, which drops the subproofs of
failed branches. `Top(o)` carries no context. `proof.check(mode)` in a
`debug_assert!` and in every test; the hand-written proofs in
`core/src/proofs/check.rs` show what the terms of the classic examples look
like. Step 1 did not build the interval
counts or the count equations: they are this step's. `submasks` takes at
most 63 members: a `⊗` split over a larger context must not panic; decide
what happens (the spec's lazy contexts are step 15 material, so an
`Unknown` outcome with a reason is acceptable, as is a count-pruned
enumeration that does not need a `u64` mask). Atom-only sequents
(`Fragment::EMPTY`) and every fragment up to MALL dispatch here.

## What to build

1. **The engine as the spec states it**: asynchronous phase run to
   completion (the `&` branching included) producing stable sequents; memo
   of stable sequents only, `Proved(term index)` or `Failed`, with a
   configurable cap and arbitrary eviction; `prove` and `focus` as in the
   MALL-Seq specification, including the immediate failure and success
   tests, the interval count invariant per atom (precomputed per occurrence
   as sparse rows), the `⊗` split by submask enumeration over the compacted
   members of Γ in Gray-code order with the count vector maintained
   incrementally and the forced split when a factor is a literal, `1`, `⊥`,
   `⊤`, `0`, and the Mix rule tried last only when the count equation admits
   it. Units and Mix are rule-set switches driven by `Fragment` and `Mode`
   (D7). In MLL without additives, the plain balance and the
   `c = t − p + 2 − #1 + #⊥` equation are valid prunes; in MALL only the
   interval form is (spec "Pitfalls"). Get this right and test it with the
   sequents the spec names (`⊢ a ⊕ b, a⊥` is provable; the unit checks in
   "MLL variants").
2. **Explicit control of recursion depth**: an explicit stack or a search
   thread with a large stack behind the `Options` (the CLI decides in step 4;
   `core` must not spawn threads unconditionally, D11).
3. **Proof construction** through the step 2 builder, memo entries pointing
   at term indices so a memo hit reuses the subproof; every proof returned
   passes the step 2 checker in debug builds (`debug_assert!`) and in every
   test.
4. **`search` front door**: `Options` (memo cap, and room for the later
   knobs: copy bound, time limit, thread count, determinism, engine and
   fragment overrides), `Outcome` (D9) with `Statistics` (nodes visited,
   memo entries, splits enumerated), `prove` with dispatch on
   `Sequent::fragment()` and `Mode`. A time limit needs a clock; put the
   deadline check behind a small trait or a closure the caller supplies so
   `core` does not depend on `std::time` (D11), and let the CLI pass one.
5. **Tests as the spec's "Testing strategy" says**: a random proof generator
   (build cut-free proofs bottom-up, read off the conclusion; every
   generated sequent must be proved, and the returned proof must pass the
   checker), mutants (swap one atom name; whatever is not caught by counts
   must still be decided consistently by the engine with and without the
   memo), the hand-written sequents from the spec, the classic small ones
   (`⊢ a⊥, a`; `⊢ (a ⊗ b)⊥, a ⊗ b`; `⊢ a ⊗ b, a⊥ ⅋ b⊥`; `⊢ a ⊗ b, a⊥, b⊥`
   is provable, `⊢ a ⅋ b, a⊥, b⊥` is not without Mix and is with it;
   `⊢ a & b, a⊥ ⊕ b⊥`; additive distributivity holds in one direction
   only: `(a & b) ⊕ (a & c) ⊢ a & (b ⊕ c)` is provable and its converse is
   not, while `a ⊗ (b ⊕ c)` and `(a ⊗ b) ⊕ (a ⊗ c)` prove each other; write
   them properly), and a
   Kanovich-style Horn encoding of a small 3-Partition instance as a slower
   test (`#[ignore]` if it exceeds a second in debug). Use the parser for
   test inputs; keep them readable.
6. **Documentation.** `.claude/rules/core.md` gets the engine's invariants
   (stable sequents, memo validity, the count prunes and where each is
   sound). CLAUDE.md's crate paragraph if needed.

## Constraints

- No `unsafe`; performance matters: no allocation per node in the hot path
  (scratch buffers reused, bitsets by value where `W` is small), hashing with
  the step 1 hasher, `#[inline]` where profiles justify, not everywhere.
- Determinism: with the same input and options the engine returns the same
  proof.
- Human-readable: the engine's state machine should be recognisable from the
  spec's rules; name the phases and rules as the spec does.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`cargo deny check` if a dependency changed, `nix flake check` at the end
(run `jj st` first). A `cargo test --release` run of the generator tests
with a larger sample is a good final smoke test; report its numbers.

## Deliverables

- Thematic jj commits ("Add interval counts per occurrence", "Add the
  focused engine for MALL", "Add Mix and the MLL count prunes", "Add the
  search front door", "Generate random proofs for testing", …).
- `plan/reports/03-focused-engine.md`: the engine's public API, the
  `Options`/`Outcome`/`Statistics` types step 4 will expose, what the
  dispatch currently does, performance observations, decisions, deviations,
  open questions. Commit it with the last change.

# Step 6: proof-net search for MLL

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D6a, D7, D8) and every report in `plan/reports/`,
  especially `05-proof-nets.md` (the structures and the criterion you
  search with).
- `proof-search-specifications.md`: "MLL" in full (verdict, the MLL-Net
  specification: preprocessing, search, symmetry breaking; pitfalls; data
  layout), "MLL variants" (Mix), "Cross-cutting engineering notes"
  (differential testing, known-hard families). The parallelization
  paragraph is for step 12: keep all search state per worker, no globals.
- `.claude/rules/core.md`, `core/src/nets/**`, `core/src/search/**`.

## Goal

`search::net`: axiom-linking enumeration over a step 5 proof structure with
the O(1) rejections, the exact acyclicity test at the spec's cadence, the
final correctness check and sequentialization into a checked proof term. It
becomes the dispatch row for unit-free MLL with or without Mix (D8), and it
agrees with the focused engine on every input.

## What to build

1. **Preprocessing**: the count equation `c = t − p + 2` and the per-atom
   balance (with Mix the inequality), rejecting early.
2. **Search**: minimum-remaining-values choice of the next unlinked literal
   (candidate counts kept in a small array updated on link/unlink), the two
   O(1) rejections (the lowest common ancestor within one conclusion is a
   `⊗`; same component in the ⅋-free skeleton), the exact test every link
   when `V ≤ 200` and every fourth link otherwise (tunable through
   `Options`), backtracking with an explicit stack and the step 5
   snapshot/rollback.
3. **On a complete linking**: the step 5 criterion, then sequentialization;
   return the term (it passes the step 2 checker; `debug_assert!` it).
4. **Symmetry breaking** for equal conclusions as the spec allows, if it is
   simple; otherwise note it as future work.
5. **Dispatch**: unit-free MLL goes to `net` by default; `--engine focus`
   still reaches the focused engine; `--engine net` on a non-MLL input is a
   clear error. Update the fragment/engine line the CLI prints. `--format
   net` on a net-engine result prints the net that was found, without the
   round trip through the derivation.
6. **Tests**: differential testing against the focused engine on random
   unit-free MLL sequents (the step 3 generator restricted, plus random
   *unprovable* sequents with balanced atoms so the counts do not reject
   them), the classic examples, sequents where every atom is distinct
   (decided without backtracking; assert the node count), and a Matsuoka or
   Kanovich hard-family instance as an ignored slow test.
7. **Documentation**: `.claude/rules/core.md` gets the search's invariants
   (when each rejection is sound, the cadence, the MRV order).

## Constraints

- No `unsafe`. Per-problem allocation once, per-link work O(1) amortised
  where the spec says so; no `String`s or `Vec<Vec<_>>` in the hot path.
- Deterministic results for the same input and options.
- The spec's "Pitfalls" list is a checklist; go through it in the report.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`cargo deny check` if a dependency changed, `nix flake check` at the end
(`jj st` first). Report timings of net versus focus on the differential
sample in release mode, and on the hard-family instance.

## Deliverables

- Thematic jj commits ("Search axiom linkings with incremental pruning",
  "Route unit-free MLL to the net engine", "Compare the net and focused
  engines on random sequents", …).
- `plan/reports/06-net-search.md`: API, the pitfalls checklist with how
  each is handled, timings, decisions, deviations, open questions, and what
  step 8 needs for the IMLL embedding and step 12 for cube-and-conquer.

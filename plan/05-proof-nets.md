# Step 5: proof nets as a representation

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D5, D6, D6a, D12) and every report in `plan/reports/`.
- `proof-search-specifications.md`: "MLL (unit-free multiplicative linear
  logic)" in full, in particular "Colored structure graph", "Exact
  acyclicity test", "Connectedness", "Sequentialization", "Pitfalls" and
  "Data layout"; "MLL variants" for Mix; the "Proof-net checks" bullet under
  "Testing strategy". Step 6 will build the *search* on what you build here;
  read its prompt `plan/06-net-search.md` so the structures serve it (the
  incremental use of the acyclicity test on partial structures, the
  ⅋-free skeleton, the union-find).
- `plan/notes/export-targets.md` § Crates for `ena`.
- `.claude/rules/core.md`, `core/src/proofs/**`, `core/src/occurrences/**`.

## Goal

`nets`: proof structures for MLL (with or without Mix) over the occurrence
forest, a correctness checker independent of any search, sequentialization
into a proof term that the step 2 checker accepts, and desequentialization
of an MLL derivation into a net. Plus a textual representation of a net
(for the CLI's `--format net` and for tests); its graphical form is step 11.

## What to build

1. **Proof structures.** A `ProofStructure` over a forest: the formula
   trees are already there; a structure adds the axiom links (`partner:
   Vec<OccId>` with a sentinel for unlinked literals, plus the derived list
   of links) and knows whether it is complete (every literal linked) and
   whether Mix is allowed. Constructors: empty; from a list of links (with
   validation: dual literals of the same atom, each literal at most once);
   `link`/`unlink` that step 6 will call in a hot loop, so keep them O(1)
   and allocation-free, with the coloured graph and the ⅋-free skeleton
   union-find (with snapshot/rollback, `ena` if it fits) maintained
   incrementally as the spec's search needs them. Decide what lives in
   `ProofStructure` and what in step 6's search state; document the split.
2. **The coloured structure graph and the criterion.** The graph exactly as
   the spec describes (⅋ premise edges share a colour unique to their ⅋,
   every other edge its own colour), as CSR adjacency with a scratch
   `deleted` bitset; the Yeo-theorem deletion procedure as the exact
   acyclicity test, meaningful on partial structures; the connectedness
   equation `E_sw = V − 1` for MLL without Mix, checked only after
   acyclicity. `is_correct(&self) -> Result<(), NetError>` with an error
   that names a switching cycle or the disconnection. Write an independent
   second criterion as a test helper (Danos contractibility, or
   Danos–Regnier: enumerate switchings when small and check each is an
   acyclic connected graph) and assert they agree on generated structures.
3. **Sequentialization** (net → proof term): by the splitting-tensor lemma
   as the spec states (peel ⅋ conclusions, find a splitting ⊗ by deleting
   it and counting components under one switching, recurse), O(n²) total,
   emitting `Par`, `Ax`, `Tensor` and `Mix` nodes through the step 2
   builder; the resulting term must pass the step 2 checker (assert in
   tests, `debug_assert!` in the function). With Mix, a structure may split
   into components without a splitting ⊗: emit Mix.
4. **Desequentialization** (derivation → net): from a proof term of an MLL
   derivation, read off the axiom links (each `Node::Ax` names its two
   literal occurrences; `Proof::nodes()` lists them, and the derivation
   view's `Rule::Ax` inferences carry the same pairs), build the structure
   and assert it is correct. Two
   derivations that differ only by rule permutations give the same net:
   test it.
5. **Text form.** A readable listing of a net: the conclusions as
   formulas, the links as pairs of literals with their positions (e.g.
   `a[3] — a⊥[7]`), the verdict of the criterion. This is what
   `--format net` prints in the CLI: a variant of `Format` in
   `cli/src/argument_parsing.rs` with a doc comment and its arm in the
   `match format` of `prove` in `cli/src/prove.rs` (`plan/reports/04-api-and-cli.md`,
   "What a later step does to add an engine or an output format"); the
   rendering itself lives in `core`. For a proved MLL sequent the
   derivation is desequentialized and the net printed; for other fragments
   the format is rejected with a clear message. serde for nets if it is a
   few lines (links plus the sequent).
6. **Documentation.** `.claude/rules/core.md`: the net model, the graph
   colouring and why ⊗ premise edges must not share a colour, when each
   check is sound, the relation between nets and terms. CLAUDE.md if
   commands or modules changed. README's feature list.

## Constraints

- No `unsafe`. Per-structure allocation once; link/unlink O(1) amortised;
  the criterion allocation-free apart from its scratch bitset.
- The criterion is a checker in the sense of D6a: independent of the search
  that step 6 writes and of the proof checker of step 2; its result is the
  ground truth for nets the way the step 2 checker is for terms.
- The spec's "Pitfalls" list is a checklist; go through it in the report.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`cargo deny check` if a dependency changed, `nix flake check` at the end
(`jj st` first). Tests: the classic nets (the axiom-tensor-par examples, a
cyclic structure, a disconnected structure with and without Mix), random
derivations from the step 3 generator restricted to MLL round-tripping
derivation → net → derivation with the checker at both ends, and the two
criteria agreeing on random linkings of random sequents (correct or not).

## Deliverables

- Thematic jj commits ("Add proof structures over the occurrence forest",
  "Add the coloured graph and the Yeo acyclicity test", "Sequentialize
  proof nets", "Desequentialize MLL derivations", "Print proof nets", …).
- `plan/reports/05-proof-nets.md`: the API (types, what step 6 calls in its
  hot loop and its costs), the pitfalls checklist, decisions, deviations,
  open questions, and what steps 6, 10 (drawing a net) and 14 (boxes for
  MELL) need to know.

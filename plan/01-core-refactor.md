# Step 1: refactor the core for proof search

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md`: the whole plan, and the design decisions D1 to D12, which
  bind this step. This step realises D1, D2, D3, D4 and D5.
- `proof-search-specifications.md`, the sections "Purpose, scope and
  conventions" and "Common infrastructure (shared by all engines)". Skim the
  rest so the data structures you build carry every engine (the "Data
  layout" subsections say what each needs).
- `.claude/rules/core.md` (loads when you read `core/`), which describes the
  data model you are changing, and `core/src/**`, `cli/src/**`, both small.

## Goal

After this step, the `linlog` crate has the data model every later step
builds on, with parsing, printing, serialization and the existing tests still
working, and nothing of proof search itself yet.

## What to build

1. **Simplify the generics (D2, D3).** Remove the `Logic` type parameter,
   the `LL`/`MLL` marker types, the `Index` trait and the `I` parameter. The
   arena index becomes `u32` behind a newtype; the variable index likewise.
   `subenum` leaves `Cargo.toml`. `Sequent` is a plain type. Keep the
   invariants (topological arena, hash-consing, `optimize`, `verify_integrity`)
   and the parser, `Display` and serde behaviour; the JSON format of a sequent
   must not change (the tests in `core/tests/parse.rs` and the serde round
   trips pin it; add a serde round-trip test if none exists).
2. **Fragments and modes (D2).** A `Fragment` value describing which
   connective classes a sequent uses (multiplicatives, multiplicative units,
   additives, additive units, exponentials; make the representation cheap to
   compare and to print, e.g. a small flags type with named constants such as
   `MLL`, `MALL`, `MELL`, `LL`, and a `Display` giving the usual names). A
   `Mode` value: classical or intuitionistic, linear or affine, Mix on or
   off. `Sequent::fragment()` computes the smallest fragment in one pass.
   The intuitionistic shape test of D1 belongs to step 8, but design
   `Fragment`/`Mode` so it slots in.
3. **The occurrence forest (D5).** A type built from a `&Sequent` with the
   spec's layout: DFS preorder numbering over the roots in `term_ids` order,
   per occurrence its arena term, kind, parent, children, subtree size,
   depth, polarity (positive `⊗ 1 ⊕ 0 !`, negative `⅋ ⊥ & ⊤ ?`, atoms by
   bias), and for literals the atom and sign; per atom the lists of positive
   and negative literal occurrences; the atom bias rule of the spec (the
   literal with fewer occurrences is positive; ties broken deterministically)
   computed once per forest. Provide the queries the engines need: subtree
   range, lowest common ancestor within one root (a parent walk is fine at
   these sizes), the root of an occurrence, the formula of an occurrence as a
   `Display`able view. Occurrence ids are a `u32` newtype. Steps 5 and 10
   will draw the forest (proof nets, SVG), so a stable, documented child
   order matters.
4. **Bitsets over occurrence ids.** Choose between `fixedbitset` and a small
   hand-written `Box<[u64]>` type; adopt a crate through the `new-tool` skill.
   Whatever you pick must hash fast (a `foldhash`/`rustc-hash` hasher, adopted
   the same way; see `plan/notes/export-targets.md` § Crates) and support the
   operations the spec's engines use: insert, remove, contains, union,
   difference, iteration in id order, `is_empty`, count, submask enumeration
   over a compacted member list (the spec's MALL section, "Pitfalls").
   Keep the per-problem width; the spec's `u64` specialisation for `n ≤ 64`
   is an optimisation for later, unless it comes for free.
5. **Layout (D4).** Replace `core/src/linear/` with the module layout of D4
   (empty `search`, `proofs`, `nets` and `export` modules with doc comments
   are fine as placeholders; do not stub engines). Re-export the public API
   from `lib.rs` so a user writes `linlog::Sequent`, `linlog::Fragment`, etc.
6. **Documentation.** Rewrite `.claude/rules/core.md` for the new model
   (the file is what a future session relies on; keep its style: invariants
   and the things the code cannot say). Update CLAUDE.md's crate paragraph
   and README's architecture paragraph where they no longer hold. Update
   `.claude/agents/crate-source-explorer.md` for crates added or removed.

## Constraints

- No `unsafe`. No dependency for its own sake; each one scoped to the crate
  and feature that uses it, with only the crate features used.
- Performance is a goal: compact arrays of `u32`, no per-occurrence heap
  allocation, no `String` inside the forest.
- Human-readable API: short names, doc comments that say what a thing is,
  and a doc example on `Sequent::fragment()` and on building a forest.
- `core` must stay buildable for wasm in principle (D11): no threads, no
  `std::time`, no OS access in this step.
- If a decision in `plan/README.md` turns out wrong while you implement it,
  say so in the report and choose the fix that keeps the later steps intact;
  do not silently deviate.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`
(features changed), `cargo deny check` (dependencies changed), and
`nix flake check` before you finish (run `jj st` first so nix sees new
files). Add unit tests for the forest (preorder ranges, parent/child
consistency, literal lists, bias) and for fragment detection on a table of
sequents.

## Deliverables

- Thematic jj commits, e.g. "Drop the Logic and Index type parameters",
  "Add Fragment and Mode with detection", "Add the occurrence forest",
  "Adopt fixedbitset for occurrence sets", "Document the new core model".
- `plan/reports/01-core-refactor.md`: what was built, the decisions taken
  where this prompt left room (bitset crate, hasher, layout), deviations from
  the spec or the plan with reasons, open questions, and what steps 2 and 3
  must know (type names, module paths, how to build a forest and iterate
  literals). Commit it with the last change.

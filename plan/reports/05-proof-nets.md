# Step 5 report: proof nets as a representation

Session of 2026-09-29, from `plan/05-proof-nets.md`. Realises plan decision
D6a: proof nets of unit-free MLL, with or without Mix, as a second
first-class representation with its own correctness checker, converted to
and from proof terms.

## Outcome

`linlog::nets` exists: `ProofStructure` is a proof structure over the
occurrence forest of its sequent, built empty, from a validated list of
links, or from a proof; it links and unlinks literals in constant time
while keeping the coloured structure graph and the `⅋`-free skeleton
current; `is_correct` decides the Danos–Regnier criterion through Yeo's
deletion test and names a switching cycle or the parts a structure falls
into; `sequentialize` turns a net into a proof term the step 2 checker
accepts; `from_proof` reads the net off an MLL proof; `Display` prints the
sequent, the links with their positions and the verdict; serde gives the
net a JSON form. The CLI's `prove` and `check` have `--format net`. Seven
commits, "Add proof structures over the occurrence forest" to "Document
proof nets", then this report. All checks pass at the last change:
`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace` (61 unit tests in core with 2 ignored, 9 parse and
9 serialize integration tests, 9 doc tests, 2 unit and 5 integration tests
in the CLI), `cargo test -p linlog --no-default-features`,
`cargo hack check --feature-powerset -p linlog`, and `nix flake check`
("all checks passed!"). No dependency was added.

## The API

Everything is re-exported from the crate root; the module is
`linlog::nets`.

| item | role |
|---|---|
| `ProofStructure::new(forest, mix)` | the empty structure; `Error::NetFragment` outside unit-free MLL |
| `ProofStructure::from_links(forest, mix, &[(x, y)])` | validated: literals, dual, each once, in range (`NetError`, wrapped as `Error::InvalidNet`) |
| `ProofStructure::from_proof(&proof, mix)` | desequentialization; the net of the `Ax` nodes, returned only if correct |
| `link(x, y)`, `unlink() -> Option<(x, y)>` | push and pop a link; O(1) plus one union-find `find` |
| `partner(o)`, `links()`, `is_complete()`, `unlinked()` | the linking as it stands |
| `same_component(x, y)` | the skeleton's rejection: a link would close a cycle no switching breaks |
| `scratch() -> Scratch` | working memory for the tests, allocated once |
| `is_acyclic(&mut Scratch) -> bool` | the Yeo test, allocation-free, meaningful on partial structures |
| `is_correct() -> Result<(), NetError>` | the criterion with witnesses: `Empty`, `Unlinked`, `SwitchingCycle(Vec<OccId>)`, `Disconnected(Vec<Vec<OccId>>)` |
| `sequentialize() -> Result<Proof, NetError>` | net to term, `debug_assert!`-checked |
| `Display`, serde | the text form and `{"sequent", "mix", "links"}` |
| `NetError`, `NetError::describe(&forest)` | ids, or formulas as `~A[0]` |
| `Error::InvalidNet(NetError)`, `Error::NetFragment(Fragment)` | the crate-level wrappers |

What step 6 calls in its hot loop, and what it costs, with `n` occurrences,
`V + E` about `2.5 n`:

- `link(x, y)`: two `partner` writes, two CSR slot writes, a push, and one
  union by rank, whose two `find`s walk at most `log n` parents. No
  allocation once built (the link stack has its capacity).
- `unlink()`: the reverse, with one undo-log pop. Links must be undone in
  reverse order of their making; a `debug_assert!` does not check this,
  the undo log simply undoes the last union, so an out-of-order unlink
  would corrupt the skeleton silently. A backtracking search never does
  that.
- `same_component(x, y)`: two `find`s.
- `Forest::lca(x, y)`: a parent walk from `x`, `None` across roots.
- `is_acyclic(&mut scratch)`: one depth-first search over the whole graph
  per round, rounds being the nesting depth of cycles through both
  premises of a `⅋`; measured in release mode on chains of `(a ⅋ ~a)`
  blocks, about 30 ns per occurrence (n = 3199: 92 µs), the same on
  correct and cyclic structures.
- `is_correct()`: allocates a `Scratch`; `is_acyclic` plus, on failure, the
  witness. The cycle witness runs one deletion procedure per edge among
  the stuck vertices, so a cycle through the whole net costs quadratic
  time (803 vertices: 12.6 ms), a short one microseconds; call it once per
  complete linking, or use `is_acyclic` and the count when only the
  verdict matters.
- `sequentialize()`: O(n²) (n = 3199: 15 ms; n = 9599: 122 ms), allocates
  the proof and one scratch. It recurses as deep as the derivation is
  high, so step 6 runs it on its search thread.

## What lives where

The structure holds what the criterion and the conversions need and what a
search must keep consistent on every link: `partner`, the link stack, the
CSR graph and the skeleton union-find with its undo log. A search keeps
its own candidate counts per literal, the choice order, the explicit
stack of decisions, the statistics, and one `Scratch`. The two O(1)
rejections are the structure's `same_component` and the forest's `lca`;
the structure does not apply them in `link`, because a search wants to
count and order candidates before linking.

## The criterion, as implemented

The graph is the spec's: occurrences as vertices, premise edges of `⊗` and
`⅋` and the links as edges, in CSR layout with a vertex's parent edge in
its first slot, its children next, and for a literal an axiom slot last
that `link` fills. The colouring (a `⅋`'s two premise edges share a colour
of their own, every other edge has its own) is not stored: with it, Yeo's
deletion condition reduces to bridges. A vertex that is not a `⅋` sees
distinct colours on all its edges, so "no component of `G − z` meets `z`
in two colours" says every neighbour is in a component of its own, that
is, every incident edge is a bridge; a `⅋` sees its premise colour twice
and its parent's once, so the condition says the parent's component
holds neither premise, that is, the parent edge is a bridge, or is absent.
One iterative Tarjan search per round computes the bridges; a round
deletes every vertex the bridges allow (deleting several at once is sound
because deletability only grows as vertices go); when a round deletes
nothing, the vertices left carry a properly coloured cycle. The
connectedness equation `2t + p + k = V − 1` is tested after acyclicity
only, and not at all with Mix. A switching cycle is isolated for the error
by removing, among the stuck vertices, each edge in turn and keeping it out
when a cycle survives; an edge found necessary stays necessary as edges go,
so one pass leaves exactly one cycle, which is walked. The disconnection
witness is the components of the switching that keeps every left premise,
each named by its vertices without a parent edge there.

The second criterion, in `nets::graph`'s tests, enumerates every switching
(up to sixteen `⅋`s) and tests each for a cycle and for connectedness with
a union-find. On 400 random linkings (random pairings, partial and
complete, of generated provable sequents, sequents that need Mix, and
mutants), with and without Mix, the two agree on acyclicity after every
link and on the verdict at the end, every named cycle is a simple cycle
that passes no `⅋` through both premises, and every disconnection has
`V − E_sw` parts, each root in exactly one; the sample had 164 nets, 25
cycles, 11 disconnections and the rest incomplete.

## Sequentialization

By the splitting-tensor lemma, on the plain graph of a sub-net (its formula
trees and links, no switching). Per stage: every `⅋` conclusion is opened
and its rule emitted below the rest; one depth-first search from the
conclusions gives the connected parts and the bridges; with Mix, several
parts are proved separately and joined with binary Mix; two literals are
an axiom; otherwise the `⊗` conclusion with the smallest id whose premise
edge is a bridge is splitting, the conclusions reached from its left
premise go to the left premise's net and the rest to the right. Handled
conclusions are deleted in the scratch, so a sub-net is what its
conclusions reach. The random round trip (60 generated MLL derivations per
mode, found by the focused engine): the proof's net is correct, its
sequentialization passes the checker, and has the same net.

## Desequentialization

`from_proof` reads the `Ax` nodes of the arena (each names its two
literals) and builds the structure with `from_links`, so an `Ax` on
non-literal or non-dual occurrences is refused; then the criterion runs,
and the net is returned only if correct, which every proof the checker
accepts gives. The test pins that two derivations of
`⊢ A ⊗ B, ~A, ~B ⊗ C, ~C` that apply the tensors in either order give the
same links, that a proof with Mix is a net only with `mix`, and that a
proof outside MLL is `Error::NetFragment`. The derivation view's `ax`
inferences were not used: the nodes are one indirection fewer.

## The text and JSON forms, and the CLI

```
⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C
A[1] — ~A[6]
B[2] — ~B[7]
C[4] — ~C[8]
proof net
```

The sequent, one link per line sorted by the first id, the literal printed
as the forest prints it with its occurrence id in brackets, then `proof
net`, `proof net with Mix`, or `not a proof net: ` and the reason with
formulas. JSON: `{"sequent": …, "mix": false, "links": [[1,6],[2,7],[4,8]]}`,
reading back with the links validated. `linlog prove --format net` prints
the verdict line and then this for a proved sequent; `linlog check --format
net` the same after `valid proof of …`. Both refuse, before searching, a
sequent outside unit-free MLL (with core's `Error::NetFragment` message)
and affine or intuitionistic mode.

## The pitfalls checklist

- *Counts are necessary conditions for MLL only.* The count that this step
  uses, the connectedness equation, is applied where nets exist only,
  since `ProofStructure::new` refuses every other fragment; the count
  prunes themselves are step 6's.
- *Atomic axioms only.* A link joins two literals (`NetError::NotLiteral`),
  and `from_proof` reads `Ax` nodes, which the checker accepts on literals
  only.
- *The LCA rejection within one conclusion only.* `Forest::lca` is `None`
  across roots; the rule is restated in `.claude/rules/core.md` for step 6.
- *`⊗` premise edges get distinct colours.* The colouring is implicit; the
  reduction to bridges is only valid because every edge but a `⅋`'s
  premises has a colour of its own, and `⊢ A ⊗ ~A` with its link is a
  pinned cycle. `⊢ A ⅋ ~A` pins the other direction.
- *Connectedness after acyclicity.* `is_correct` tests the equation only
  after the deletion procedure emptied the graph.
- *Contexts as sets of ids.* Links are between occurrence ids; two
  occurrences of one formula are distinct literals.
- *Units break the linking model.* `Error::NetFragment` for `MLL with
  units`, before any link.
- *Recursion depth.* Nothing in the criterion recurses (the searches are
  iterative). `sequentialize` recurses to the derivation's height, and the
  CLI does not call it; step 6 must call it on its search thread.

## Decisions where the prompt left room

- **No `ena`.** The skeleton needs union, find and undo of the last union,
  nothing keyed or valued; the spec's own data layout is an undo log of
  what a union changed. That is sixty lines with a test (`skeleton.rs`),
  against fetching a crate, wrapping `OccId` in its key and value traits
  for a structure that carries no value, and taking on its dependency
  tree. `ena` remains the candidate if a later step needs snapshots of
  many unions at once.
- **Colours are not stored.** The bridge reduction makes them a comment;
  storing a `u32` per slot that nothing reads would only invite drift
  between the stored colour and the rule the test applies.
- **Deletability through bridges, in rounds**, rather than a per-vertex
  component count: one linear search per round decides every vertex at
  once, and the number of rounds is small in practice (one for a forest
  of trees with a few links; the nesting depth of `⅋`-through-both-premises
  cycles in general).
- **Witness by minimisation**, not by a constructive proof of Yeo's
  theorem: the general problem of finding a properly coloured cycle needs
  matching machinery, while re-running the exact test per edge is short,
  obviously right given the test, and runs only on the error path.
- **`ProofStructure` is the type**, not `ProofNet`: the plan's `ProofNet`
  is a structure that passed `is_correct`. A newtype for "verified" was not
  added; `sequentialize` re-runs the criterion, which is cheap next to the
  sequentialization.
- **`Scratch` is explicit** for the hot-loop test and allocated internally
  by `is_correct`, so the checker's signature stays `&self` and the search
  pays nothing per test.
- **The links are a stack** with `unlink()` taking no arguments: the undo
  log is LIFO, and an API that pretended to unlink any link would either be
  O(n) or wrong.
- **An axiom is `Ax(min, max)`** and the splitting `⊗` is the lowest id,
  so sequentialization is deterministic and does not depend on the order
  links were made.
- **`Empty` is its own error**: `⊢` is not provable, with or without Mix,
  and the acyclicity of nothing would otherwise call it a net with Mix.
- **`Disconnected` names parts by their topmost vertices** under the left
  switching, since a part may hold no root (a right premise under a cut).
- **`mix` is a parameter of `from_proof`**, not read off the proof: the
  mode is the user's choice, and a Mix-free proof found in Mix mode is
  still a net with Mix.
- **The CLI refuses `--format net` before searching**, using core's error
  value for the fragment message, and refuses affine and intuitionistic
  mode with a message of its own; `check --format net` is the same after
  the check.

## Deviations from the spec or the prompt, with reasons

- **The spec's splitting test is wrong.** "Delete it and count components
  under one arbitrary switching (exactly two ⇒ splitting)" cannot
  distinguish: a correct net without Mix is a tree under every switching,
  so deleting any `⊗` conclusion, which loses two edges, leaves exactly two
  components whether or not it is splitting (`⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C`:
  the first tensor is not splitting, the second is, and both leave two
  components under either switching). The lemma is about the plain graph:
  a `⊗` is splitting iff its premise edge is a bridge there. This is what
  the code does and what `.claude/rules/core.md` records.
- **The spec's "O(n²)" is met** through one bridge search per stage rather
  than one component count per candidate, which would have been cubic.
- The spec's `colors: u32` are not stored (above).
- The report's numbers come from release builds of a scratch program
  outside the repository, not from a committed benchmark; step 13 owns
  benchmarks.
- No decision in `plan/README.md` turned out wrong. D12's drawing
  convention (formula trees downwards from the conclusions, links as arcs
  above the literals) is what the preorder and `links()` give directly.

## Review

A fresh-context reviewer (a sub-agent that had not seen the design being
made) read the spec's MLL section and the code, argued each claim, and
wrote an independent harness in a throwaway crate outside the repository:
brute-force enumeration of switchings with its own union-find (exact on
partial structures too, over the `⅋`s whose premises reach a link), Danos
contractibility for complete structures without Mix, cross-validated
against the enumeration, random switchings for large Mix nets, and the
step 2 checker as the certificate for every sequentialized proof. It
compared 413 745 structures: random formula trees over one to four atoms
with random full and partial pairings, random cut-free derivations with
their own linking, partner-swap mutants and fresh re-pairings, Mix-heavy
and `⅋`-heavy derivations, nets of up to 395 occurrences and sequents of
up to 199 literals, and hand probes (the empty sequent, one axiom, one
unlinked literal, repeated roots, tensor chains of 200 000 conclusions, a
Mix net of 20 000 parts, cyclic chains of 9000 vertices). Per structure it
ran `is_acyclic` after every link (2.4 million incremental checks),
`same_component` against a brute-force skeleton, `is_correct` against the
reference, every witness's validity, sequentialization with the checker
and the equality of links, `from_proof` back, `Display`, then unlinked
everything checking the verdicts on the way down, and relinked in a
shuffled order. No disagreement, no invalid witness, no panic; all
172 735 correct nets sequentialized into checked proofs with the net's
links (about 250 000 Mix nodes emitted, never for a connected net).

It confirmed each claim with an argument: the colouring's equivalence
with switchings (the graph is simple, and the only same-colour pair at a
vertex is a `⅋`'s premises); the bridge form of Yeo's condition (a `⅋`
premise edge at a non-`⅋` vertex is one of a pair whose other member is at
the sibling); batching within a round (a bridge of the round's graph stays
one as vertices go, so every deletion is Yeo-deletable in the current
graph, and Yeo's theorem supplies progress); the witness minimisation
(kept-in edges lie on every later cycle, so exactly one survives and the
last successful test leaves its vertices as the stuck set); the count;
the parts (every component has a cut vertex); the splitting test (the spec
is wrong as stated, and should be corrected at "counting components under
one arbitrary switching"); and the Mix case (Fleury–Retoré: a connected
acyclic structure without `⅋` conclusions sequentializes, and its last
rule can only be a splitting `⊗`).

Its fragilities, none a soundness issue, and what was done: `is_bridge`
now debug-asserts that the last search reached both ends, and `search`
that the scratch belongs to the graph; `from_proof`'s doc says the checker
is not run, so the net of a rejected term can still be a net. Left as
follow-ups below: `sequentialize` recursing to the derivation's height
(50 000 conclusions overflow an 8 MiB stack; 20 000 take 5.5 s), and
`is_correct`, hence `Display` and `from_proof`, always isolating the
witness on a cyclic structure (a 9000-vertex cycle costs about a second).

## Open questions and follow-ups

- **A linear-time criterion** (Nguyễn's unique perfect matching, or
  Guerrini's contraction) if the rounds ever show up in step 6's profile;
  the bridge-based test is linear per round, and the rounds are few on
  every structure measured.
- **Sequentialization is quadratic** through one full-width search per
  stage (`disc.fill` over all vertices, and one search per stage over the
  sub-net); Guerrini's linear algorithm is the upgrade if a 10 000-node
  net's 122 ms ever matters.
- **`Display` runs the criterion**, allocating a scratch and, for a cyclic
  structure, isolating the witness; fine for a human-facing form. A
  boolean fast path for callers that want no witness is `is_acyclic` plus
  the count, which step 6 uses; `is_correct` could take a flag if a caller
  wants the verdict on huge wrong structures.
- **`sequentialize` recurses** to the derivation's height; the CLI never
  calls it, and step 6 runs it on the search thread, but an iterative
  version is the durable fix if a library user hits the main stack's limit
  (the reviewer's figure: 50 000 conclusions).
- **The spec's sequentialization paragraph** should be corrected by the
  planning session, as step 3's corrections were recorded: a splitting `⊗`
  is found by a bridge test on the plain graph, not by counting components
  under a switching.
- **`links()` is in link order**; only `Display` sorts. A canonical order
  for comparing nets is a sort by the caller (the tests do it).
- **Nullary Mix** stays out: the empty net is an error, matching step 2's
  decision that `⊢` has no proof.
- **Symmetry breaking and candidate ordering** are step 6's; the structure
  offers nothing for them beyond `unlinked()` and the forest's literal
  lists.

## For step 6

- Build one `ProofStructure::new(forest, mode.mix)` and one `scratch()`
  per problem; `link`/`unlink` in stack order; `same_component` and
  `Forest::lca` for the O(1) rejections before linking; `is_acyclic` at
  the spec's cadence; on a complete linking `is_correct` and then
  `sequentialize` on the search thread, and `debug_assert!` the proof.
  `from_proof` is the way to print the net of a focused-engine proof;
  `--format net` on a net-engine result should print the found structure
  directly, which is `Display` on it.
- The count equation and the per-atom balance are not in the structure.
  The criterion's connectedness equation `2t + p + k = V − 1`, with
  `V = 2k + t + p`, is `k = t + 1`, and with the tree identity
  `c = 2k − t − p` that is the spec's `c = t − p + 2`: the count equation
  is the connectedness equation of a complete linking. Step 6's
  preprocessing is therefore the same test before any link, plus the
  per-atom balance, and costs nothing from the structure.
- `Statistics` wants counters for links tried and exact tests run
  (`#[non_exhaustive]`, with lines in `serialize/search.rs`).

## For step 10

- A net is the forest (preorder, `left`/`right`, `depth`) plus `links()`;
  the drawing of D12 is the formula trees from the roots downwards and one
  arc per link above the literals. The text form's `[id]` positions are
  the occurrence ids, which the SVG can label.

## For step 14

- Boxes need vertices that are not occurrences (a box's border, the
  auxiliary doors), which the CSR graph over `forest.len()` vertices does
  not have; the criterion for MELL nets with boxes treats a box as one
  vertex of the enclosing net, so the graph would be built per depth
  level rather than extended. The bridge reduction of the deletion
  condition holds for any colouring where only `⅋`-like vertices repeat
  a colour.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo test -p linlog --no-default-features`,
`cargo hack check --feature-powerset -p linlog`, rustdoc with warnings
denied, and `nix flake check` ("all checks passed!"): all pass at the last
code change, "Document proof nets", on top of which this report sits. The
timings above come from a release build of a scratch program outside the
repository (chains of `a ⊗ (a ⊗ …)` with distinct atoms and of
`(a ⅋ ~a) ⊗ (…)` blocks, cross-linked for the cyclic cases).

# Linear Logic Proof Search: Algorithms per Fragment (Rust implementation spec)

Sep 29, 2026 · @Fabian

## Purpose, scope and conventions

This is an implementation brief for a Rust proof-search engine covering propositional linear logic fragment by fragment. For each fragment it names the algorithm to implement first, the fallback, the known complexity bound, the pitfalls, the data layout and where parallelism pays off. Each fragment section is written so a coding agent can implement it without reading the papers; the references section says where to look when a detail is unclear.

**Syntax.** Classical linear logic (CLL) formulas are kept in negation normal form: atoms `a`, `a^⊥`; multiplicatives `⊗`, `⅋`, units `1`, `⊥`; additives `&`, `⊕`, units `⊤`, `0`; exponentials `!`, `?`. Negation is a defined operation on formulas (De Morgan), never a connective. Linear implication `A ⊸ B` abbreviates `A^⊥ ⅋ B`. Intuitionistic linear logic (ILL) formulas use `⊸`, `⊗`, `1`, `&`, `⊕`, `⊤`, `0`, `!` and are handled two-sided.

**Sequents.** CLL fragments use one-sided sequents `⊢ Γ` (Γ a multiset). ILL fragments use two-sided `Γ ⊢ A` with exactly one formula on the right. Exponential fragments split the left context into an unrestricted zone `Θ` (formulas under `!`, reusable) and a linear zone `Δ` (used exactly once), written `Θ ; Δ ⊢ A` (dyadic sequents, Andreoli 1992). Cut is admissible in every fragment here, so search is always cut-free; cut-elimination is never run during search.

**Polarity and focusing.** Positive connectives: `⊗`, `1`, `⊕`, `0`, `!` and positive atoms. Negative: `⅋`, `⊥`, `&`, `⊤`, `?` and negative atoms. Negative rules are invertible (applied eagerly, no backtracking); positive rules are applied in maximal chains on one chosen formula (focus). Every algorithm below is a focused proof system in Andreoli's sense; unfocused sequent search is never the recommendation because its search space is exponentially larger for no gain in completeness.

**Fragment names.** MLL = `⊗ ⅋` (unit-free unless stated); MLL+units adds `1 ⊥`; MALL adds `& ⊕ ⊤ 0`; MELL = MLL + `! ?` (usually with units); LL = everything. Prefix I = intuitionistic (IMLL/MILL, IMALL/MAILL, IMELL, ILL). LLW/affine = LL plus weakening. Lambek calculus = non-commutative IMLL without empty antecedents.

**How to read each fragment section.** Each has: (1) complexity and decidability, (2) survey of algorithm families with the evidence for ranking them, (3) the recommended algorithm as a specification (inputs, state, rules, termination, output), (4) pitfalls, (5) data layout, (6) parallelization. Recommendations are ranked on published worst-case bounds, published benchmark comparisons where they exist, and engineering simplicity; where no head-to-head benchmark exists this is said explicitly rather than guessed.

## Errata found during implementation

Corrections established while implementing the engines, each confirmed by an
independent reference implementation; the original text below is left as
written.

- **MALL-Seq, immediate failure (a `0` in a stable sequent).** Not fatal
  when some member has a `⊤` below it: `⊢ 0, ⊤ ⊕ b` is provable by focusing
  on the `⊕` and letting the `⊤` absorb the `0`. Focusing on `0` itself
  still fails. The interval row of `⊤` is therefore not `(0, 0)`: a member
  with a `⊤` below it passes any count.
- **MALL-Seq, immediate failure (no positive non-literal formula and not a
  dual pair).** Sound only without Mix: `⊢ a, ~a, b, ~b` is provable with
  Mix. With Mix the sequent falls through to the Mix step.
- **MLL-Seq, forced splits.** Only a positive literal (its dual alone),
  `1` (the empty context) and `0` (no split at all) force a side of a `⊗`
  split; `⊥` and `⊤` do not (`⊢ ⊥ ⊗ b, a, ~a, ~b` needs `{a, ~a}` on the
  `⊥` side, and a `⊤` side accepts any subset while the other side's
  provability is not monotone).
- **MLL-Net, sequentialization.** "Delete the `⊗` conclusion and count
  components under one arbitrary switching (exactly two ⇒ splitting)"
  cannot distinguish: a correct net without Mix is a tree under every
  switching, so deleting any `⊗` conclusion leaves exactly two components
  whether it is splitting or not (`⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C`: only the
  second tensor is splitting, both leave two components). The splitting
  lemma concerns the plain graph: a `⊗` conclusion is splitting iff its
  premise edge is a bridge there. One bridge search per stage keeps the
  O(n²) total.

## Overview: fragment, complexity, what to implement

The literature contains no head-to-head benchmark across algorithm families for any fragment; the only cross-prover comparison found is Chaudhuri–Pfenning (CADE 2005), where a focused inverse method beat Gandalf, linTAP and llprover on their MALL/ILL suite. The rankings below therefore rest on worst-case bounds, the structure of each search space, and the published internal benchmarks (Lygon: lazy splitting 3–15× over naive; Chaudhuri: focusing 1.3×–600× over unfocused). Every recommendation is a focused system; the differences are in how the ⊗-split and the exponentials are managed.

| Fragment | Complexity (source) | Implement first | Fallback / alternative | Why |
| --- | --- | --- | --- | --- |
| MLL (unit-free) | NP-complete (Kanovich 1992) | Proof-net search: enumerate axiom linkings with count invariants and incremental acyclicity pruning; final near-linear correctness check (Danos contractibility; Guerrini 1999/2011) | Focused sequent search over occurrence bitsets with memoization | Choice points are only axiom pairings; sequents whose atoms are all distinct are decided in linear time |
| MLL + Mix | NP-complete | Same as MLL with the connectedness condition dropped; correctness = uniqueness of the perfect matching (Nguyễn 2020, linear time) | Bitset sequent search with the mix rule | Correctness is a pure matching property |
| MLL + units (1, ⊥) | NP-complete (Lincoln–Winkler 1994) | Focused bitset sequent search with memoization; units are rules, not net nodes | Proof-net search after eliminating units where possible | No canonical proof nets with units (Heijltjes–Houston 2016); provability is still NP |
| Cyclic MLL / Lambek L, L\* | NP-complete (Pentus 2006); L() and L(/) in P (Savateev) | Planar proof-net search (planarity prunes linkings); cubic DP for one-division fragments | Chart parsing O(n⁵) for bounded order (Fowler 2008) | Non-commutativity removes most linkings |
| ALL (additives only) | Linear time (Heijltjes–Hughes 2015) | Direct linear algorithm via additive proof nets / Petri-net construction | Bitset search (trivially small) | Special case, cheap to include |
| MALL (± units) | PSPACE-complete, already for unit-free and intuitionistic (LMSS 1992) | Focused depth-first search over occurrence bitsets with a memo table of proved/failed sequents; interval count invariants; explicit ⊗-split enumeration filtered by counts | (a) Lazy I/O contexts with strict/lax zones (RM3, Cervesato–Hodas–Pfenning 2000) when contexts are large; (b) focused inverse method (Chaudhuri 2006) for theory-heavy inputs | Sequents are sets of subformula occurrences, so memoization is exact and & duplication is free |
| MELL | Decidability open since 2004; refuted claim 2015/2019; positive claim in an unrefereed July 2026 preprint; TOWER-hard regardless (Lazić–Schmitz 2015) | Focused search on dyadic sequents (unrestricted zone + linear zone) with iterative deepening on the number of copy steps; memo keyed by (bound, zone contents); answers Proved / Unprovable / Unknown | Focused inverse method with subsumption (complete for provable sequents); Petri-net reachability engine for the !-Horn fragment | No decision procedure with a usable bound exists; the copy bound is the only knob |
| Affine LL (LLW) and affine fragments | Decidable (Kopylov 1995); TOWER-complete (Lazić–Schmitz 2015); affine MLL NP-complete | MELL engine plus weakening, with the supermultiset-ancestor prune (Dickson's lemma gives termination) | Kopylov's normal-form procedure (not practical) | Weakening turns the loop check into a decision procedure |
| IMLL / MILL | NP-complete (Kanovich 1992 Horn fragment) | Essential-net search (Lamarche) with directed acyclicity + dominator condition, incremental via transitive closure (Moot 2008) | Embed into MLL (conservative for the multiplicative fragment) | Directed nets make incremental exact checks cheap |
| IMALL / MAILL | PSPACE-complete (LMSS 1992) | Two-sided focused bitset search with memo; RM3 is the lazy alternative | Inverse method | Same engine as MALL, one succedent |
| IMELL, full ILL | IMELL open; full ILL undecidable (LMSS 1992; Forster–Larchey-Wendling 2019 for {&, ⊸, !}) | Semi-decision: two-sided MELL engine with copy bound | Inverse method (Sympli-style saturation) | Nothing complete and terminating exists |
| !-Horn fragment | Ackermann-complete (Kanovich 1995 + Petri-net reachability bounds 2019–2021) | Translate to Petri-net reachability; call a reachability engine (e.g., KReach) | MELL engine | Exact correspondence, mature tools |
| Full LL | Undecidable (LMSS 1992) | MELL engine extended with additives (semi-decision) | Inverse method | Bound-limited search only |

Sources for the table are listed in the References section; each fragment section below gives the precise statements.

## Common infrastructure (shared by all engines)

All engines share one representation: a hash-consed formula arena, an occurrence forest per problem, and sequents encoded as bitsets or count vectors over occurrence ids. Build this once and test it independently before any engine.

**Formula arena.** `struct Formula { kind: Kind, a: FId, b: FId, atom: AtomId, hash: u64, size: u32 }` stored in a `Vec<Formula>` indexed by `FId(u32)`; `Kind` ∈ {Atom, NegAtom, Tensor, Par, One, Bot, With, Plus, Top, Zero, Bang, Quest}. Parse into negation normal form; `dual(f)` is precomputed and stored as a field so negation is O(1). Hash-cons with a `HashMap<(Kind,FId,FId,AtomId), FId>` so equal subformulas share an id (needed for memo hits and for the count invariants). Two-sided ILL sequents `Γ ⊢ A` are stored as-is with polarity flags; do not translate to classical form except where a section says the embedding is conservative.

**Occurrence forest.** For one problem, number every subformula occurrence of the sequent in DFS preorder, giving `OId(u32)` in `0..n`. Because it is preorder, the occurrence subtree of `o` is the contiguous range `[o, o + size(o))`, so "the atoms below this ⊗" is a bitmask slice. Store per occurrence: `fid`, `parent`, `left/right child`, `polarity`, `depth`, the index of the ⅋/⊗/&/⊕ node above, and for literals the atom id and sign. Precompute per atom name the lists of positive and negative literal occurrences (used by proof-net search and count checks) and an LCA structure (Euler tour + sparse table, or plain parent-walk since trees are small).

**Sequent encoding.** In cut-free MLL, MALL and their intuitionistic versions, every occurrence appears at most once in any sequent of any proof (rules only decompose; & duplicates the context into two separate premises). A sequent is therefore a *set* of occurrence ids: a fixed-width bitset `[u64; W]` with `W = ceil(n/64)` chosen per problem. For MELL the unrestricted zone Θ is also a set (contraction is implicit), but the linear zone can hold several copies of the same occurrence after repeated copying, so it is a sparse count vector (`SmallVec<(OId, u8)>` sorted by id) or a bitset plus an overflow map. Hash sequents with a fast non-cryptographic hasher (ahash/fxhash) over the raw words.

**Polarity and atom bias.** Positive: ⊗, 1, ⊕, 0, !; negative: ⅋, ⊥, &, ⊤, ?. For each atom *name* choose once which literal is positive (bias); all engines must use the same choice within a problem. Any consistent choice is complete (Andreoli 1992; Chaudhuri–Pfenning–Price 2008 show the choice only changes search behaviour). Default: make the literal with fewer occurrences positive, so fewer formulas are focus candidates.

**Focused proof search skeleton (backward).** State: `⊢ Θ ; Γ ⇑ L` (asynchronous, L a list of negative formulas to decompose) or `⊢ Θ ; Γ ⇓ F` (synchronous, focus on F). Phases:

1. Asynchronous phase: pop the head of L. ⅋ → push both subformulas; ⊥ → drop; ⊤ → success; & → two premises with identical Γ (conjunctive branching); ? → move subformula into Θ; negative atom or positive formula → move into Γ. No backtracking in this phase; order is irrelevant for completeness, so use a fixed order and never revisit.
2. Decide: with L empty, choose a positive formula in Γ (rule D1) or, in exponential fragments, a formula in Θ (rule D2, does not consume). This and the ⊗-split are the only backtrack points.
3. Synchronous phase on the focus F: ⊗ → split Γ into two parts and continue in focus on each factor; ⊕ → choose a side (backtrack point); 1 → Γ must be empty; 0 → fail; ! → Γ must be empty, then release into the asynchronous phase with the subformula; positive atom p → Γ must be exactly `{p⊥}` (initial rule, atomic only; identity on compound formulas is never needed). When the focus becomes negative, release it into L.

Termination in MLL/MALL: every rule strictly reduces the multiset of occurrence sizes, so depth is bounded by n. In exponential fragments only D2 breaks this, which is why the MELL section bounds D2.

**Count invariants (necessary conditions, used as prunes).** For unit-free MLL, a provable sequent with `c` formulas, `t` tensor occurrences, `p` par occurrences and `2k` literal occurrences satisfies `k = t + 1` (each ⊗ merges two subproofs, each axiom starts one) and `p = k + 1 − c`, i.e. `c = t − p + 2`. Per atom name, positive and negative literal occurrences must be equal in number. With Mix, the equalities on `t` and `p` become `c ≥ t − p + 2`; the per-atom balance still holds. With additives the plain balance is *unsound* (⊢ a ⊕ b, a⊥ is provable but b is unbalanced); use interval counts: for each atom, compute per subformula a pair `(min, max)` of (positive − negative) occurrences, with ⊗/⅋ summing intervals and &/⊕ taking the interval hull; the sequent is prunable if 0 is outside the summed interval for some atom (van Benthem/Morrill count invariance with additives). Units and exponentials contribute `(0,0)` for atoms, but exponentials make the invariant unsound for any atom below a `?`, so skip those atoms in exponential fragments.

**Memoization contract.** A memo entry `sequent → Proved(proof) | Failed` is valid without conditions in MLL and MALL because cut-free provability depends only on the sequent. In MELL entries are valid only relative to the copy bound in force: store `Failed(bound_used)` and treat it as a hit only when the current remaining bound is ≤ the stored one. Never memoize across problems with different occurrence numberings.

**Proof output.** Every engine returns a proof term over occurrence ids: `Ax(o1,o2) | Tensor(o, left, right) | Par(o, p) | With(o, p1, p2) | Plus(o, side, p) | One(o) | Bot(o, p) | Top(o) | Bang(o, p) | Quest(o, p) | Copy(o, p)`. A separate checker (no search, linear time) re-derives the sequent from the term; every engine's output must pass it in tests. This is also the export format for Coq/Rocq certificates via Yalla, as Click & coLLecT and LL\_prover already do.

## MLL (unit-free multiplicative linear logic)

**Verdict.** Implement proof-net search (axiom-linking enumeration with an exact polynomial incremental acyclicity test), and keep the bitset sequent engine of the MALL section as the reference implementation and as the engine for units. The reason is structural: a cut-free MLL proof is determined by its axiom linking, so the only choice points are which positive literal pairs with which negative literal of the same atom; when every atom name occurs once with each sign there is exactly one candidate linking and the sequent is decided in linear time by the correctness check alone. Sequent search, even focused with lazy splitting, still branches on ⊗-splits in that case.

**Complexity.** Provability is NP-complete: NP membership because every cut-free proof has one rule per connective occurrence (LMSS 1992), hardness via the Horn fragment ⊗/⊸ over atoms (Kanovich, LICS 1992; ILLC report 1991). Hardness survives severe restrictions: two literals, or one literal plus ⊥, or constants only (Lincoln–Winkler 1994, from 3-Partition); the one-literal Horn fragment is linear time. Proof-structure *correctness* is NL-complete (Jacobé de Naurois–Mogbil 2011) and linear time in practice (Guerrini 1999/2011; Murawski–Ong 2000/2006).

**Algorithm families and the evidence.**

| Family | Choice points | Per-node cost | Evidence |
| --- | --- | --- | --- |
| Focused sequent search, naive ⊗-split enumeration | ⊗-splits (2^\|Γ\|) and axiom choice | O(1) | Click & coLLecT's auto-prover does exactly this with a bitmask over the context and a 3 s timeout (source read) |
| Focused search, lazy I/O contexts (Hodas–Miller 1994; RM1–RM3, Cervesato–Hodas–Pfenning 2000) | Left premise consumes greedily, residue returned | O(1) amortised | Lygon: 3–15× over naive splitting (Winikoff–Harland 1995); no comparison to proof nets |
| Proof-net construction (Galmiche–Perrier 1992; Galmiche 2000; Andreoli 2001; Matsuoka's Proof Net Calculator; Moot 2008 for essential nets) | Axiom pairings only (∏ over atoms of multiplicity!) | Incremental check O(1)–O(n·m), final check linear | Matsuoka reports solving larger encoded NP instances than naive enumeration; Moot's provers parse wide-coverage grammars; no controlled benchmark |
| Connection / matrix method (Kreitz–Mantel–Otten–Schmitt 1997) | Connections + prefix unification | Unification | No experiments published |
| SAT/ILP/ASP encodings | Solver-internal | — | No paper found; only the reverse direction (NP problems → MLL) exists (Matsuoka 2017) |

No published head-to-head exists, so the ranking is: proof nets when atoms are mostly distinct or the context is large; bitset sequent search when contexts are small (≤ 20 formulas) or units are present; I/O contexts when contexts are large and memory is tight.

### Specification: MLL-Net (proof-net search)

**Input.** Occurrence forest of ⊢ Γ; literal lists per atom name. **Output.** `Proved(term)` or `Unprovable`.

**Preprocessing (reject early).** Let `c` = number of conclusions, `t` = ⊗ occurrences, `p` = ⅋ occurrences, `2k` = literal occurrences. Reject unless `c = t − p + 2` and, for each atom name, positive and negative occurrence counts agree. (With Mix: reject unless `c ≥ t − p + 2`.)

**Colored structure graph.** Vertices = all occurrence nodes. Edges: for each ⊗ or ⅋ node, an edge to each premise; for each chosen axiom link, an edge between the two literals. Colors: the two premise edges of a ⅋ share one color unique to that ⅋; every other edge has its own color. A *switching cycle* (a cycle surviving some Danos–Regnier switching) is exactly a properly edge-colored cycle in this graph, because the only way to violate proper coloring is to traverse both premises of a ⅋ consecutively. Unlinked literals are leaves and lie on no cycle, so the test is meaningful on partial structures.

**Exact acyclicity test (polynomial, simple).** By Yeo's theorem, an edge-colored graph with no properly colored cycle has a vertex `z` such that no connected component of `G − z` is joined to `z` by edges of two different colors; such a vertex can be deleted and the property persists. Algorithm: repeat { find a deletable `z`; delete it }; if no vertex is deletable while edges remain, a switching cycle exists. Vertices on a properly colored cycle are never deletable, so the procedure is exact. Cost O(V·(V+E)) naive; the FSCD 2025 paper by Di Guardia, Laurent, Tortora de Falco and Vaux Auclair is the reference connecting Yeo's theorem to MLL sequentialization. Nguyễn (LMCS 2020) gives a linear-time alternative via uniqueness of a perfect matching; use it if the Yeo test shows up in profiles.

**Connectedness (MLL without Mix).** Given acyclicity, every switching is a forest with the same edge count `E_sw = 2t + p + k`; it is a tree iff `E_sw = V − 1`. So DR-correctness = acyclicity + this one equation. With Mix, drop the equation.

**Search.**

1. Maintain `partner: Vec<u32>` (unlinked = `u32::MAX`), a union-find with undo log over the *⅋-free skeleton* (tree edges except ⅋ premise edges, plus axiom edges), and the colored graph.
2. Choose the unlinked literal `x` with the fewest admissible partners (minimum remaining values). Admissible `y`: unlinked, dual sign, same atom, and neither of the two O(1) rejections below fires.
3. O(1) rejections for a candidate link `x–y`: (a) `x` and `y` lie in the same conclusion and their lowest common ancestor is a ⊗ (the tree path plus the link is a switching cycle); (b) `find(x) == find(y)` in the ⅋-free skeleton (a cycle avoiding all ⅋ nodes).
4. Add the link; run the exact acyclicity test when `V ≤ 200` or every fourth link otherwise (tunable); on failure undo and try the next `y`.
5. When all literals are linked: run the exact test and the connectedness equation; on success sequentialize and return the term.
6. Backtrack over `y`; if `x` has no admissible partner, backtrack to the previous literal.

**Sequentialization (net → proof term).** While a conclusion is a ⅋, emit `Par` and replace it by its premises. If all conclusions are literals, the structure is one axiom: emit `Ax`. Otherwise some ⊗ conclusion is *splitting* (Girard's splitting lemma): removing it leaves two correct sub-nets. Test each ⊗ conclusion by deleting it and counting components under one arbitrary switching (exactly two ⇒ splitting); recurse on the two parts. O(n²) total; Guerrini 2011 gives linear time if needed.

**Symmetry breaking (optional).** If two conclusions have the same formula id, swapping them maps proofs to proofs; require the lexicographically smaller linking on the first copy. Do not attempt symmetry breaking inside formulas.

### Fallback: MLL-Seq (bitset sequent search)

Use the focused skeleton with sequents as bitsets, a memo table, and explicit ⊗-splits: for focus `A ⊗ B` with context `Γ`, enumerate submasks `S ⊆ Γ` for the left premise in Gray order while maintaining the per-atom count vector incrementally; skip `S` unless both `S ∪ {A}` and `(Γ∖S) ∪ {B}` are atom-balanced. Axiom: focus on positive literal `p` succeeds iff `Γ = {p⊥}`. This engine also handles units (`1` needs `Γ = ∅`; `⊥` is dropped asynchronously) and Mix (add the rule that splits `Γ` into two provable parts, tried only when no other rule applies). Contexts above \~20 formulas should switch to lazy I/O contexts (RM1) because submask enumeration is 2^|Γ|.

### Pitfalls

- The plain per-atom balance and the `c = t − p + 2` equation are necessary conditions for MLL sequents only; they are unsound as prunes once additives or exponentials appear (see the count-invariant note in the infrastructure section).
- Using the identity rule on compound formulas is never needed; atomic axioms are complete. Allowing compound identities in proof-net search would break the linking model.
- The LCA rejection is only valid within one conclusion formula; literals in different conclusions have no tree path.
- Coloring: ⊗ premise edges must get distinct colors; giving them a shared color would wrongly forbid cycles through a ⊗ and make the test unsound.
- Connectedness must be checked *after* acyclicity; the edge-count equation alone says nothing about cyclic structures.
- Sequent search must treat the context as a set of occurrence ids, never as a multiset of formulas; two occurrences of the same formula are distinct ids.
- Units break the linking model (⊥ needs a jump target); do not extend MLL-Net to units, route them to MLL-Seq.
- Recursion depth equals the number of links (≤ n/2); use an explicit stack or a large thread stack anyway.

### Data layout

- Occurrence arrays in preorder (`parent`, `first_child`, `size`, `kind`, `atom`, `sign`), all `u32`.
- `partner: Vec<u32>`; literal lists per atom name as `Vec<OId>` split by sign; candidate counts kept in a small array updated on link/unlink.
- Union-find with an undo log (`Vec<(root, old_parent, old_rank)>`) so backtracking is O(1) per link.
- Colored graph as CSR adjacency with a `deleted: BitSet` scratch for the Yeo test; colors are `u32` (⅋ id for its premise edges, otherwise a fresh id).
- Memo table for MLL-Seq: `HashMap<Box<[u64]>, Entry>` with ahash; store the proof as a term index into an arena, not as an owned tree.

### Parallelization

- MLL-Net has no shared state, so it parallelizes as cube-and-conquer: enumerate the first `d` link choices sequentially into cubes (`d` chosen so there are \~8–32× more cubes than cores), then solve cubes with `rayon::scope` or a work-stealing deque, with an `AtomicBool` stop flag once a proof is found.
- Link the atom names with the smallest multiplicity first; they are nearly forced and shrink the cubes.
- MLL-Seq shares the memo table; use a sharded concurrent map (`DashMap` or per-shard `Mutex<HashMap>`) and accept duplicate work on races; entries are immutable facts, so no invalidation is needed.
- Results are nondeterministic in *which* proof is found first; if a canonical proof is required, run the sequential engine or collect all cube results and pick the smallest.

## MLL variants: units, Mix, cyclic MLL and Lambek

**Units (1, ⊥).** Provability stays NP-complete (Lincoln–Winkler 1994: even constant-only MLL is NP-hard). Use MLL-Seq. Rules: `1` is positive and succeeds only with an empty context; `⊥` is negative and is dropped in the asynchronous phase. Prune: focus on `1` only when `Γ = {1}` after the asynchronous phase. Count invariant with units (leaves are axioms and `1` rules; `⊥` adds a formula): `c = t − p + 2 − #1 + #⊥`, with per-atom balance unchanged; with Mix the equality becomes `≥`. Checks: ⊢ 1 (1 = 1), ⊢ ⊥, 1 (2 = 2), ⊢ 1 ⊗ 1 (1 = 1), ⊢ ⊥ ⅋ 1 (1 = 1), and ⊢ ⊥ ⅋ ⊥ fails (1 ≠ 3), matching provability. Proof equivalence with units is PSPACE-complete (Heijltjes–Houston 2016), which is why no canonical nets exist; provability is unaffected.

**Mix.** Adding the binary Mix rule (⊢ Γ and ⊢ Δ give ⊢ Γ, Δ) keeps NP membership; hardness proofs are believed to survive but no source states it explicitly. In MLL-Net, drop the connectedness equation; correctness is DR-acyclicity, equivalently uniqueness of the perfect matching of the R&B graph (Nguyễn 2020), which the Yeo test decides. In MLL-Seq, add a Mix step tried last: split `Γ` into two non-empty parts, each provable; with memoization this is a partition search over submasks and is only worth trying when `c > t − p + 2`.

**Cyclic MLL and the Lambek calculus.** Derivability in L and L\* is NP-complete (Pentus 2006), and so are the multiplicative fragments of cyclic and non-commutative LL, whose derivability coincides with L\*. Product-free fragments with both divisions are NP-complete (Savateev 2009); the one-division fragments L() and L(/) are in P by a cubic dynamic program (Savateev 2010); the multiplicative-additive Lambek calculus is PSPACE-complete already for (, ∧) and (, ∨) (Kanovich–Kuznetsov–Scedrov 2019). Implementation: MLL-Net with a *planarity* constraint on the axiom linking. Order the literal occurrences left-to-right as they appear in the cyclic sequent; a link `x–y` is admissible only if no existing link `u–v` crosses it (`x < u < y < v` or `u < x < v < y` in cyclic order). Maintain links in a balanced structure or, since n is small, check crossings against a stack of open links in O(1) amortised by processing literals left to right (well-formed bracket structure). Planarity removes almost all linkings; Moot's Grail family and the lambekseq prover are working examples of this design. For grammar-sized inputs with bounded formula order, Fowler's chart parser (O(n⁵) for bounded order) is the alternative, but it is a parser, not a general prover.

**IMLL by embedding.** For the purely multiplicative intuitionistic fragment (⊗, ⊸, 1), `Γ ⊢ A` is provable iff `⊢ Γ⊥, A` is provable in MLL, so both MLL engines apply after the translation; the essential-net engine in the intuitionistic section is the native alternative.

## MALL (multiplicative-additive linear logic)

**Verdict.** Implement focused depth-first search over occurrence bitsets with a memo table of stable sequents (MALL-Seq). Add the focused inverse method as a second engine for inputs with many hypotheses. The lazy I/O model with strict/lax zones (RM3) is the alternative when contexts exceed 64 formulas.

**Complexity.** PSPACE-complete (Lincoln–Mitchell–Scedrov–Shankar, APAL 1992) by a log-space encoding of QBF: `&` plays ∀, `⊕` plays ∃, extra literals enforce quantifier order. The encoding uses no constants and is two-sided-intuitionistic, so unit-free MALL and IMALL are already PSPACE-hard; units do not change the class. Refinement: focused MALL search alternates existential (decide, ⊕, ⊗-split) and universal (`&`) phases, and Das (JAR 2020) shows that bounding the number of alternations gives fragments complete for each level Σ^p\_k / Π^p\_k of the polynomial hierarchy; the search tree of a focused prover *is* an alternating machine run. Additive-only LL (ALL) is linear time (Heijltjes–Hughes 2015). Cut-free proofs can be exponentially larger than cut-full ones on some families (Akbar Tabatabai–Jalali 2026), so no backward engine is polynomial on all provable inputs. Proof-net correctness for MALL is NL-complete (Jacobé de Naurois–Mogbil 2011); conflict nets (Hughes–Heijltjes 2016) have polynomial correctness in the size of the net, but nets can be exponentially larger than the sequent, so nets are not a search vehicle for MALL.

**Algorithm families and the evidence.**

| Family | Handles `&` sharing by | Handles ⊗-split by | Evidence |
| --- | --- | --- | --- |
| Focused backward search, explicit contexts (this spec) | Copying the bitset | Submask enumeration filtered by interval counts, memoized | Memoization is exact because sequents are sets of occurrences; no published benchmark |
| Lolli/Lygon lazy I/O contexts, RM1–RM3 (Hodas–Miller 1994; Cervesato–Hodas–Pfenning 2000) | Strict zone: `&`-branches must consume identical resources, checked by residues; slack flag for ⊤ | Left premise consumes greedily, residue to the right premise | Lygon: 3–15× over naive splitting; the model behind Lolli, Lygon, LolliMon, Celf |
| Boolean-constraint distribution (Harland–Pym 1997/2003) | Same constraint variables in both `&` premises | Boolean variable per context formula per split; solve lazily or eagerly | Eager mode is explicitly parallel-friendly; no benchmark |
| Focused inverse method (Chaudhuri–Pfenning CSL/CADE 2005; Chaudhuri 2006) | Forward rules; sequents stored in a database with subsumption | No splitting: forward ⊗ combines two derived sequents | Only cross-prover comparison: beat Gandalf, linTAP, llprover on QBF, blocks-world, coins and affine suites; focusing gave 1.3×–600× over unfocused |
| Game/QBF view (Delande–Miller–Saurin 2010; Das 2020) | ∀ | ∃ | Structural, no solver reported |

### Specification: MALL-Seq

**Input.** Occurrence forest; atom bias. **Output.** `Proved(term)` or `Unprovable` (always terminates).

**Stable sequents.** Run the asynchronous phase to completion first: it is deterministic and branches only at `&` (two premises with identical `Γ`). `⊤` closes its branch; `⊥` is dropped; `⅋` pushes both premises; `?` does not occur here. What remains is a *stable* sequent: a bitset of positive formulas and negative atoms. Memoize only stable sequents.

**prove(Γ) for a stable Γ:**

1. Memo lookup; return on hit.
2. Immediate failure: `Γ` contains a `0` occurrence; or `Γ` has no positive non-literal formula and is not exactly a dual pair `{p, p⊥}`; or the interval count invariant excludes 0 for some atom.
3. Immediate success: `Γ = {p, p⊥}` with `p` the positive literal.
4. For each positive non-literal formula `F ∈ Γ` in heuristic order (fewest feasible continuations first; ⊗ with a literal factor first because its split is forced): `focus(Γ ∖ {F}, F)`; return `Proved` on the first success, recording the term.
5. Record `Failed` and return.

**focus(Γ, F):**

- `F = A ⊕ B`: try `focus(Γ, A)` then `focus(Γ, B)`.
- `F = A ⊗ B`: for each submask `S ⊆ Γ` (Gray-code order, count vector updated incrementally) with `S ∪ {A}` and `(Γ∖S) ∪ {B}` passing the interval count check: if `focus(S, A)` and `focus(Γ∖S, B)` both succeed, return. If `A` is a positive literal, the only candidate is `S = {A⊥}`.
- `F = 1`: succeed iff `Γ = ∅`. `F = 0`: fail.
- `F` a positive literal `p`: succeed iff `Γ = {p⊥}`.
- `F` negative: release — run the asynchronous phase on `Γ ∪ {F}` and call `prove` on each resulting stable sequent (all must succeed).

**Interval counts.** For each occurrence `o` and atom `a` precompute `lo[o][a], hi[o][a]`: literal `±a` gives `(±1, ±1)`; other literals `(0, 0)`; `⊗`, `⅋` sum; `&`, `⊕` take the hull `(min(lo₁, lo₂), max(hi₁, hi₂))`; units `(0, 0)`. A sequent is feasible only if `Σ lo ≤ 0 ≤ Σ hi` for every atom. Store as sparse rows over the atoms occurring below `o`.

**Termination and space.** Every call strictly decreases the multiset of occurrence sizes, so depth ≤ n. Without the memo the search runs in polynomial space (the PSPACE algorithm); with the memo it trades space for time. Cap the memo (e.g., 2^24 entries) and evict arbitrarily; eviction only costs time.

### Alternative: RM3 lazy contexts (summary of Cervesato–Hodas–Pfenning 2000)

Contexts are passed as input and returned as output; the ⊗ left premise receives all of `Γ` and returns the unconsumed residue for the right premise. A strict zone `Ξ` must be fully consumed and a lax zone `Δ` may be; `⊤` consumes `Ξ` and raises a slack flag that permits later goals to leave resources; for `A & B` the resources consumed by `A` become the strict zone for `B` (or, if `A` raised slack, the residue becomes lax for `B`). This gives early failure for `&` mismatches without enumerating splits, at the price of losing the memo (residue-dependent results). Use it when `|Γ| > 64` or memory is constrained.

### Alternative: focused inverse method (summary of Chaudhuri 2006)

Forward saturation from initial sequents `⊢ p, p⊥` for literals in the goal's subformula closure, applying focused big-step rules forward and keeping a database of derived sequents with subsumption (in MALL: set equality on occurrences; with the affine flag, subset). Terminates for MALL because the space of sequents is finite; complete for MELL/ILL as a semi-decision procedure. Strength: no ⊗-split guessing and natural sharing across hypotheses; weakness: database size, and the cost of the "is this sequent new?" test, which dominates (index with feature-vector or path indexing as in Imogen).

### Pitfalls

- Never prune MALL with the plain atom balance; ⊢ a ⊕ b, a⊥ is provable with `b` unbalanced. Only the interval form is sound.
- Memoize stable sequents only; memoizing mid-asynchronous states wastes memory and duplicates keys.
- Every positive formula must be tried as focus; skipping a candidate breaks completeness. Literals are the exception: focusing a literal can only succeed in the dual-pair case handled in step 3.
- `⊤` succeeds only in the asynchronous phase; after it, no `⊤` remains. `0` in a stable sequent is fatal.
- The submask enumeration must be over a compacted index of `Γ`'s members (positions 0..|Γ|), not over the global occurrence bitset, or the loop runs over 2^n.
- The `&` rule duplicates the context; with bitsets this is a copy of `W` words and needs no sharing machinery.
- DFS recursion depth is bounded by n, but n can be thousands: use an explicit stack or spawn the search on a thread with a large stack.
- Units: `1` needs `Γ = ∅`; `⊥` is dropped; the count equation of MLL does not apply (slices differ), only the intervals.

### Data layout

- Sequent bitsets `Box<[u64]>` of width `W`; for `n ≤ 64` specialize to `u64` (one machine word per sequent, memo key by value).
- Memo: `HashMap<Key, Entry>` with ahash, `Entry = Proved(TermIdx) | Failed`; keys stored in an arena, the map holding `u32` offsets, to halve memory.
- Interval counts as `Vec<(AtomId, i16, i16)>` per occurrence, sorted by atom; sums maintained in a scratch array indexed by atom.
- Split enumeration state: `members: SmallVec<[OId; 64]>` for `Γ`, current submask `u64`, running count vector.
- Proof terms in a `Vec<Term>` arena with `u32` children.

### Parallelization

- The search tree is an and/or tree in the exact sense of Das's alternation: `&` premises are and-nodes (both must succeed, may run in parallel and share the memo), decide/⊕/⊗-split alternatives are or-nodes (first success wins, needs a stop flag).
- Shared memo table across workers (`DashMap`, or sharded `RwLock<HashMap>`); every entry is an immutable fact, so races only cause duplicated work.
- Split work near the root, cube-and-conquer style: enumerate the first two levels of or-choices sequentially into cubes, run cubes on a work-stealing pool (rayon), steal in large chunks from random victims (Karp–Zhang; Rao–Kumar). Deep or-choices are cheap and stay sequential.
- Portfolio as the cheapest first step: run the same engine with different focus orderings and atom biases on different cores (randoCoP-style restarts showed solid gains for connection-style search).
- Cancellation: rayon tasks cannot be killed; every task must poll the stop flag at each `prove` entry.

## MELL (multiplicative-exponential linear logic)

**Verdict.** No implementable decision procedure exists. Implement a focused semi-decision procedure on dyadic sequents with iterative deepening on the number of copy steps (MELL-Seq), returning `Proved`, `Unprovable` (only when a level completed exhaustively) or `Unknown`. Add the !-Horn → Petri-net route for that fragment, and the affine variant, which *is* a decision procedure.

**Decidability status (as of 29 Sep 2026).** Provability in MELL is inter-reducible with reachability in branching VASS (de Groote–Guillaume–Salvati, LICS 2004). Bimbó (TCS 2015) claimed decidability; Straßburger (TCS 2019) showed the third step of that proof does not go through and gave a decidability proof only for RMELL (MELL with unrestricted contraction). The Stanford Encyclopedia (rev. 2023) and the MFCS 2025 paper on 2-dimensional BVASS both list the general problem as open. A preprint of 10 July 2026 by Bizière, Leroux and Sutre (arXiv 2607.09558) proves that every unreachable BVAS configuration is separated by a semilinear inductive invariant, concludes that BVAS reachability is decidable, and states in its introduction that MELL provability is decidable as a corollary. It is v1, has no listed venue, gives no complexity bound, and its algorithm is a pure enumeration of executions against candidate invariants; no independent confirmation was found. Treat it as *claimed, not yet vetted*. Even if correct, provability is TOWER-hard already for affine MELL (Lazić–Schmitz, TOCL 2015), so no engine can be efficient on all inputs. IMELL is also open; the Lazić–Schmitz bounds hold for intuitionistic versions.

**Related exact results.** The !-Horn fragment (Horn clauses under `!`, multiplicative goals) is exactly Petri-net reachability (Kanovich, APAL 1995), hence Ackermann-complete (Leroux–Schmitz 2019 upper bound; Czerwiński–Orlikowski and Leroux, FOCS 2021 lower bounds); adding `⊕` or `&` to Horn clauses makes it undecidable. Propositional affine LL (LLW, all connectives plus weakening) is decidable (Kopylov, LICS 1995) and TOWER-complete; contractive LL is Ackermann-complete; implicational relevant logic and IMLLC/IMELLC are 2-EXPTIME-complete (Schmitz, JSL 2016). Elementary affine logic is decidable (Dal Lago–Martini 2004). MELL proof-structure correctness is NL-complete; Guerrini–Masini (TCS 2001) give a confluent parsing system for MELL nets; no linear-time MELL correctness algorithm was found.

**Existing provers and what they do.** llprover (Tamura; Prolog) caps contractions at 3 per branch by default. Click & coLLecT (Callies–Laurent; OCaml) does iterative deepening on a bound for the exponential focusing rule with a 3-second time limit and reports unknown beyond it. linTAP (Mantel–Otten 1999) is a prefixed-tableau prover for MELL using prefix unification with an iteratively increased multiplicity, following the matrix characterization of Kreitz–Mantel (JAR 2004). Chaudhuri's focused inverse method treats the unrestricted zone as a set with subsumption `Θ ⊆ Θ'`, `Γ = Γ'`, complete for provable sequents. None of these publishes MELL benchmarks beyond internal ones.

### Specification: MELL-Seq

**Sequents.** Dyadic `⊢ Θ ; Γ`, with `Θ` a *set* of occurrence ids (formulas that arrived under `?`; contraction and weakening are implicit) and `Γ` a *multiset* of occurrence ids (copies from `Θ` can put the same occurrence into `Γ` more than once).

**Rules (on top of the MALL-Seq skeleton).**

- Asynchronous `?A`: remove `?A` from the list, add occurrence `A` to `Θ`.
- Synchronous `!A` in focus: succeed only if `Γ = ∅`; then release `A` into the asynchronous phase with the same `Θ`.
- Decide D1: focus on a positive formula in `Γ` (consumes it). Decide D2: focus on a positive formula in `Θ` *without* removing it; this is the only rule that can repeat, and it counts against the bound. A negative formula in `Θ` is copied by D2 and immediately released.
- Initial rule with `Θ`: focus on a positive literal `p` (taken from `Γ` by D1 or from `Θ` by D2) succeeds iff `Γ = {p⊥}`, or `Γ = ∅` and `p⊥ ∈ Θ`. These are Andreoli's two initial rules `⊢ Θ ; p⊥ ⇓ p` and `⊢ Θ, p⊥ ; · ⇓ p`.
- Units: `1` needs `Γ = ∅` (Θ arbitrary); `⊥` is dropped.

**Bound.** `b` = maximum number of D2 steps on any root-to-leaf branch. Run `b = 0, 1, 2, …` up to a configured limit. A level reports `Exhausted` if some branch failed *because* the bound was hit, `Failed` if the search space at that bound was fully explored without hitting the bound. Answers: `Proved` at the first level with a proof; `Unprovable` if a level returns `Failed` (the search was complete at that level, and larger bounds only add D2 steps that were never reached); `Unknown` when the limit is reached with `Exhausted`.

**Loop check.** If a stable sequent `(Θ, Γ)` recurs above itself on the current branch, prune; a minimal proof never repeats a sequent, so completeness is preserved.

**Memo.** Key `(Θ, Γ)`; value `Proved(term)` or `Failed{bound_remaining}`. A `Failed` entry is a hit only when the current remaining bound is ≤ the stored one. Entries survive across iterative-deepening levels; that is where most of the re-exploration cost is recovered.

**Copy heuristics (D2).** Try D1 before D2. Among `Θ` formulas, prefer positive ones whose literal occurrences match atoms currently unmatched in `Γ`; skip D2 on a `Θ` formula while an identical copy of it is already in `Γ` unconsumed (a second copy cannot help until the first is used).

**Count prunes.** Interval counts are valid only for atoms that occur nowhere below a `?` or `!` in the whole problem; disable them for the others.

### Affine variant (LLW, affine MELL): a decision procedure

Weakening is admissible, so any proof with a premise sequent that is a supermultiset of a sequent below it on the same branch (`Θ' ⊇ Θ` and `Γ' ⊇ Γ` as multisets) can be shortened by weakening away the surplus. Prune such premises. Multiset inclusion over finitely many occurrence ids is a well-quasi-order (Dickson's lemma), branching is finite, so by König's lemma the pruned search tree is finite: the engine terminates and decides. This is the syntactic core of Kopylov's argument and of Straßburger's RMELL proof; Larchey-Wendling (JAR 2020) gives a constructive, mechanised template for exactly this kind of redundancy-free search. Implementation: keep the branch's stable sequents in a stack and test inclusion against each (O(depth · W)); with weakening also allow `Γ ⊋ {p⊥}` in the initial rule and `Γ ≠ ∅` under `!` and `1`. This prune is *unsound* in MELL proper, where weakening is unavailable outside `Θ`.

### !-Horn fragment via Petri nets

A sequent `!(X₁ ⊸ Y₁), …, !(Xₖ ⊸ Yₖ), W ⊢ Z` with each `Xᵢ, Yᵢ, W, Z` a tensor product of atoms is provable iff the marking `Z` is reachable from `W` in the Petri net whose places are the atoms and whose transitions consume `Xᵢ` and produce `Yᵢ` (Kanovich 1995). Detect the fragment syntactically, build the net, and call a reachability engine (KReach implements Kosaraju's procedure; coverability engines suffice when weakening is present). The ILLTP library ships 3,137 such problems from the Model Checking Contest, so this route has a ready benchmark.

### Pitfalls

- `Unprovable` may only be reported when a whole level finished without any branch hitting the bound; reporting it after `Exhausted` is unsound.
- The memo key must include `Θ`; a `Failed` entry recorded with remaining bound `r` may be reused only when the current remaining bound is ≤ `r`; reusing it with a larger remaining bound is unsound.
- Copies: D2 must not remove the formula from `Θ`; weakening of `Θ` is implicit (unused members are fine).
- `!A` in focus with non-empty `Γ` fails immediately; in a ⊗ split with an `!` factor, that factor's part of the split is forced to be empty.
- Bounding total copies instead of copies per branch changes completeness in the limit only if the limit grows without bound; per-branch is the standard (llprover, Click & coLLecT) and interacts correctly with memo entries.
- The inverse method is the better engine when `Θ` is large and `Γ` small (theory-heavy inputs); it does not terminate on unprovable inputs either.

### Data layout

- `Θ`: bitset of width `W`. `Γ`: `SmallVec<[(u32, u8); 16]>` sorted by occurrence id with counts, or a bitset plus a side map when all counts are 1 (the common case; promote to counts lazily).
- Memo key: hash of `(Θ words, Γ entries)`; store both zones in an arena and keep `u32` offsets in the map.
- Branch stack of stable sequents for the loop check and the affine prune.
- Bound state: remaining D2 budget carried in the frame.

### Parallelization

- Same and/or structure as MALL; D2 alternatives are or-nodes.
- Run one deepening level at a time; parallelize within a level over root-near or-choices; do not run levels concurrently (the memo makes each level cheaper than restarting).
- The shared memo now carries bound information; use a compare-and-swap update so an entry's `bound_remaining` only ever increases.

## Intuitionistic fragments (IMLL, IMALL, IMELL, ILL) and affine variants

**Verdict.** IMLL: essential-net search (directed proof nets with a dominator condition), because the intuitionistic orientation makes the incremental exactness test a directed-reachability query, which is cheaper than the classical Yeo test. IMALL: the MALL-Seq engine in two-sided form (bitset of hypotheses plus one goal id). IMELL and full ILL: the MELL-Seq engine in two-sided form, semi-decision only. Affine ILL: the same with the supermultiset prune, which decides.

**Complexity.**

| Fragment | Result | Source |
| --- | --- | --- |
| IMLL (⊗, ⊸, 1) | NP-complete; hardness already for Horn sequents over atoms | Kanovich, LICS 1992 (hardness); LMSS 1992 (NP membership) |
| Affine IMLL / MLL | NP-complete (Vertex Cover) | LMSS 1992; refined bounds Kopylov, APAL 1995 |
| ⊸-only (BCI) | In NP (linear proofs); NP-hardness not located in the literature | — |
| IMALL | PSPACE-complete; the QBF encoding is two-sided | LMSS 1992 (remark in text) |
| Purely additive with units | Linear time | Heijltjes–Hughes, LICS 2015 |
| IMELL | Open; TOWER-hard | Lazić–Schmitz 2015 (intuitionistic versions §4.1.2) |
| IMLLC, IMELLC (contraction, no weakening) | 2-EXPTIME-complete | Schmitz, JSL 2016 |
| Full propositional ILL | Undecidable; already for {&, ⊸, !} | LMSS 1992; Forster–Larchey-Wendling, CPP 2019 (Coq-certified) |
| Affine ILL (all connectives + weakening) | Decidable, TOWER-complete | Kopylov 1995/2001; Lazić–Schmitz 2015 |
| Multiplicative subexponential logic (3 labels) | Undecidable, classical and intuitionistic | Chaudhuri 2014; Larchey-Wendling, FSCD 2021 (Coq) |

**Provers and evidence.** Lolli (I/O contexts), LolliMon (backward chaining with backtracking outside the monad, forward chaining with saturation inside, discrimination-tree indexing) and Celf are the logic-programming lineage. Sympli (Chaudhuri, SML) is the propositional inverse-method prover; LL\_prover (OCaml) offers backward search or inverse method with an ILL flag, reads the LLTP format and emits Coq certificates checkable by Yalla; Olarte's Maude provers are the focused prototypes used for ILLTP. Grail, LinearOne (first-order MILL, proof nets), CatLog3 (focusing plus count invariance) and lambekseq are the categorial-grammar provers. The ILLTP library (Olarte, de Paiva, Pimentel, Reis, 2019) has 4,494 ILL problems; with a 5-minute timeout the Maude prototype solved only about 76–83 of the 358 ILTP-derived problems per translation, and the authors state that no systematic efficiency study of ILL provers exists. Nothing found for 2020–2026 changes that picture, so a careful Rust implementation of the engines here would be state of the art by default.

### Specification: IMLL-Net (essential nets)

**Structure.** Translate `Γ ⊢ A` into Lamarche's polarized proof structure: every subformula occurrence is *input* (hypothesis side) or *output* (goal side); `A ⊸ B` in output position has an input premise `A` and an output premise `B`; in input position the polarities flip. Edges are directed by polarity (take the exact orientation from Moot 2008, §2–3, or Murawski–Ong 2000/2006). Axiom links connect an input literal to an output literal of the same atom, as directed edges.

**Correctness (Lamarche; Murawski–Ong).** A complete structure is a proof net iff (1) the directed graph is acyclic, and (2) for every output-`⊸` node `l` with input premise `a`, every directed path from the goal root into the sub-net of `a` passes through `l`; equivalently `l` dominates `a` in the dominator tree from the root. Both conditions are checkable in linear time with a dominator-tree algorithm (Murawski–Ong), or in O(n²) with reachability.

**Incremental search (Moot 2008).** Keep the transitive closure `R` of the partial net (bit-matrix, `n` rows of `W` words). Candidate link `x → y` is rejected if `R[y][x]` (would close a cycle) or if it would create a path violating a dominator condition, tested with the closure augmented by the set of nodes that must not be bypassed. After accepting a link, update `R` by `R[u] |= R[y]` for all `u` with `R[u][x]`, i.e. O(n · W) word operations. Order literals by fewest admissible partners; the atom-balance and count equation of MLL apply after translating to `⊢ Γ⊥, A`. At the end run the full correctness check and sequentialize (splitting `⊗`/`⊸` lemma as in MLL, or read the sequent proof off the dominator tree).

**Embedding alternative.** For the fragment ⊗, ⊸, 1 only, `Γ ⊢ A` is provable iff `⊢ Γ⊥, A` is provable in MLL, so MLL-Net and MLL-Seq apply directly. Do not use the embedding for fragments with additives or `0`; run the two-sided engine instead.

### Specification: two-sided focused engine (IMALL, IMELL, ILL)

Sequents `Θ ; Δ ⊢ A` with `Θ` a set, `Δ` a bitset (multiset for exponentials) of hypothesis occurrences and `A` one goal occurrence. Polarities: positive `⊗, 1, ⊕, 0, !`; negative `⊸, &, ⊤`; atoms by bias. Asynchronous phase: right-negative goals decompose invertibly (`A ⊸ B` moves `A` into `Δ`, `A & B` branches with the same `Δ`, `⊤` succeeds); left-positive hypotheses decompose invertibly (`A ⊗ B` splits into two hypotheses, `1` is dropped, `A ⊕ B` branches with the same goal, `0` on the left succeeds immediately, `!A` moves `A` into `Θ`). Stable sequents have positive or atomic goals and negative or atomic hypotheses; memoize them. Decide: right-focus on a positive goal, or left-focus on a negative hypothesis from `Δ` (consumed) or `Θ` (copied, bounded). Left-focus on `A ⊸ B`: split `Δ` into the part proving `A` (right focus) and the part continuing with `B` as a hypothesis; left-focus on `A & B`: choose a side; initial rule: goal atom `p` with `Δ = {p}` (or `Δ = ∅`, `p ∈ Θ`). Everything else (counts, memo, bound, affine prune, parallelism) is inherited from the MALL/MELL sections; counts are computed on the translation `⊢ Γ⊥, A`.

### Pitfalls

- `0` as a hypothesis proves any goal (ex falso for the additive unit); forgetting this rule breaks completeness. `⊤` as a hypothesis is inert.
- Left-focusing on `A ⊸ B` splits the context like a ⊗; the goal stays on the `B` side. The `A` premise is a right-focus, so `A`'s polarity decides whether it is immediate or released.
- Promotion (`!A` as goal) needs `Δ = ∅`; with weakening (affine) that condition is dropped.
- The single-succedent invariant means the memo key is `(Θ, Δ, goal)`; do not reuse classical one-sided keys.
- In essential nets the direction of axiom links matters (input literal to output literal); an undirected implementation silently accepts cyclic structures.
- Classical LL is not conservative over full ILL (Schellinx 1991; the counterexamples involve `0`); keep a native two-sided engine for anything beyond the multiplicatives.

### Data layout

- Hypothesis bitset as in MALL; goal as a `u32`; `Θ` bitset; interval counts on the classical translation.
- Essential nets: `reach: Vec<Bitset>` closure matrix with an undo log of changed rows (store the row copies touched per link; rows are small); `dominator` recomputed only at the final check.

### Parallelization

- As MALL/MELL; for IMLL-Net the closure matrix is per worker (cube-and-conquer, no sharing).

## Other fragments: full LL, additive-only, contractive, first-order, subexponentials, fixed points

**Full propositional LL.** Undecidable (LMSS 1992, via and-branching two-counter machines); the encoding lives in the intuitionistic fragment too. Implement as MELL-Seq plus the additive rules of MALL-Seq: the same dyadic sequents, the same copy bound, the same memo. Additives and exponentials combine without new rules; the only interaction to check is that `&` duplicates `Γ` *and* keeps the same `Θ`, and that the copy budget is per branch (each `&` premise inherits the current remaining budget).

**Additive-only LL (ALL, with or without units).** Provability is decidable in linear time in the product of the sizes of the two formulas of `A ⊢ B` (Heijltjes–Hughes, LICS 2015, via additive proof nets and a Petri-net construction). A direct implementation: an `A ⊢ B` sequent in ALL is provable iff a simple recursive procedure on the pair `(A, B)` succeeds, memoized on subformula pairs (`&` on the right and `⊕` on the left branch conjunctively, `⊕` on the right and `&` on the left branch disjunctively, `⊤` right and `0` left close, atoms must match). With memoization on `(occurrence of A, occurrence of B)` this is O(|A|·|B|). Include it as a fast path when a sequent has no multiplicatives or exponentials; in the classical one-sided form the sequent has exactly two formulas.

**Contractive fragments (contraction, no weakening).** Propositional LL with contraction is Ackermann-complete (Lazić–Schmitz 2015); IMLLC and IMELLC are 2-EXPTIME-complete and implicational relevance logic R→ is 2-EXPTIME-complete (Schmitz 2016, via BVASS coverability); RMELL is decidable (Straßburger 2019). Decision procedures exist but are impractical; the MELL-Seq engine with contraction allowed on all formulas plus the *submultiset*-ancestor prune (the contraction-side analogue of the affine prune, justified by Kripke's lemma as in Straßburger's RMELL proof) terminates. Larchey-Wendling (JAR 2020) is the mechanised reference for the redundancy-free search framework.

**First-order fragments.** First-order MLL adds only unification to MLL-Net: axiom links must unify their atoms' arguments, and `∀`/`∃` become invertible/positive rules; provability stays NP (proof size is linear, unification is linear) and NP-hard by inclusion. First-order MALL (MALL₁) is NEXPTIME-hard (Lincoln–Scedrov 1994); decidability was announced but NEXPTIME membership is not established in the literature found. First-order additive LL proof search is NP-complete (Heijltjes–Hughes 2015). Moot's LinearOne (first-order MILL, proof nets with unification) and Chaudhuri's first-order focused inverse method are the working designs; do not build first-order support before the propositional engines pass their tests.

**Subexponentials (SELL).** Multiplicative subexponential logic with one unrestricted and two incomparable linear labels is undecidable, classically and intuitionistically (Chaudhuri 2014; Larchey-Wendling FSCD 2021, Coq). SELLF (OCaml) and Olarte–Pimentel–Rocha's Maude L-framework are frameworks rather than efficient provers. If needed, generalize MELL-Seq's single `Θ` zone to one zone per label with the label's structural rules and the promotion side condition (`!ᵃA` requires every remaining zone to be ≥ `a` in the preorder); no decision procedure exists in general.

**Fixed points (μMALL).** No dedicated prover implementation was found; the theory is infinitary/circular proofs (Baelde–Doumane–Saurin). Out of scope for a first implementation.

**Lambek variants beyond the multiplicative core.** Lambek with a relevant modality is undecidable even unidirectional, but decidable and in NP when the modality is restricted to primitive types (Kanovich–Kuznetsov–Scedrov 2016); two of Morrill's CatLog calculi are undecidable (KKS 2021); full Lambek with contraction is undecidable (Chvalovský–Horčík). Stay within L, L\*, the product-free and one-division fragments for anything that must terminate.

## Cross-cutting engineering notes

**Certificates first.** Every engine returns a proof term over occurrence ids, and an independent linear-time checker validates it against the sequent (the checker is a small interpreter of the unfocused rules; it must not share code with the search). Reject any engine change whose proofs fail the checker. Export the same terms to Rocq via Yalla's format the way Click & coLLecT and LL\_prover do; that is the only external certification path found.

**Testing strategy.**

- Differential testing: MLL-Net against MLL-Seq on random unit-free sequents (they must agree on every input); the two-sided engine against the classical engine on IMLL via the embedding; MALL-Seq with the memo against MALL-Seq without it.
- Generated positives: build random cut-free proofs bottom-up (choose rules, invent atoms) and read off the conclusion; the engine must prove it. Generated negatives: mutate a provable sequent by swapping one literal's atom name; most mutants are unprovable and the count invariants catch many, so also keep mutants that pass the counts.
- Known-hard families: Kanovich's Horn encodings of 3-Partition and Matsuoka's encodings of 3D-Matching and Partition (Matsuoka 2017/2018) for MLL; the LMSS QBF encoding for MALL (Chaudhuri's qbf1–3 suites); ILLTP's 271 Kleene-theorem and 1,086 ILTP-derived problems for ILL; ILLTP's 3,137 Petri-net problems for !-Horn. Track solved-within-timeout counts per family as the regression metric.
- Focusing invariants as debug assertions: after the asynchronous phase no negative non-atomic formula remains; a stable sequent is never memoized twice with different results; the copy budget never goes negative.
- Proof-net checks: for every complete linking accepted by the Yeo test plus the edge equation, the contractibility check (Danos) must also accept, and vice versa.

**Benchmark harness.** Read the LLTP/ILLTP `fof(...)` syntax and a plain infix syntax; run each problem with a timeout on a fresh engine instance; record `Proved`/`Unprovable`/`Unknown`, wall time, memo size and node count; store results as CSV. Compare against LL\_prover (OCaml, reads LLTP) and Click & coLLecT's auto-prover (3 s cap) as external baselines; llprover and linTAP are the older Prolog baselines.

**Crate layout (workspace).**

1. `ll-syntax`: parser, formula arena, hash-consing, NNF, dual, pretty-printer, LLTP reader.
2. `ll-occ`: occurrence forest, bitsets, interval counts, LCA, atom tables.
3. `ll-proof`: proof terms, the independent checker, Rocq/Yalla export.
4. `ll-net`: MLL-Net and IMLL-Net (colored graph, Yeo test, closure matrices, sequentialization).
5. `ll-focus`: the focused skeleton with pluggable rule sets (MLL, MALL, MELL, ILL, affine, contractive flags), memo tables, iterative deepening.
6. `ll-inverse`: forward saturation engine with subsumption indexing (feature-vector index as in Schulz 2013; path indexing as in Imogen).
7. `ll-petri`: !-Horn detection and export to a Petri-net reachability tool.
8. `ll-cli` and `ll-bench`.

**Parallel runtime.** One `rayon` pool; cube-and-conquer at the root (enumerate the first 2–3 or-choices sequentially, then `scope` over cubes); an `AtomicBool` stop flag polled at each `prove` entry; shared memo via `DashMap` (or 64 shards of `Mutex<HashMap>` keyed by the top hash bits); per-worker scratch (bitsets, closure rows, undo logs) allocated once in a `bumpalo` arena and reset per cube. Determinism switch: with `--deterministic` run the sequential engine, since parallel first-success changes which proof is returned. The QBF analogy (Das 2020) is the design guide: `&` nodes are and-parallel and profit from the shared memo; decide/⊕/split nodes are or-parallel and profit from randomized orderings (portfolio) more than from fine-grained splitting; steal near the root in big chunks (Karp–Zhang; Rao–Kumar).

**Known gaps in the literature (opportunities).** No SAT/QBF encoding of MLL or MALL provability has been evaluated; no memoized subset-DP formulation of MALL search appears in print although it follows directly from the subformula property; no learned heuristics for LL search exist; no benchmark comparison across algorithm families exists for any fragment. A Rust implementation that reports ILLTP numbers for the engines above would be the first systematic study.

## References

Opened during this survey unless marked (abstract only). Grouped by use.

**Complexity and decidability**

- Lincoln, Mitchell, Scedrov, Shankar, [Decision problems for propositional linear logic](https://curien.galene.org/ECI2023/Lincoln%2B-decision-LL.pdf), APAL 56, 1992 (MALL PSPACE-complete, LL undecidable, affine MLL NP-complete).
- Kanovich, [Horn programming in linear logic is NP-complete](https://lics.siglog.org/1992/Kanovich-Hornprogramminginli.html), LICS 1992; [The complexity of Horn fragments of linear logic](https://www.sciencedirect.com/science/article/pii/016800729490085X), APAL 69, 1994; [Petri nets, Horn programs, linear logic and vector games](https://www.sciencedirect.com/science/article/pii/016800729400060G), APAL 75, 1995.
- Lincoln, Winkler, [Constant-only multiplicative linear logic is NP-complete](https://www.sciencedirect.com/science/article/pii/0304397594001081), TCS 135, 1994 (abstract only).
- Lincoln, [Deciding provability of linear logic formulas](https://www.cambridge.org/core/books/abs/advances-in-linear-logic/deciding-provability-of-linear-logic-formulas/960C431D7336A81B6EDC04E4F6554FE3), in Advances in Linear Logic, 1995 (bibliographic).
- Lincoln, Scedrov, [First-order linear logic without modalities is NEXPTIME-hard](https://www.csl.sri.com/~lincoln/papers/mall1-hard.pdf), TCS 135, 1994.
- de Groote, Guillaume, Salvati, [Vector addition tree automata](https://inria.hal.science/inria-00100081), LICS 2004.
- Lazić, Schmitz, [Non-elementary complexities for branching VASS, MELL, and extensions](https://arxiv.org/abs/1401.6785), CSL-LICS 2014 / ACM TOCL 2015.
- Bimbó, [The decidability of the intensional fragment of classical linear logic](https://www.sciencedirect.com/science/article/pii/S0304397515005290), TCS 597, 2015 (abstract only).
- Straßburger, [On the decision problem for MELL](https://www.lix.polytechnique.fr/~lutz/papers/OnDeciMELL.pdf), TCS 768, 2019.
- Bizière, Leroux, Sutre, [Solving the reachability problem for branching vector addition systems via semilinear inductive invariants](https://arxiv.org/abs/2607.09558), arXiv 2607.09558, July 2026 (preprint, unrefereed). Precursors: [MFCS 2025](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.MFCS.2025.22), [FoSSaCS 2026](https://link.springer.com/chapter/10.1007/978-3-032-22730-0_4).
- Kopylov, [Decidability of linear affine logic](https://www.cs.cornell.edu/people/kopylov/papers/llw/), LICS 1995 / Inf. & Comp. 2001; [On NP-completeness in linear logic](https://www.cs.cornell.edu/people/kopylov/papers/np/index.htm), APAL 75, 1995.
- Schmitz, [Implicational relevance logic is 2-ExpTime-complete](https://arxiv.org/abs/1402.0705), JSL 2016.
- Leroux, Schmitz, [Reachability in vector addition systems is primitive-recursive](https://arxiv.org/abs/1903.08575), LICS 2019; Czerwiński–Orlikowski and Leroux, FOCS 2021 (Ackermann lower bounds).
- Das, [From QBFs to MALL and back via focussing](https://arxiv.org/abs/1906.03611), IJCAR 2018 / JAR 2020.
- Heijltjes, Hughes, [Complexity bounds for sum-product logic via additive proof nets and Petri nets](https://researchportal.bath.ac.uk/en/publications/complexity-bounds-for-sum-product-logic-via-additive-proof-nets-a/), LICS 2015.
- Heijltjes, Houston, [No proof nets for MLL with units: proof equivalence in MLL is PSPACE-complete](https://ar5iv.arxiv.org/html/1510.06178), CSL-LICS 2014 / LMCS 2016.
- Pentus, [Lambek calculus is NP-complete](https://www.sciencedirect.com/science/article/pii/S0304397506002702), TCS 357, 2006; [Complexity of the Lambek calculus and its fragments](https://www.aiml.net/volumes/volume8/Pentus.pdf), AiML 2010. Savateev, [Unidirectional Lambek grammars in polynomial time](https://link.springer.com/article/10.1007/s00224-009-9208-4), ToCS 2010. Kanovich, Kuznetsov, Scedrov, [The complexity of multiplicative-additive Lambek calculus: 25 years later](https://link.springer.com/chapter/10.1007/978-3-662-59533-6_22), WoLLIC 2019.
- Forster, Larchey-Wendling, [Certified undecidability of intuitionistic linear logic via binary stack machines and Minsky machines](https://www.ps.uni-saarland.de/Publications/documents/ForsterLarchey-Wendling_2018_Undecidability-ILL.pdf), CPP 2019. Larchey-Wendling, [Constructive decision via redundancy-free proof-search](https://link.springer.com/article/10.1007/s10817-020-09555-y), JAR 2020; [Synthetic undecidability of MSELL via FRACTRAN mechanised in Coq](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2021.18), FSCD 2021. Chaudhuri, [Undecidability of multiplicative subexponential logic](https://arxiv.org/abs/1502.04769), 2014.
- Akbar Tabatabai, Jalali, [Proof complexity of linear logics](https://arxiv.org/abs/2601.22393), arXiv 2026. Suzuki, Sano, [Undecidability of linear logics without weakening](https://arxiv.org/abs/2509.00644), arXiv 2025.

**Proof nets and correctness algorithms**

- Danos, Regnier, [The structure of multiplicatives](https://link.springer.com/article/10.1007/BF01622878), Arch. Math. Logic 1989.
- Guerrini, [A linear algorithm for MLL proof net correctness and sequentialization](https://www.sciencedirect.com/science/article/pii/S0304397510007127), TCS 412, 2011 (LICS 1999).
- Murawski, Ong, [Dominator trees and fast verification of proof nets](https://www.lfcs.inf.ed.ac.uk/events/lics/2000/MurawskiOng-DominatorTreesandFa.html), LICS 2000; [Fast verification of MLL proof nets via IMLL](https://dl.acm.org/doi/10.1145/1149114.1149116), ACM TOCL 2006.
- Jacobé de Naurois, Mogbil, [Correctness of linear logic proof structures is NL-complete](https://www.sciencedirect.com/science/article/pii/S0304397510007115), TCS 412, 2011.
- Retoré, [Handsome proof-nets: perfect matchings and cographs](https://www.sciencedirect.com/science/article/pii/S030439750100175X), TCS 294, 2003.
- Nguyễn, [Unique perfect matchings, forbidden transitions and proof nets for linear logic with Mix](https://lmcs.episciences.org/6172), LMCS 16(1), 2020.
- Di Guardia, Laurent, Tortora de Falco, Vaux Auclair, [Yeo's theorem for locally colored graphs: the path to sequentialization in linear logic](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2025.16), FSCD 2025.
- Bagnol, Doumane, Saurin, [On the dependencies of logical rules](https://link.springer.com/chapter/10.1007/978-3-662-46678-0_28), FoSSaCS 2015. Matsuoka, [A new correctness criterion for MLL proof nets](https://arxiv.org/abs/1902.09693), 2019.
- Hughes, van Glabbeek, Proof nets for unit-free MALL, LICS 2003 / TOCL 2005; Hughes, [Abstract p-time proof nets for MALL: conflict nets](https://arxiv.org/abs/0801.2421); Hughes, Heijltjes, Conflict nets, LICS 2016. Bagnol, [MALL proof equivalence is Logspace-complete via BDDs](https://arxiv.org/abs/1502.01993), TLCA 2015.
- Guerrini, Masini, [Parsing MELL proof nets](https://www.sciencedirect.com/science/article/pii/S0304397599002996), TCS 254, 2001. Lamarche, [Proof nets for intuitionistic linear logic: essential nets](https://inria.hal.science/inria-00347336), INRIA RR 2008 (ms. 1994).

**Proof search algorithms and provers**

- Andreoli, Logic programming with focusing proofs in linear logic, JLC 2(3), 1992 ([PDF](https://www.cs.cmu.edu/~fp/courses/15816-s12/misc/andreoli92jlc.pdf)); [Focussing and proof construction](https://www.sciencedirect.com/science/article/pii/S0168007200000324), APAL 107, 2001. Andreoli, Mazaré, [Concurrent construction of proof-nets](https://link.springer.com/chapter/10.1007/978-3-540-45220-1_3), CSL 2003.
- Hodas, Miller, [Logic programming in a fragment of intuitionistic linear logic](https://www.lix.polytechnique.fr/~dale/papers/ic94.pdf), Inf. & Comp. 1994. Cervesato, Hodas, Pfenning, [Efficient resource management for linear logic proof search](https://www.cs.cmu.edu/~fp/papers/elp96.pdf), ELP 1996 / TCS 232, 2000.
- Harland, Pym, [Resource-distribution via Boolean constraints](https://arxiv.org/abs/cs/0012018), CADE 1997 / ACM TOCL 2003. Winikoff, Harland, [Implementing the linear logic programming language Lygon](https://www.academia.edu/19749815/Implementing_the_linear_logic_programming_language_Lygon), ILPS 1995.
- Chaudhuri, [The focused inverse method for linear logic](http://reports-archive.adm.cs.cmu.edu/anon/2006/CMU-CS-06-162.pdf), PhD thesis, CMU 2006. Chaudhuri, Pfenning, [A focusing inverse method theorem prover for first-order linear logic](https://www.cs.cmu.edu/~fp/papers/cade05.pdf), CADE 2005 (the cross-prover benchmark). Chaudhuri, Pfenning, Price, [A logical characterization of forward and backward chaining in the inverse method](https://www.cs.cmu.edu/~fp/papers/fwdbwd07.pdf), JAR 2008. Chaudhuri, Miller, Saurin, [Canonical sequent proofs via multi-focusing](https://www.lix.polytechnique.fr/~dale/papers/tcs08trackb.pdf), 2008.
- Galmiche, [Connection methods in linear logic and proof nets construction](https://www.sciencedirect.com/science/article/pii/S0304397599001760), TCS 232, 2000 (abstract). Kreitz, Mantel, Otten, Schmitt, [Connection-based proof construction in linear logic](https://www.jens-otten.de/papers/linlogic_cade97.pdf), CADE 1997. Mantel, Otten, [linTAP](https://www.jens-otten.de/papers/lintap_tab99.pdf), TABLEAUX 1999. Kreitz, Mantel, [A matrix characterization for MELL](https://link.springer.com/article/10.1023/B:JARS.0000029976.22387.ac), JAR 2004 (abstract).
- Tammet, [Proof strategies in linear logic](https://cdn.aaai.org/Symposia/Fall/1993/FS-93-01/FS93-01-022.pdf), JAR 12, 1994 (AAAI FS-93 version).
- Moot, [Graph algorithms for improving type-logical proof search](https://arxiv.org/abs/0805.2303), 2008; [Grail](https://github.com/RichardMoot/Grail); [LinearOne](https://github.com/RichardMoot/LinearOne); Moot, Puite, Proof nets for the multimodal Lambek calculus, Studia Logica 2002. Morrill, [CatLog3](https://link.springer.com/article/10.1007/s10849-018-09277-w), JoLLI 2019. Fowler, [Efficiently parsing with the product-free Lambek calculus](https://aclanthology.org/C08-1028.pdf), COLING 2008.
- Matsuoka, [Direct encodings of NP-complete problems into Horn sequents of MLL](https://software.imdea.org/Conferences/hcvs17/papers/paper_1.pdf), HCVS 2017; [Proof Net Calculator](https://staff.aist.go.jp/s-matsuoka/PNCalculator/index.html).
- Delande, Miller, Saurin, [Proof and refutation in MALL as a game](https://www.lix.polytechnique.fr/~dale/papers/apal-games.pdf), APAL 2010.
- Tools: [Click & coLLecT](https://github.com/ComputerAidedLL/click-and-collect) (OCaml; focused auto-prover with bitmask splits, exponential iterative deepening, 3 s cap); [llprover](https://cspsat.gitlab.io/llprover/) (Prolog, contraction cap 3); [LL\_prover](https://github.com/wujuihsuan2016/LL_prover) (OCaml, backward + inverse, ILL flag, Yalla certificates); [Sympli](https://github.com/chaudhuri/sympli) (SML inverse method); [Linear-Logic-Prover-in-Maude](https://github.com/carlosolarte/Linear-Logic-Prover-in-Maude); [LolliMon](https://github.com/clf/lollimon); [SELLF](https://github.com/meta-logic/sellf); [lambekseq](https://github.com/PterosDiacos/lambekseq); [lolli](https://github.com/ibrahimcesar/lolli) (Rust, educational focused prover, 2026); [Yalla](https://github.com/olaure01/yalla) (Rocq deep embedding; JAR 2026).
- Benchmarks: Olarte, de Paiva, Pimentel, Reis, [The ILLTP library for intuitionistic linear logic](https://arxiv.org/abs/1904.06850), 2019; repository [meta-logic/lltp](https://github.com/meta-logic/lltp).

**Parallel search and indexing**

- Gupta, Pontelli, Ali, Carlsson, Hermenegildo, [Parallel execution of Prolog programs: a survey](https://cliplab.org/papers/partut-toplas.pdf), TOPLAS 2001. Karp, Zhang, [Randomized parallel algorithms for backtrack search and branch-and-bound](https://dl.acm.org/doi/10.1145/174130.174145), JACM 1993. Rao, Kumar, Ramesh, [A parallel implementation of iterative-deepening-A\*](https://cdn.aaai.org/AAAI/1987/AAAI87-032.pdf), AAAI 1987. Heule, Kullmann, Wieringa, Biere, [Cube and conquer](https://www.cs.utexas.edu/~marijn/publications/cube.pdf), HVC 2011. Raths, Otten, [randoCoP](https://jens-otten.de/papers/randocop_paar08.pdf), PAAR 2008.
- Schulz, [Simple and efficient clause subsumption with feature vector indexing](https://link.springer.com/content/pdf/10.1007/978-3-642-36675-8_3.pdf), 2013. McLaughlin, Pfenning, [Efficient intuitionistic theorem proving with the polarized inverse method (Imogen)](https://www.cs.cmu.edu/~fp/papers/imogen09.pdf), CADE 2009. Sekar, Ramakrishnan, Voronkov, Term indexing, Handbook of Automated Reasoning, 2001.
- Rust crates: [fixedbitset](https://docs.rs/fixedbitset), [bumpalo](https://docs.rs/bumpalo), [lasso](https://docs.rs/lasso), [rayon](https://docs.rs/rayon), [varisat](https://github.com/jix/varisat), [splr](https://github.com/shnarazk/splr), [scryer-prolog](https://github.com/mthom/scryer-prolog) (WAM with tabling; no or-parallelism).

**Surveys and reference pages**

- Di Cosmo, Miller, [Linear Logic](https://plato.stanford.edu/entries/logic-linear/), Stanford Encyclopedia of Philosophy (rev. 2023). Lincoln, [Linear logic](https://www.csl.sri.com/papers/sigact92/sigact92.pdf), SIGACT News 1992. Vale, [Efficient proof net verification and sequentialization](https://arthurovale.github.io/files/EfficientProofNets.pdf), survey 2018. Wikipedia, [Vector addition system](https://en.wikipedia.org/wiki/Vector_addition_system).

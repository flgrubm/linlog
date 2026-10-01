---
paths:
  - "core/**"
---

# linlog core: data model and invariants

Loaded when a file under `core/` is read. What follows is what the code relies
on but does not say in one place.

## Sequents are arena-allocated DAGs

`Sequent` (`core/src/sequents/mod.rs`) has three fields, all `pub(crate)`:
- `terms: Vec<Term>`: every subformula. Children are referenced by arena
  index (`TermId`, a `u32` newtype), never by pointer.
- `roots: Vec<TermId>`: the root formulas that make up the sequent, in the
  order the sequent lists them.
- `atoms: Vec<String>`: atom names. `Var(a)`/`DualVar(a)` index into this
  with `Atom`, a `u32` newtype.

**A term only references subterms with a strictly smaller index**, so the
arena is topologically sorted: one ascending pass sees every subterm before
its parents, one descending pass sees every parent before its subterms
(`fragment()` and the forest's size computation rely on the latter).
`verify_integrity()` checks it, `Formula` printing debug-asserts it, and
deserialization runs the check. Code that builds or rewrites an arena must
preserve it.

`optimize()` runs after parsing. It deduplicates atom names, hash-conses
identical terms, drops unreachable ones and sorts `roots`. Nothing else
guarantees that every arena term is reachable: a deserialized sequent may
carry junk terms, and `fragment()` and `Forest` walk from the roots for that
reason. `Sequent::add` merges two sequents by offsetting atom and term indices.

The public surface is read-only accessors (`terms`, `term`, `roots`,
`atom_names`, `atom_name`, `atom`, `formula`) plus `optimize`, `add` and
`verify_integrity`; construction goes through the parser or serde. Tests inside
the crate build arenas as struct literals.

## One-sided, negation normal form

When parsing, terms on the left of `⊢` get negative polarity. Negation is
pushed down to atoms with `Term::dual()`, so there is no general negation
node, only `DualVar`. `A ⊸ B` becomes `A^⊥ ⅋ B`. Printing therefore gives
`A |- A` as `⊢ ~A, A`. Intuitionistic sequents use the same model (plan
decision D1): an ILL sequent is a one-sided sequent of a particular shape,
read back by `Reading` (below); there is no second data model.

## The intuitionistic reading

`occurrences/reading.rs` reads a one-sided sequent as a two-sided
intuitionistic one: `Reading::new(&forest)` gives every occurrence a
`Position`, `Input` (a hypothesis, or the antecedent of a goal) or
`Output` (the goal, or the antecedent of a hypothesis), and names the
`goal` root, or fails with a `ShapeError` (`describe(&forest)` for
formulas). This is Lamarche's polarization, and what the two-sided engine,
the checker, the two-sided derivation and the future essential nets read.

- **The grammar.** In output position `⊗ ⊕ & ! 1 ⊤ 0`, atoms `a`, and
  `A ⊸ B` stored as `A⊥ ⅋ B`; in input position the duals: `⅋ & ⊕ ? ⊥ 0
  ⊤`, `~a`, and `A ⊗ B⊥` for a hypothesis `A ⊸ B`. The position flips at
  the antecedent of an implication (an output `⅋` or an input `⊗`) and
  nowhere else. `Reading::implication(o)` returns (antecedent, consequent)
  for exactly those occurrences; `formula(o)` prints an occurrence as the
  intuitionistic formula its position makes it (`⊥` as `1`, an input `⊤`
  as `0`, an input `⊗` as `⊸`); `Display` prints `Γ ⊢ A`.
- **The choices, made deterministically.** A bottom-up pass computes which
  positions each occurrence can take (`⊤` and `0` both, `Var` output
  only, and so on); the first occurrence with neither, in descending id
  order, is `ShapeError::Formula` (a minimal offending subformula). The
  goal is the root that can only be output (two such roots:
  `SeveralGoals`; none that can be output: `NoGoal`), else the *last*
  root, by id, that can be output. Inside an implication the left factor
  is the antecedent when that reading works and the right one otherwise,
  so `b ⅋ ~a` reads as `a ⊸ b` too (the symmetric reading).
- **Ambiguity is real and cannot be resolved from the arena.** Only
  formulas built from `⊤` and `0` alone can stand on either side, and for
  those the written succedent is lost: `Sequent::optimize` sorts the roots
  by term and hash-conses, so a `⊤`-built succedent equal to a hypothesis
  subterm gets an early id and another root becomes the goal (`0, ⊤ ⊢ ⊤`
  prints as `0, 0 ⊢ 0`). A review brute-forced 607 464 such sequents and
  found no pair of readings that differ in provability, so the verdict is
  unaffected; only the two-sided print and derivation show the other
  reading. Engine, checker and view all call `Reading::new` on the same
  forest, which is what keeps them consistent; never hand one of them a
  reading of a different forest.
- **`Fragment::name_in(mode)`** is the mode-aware name (`IMLL`, `IMLL with
  units`, `IALL`, `IMALL`, `IMELL`, `ILL`; the classical `Display` is
  unchanged), and the JSON of an `Outcome` uses it; a `Fragment` reads
  back from either spelling.

## Terms, kinds, fragments

- `Term` (`sequents/term.rs`) is the full classical connective set; `Kind` is
  the same enum without payloads, for the forest's per-occurrence array and
  for tables keyed by connective. `Kind::polarity()` is `None` for literals
  because a literal's polarity is the per-atom bias (below).
- There are no fragment-typed sequents. `Fragment` (`fragment.rs`) is a value:
  five connective-class flags, the usual fragments as constants
  (`MLL`, `MLL_WITH_UNITS`, `ALL`, `MALL`, `MELL`, `LL`; `ALL` is
  additive-only, `LL` is everything), `contains` as the subset order, and
  `Display` naming the smallest named fragment containing the value. The
  empty fragment (atoms only) prints as `MLL`. `Sequent::fragment()` is the
  detection. `Mode` (same file) is what the user asks beyond the sequent:
  intuitionistic, affine, Mix; three bools with builder methods.

## The occurrence forest

`Forest` (`occurrences/mod.rs`) is what every engine, checker and net works
on; a sequent inside a search is an `OccSet` of occurrence ids, never a list
of terms. Invariants the code relies on:

- **Numbering is DFS preorder**: roots in `Sequent::roots` order, and under
  a binary connective the left subterm before the right one. Hence a root
  precedes its subtree, `left(o) == o + 1`, `right(o)` follows the left
  subtree, and `subtree(o)` is the id range `o .. o + size(o)`; `is_below`
  is two comparisons. Renderers (proof nets, SVG) depend on this child order.
  The numbering is a pure function of the sequent, so ids are stable across
  runs and serializations of the same sequent; never memoize across forests.
- Two occurrences of one arena term (a shared subterm, or repeated roots)
  are distinct ids with the same `TermId`.
- `parent` is stored raw as `u32` with `u32::MAX` for a root; the accessor
  returns `Option<OccId>`. That is why a forest refuses a sequent with
  `u32::MAX` or more occurrences (`Error::TooManyOccurrences`), which only a
  deeply shared arena from JSON can produce.
- **Atom bias**: per atom, the literal (`Sign::Var` or `Sign::DualVar`) with
  fewer occurrences is positive; a tie makes `Var` positive. So an atom that
  occurs with one sign only has all its literals negative, which is intended:
  they are never focus candidates. The bias is computed once per forest and
  every engine must use it; `polarity(o)` already applies it to literals.
- Literal lists are one `Box<[OccId]>` in CSR layout, grouped by atom, then
  sign (`Var` first), ascending ids within a group; `literals(atom, sign)`
  slices it. `all_literals()` is the whole thing.
- The forest owns a clone of its `Sequent` so that `formula(o)` can print.
  Everything else per occurrence is a `Box<[u32]>` or narrower; keep it that
  way (no per-occurrence heap objects, no strings).
- `lca` is a parent walk from the first argument and is `None` across roots;
  the net search's cycle rejection is only valid within one root.

`OccSet` (`occurrences/set.rs`) is `Box<[u64]>` with the forest's width fixed
at creation (`Forest::empty_set`, `root_set`, `OccSet::empty(len)`).
Word-wise operations debug-assert equal widths; combining sets of different
forests is a bug the release build will not catch. `Hash` is over the words,
through the crate's `hash::HashMap` (foldhash with a fixed seed: reproducible
runs, no OS randomness, works on wasm). `submasks(len)` enumerates the
submasks of a compacted member list (at most 63 members) in Gray-code order,
yielding the flipped position so the two sides of a split are updated by one
`toggle` each; the empty submask is the starting state, not an item.

## Proofs are terms over occurrence ids

`Proof` (`proofs/mod.rs`) owns its `Forest` and an arena `Box<[Node]>` of
rule instances; `Node` is a 16-byte `Copy` enum (a static assertion pins the
size): the rule, the occurrence it acts on, and the premises as `NodeId`s.
The rules are the dyadic calculus's (`⊢ Θ ; Γ`): `Quest` moves a formula
into `Θ`, `Copy` uses a `Θ` formula without consuming it, `Bang` needs an
empty linear zone; `Weaken` exists for affine mode and `Mix` for Mix. The
ILL rules have no tags of their own: on the lowered sequent (D1) every one
is a classical rule (`⊸L` is `⊗`, `⊸R` and `⊗L` are `⅋`, `&L` is `⊕`, `⊕L`
is `&`, `1L` is `⊥`, `0L` is `⊤`, `!L` is dereliction), so the same terms
serve step 8. Invariants:

- **A premise precedes its conclusion** (strictly smaller index), the root
  is the last node, and every node is reachable from the root. `Proof::new`
  verifies the bounds and the order of what the root reaches, drops the
  rest (an engine's arena holds the subproofs of failed branches) and
  renumbers; it does not check the proof. A subproof two nodes share (a
  memo hit) is stored once, so the arena is a DAG and the derivation view
  unfolds it.
- A node never records the sequent it proves; the checker derives it. So
  `Top(o)` does not say what context the `⊤` absorbs, and an engine need
  not record it.
- Occurrence ids are those of the proof's own forest; a proof is meaningful
  only with it. Serialization stores the sequent and rebuilds the forest,
  which is deterministic (D5).
- In affine mode the spec's relaxed rules (a non-empty context under `!`
  or `1`, `Γ ⊋ {p⊥}` in the axiom) are not rules of the term: the engine
  emits `Weaken` nodes *below* the `!`, `1` or `ax` for the surplus. The
  checker's `!`, `1` and `ax` are the same in both modes. `Weaken` of a
  `?` formula is the standard `?w` and is allowed in every mode; the dyadic
  engines express the same thing as a `Quest` whose formula goes unused.

## The checker

`proofs/check.rs` is the reference for what a proof is. It shares no code
with any engine and must not: an engine's proofs are validated by something
that cannot repeat the engine's mistakes. Engines only call `Proof::check`.

- One bottom-up pass in arena order, no recursion. Each node gets a
  `Derived { theta, gamma, any }`: `gamma` is the linear zone as a sorted
  multiset (`proofs/multiset.rs`, a `Vec<OccId>` with repeats: copies can
  repeat an occurrence and every subformula of a copied formula), `theta`
  the **least** unrestricted zone the subproof needs (`Copy` adds the
  occurrence, `Quest` removes its subformula), and `any` says a `⊤` above
  absorbs any further linear context. The claim the tests and the review
  rest on: `Derived` characterises exactly the set of dyadic sequents the
  subterm proves. At the root, `theta` must be empty (every copy has its
  `?` step below it) and `gamma` must equal the roots, or be a sub-multiset
  of them under `any`.
- The `any` flag is what makes `⊤` checkable without a recorded context:
  consuming a subformula from an absorbing premise succeeds when it is
  absent; `⊗` and Mix sum the zones and or the flags; `&` needs equal zones,
  or the absorbing side's zone included in the exact side's (result exact),
  or the pointwise maximum when both absorb; promotion needs the zone empty
  after its subformula and resets the flag, because `⊢ !A, Δ` holds only for
  `?` contexts, which `Θ` already covers. The derivation view instantiates
  the absorbed context top-down.
- **Intuitionistic mode is the classical check plus the one-succedent
  condition** against the sequent's `Reading` (`Problem::Shape` when there
  is none). Every ILL rule is a classical node, so what is checked is only
  that every sequent of the proof has one goal: (R1) every derived `gamma`
  holds at most one occurrence in output position, and exactly one unless
  `any` (a `⊤` above supplies the goal); (R2) in `take`, an absent child in
  output position may be absorbed by `any` only if the premise's zone has
  no output already (else the premise's sequent would have two goals);
  (R3) `Weaken` never weakens an output; Mix is `Forbidden`. Any failure
  is `Problem::Succedents(n)`. These are sound and complete for "some
  top-down instantiation of the absorbed contexts is an ILL derivation":
  `Ax` has one output by construction, `Bang` resets `any`, `Θ` holds only
  input occurrences (subformulas of `?`), and at a `⊗` the fixed output
  counts of the two premises sum to two, so the side with none must
  absorb, which R1 guarantees. The derivation view builds that
  instantiation. A review compared the checker with an independent
  two-sided prover on 8 000 classical proofs of ILL-shaped sequents.
- The rule interpretation was reviewed against an independent top-down
  reference checker with random proofs, mutants and random terms (step 2's
  report); the intricate cases that review named are pinned in
  `check.rs`'s `accepts_every_rule`, so keep them when the checker changes.
- `CheckError` reports ids, not formulas: `node`, its `rule`, the derived
  `premises` as `Dyadic` sequents and a `Problem`. `Display` prints ids too;
  `describe(&forest)` prints the same message with formulas (nodes keep
  their ids), which is what the CLI shows. Both go through one writer
  (`CheckError::write`), so a new `Problem` gets one arm.

## The derivation view

`proofs/derivation.rs` unfolds a checked term into the tree of standard
one-sided inferences (`Inference { sequent, rule, principal, premises }`,
premises before conclusions, root last, `Rule` with the usual spellings).
The sequent is the ids in ascending order with repeats; `principal` is a
position in it, `None` for `ax` (its sequent is the two literals) and Mix.
Step 5 reads axiom links off the `ax` inferences (or the `Ax` nodes).

`Derivation::two_sided` (`Proof::two_sided_derivation`) is the same tree
read two-sided: it checks the proof in intuitionistic affine mode (so
`wk` shows where used), keeps the `Reading` (`Derivation::reading`), names
each rule by the position of its principal formula
(`Rule::intuitionistic`: `⊗` on a hypothesis is `⊸L`, `⅋` on one `⊗L`,
`⊕₁` on one `&L₁`, `&` on one `⊕L`, `⊥` is `1L`, an input `⊤` is `0L`, a
dereliction `!L`, a promotion `!R`, `?c`/`?w` are `!c`/`!w`; the axiom and
`wk` keep their names), and the renderer prints `Γ ⊢ A` with the
hypotheses in id order. The one place the reading changes the tree: at a
`⊗` where both premises absorb, the goal among the absorbed formulas goes
to the premise that has none (the classical rule gives everything to the
left one), which is what makes the instantiation an ILL derivation. The
`Inference` sequents are the same ids as one-sided; only the rule names
and the rendering differ.

The dyadic-to-standard translation is *not* Andreoli's (which contracts all
of `Θ` at every `⊗` and weakens all of it at every leaf): the standard
sequent of a subproof is `⊢ ?Θ, Γ` for the least `Θ` the checker derived,
so structural rules appear only where needed:
- `Copy` is `?d`, plus `?c` below it when the formula is used again above.
- `Quest` is no inference when its formula is used above (the sequents
  coincide) and `?w` otherwise.
- `⊗` and Mix contract, below the rule, the `?` formulas both premises use.
- `&` weakens, above a premise, the `?` formulas only the other premise
  uses, unless a `⊤` in that premise absorbs them.
- Whatever a `⊤` absorbs flows down to it through every rule; a `⊗` split
  gives the absorbed part to the absorbing premise.
The builder recurses over the tree, so its depth is the derivation's
height, and a DAG with heavy sharing unfolds to a tree exponentially
larger than the arena. `Derivation::new` checks the term first (any mode)
and fails as the checker would.

`Rule::Open` is the rule of an open goal in the derivation of a proof in
progress (below) and appears nowhere else; `Rule::classical` maps every
two-sided name back to the classical rule it is on the one-sided sequent,
and `Rule::from_str` reads a rule from its name or an ASCII spelling.
`Derivation::from_parts` wraps inferences that already have the
derivation's shape, and `Derivation::of_goal` unfolds a proof whose root
concludes a goal rather than the roots (as `prove_goal` returns it) into
inferences, for grafting.

`proofs/fmt.rs` draws the tree: premises side by side, bottom-aligned, three
columns apart; a bar of `─` spanning their conclusions or the conclusion,
whichever is wider, with the rule name after it; the conclusion centred
under the bar; an open goal is its sequent alone, with no bar, which is
how a leaf without a rule is told from a closed one. Widths are character
counts (every symbol used is one column in a monospace font), lines are
trimmed on the right, and there is no trailing newline. The renderings are
pinned in tests, so a layout change is a test change.

## Interactive proving

`proofs/interactive.rs` (feature `interactive`) is the state a client
holds for step-by-step proving: the forest, the mode, the inferences of a
derivation of the standard calculus with open goals as leaves, and the
steps taken. It reuses `Inference` and `Rule` and shares no second
representation with anything. What the code relies on:

- **The arena is top-down.** Inference 0 concludes the sequent; a step
  closes one open goal in place (its rule, principal and premises are
  filled in) and appends the goals it opens at the end, so a premise has a
  larger index than its conclusion, the reverse of `Derivation`'s order;
  `derivation()` renumbers into postorder, so its ids are not the state's.
  Since steps only append, the inferences a step added are a suffix of the
  arena as long as no later step exists, which is why `undo` is "truncate
  to the smallest index in the closed goal's subtree, reopen the goal" and
  why the history is just the list of goals closed, in order. A search
  graft is one step (its whole subtree is the suffix).
- **Positions, not ids, address formulas**: `position` indexes the goal's
  sequent (ascending ids with repeats), as Click & coLLecT's
  `formulaPosition` does; the split of a `⊗` or Mix is the positions of
  the context formulas going left. Equal ids at different positions are
  interchangeable (the sequent is a multiset), so which copy the client
  picks does not matter.
- **Validation at application time is complete for the checker**, so that
  a derivation built through `apply` always translates into a term the
  checker accepts: `expand` checks the connective, the mode (`wk` only
  affine, Mix only with Mix), the context (`ax` exactly the dual, `1`
  alone, `!` with a `?`-only context, since the term's `Bang` needs the
  linear zone empty), the split, and in intuitionistic mode R1 (every
  premise has exactly one occurrence in output position) and R3 (never
  weaken the output). R2 of the checker (an absent output child absorbed
  by a `⊤` only if the premise has no output already) is implied: the
  linear zone the term derives for an inference is a sub-multiset of the
  inference's non-`?` formulas, so where the child is absent the zone's
  outputs are among the sequent's other formulas, of which R1 leaves none
  (`Θ` members are inputs, so a `Copy` never triggers R2 either). A
  fresh-context review confirmed this on about 2 500 random derivations
  in every mode, each translated and checked. In intuitionistic mode a rule is accepted under its
  classical or its two-sided name and recorded under the two-sided one,
  so a two-sided state's inferences look like `Derivation::two_sided`'s.
- **The standard-to-dyadic translation** (`Terms`, in `proof()`) is the
  view's table read upwards: every rule is its node; `?d` is `Copy(A)`;
  `?c` and `?w` are nothing; and every `?` formula gets its `Quest` node
  *where it enters the derivation*: below the rule that introduces it as
  a subformula (`Terms::premise`) or below the root for a `?` root. So a
  `?` formula is in `Θ` from its entry upwards, a contraction's second
  instance is the same `Θ` member, and a weakened instance is simply not
  copied (the unused `Quest` is what the view shows as `?w`). This keeps
  the dyadic linear zone free of `?` formulas above their entry, which is
  what `Bang` needs, and makes every `Copy` sit above its `Quest`, which
  is what the checker's empty root `Θ` needs. The term's own derivation
  view may place structural rules elsewhere than the user did (it
  contracts below a `⊗`, the user contracted above the root, say); the
  state's derivation is the user's tree, the term's is the checker's.
  `proof()` runs the checker on the term, always: the layer is not trusted
  more than an engine.
- **Search from a goal**: `close` calls `prove_goal` on the goal's
  sequent, and grafts `Derivation::of_goal` of the proof found; the goal's
  fragment is its own (`search::goal_fragment`), so the prunes are those
  of the goal, and the net engine never runs off the roots. The outcome
  returned is the search's, its proof the proof of the goal alone.
  `split_passes` lends the client the focused engine's count prunes
  (`focus::split_passes`, which builds the engine's `Rules` and tallies
  for the two sides) as a "this split cannot close" test; a split that
  passes may still fail.
- **Reading a state back** (`from_parts`, used by deserialization) checks
  the shape (root 0 concludes the sequent, every other inference is the
  premise of exactly one earlier one, sequents ascending and within the
  forest, a principal position exactly for the rules that have one),
  replays every closed inference (`replay` recovers the split of a `⊗`
  from the left premise less the subformula, and of a Mix from the left
  premise alone, whose first formula stands for the position; then
  `expand`'s premises must equal the recorded ones) and checks the history
  from the last step back: its entries are distinct closed inferences, and
  the inferences a step added, which are those of its subtree that exist
  at that point (the later ones belong to later steps), are the suffix of
  the arena then. So a loaded state is as trustworthy as one built through
  the API; the checker at the end is the final word anyway. A history that
  does not cover every closed inference is allowed (those steps are just
  not undoable). A review fed hundreds of API-built states through JSON
  and found the first version of these checks rejecting chains of steps,
  every Mix and every graft (`graft` now appends a found derivation in
  reverse so that premises keep larger indices); keep the round trip of
  such states in `core/tests/serialize.rs`.
- **The reading is recomputed** (`Interactive::reading`, O(n)) whenever
  intuitionistic mode needs positions, since `Reading` borrows the forest
  and the state owns it; `new` guarantees it exists.

## Proof search: the front door

`search/mod.rs` is what a front end calls: `prove(&sequent, mode,
&options)` and `prove_until(…, stop)` return `Result<Outcome, Error>`, and
`prove_goal(&forest, goal, mode, &options, stop)` decides any multiset of
occurrences of a forest, given in any order, `prove_until` being that on
the roots: the goal's own fragment (`goal_fragment`, over the subtrees)
picks the prunes and the engine, the net engine only for the roots
(`Error::NetGoal` when forced elsewhere, since a structure's conclusions
are the forest's roots), the additive path for any two additive-only
occurrences (`additive::search_goal`), the focused engine otherwise; in
intuitionistic mode a goal must have exactly one occurrence in output
position (`Error::GoalOutputs`). The proof of a goal other than the roots
has a root that concludes the goal, so `Proof::check` rejects it; only
`Interactive` consumes such proofs, by grafting their derivation. Where
`Outcome` carries the `Verdict` (`Proved(Box<Proof>)`, `Unprovable` only
after an exhaustive search, which with exponentials means a deepening
level that never hit the copy bound, `Unknown(Reason)`, with
`Reason::CopyBound` when every level hit it), the `Fragment` searched in,
the `Mode`, the `Engine` that ran, the `Statistics`, and `net`, the
`ProofStructure` the net engine found (`None` from the focused engine).
`Options` has private fields and setters (`memo_limit`, `recursion_limit`,
`engine`, `fragment`, `test_period`, `copies`, `jobs`, `portfolio`), the
constants `DEFAULT_MEMO_LIMIT`, `DEFAULT_RECURSION_LIMIT` and
`DEFAULT_COPIES`, which the CLI shows as its defaults, and `stack_size()`,
the stack a thread needs at the recursion limit, which sizes the CLI's
search thread and the parallel pool's workers alike;
`Reason`, `Statistics`, `Engine` and `Outcome` are `#[non_exhaustive]` so
later steps add variants and fields without a breaking change.
`Statistics` has one set of counters for both engines: `nodes` is stable
sequents for `focus` and literals chosen for `net`; `memo_hits`,
`memo_entries` and `splits` are the focused engine's, `links` and `tests`
the net engine's, and the others stay zero.

- The dispatch is plan decision D8. Unit-free MLL (the empty fragment
  included) in classical mode goes to `net` when no literal occurs more
  than `NET_MULTIPLICITY` (2) times (`prefers_net`: equal literals are
  interchangeable partners, and the linking search pays a permutation's
  worth of nodes for every wrong choice among them, which the focused
  engine's counts refute at once), else to `focus`. Multiplicity is a
  proxy, measured by the benchmarks (the `engines` runs of
  `bench/RESULTS.md`, read in `plan/reports/14-benchmarks.md`): the net engine loses on Horn encodings (literals six times and
  more, by one to four orders of magnitude) and on equal literals inside
  one pure `⊗` or `⅋` tree (a sequent of five blocks `x ⊗ x ⊗ x ⊗ x`
  against `~x ⅋ ~x ⅋ ~x ⅋ ~x` with one defect: over 10 s against 20 ms at
  multiplicity 4), and wins by up to five orders on literals repeated
  three or four times across different conclusions (`wide-m3`,
  `wide-m4`). No threshold serves both; the feature that separates them
  is equal literals under one pure tree, which the leaf symmetry break
  below would take from the net engine's weaknesses. Every other
  classical input, exponentials included, and everything in affine mode
  goes to `focus`.
  Before both: exactly two roots in the additive fragment with at least
  one additive connective go to `additive` (atoms alone stay with `net`).
  Intuitionistic mode first computes the `Reading`
  (`Error::NotIntuitionistic`, whose message has ids; the CLI describes it
  with formulas) and refuses Mix (`Error::IntuitionisticMix`: a Mix premise
  would have no goal); then the same rows, with `two_sided` in place of
  `focus`, and `net` on unit-free IMLL by the embedding (below).
  `Options::engine` forces an engine; `Engine::Net` on a fragment outside
  unit-free MLL, asserted or detected, is `Error::NetFragment`, and in
  affine mode `Error::NetMode`; `Focus` in intuitionistic mode and
  `TwoSided` in classical mode are `Error::EngineMode`; `Additive` on
  anything but two additive-only formulas is `Error::NotAdditive`. A new
  engine gets an `Engine` variant (its `Display` is its name in text and
  JSON), a row in `prove_until`, and a value of `--engine` in the CLI
  (`.claude/rules/cli.md`).
- **IMLL by embedding.** In intuitionistic mode the net engine runs on the
  one-sided sequent unchanged and its proof is returned as it is: every
  cut-free MLL proof of a sequent with one output-shaped root keeps
  exactly one output on every sequent (an all-input MLL sequent without
  units is unprovable, since every leaf has an output, so the split of a
  hypothesis `A ⊸ B` can never take the goal to the antecedent's side),
  hence any sequentialization of a classical net of an IMLL sequent passes
  the intuitionistic checker and no essential-net condition is needed for
  the verdict. With `1` the lowered sequent has units and goes two-sided.
- `Options::fragment` asserts a fragment: a sequent outside it is
  `Error::FragmentMismatch`, and the search runs in the asserted fragment,
  which switches off the prunes that only hold in the smaller one and
  picks the engine (`--fragment mall` on an MLL input runs `focus`).
- The crate has no clock (D11): a time limit is a closure the caller gives
  `prove_until`, polled once per node (a stable sequent, or a literal
  chosen) and, in the focused engine, once every `SPLITS_PER_POLL` (4096)
  splits of a `⊗` or Mix enumeration (`poll_splits`): one stable sequent
  of a 20-token Petri net enumerates tens of millions of splits whose
  premises fail in focus, and without that poll a 2 s limit ran past five
  minutes. It answers `Unknown (Reason::Stopped)`. The crate docs in
  `lib.rs` show the common path (parse, fragment, prove, derivation, JSON)
  as a doc test; keep it the shortest correct program when the API moves.
  The focused engine recurses on the caller's stack, bounded by
  `Options::recursion_limit`, and the net engine's sequentialization
  recurses to the derivation's height; a caller that raises the limit or
  proves a huge net runs the search on a thread with a larger stack.

## The focused engine

`search/focus/mod.rs` is the spec's MALL-Seq and MELL-Seq in one engine, for
every classical fragment up to full LL, with units, Mix, the exponentials
and affine mode as rule switches (`Rules`, from `Fragment` and `Mode`), and
the spec's two-sided engine for every intuitionistic fragment when given
the sequent's `Reading` (`Engine::TwoSided` is that configuration). Its
functions are the spec's rules: `asynchronous` (the phase `⊢ Θ ; Γ ⇑ L`),
`quest` (`?` into `Θ`), `prove` (a stable sequent), `focus` (`⊢ Θ ; Γ ⇓ F`),
`initial` (the two initial rules), `split` (the `⊗` rule), `mix`. What it
relies on:

- **Two-sided is one constraint.** Every rule of the two-sided focused
  calculus is a rule of this engine on the lowered sequent (`⊸R` and `⊗L`
  are `⅋`, `⊸L` is a `⊗` in input position, `!L` is `quest` plus a copy,
  and so on), and starting from one output-shaped root every rule keeps
  exactly one output on each premise by itself, except the split of a
  hypothesis `A ⊸ B`, where the goal must go with the consequent `B⊥`.
  So `split`, in its free enumeration, fixes the one output member of `Γ`
  on the consequent's side (`Reading::implication`) and enumerates the
  rest; the forced splits need no change (the dual of an output positive
  literal is a hypothesis in `Γ` or `Θ`, the dual of an input positive
  literal is the goal itself or nothing, `1` and `!` are output-only, a
  `0` factor fails), `Θ` holds only input occurrences, a leaf's `weakened`
  never sees an output (debug-asserted), promotion needs `Γ` empty as
  before, and the count prunes are necessary conditions on the lowered
  sequent, hence sound. Mix is refused before the engine runs. The memo,
  the copy budget, the loop check and the pools are indifferent to
  positions: the key `(Θ, Γ)` determines the two-sided sequent. A
  fresh-context review compared the engine with an independent unfocused
  two-sided prover on about 60 000 sequents over every ILL connective,
  linear and affine, with no disagreement.

- **Dyadic sequents.** `Θ`, the unrestricted zone, is an `OccSet` of the
  subformulas of the `?` formulas decomposed on the branch; it only grows
  along a branch, is shared by every premise, and a `?A` whose `A` is
  already there changes nothing. `Γ`, the linear zone, is a `Context`
  (`focus/context.rs`): a bitset plus a sorted list of the extra copies of
  occurrences present more than once, empty until a copy repeats an
  occurrence (a copied `~a ⅋ ~a` releases the same `~a` twice), which is
  the one allocation on the hot path. Member lists (`gamma.iter()`) carry
  repeats, and a split enumerates positions, so its Gray-code toggling
  uses the mask bit, never `contains`.
- **Stable sequents only.** The asynchronous phase runs to completion (`⅋`
  opens, `⊥` drops, `⊤` closes with a `Top` node and the pending `⅋`/`⊥`
  nodes wrapped around it, `&` branches on copies of the state, `?` moves
  its subformula into `Θ` under a `Quest` node); what reaches `prove` is
  `Θ` plus a `Γ` of positive formulas and negative literals, and only those
  are memoized. `search_goal` starts from any multiset of occurrences with
  an empty `Θ` (an interactive prover's open goal); `search` starts from
  the roots.
- **The copy budget.** A copy (rule D2: focus on a `Θ` member, which stays
  there, under a `Copy` node) costs one unit of a per-branch budget passed
  down the calls; so does the initial rule `⊢ Θ, p⊥ ; · ⇓ p`, emitted as
  `Copy(p⊥)` above `Ax(p, p⊥)`, so that the bound counts every `?d` of the
  derivation. `run` deepens the budget from 0 to `Options::copies`. A
  level whose search skipped a copy for lack of budget sets `exhausted`;
  `Unprovable` is answered only by a level that ends with the flag clear,
  and `Reason::CopyBound` when every level set it. The flag is saved and
  cleared around each stable sequent's decision so the memo entry can say
  whether *that* subtree was cut. Without exponentials there is one level
  with budget 0 and nothing can set the flag.
- **Memo contract with the bound** (`focus/memo.rs`). The key is both
  zones. `Proved(NodeId)` is a fact at any budget (a proof is a proof; one
  found with more copies than the current level allows is still returned,
  so the bound limits the search, not the proof: `Proved` at level `k`
  does not mean a proof with at most `k` copies per branch, and a
  reported minimum would have to be budget-aware).
  `Failed(Complete)` (the subtree was explored to the end without hitting
  the budget, and without a prune that depends on an ancestor, below) is a
  fact at any budget, since more budget adds nothing that was not tried.
  `Failed(Exhausted(r))`, cut by the budget with `r` copies left, applies
  only when at most `r` are left now (`Memo::get`), and its hit sets
  `exhausted`; a later entry only raises `r`, and `Complete` or `Proved`
  replace it. Entries survive across levels; that is where the
  re-exploration of deepening is recovered. Never memoize across forests.
- **The loop check** uses the branch stack of stable sequents (`stack`,
  live up to `stack_len`, entries reused), on with exponentials only: a
  stable sequent equal to an ancestor is pruned, because a smallest proof
  of the ancestor never passes through it. Such a failure is a fact about
  the branch, not the sequent: `dependency` records the shallowest
  ancestor depth a prune below relied on, a failure that carries a
  dependency on an ancestor is not memoized, and the dependency is
  discharged at that ancestor, whose own failure is genuine (a proof of
  the repeat would be a proof of the ancestor). Order in `prove_stable`: a
  `Proved` or `Complete` memo entry answers first; then the stack; then an
  `Exhausted` entry, so that a repeated sequent is pruned rather than
  reported as cut by the budget. Pruned branches never set `exhausted`.
- **The spec's affine prune is wrong and is not implemented.** It prunes a
  stable sequent that *contains* an ancestor as a multiset, arguing that
  weakening shortens the proof; but weakening turns a proof of the smaller
  sequent into one of the larger, never the reverse, and `⊢ ?(a ⅋ ~a)` is
  provable only through `⊢ a ⅋ ~a ; a, ~a`, which contains the root. A
  review found 426 wrong `Unprovable` verdicts in 5 200 random affine
  sequents with it. The same holds with the zones equal. So affine mode
  is not a decision procedure here: it runs the bounded, loop-checked
  search of linear mode with weakening, and answers `CopyBound` like it.
  (The prune in the other direction, a sequent *contained in* an
  ancestor, is sound but useless: it is the useful branch.)
- **Affine mode** has no relaxed rules in the term: a leaf (`Ax`, `One`,
  `Bang`) weakens every leftover member of `Γ` below itself (`weakened`,
  one `Weaken` per copy); weakening never goes above a promotion. Nothing
  but `0` forces a split in affine mode (`forced_side`), every dual pair
  or literal with its dual in `Θ` closes a stable sequent (`initial`, which
  goes on to the next pair when the budget refuses a copy), `1` and `!`
  are candidates with any context, a `0` is not fatal (it is weakened at a
  leaf), and the interval check and the count equation are off
  (`Rules::intervals`, `Rules::equation`): weakening discards any
  imbalance.
- **The rules with `Θ`.** D1 candidates first (`⊗`, `⊕`; `1` and `!` only
  when alone), then the copies from `Θ`: a member with an unconsumed copy
  in `Γ` is skipped (a second copy cannot help before the first is used,
  and the two are the same formula), those with a literal whose dual is a
  member first (`meets`), then by id; a negative `Θ` member is copied and
  released. `!A` in focus needs `Γ` empty and releases `A` into an empty
  `Γ` under a `Bang`. A positive-literal factor of a `⊗` takes its dual
  from `Γ` when there is one and otherwise leaves its side empty for the
  `Θ` initial rule; the dual in `Γ` first loses no proof, since the copies
  are the same formula and a proof that spends this one elsewhere and
  copies here is the same proof with the roles swapped, but the swap moves
  a copy to another branch, so a sequent may need one level more than its
  best proof's copies per branch (`⊢ ?~p, ?p, ~p, p ⊗ ⊥` is proved at
  bound 2, not 1).
- **Memo validity without exponentials** is unconditional, as before: cut-
  free provability of a set of occurrences depends on the set alone, and
  every entry is `Proved` or `Complete`. When the table is full it is
  cleared (`Options::memo_limit`; zero switches it off; the arena is
  append-only, so a `Proved` id never dangles).
- **A `0` is fatal only without a `⊤`.** The spec calls a `0` in a stable
  sequent fatal, but `⊢ 0, ⊤ ⊕ b` is provable through the `⊕`; the
  immediate failure applies only when no member has a `⊤` below it
  (`Tally::absorbs`), and not in affine mode. The other immediate tests: a
  dual pair succeeds; a literal-only sequent fails without Mix and with an
  empty `Θ`; an unbalanced sequent fails.
- **Counts** (`focus/counts.rs`): per occurrence a sparse row of intervals
  per atom (literals `±1`, `⊗`/`⅋` sum, `&`/`⊕` hull, units nothing), an
  `absorbs` flag (a `⊤` at or below it: the row is meaningless and any set
  containing the occurrence passes), and a `weight` `t − p − #1 + #⊥`. The
  interval check is sound in every fragment without exponentials (proof by
  induction on the rules, with `⊤` covered by the flag and `0` as `(0, 0)`).
  With exponentials, an atom with a literal below any `?` or `!` in the
  problem gets no row entries anywhere (its copies and discards break the
  balance; `Θ` members are not in the tally, and they contain only such
  atoms), and a `⊤` below any `?` or `!` switches the check off altogether
  (`absorbs_from_copies`: a copy of it absorbs any imbalance). The hull
  for `&` is the spec's choice; the intersection would be sound too and
  stronger, and is a follow-up. The count equation
  `c = t − p − #1 + #⊥ + 2` (`≥` with Mix, and `>` for a Mix to be worth
  trying) is only sound without additives, additive units or
  exponentials and without weakening, and `Rules::equation` switches it on
  for exactly those cases; `⊢ a ⊕ b, ~a` is the counterexample the spec
  names. A `Tally` keeps a set's sums incrementally, so a split moves one
  row per flip.
- **Focus candidates.** Every `⊗` and `⊕` of a stable sequent; `1` and `!`
  only when alone (they need an empty context; any, in affine mode); never
  a literal (a positive literal in focus succeeds only in the initial
  cases). Order: `1` and `!`, a `⊗` with a forced split, `⊕`, a `⊗` whose
  split is enumerated; ascending ids within a class; then the copies.
  This order is what makes the run deterministic, with the memo, which is
  only looked up, never iterated.
- **Forced splits.** A factor that is a positive literal takes exactly its
  dual from the context, and the first dual occurrence when there are
  several (they are the same formula, so the residues are equal
  multisets), or nothing when the context has none; a factor `1` or `!`
  takes the empty context; a factor `0` fails the candidate, not the
  sequent. `⊤`, `⊥` and negative literals force nothing: `⊢ ⊥ ⊗ b, a, ~a,
  ~b` needs `{a, ~a}` on the `⊥` side.
- **`split_passes`** is the count test of a split as a function (the
  engine's `Rules::new` and two tallies), for the interactive state's
  helper; keep it equal to `sides_pass`.
- **Free splits** enumerate the submasks of the compacted members in
  Gray-code order (`submasks`), the empty submask first, two tallies moved
  per flip, and both sides must pass the counts before either premise is
  searched. More than 63 members (copies counted) is
  `Reason::ContextTooWide` for the whole search, never a silent failure;
  the spec's lazy contexts or a branch-and-bound over the
  members are the ways past it.
- **Mix** is tried last on a stable sequent, after the copies, with the
  first member fixed on the left so each partition comes up once, the
  trivial partition skipped, and each part decided by `prove` with the same
  `Θ` and budget, so the memo shares parts between partitions. Refuting a
  wide sequent with Mix costs about `3^k` stable sequents for `k` members.
- **Recursion.** `prove`, `focus` and `asynchronous` count one level each
  (at most three per occurrence, plus one per `?` and per copy);
  `Options::recursion_limit` stops the search with `Reason::RecursionLimit`.
  Measured stack per level: under 2 KiB in debug builds, under 512 bytes
  in release, so the default of 2048 fits an 8 MiB main-thread stack.
- **No allocation per node once warm**: sets, contexts, keys, member lists
  and tallies come from pools on the engine (`take_*`/`give_*`); a leaked
  buffer on an error path only costs an allocation later. The memo insert
  clones its key, and a repeated occurrence grows a context's extra list;
  those are the allocations per stable sequent.
- **The atom bias hurts Horn clauses.** The forest makes the rarer literal
  positive, so clause bodies whose atoms also appear as hypotheses are
  usually negative and their `⊗` splits are enumerated instead of forced;
  refuting `families::three_partition` with bins of four takes 51 s in
  release mode for that reason (the `3-partition-no` family at size 4).
  A bias override is performance-pass material.
- **The memo can change decisiveness within the bound, never a verdict.**
  An `Exhausted` entry is a fact about the sequent alone, the loop check
  about the branch, so a run with the memo may answer `Unknown` where a
  memo-free run answers `Unprovable` (or the reverse); the generated tests
  assert only that the two never contradict.
- **`exhausted` and `dependency` are engine-wide flags** saved, cleared and
  restored by hand inside `prove_stable`; an early return between the
  decision and the restore, or a stack push anywhere else, silently
  breaks the level's completeness claim.
- **Every proof passes the checker**: `debug_assert!` in `search`, and
  every test that gets a proof calls `check`. The test-only generator
  `search/generate.rs` builds random provable sequents (and mutants of
  them) for every combination of units, additives, Mix and exponentials,
  and reports the most derelictions on one branch of the proof it read
  the sequent off, which bounds the copies the engine needs; a new rule
  set extends it rather than writing new positives by hand.

## Proof nets

`nets/mod.rs` is the proof-net model for unit-free MLL, with or without
Mix; `ProofStructure::new` refuses any other fragment
(`Error::NetFragment`), and there is no net for affine mode (the CLI
refuses it before searching); in intuitionistic mode the net is the one of
the one-sided sequent. A structure is the forest
plus `partner` (one `u32` per occurrence, `NONE` for unlinked literals and
connectives), the stack of links in the order they were made, the coloured
graph (`graph.rs`) and the `⅋`-free skeleton (`skeleton.rs`); `mix` says
whether Mix is allowed, which is the difference between the two criteria.
What the code relies on:

- **Links are a stack.** `link(x, y)` pushes and `unlink()` pops the last
  link: the skeleton's union-find has an undo log (union by rank, no path
  compression, one entry per union, so undo is O(1) and find is
  logarithmic), and a backtracking search takes links back in reverse
  order anyway. `link` debug-asserts that the literals are dual and
  unlinked; `from_links` and deserialization validate the same at the
  boundary and return `NetError`.
- **The coloured graph** (`graph.rs`): vertices are the occurrences, edges
  the premise edges of every `⊗` and `⅋` plus the links, in CSR layout
  with the parent edge in a vertex's first slot, then its children, and
  for a literal its axiom slot last (`NONE` while unlinked), so `link` and
  `unlink` write two slots. The colouring is the spec's: the two premise
  edges of a `⅋` share a colour of their own, every other edge has its own
  colour, so a cycle survives some switching iff it is properly coloured.
  `⊗` premise edges must never share a colour (that would forbid the cycle
  `⊗ – A – ~A – ⊗` in `⊢ A ⊗ ~A` and accept a wrong structure), and `⅋`
  premise edges must always share one (else `⊢ A ⅋ ~A` would be rejected).
  The colours are not stored: with this colouring, Yeo's deletion
  condition ("no component of `G − z` meets `z` in two colours") is
  exactly "every present edge of `z` is a bridge" for a vertex that is not
  a `⅋`, whose incident colours are all distinct, and "the parent edge is
  absent or a bridge" for a `⅋`, whose premise edges share a colour.
  Bridges come from one iterative Tarjan search per round (`search`,
  which skips the search-tree parent vertex, valid because the graph is
  simple: a literal's tree edge and its link never join the same pair).
- **The Yeo test is exact on partial structures.** `acyclic` deletes, in
  rounds, every vertex deletable by the round's bridges and stops when a
  round deletes nothing: everything gone means no switching cycle, and
  otherwise the vertices left carry one. Deleting several vertices found
  deletable in one round is sound because deletability only grows as
  vertices go (components of `G − z` can only split). A deletable vertex
  never lies on a properly coloured cycle, and Yeo's theorem gives a
  deletable vertex whenever there is none, which is the exactness. An
  unlinked literal is a leaf and lies on no cycle, so
  `is_acyclic(&mut Scratch)` answers the same question about a partial
  structure; it allocates nothing, the `Scratch` (from
  `ProofStructure::scratch`) holds the bitsets and the search arrays.
  The cost is one search per round, and the number of rounds is the
  nesting depth of cycles that pass through both premises of a `⅋` (each
  round peels one layer of them); a forest of unlinked trees goes in one.
- **Connectedness is a count, checked after acyclicity only.** When no
  switching has a cycle, every switching is a forest with
  `2t + p + k` edges (`t` tensors, `p` pars, `k` links), so it is a tree
  iff that is `V − 1`. Before acyclicity the equation says nothing. With
  Mix the equation is dropped: correctness is acyclicity alone. The empty
  structure is not a net in either case (`NetError::Empty`), since no rule
  concludes `⊢`. `is_correct` requires completeness first
  (`NetError::Unlinked`).
- **Witnesses are for humans and tests, not the hot loop.** `is_correct`
  allocates its own scratch. A `SwitchingCycle` is isolated from the
  vertices the procedure got stuck on by removing each edge among them in
  turn and keeping it out when a cycle survives; an edge found necessary
  stays necessary as edges go, so one pass leaves exactly one cycle, at the
  cost of one deletion procedure per edge. `Disconnected` lists the parts
  of the switching that keeps every left premise, each by its vertices
  without a parent edge there (roots and right premises of `⅋`s); a part
  may hold no root at all. `nets::graph`'s tests check every witness
  against a brute-force enumeration of switchings and assert the two
  criteria agree; keep that test when the criterion changes.
- **Sequentialization** (`sequentialize.rs`) is the splitting-tensor
  lemma on the *plain* graph of a sub-net (tree edges and links, no
  switching): a `⊗` conclusion is splitting iff its premise edge is a
  bridge there. The spec's "delete it and count components under one
  switching" is wrong: under any switching, every `⊗` conclusion of a
  correct net leaves exactly two components (the switching is a tree and
  loses two edges), so the count cannot tell. Per stage: open every `⅋`
  conclusion (its rule goes below the rest), one search from the
  conclusions gives the parts and the bridges, parts are joined with Mix
  (only with Mix; a sub-net of a connected net is connected), two
  literals are an axiom (`Ax(min, max)`), else the splitting `⊗` with the
  smallest id is applied and the conclusions reached from its left premise
  go left. Handled conclusions are deleted in the scratch, so a sub-net is
  what its conclusions reach. O(n²) in the net's size; the recursion is as
  deep as the derivation. The proof is checked in a `debug_assert!` and the
  round trip derivation → net → derivation is a test.
- **Nets and terms.** `from_proof` reads the links off the `Ax` nodes and
  returns the net only if it is correct; every proof the checker accepts
  gives a correct net, two proofs that differ by rule permutations give the
  same one, and `sequentialize` gives one proof per net, so the net is the
  canonical form of an MLL proof. The net of a term is meaningful only over
  the term's forest, like the term itself.
- What the net engine keeps outside the structure: the per-atom counts,
  the copies of literal conclusions, the explicit stack, statistics, and
  one `Scratch`. The structure offers `partner`, `unlinked`, `link`,
  `unlink`, `same_component` (the skeleton's rejection), `is_acyclic`, and
  `Forest::lca` is the other O(1) rejection.
- The text form (`Display`: the sequent, `~A[0] — A[2]` per link sorted
  by first id, then `proof net`, `proof net with Mix` or `not a proof net:
  ` with the reason in formulas) and the JSON form are pinned in tests.

## The net engine

`search/net.rs` is the spec's MLL-Net: axiom-linking search over a
`ProofStructure`, for unit-free MLL with or without Mix. A cut-free proof
of MLL is its linking, so the only choices are which dual literals to pair.
What the code relies on:

- **Preprocessing** (`counts_admit`): `c = t − p + 2` (`≥` with Mix) and
  as many `a` as `~a` per atom, else `Unprovable` at once. Both are
  necessary (induction on cut-free proofs; Mix only raises `c`). The
  equation is also sufficient for connectedness once a complete linking
  is acyclic: every switching keeps `2t + p + k` edges, the forest has
  `c = 2k − t − p` formulas (every connective is binary), so
  `c = t − p + 2` is `k = t + 1` is "`V − 1` edges", a tree. That is why
  `run` calls a complete linking that passed the exact test a proof net
  without another connectedness test, and why `search` then `expect`s
  `sequentialize` (which re-runs `is_correct`) to succeed. Do not relax
  the preprocessing or move it after the search. The empty sequent fails
  the equation in both modes.
- **Two constant-time rejections** of a candidate link, sound in every
  partial linking because a switching cycle survives every extension:
  the lowest common ancestor of two literals of one conclusion is a `⊗`
  (the tree path plus the link is a cycle kept by the switchings that
  keep the path's `⅋` premises: *some* switching, which is all the
  criterion needs; it is not kept by every switching when a `⅋` lies on
  the path, so no stronger rule follows from it), and the `⅋`-free
  skeleton already joins them (`same_component`: a cycle with no `⅋`
  premise edge, kept by every switching). `Forest::lca` is `None` across
  roots, and the rule is only valid within one root.
- **Symmetry breaking for equal literal conclusions only.** Conclusions
  that are the same literal (same atom and sign, both roots) are chained
  in id order (`copy_before`, `copy_after`), and a link is admissible only
  if the partners of the chain ascend with the conclusions. Sound because
  swapping two such conclusions is an automorphism of the structure, so it
  maps nets to nets, and the lexicographically least member of an orbit
  (partners listed by literal id) has ascending partners: if consecutive
  copies `x_i < x_{i+1}` had partners `p > q`, the swap differs at
  `{x_i, x_{i+1}, p, q}` and is smaller at `min(x_i, q)`. The argument
  covers every group at once, however the partners lie. It does **not**
  extend to equal compound conclusions with a first-literal key: two
  copies of `F` and two of `G` whose first literals link into each other's
  copies at non-first positions have an orbit in which no member sorts
  both groups; a compound extension needs keys under roots that no
  symmetry moves, and is a follow-up. The spec forbids symmetry breaking
  inside formulas, which is the loss on Horn encodings (below).
- **Choice order** (`choose`): the unlinked literal with the fewest
  admissible partners, ties to the first in atom order, `a` before `~a`,
  then id; every unlinked literal is inspected, so a literal without an
  admissible partner is a dead end found now (forward checking). The
  counting of one literal stops at the best count so far, which never
  hides a zero. Partners are then enumerated in id order over
  `forest.literals(atom, !sign)`. All of this makes the run deterministic.
- **The exact test** (`is_acyclic`) runs after every link on a structure
  of at most 200 occurrences and after every fourth link above that
  (`Options::test_period` overrides; zero counts as one), and always on a
  complete linking. Between tests a doomed branch is followed for at most
  `period − 1` links. `is_correct` (witnesses, allocation) is never called
  in the loop; `sequentialize` calls it once at the end.
- **The stack** (`Frame { literal, next }`): every frame but the top has
  its current link made, the top is looking for one; `next_partner`
  moves `next` past the partner it returns, so a linking is tried at most
  once; a dead end, a failed test or an exhausted frame takes the last
  link back, always in stack order, which the structure's undo log
  requires. `remaining[atom]` (unlinked pairs per atom) follows every
  link and unlink. The stop condition is polled once per node, in
  `decide`, so a frame's candidates run between two polls.
- **Where it loses.** Horn encodings (Matsuoka's Partition and Lincoln's
  two-literal 3-Partition, `families::partition` and
  `families::three_partition_mll`) have few atoms
  with many occurrences, and the equal literals inside `b ⊗ b ⊗ b` and
  `~b ⅋ ~b` are interchangeable, so a wrong early choice costs a whole
  symmetric subtree before a cycle appears; the focused engine refutes the
  same sequents through the counts of each `⊗` split in milliseconds
  where the net engine needs seconds or does not finish. Distinct atoms
  and wide contexts are where the net engine wins. Symmetry breaking for
  the leaves of a pure `⊗` or `⅋` tree of equal literals (sound: the
  leaves of a `⊗` tree share every switching's component, and a `⅋` tree
  opens to interchangeable conclusions) is the follow-up the plan's step
  14 should measure; until then the dispatch routes MLL with a literal of
  multiplicity above 2 to the focused engine (`prefers_net`).
- **Every proof passes the checker** (`debug_assert!` in `search`, every
  test), and every net is the net of its proof (`from_proof` in the tests'
  `run`). The differential test against the focused engine
  (`agrees_with_the_focused_engine`) covers generated provable sequents,
  their mutants, doubled sequents (equal conclusions) and random
  balanced sequents from `generate::balanced`, which pass the counts and
  are mostly unprovable; extend it rather than pinning verdicts by hand.

## The parallel runtime

`search/parallel.rs` (feature `parallel`, off by default, on in the CLI,
never on wasm: it is the one place the crate needs threads) is the
runtime; `focus/parallel.rs` and `net::parallel` are the two engines on
it. `prove_goal` takes the parallel path when `Options::jobs` is above one
and the engine is `Focus`, `TwoSided` or `Net`; the additive path stays
sequential (it is linear-time in the product of the formulas' sizes and
has no or-choices worth sharing out). What the code relies on:

- **One pool per search, no global.** `Runtime::new(jobs, stack_size)`
  builds a rayon pool of `jobs` threads with stacks of
  `Options::stack_size()` (the engine recurses on the worker's stack as
  it does on the caller's), which `prove_goal` drops with the outcome;
  `Error::ThreadPool` when the threads cannot start. Never touch rayon's
  global pool: a library must not size or seed it, and `RAYON_NUM_THREADS`
  is read only when a builder's thread count is zero, which ours never
  is.
- **The stop closure is polled on the calling thread.** `Runtime::drive`
  spawns the work into the pool from an `in_place_scope` and, on the
  calling thread, waits on a channel for the result with a one
  millisecond timeout, polling the caller's closure at each timeout and
  raising the root `AtomicBool` when it fires. So `prove_goal`'s closure
  needs no `Send` and is polled about a thousand times a second, not
  once per node: a caller that reads the clock every `n` polls (the CLI
  does, on one thread) must read it every poll on several
  (`polls_per_clock` in the CLI). A worker polls its `Flags`, the chain
  of its own cancel flag and its ancestors' up to the root, at every
  stable sequent (`prove_stable`) or literal chosen (`decide`), through
  `Stop::Flags`; the sequential engines poll the closure through
  `Stop::Closure`. rayon tasks cannot be killed, so a place that stops
  polling is a place cancellation does not reach.
- **Such places exist, in the split enumeration.** On one thread, 90 of
  the baseline's LLTP runs on files under 2 MB ran past the harness's
  kill at 10.5 s under a 5 s limit; one of them, the Petri net
  `AutoFlight_afcs_05_a_1_1`, examines 1.8 billion splits at a single
  stable sequent and stops only after 242 s, so the poll every
  `SPLITS_PER_POLL` splits does not stop that enumeration. At 16
  threads, 166 other small Petri nets, which one thread stops at 5 s,
  ran past the kill as well (`bench/results/2026-09-30/`, rows with
  `reason` `killed`). Given 600 s before the kill, such one-thread runs
  stop 10 s to 506 s after their start, and two Petri nets are proved
  after their limit (`TokenRing-20-unfolded_1_1` at 552 s under 5 s; the
  `*-generous.csv` rows). No test catches it.
- **Cube-and-conquer is nested fork-join at the first `LEVELS` (2)
  choices of a branch**, not a static enumeration: at a choice among
  alternatives (`decide_with`'s candidates and copies together, the two
  sides of a `⊕`, the free splits of a `⊗` with the assignment of the
  first `fixed` members per task, `fixed` giving the pool twice its
  threads in tasks and at most `MAX_FIXED` = 6 bits) an engine whose
  `or_depth` is below `LEVELS` spawns a worker per alternative but the
  first and runs the first on one more worker on its own thread
  (`choose_parallel`); workers have `or_depth + 1`. Every alternative
  runs on a worker, never on the spawning engine itself, because a
  worker's stop chain holds the choice's cancel flag and the spawning
  engine's does not: an alternative run in place would never be
  cancelled by a sibling's proof (a review measured a whole refutation
  spent that way). Below the levels a worker is the sequential engine.
  rayon's work stealing is what makes this "when the pool has idle
  workers": a spawned task nobody steals runs on the spawning thread
  after its own alternative. The `&` rule within the levels runs its
  right premise on a worker of the pool and its left one on a worker on
  its own thread (`with_parallel`); the `⊗` premises stay sequential
  (the first usually fails fast). Mix stays sequential after the
  parallel alternatives failed (`last_resort`).
- **A worker is a copy of the branch, not of the engine** (`Spawn`,
  `Spawn::worker`): the shared parts by reference (forest, reading,
  counts, rules, memo, arena, runtime, flags), the branch's by copy (the
  live stack of keys, `depth`, `copies`, `or_depth`), fresh pools and
  counters, `exhausted` clear and `dependency` none. The copied stack is
  what keeps the loop check's prunes below the cube; `depth` keeps the
  recursion limit's meaning for the counter, not for the stack: a pool
  thread that waits at a scope runs stolen tasks on its own stack, so
  its frames are the scope's (a choice near the root, a few dozen
  levels) plus the stolen task's, and nested waits compound; the 2×
  margin of `Options::stack_size` and its 8 MiB floor cover this at the
  default limit, and a raised limit is where an overflow would first
  show. `Engine::new` takes the counts, the rules, the memo (`Table`)
  and the arena (`Arena`) from outside for that reason; `search_goal`
  builds them and owns them.
- **Merging is the sequential rule's**: a choice's result is a proof if
  any alternative found one (a proof of one alternative wins over an
  error of another, so the pool may decide where one thread gives up
  with `RecursionLimit`), else the first error that is not a stop caused
  by cancellation (a worker's `Stopped` is ignored only when the choice's
  own `cancel` flag is raised), else a failure with `exhausted` or-ed and
  `dependency` min-ed over the alternatives that ran to their end
  (`Collected::take`); a cancelled alternative's flags are dropped, as
  the sequential engine never ran it. For `&`, a failed premise decides
  and the other's flags are dropped, both premises' flags count when both
  ran to the end (`with_parallel`). Success raises `cancel` at an
  or-node, failure or error at the `&`. A worker inserts into the memo
  only what its own `prove_stable` decided, so a cancelled worker leaves
  facts and nothing half-done.
- **The shared memo is 64 shards of the sequential `Memo`** behind one
  `Mutex` each (`memo::Shared`, the key's top hash bits choosing the
  shard, the cap split among them), and `Memo::insert`'s merge under the
  shard's lock is the compare-and-swap the bound needs: a larger
  `Exhausted` budget wins, `Complete` and `Proved` win over `Exhausted`,
  a proof stays, so an entry's validity only grows whatever the
  interleaving, and two workers deciding one sequent cost duplicated
  work, never a weaker entry. A lock is held for one map operation and
  never across a recursive call. `dashmap` was not taken: the notes
  record its maintenance as thin, the shards are twenty lines, and the
  speedup table shows no contention worth a dependency. `hits` and `peak`
  are summed over the shards (`peak` is an upper bound).
- **The arena is shared behind one `Mutex<Vec<Node>>`** (`Arena::Shared`),
  a push holding the lock for the push: an id is a position in the one
  arena every `Proved` entry refers to, and a node's premises were
  pushed before it by whichever worker built them, so the order
  `Proof::new` needs holds across threads; the memo insert happens after
  the push, so a hit always finds a complete subtree. Pushes are as many
  as rule instances on successful branches, far fewer than nodes visited.
- **Levels never overlap**: `run` deepens the copy bound on the root
  engine, which spawns nothing until its first choice and reads
  `exhausted` after every task of the level has ended (the scope waits),
  so the or-reduction over the workers is the merge above and a level is
  `Unprovable` only with every worker's flag clear.
- **The net engine's cubes are the branches of the first `d` links**
  (`Engine::explore` with a limit records a branch that reaches it and
  takes the link back; `seed` makes a cube's links on an empty
  structure; `reset` takes every link back and clears the frames but
  keeps the counters), enumerated on the root engine with the tests the
  search applies, `d` growing until there are `CUBES_PER_THREAD` (16)
  cubes per thread or the enumeration decided the sequent (a proof
  within the limit, or no branch surviving, which is `Unprovable`).
  Workers pull cubes from an atomic counter with one engine each, so the
  per-worker state is allocated once; a worker that finds a net stores
  it and raises the flag; `Unprovable` needs every cube to have ended
  `Ok(false)`, and any error or a real stop makes the verdict `Unknown`.
  The choice order (fewest admissible partners first) is the cube order,
  so cubes are already the small-multiplicity atoms first. No state is
  shared beyond the flags: the structure and the scratch are per worker.
- **A parallel run may return another proof, never another verdict**:
  every level is searched to its end by some worker with no cube
  abandoned unless a proof or an error ends it, so `Proved` and
  `Unprovable` agree with the sequential engine; only decisiveness within
  the copy bound may differ (as it does between memo and no memo), since
  the memo's contents depend on the interleaving. A parallel run may
  answer `Proved` where the sequential one answers `Unknown
  (RecursionLimit)` on another alternative. `Unknown (Stopped)` is the
  caller's stop, never a cancellation. The tests
  (`focus::parallel::tests`, `net::parallel_tests`) assert exactly this
  on the generated samples with two and four threads, with and without
  the portfolio; every proof is checked.
- **`Options::portfolio`** gives every worker a seed (`Spawn::worker`, a
  mix of the spawning engine's seed and the alternative's index, never
  zero) that `rank` uses in place of the id to order candidates and
  copies within their classes, so workers below the levels explore in
  different orders; the first alternative of every choice keeps the
  spawning engine's order. It is off by default: the table in the step
  report shows no consistent gain on the families measured.
- **Statistics** add every worker's counters (`Statistics::add`, the
  memo's read off the shared table once), so a parallel `nodes` is the
  work done, not the work one thread would have done, and the CLI's
  pinned counts use `--deterministic`.

## The additive fast path

`search/additive.rs` decides a sequent of exactly two additive-only
formulas by a recursion on pairs of subformula occurrences, one below each
root, memoized on the pair: `⊤` closes, `&` on either side needs both
subformulas against the other, two dual literals are an axiom, `⊕` on
either side tries one subformula at a time, and nothing else proves
anything. `&` is invertible and goes first; **which `⊕` to decompose is a
real choice** (a `&` below the other formula's `⊕` may need both sides of
this one: `⊢ ~c ⊕ ~a, b ⊕ (c & a)`), so both formulas' `⊕` are tried and
the memo is what bounds the work by `|A|·|B|`; a first version that
returned after the first formula's `⊕` was caught by the review. The
procedure is the same in every mode: additive rules keep one output by
themselves, and neither weakening nor Mix can help a two-formula sequent
(a proof of one formula alone ends in `⊤` leaves, which absorb the other).
`Statistics::nodes` is pairs visited, `memo_hits` and `memo_entries` the
memo's.

## Export

`export/` writes sequents and derivations as LaTeX (ebproof trees, cmll and
amssymb symbols) and Typst (curryst trees), pure functions to `String`,
each in a `Form` (`Fragment` or `Standalone`), draws them and proof
structures as SVG documents (no `Form`: an SVG is always a document), and
writes derivations as Rocq proof scripts for NanoYalla (`rocq`, in a
`Form`: the lemma, or a file with the import). What the code relies on:
- **One table per target, one printer.** `notation::Notation` is the
  symbol table (connectives, units, dual mark, turnstile, the alignment
  mark, the atom escaper); `Notation::term` and `Notation::ill` are the
  bracketing of `Sequent`'s and `Reading`'s `Display` over it, and must
  stay in step with them. A new target (SVG text, say) is a new table;
  `latex::label` and `typst::label` are the rule-label tables.
- **The walk keeps its own stack** (`notation::walk`, enter and exit
  events): exits are ebproof's postfix order, enter/exit brackets
  curryst's nesting. Nothing in the emitters recurses over the tree, so
  the output is linear in the inferences (each prints its whole sequent)
  and never as wide as the tree, unlike the text renderer. The formula
  printers recurse to the formula's depth, as `Display` does.
- **An open goal is one shape in both targets**: its sequent under
  vertical dots with no inference line (`\hypo{\vdots}` then
  `\infer[no rule]1{…}`; a curryst leaf that is a centred `grid` of
  `dots.v` over the sequent). Neither package has a per-inference dotted
  bar (ebproof 2.1.1 styles: simple, no rule, double, dashed; curryst
  0.6.0 has one stroke per tree), which is why.
- **Typst symbols are Unicode characters, not names**: Typst 0.15
  removed `times.circle` and `plus.circle`, so names break across
  versions and characters do not. `&` is `class("binary", \&)`, `?` is
  `class("normal", ?)` (Typst spaces punctuation), letters in labels are
  `upright(L)` (a string in math keeps the space before it). Only the
  two-sided LaTeX tree aligns turnstiles (`&\vdash`); a one-sided
  sequent would align at its left edge, so it stays centred.
- **Limits of the packages, not of the emitters**: Typst 0.15 refuses a
  curryst 0.6.0 tree more than about eleven inferences high ("maximum
  show rule depth exceeded": curryst nests several layout elements per
  level), while ebproof compiled a 120-high tree; TeX fails with
  "Arithmetic overflow" on a sequent line wider than its largest
  dimension (about 5.7 m). Neither can be fixed in the output.
- **Snapshots**: `core/tests/export.rs` pins standalone documents in
  `core/tests/snapshots/` (`BLESS=1` rewrites them); the flake's `export`
  check compiles exactly those files plus two CLI outputs with pdfLaTeX
  and Typst, which is what catches output that matches its snapshot but
  does not compile. The crane source keeps that directory
  (`modules/workspace.nix`), since `cleanCargoSource` alone drops it.
  `typst::CURRYST` and the nixpkgs curryst in `modules/export.nix` move
  together.
- **Euler everywhere.** The standalone LaTeX loads `eulervm` and the Typst
  page sets math in `"Euler Math"`; fragments stay font-neutral. The
  export check runs Typst and resvg with `--ignore-system-fonts` /
  `--skip-system-fonts` and fails on any output, so a font they cannot
  find fails instead of falling back silently.
- **SVG** (`export/svg/`): a third table (`NOTATION`, with the atom
  letters as mathematical italic codepoints, which a math font sets as
  math italic, and `\u{1}` standing for the raised `⊥`; `PLAIN` for the
  `<title>`), and layouts of its own: `tree.rs` (a post-order pass over
  `walk`'s exits for box widths, a pre-order pass over its enters for
  positions; uniform rows of `line_height`), `net.rs` (literals in id
  order, which is left to right; connectives by height; links as
  half-ellipses whose height is proportional to their width, so nested
  links never cross). Widths are integer thousandths of an em from
  `font.rs`'s advance table of Euler Math 0.75 (a fixed fallback outside
  it); all coordinates are integers, so the output is byte-stable.
- **A superscript or subscript is its own `<text>`**, never a `<tspan>`
  with `dy`: resvg (which Typst uses to draw SVG images) spreads
  `textLength` wrongly across such a tspan, while every renderer agrees
  on separate positioned texts. Spaces separate pieces instead of
  starting or ending one. The font has no `₁`/`₂`, so rule names'
  subscripts are lowered digits.
- The text bounds are `font::HEIGHT` (a raised `⊥`) above and
  `font::DEPTH` (a comma) below every baseline; the layouts reserve
  them for every line, and the structural test in `core/tests/export.rs`
  checks every element against the view box with the same bounds.
- **Rocq** (`export/rocq.rs`): the kernel is NanoYalla `NANOYALLA`
  (Click & coLLecT's `nanoyalla/`: `nanoll.v` is the trusted `ll`
  inductive over list sequents, `macroll.v` the derived rules), and the
  script relies on its `_ext` lemmas exactly as stated there: every rule
  takes the list `l1` of formulas before its principal one and infers the
  rest by unification, `oc_r_ext l1 (A) l2` needs both contexts without
  their `?`, `tens_r_ext l1 A B l2 : ll (l1 ++ A :: nil) -> ll (B :: l2)
  -> ll (l1 ++ tens A B :: l2)` needs the left premise's context before
  the `⊗` and the right one's after it, `ax_expansion` closes `[dual A;
  A]` and `[A; dual A]` for any formula `A` (atoms are `formula`
  binders of the lemma, so the lemma is schematic), and `ex_perm_r p l`
  proves the goal whose position `i` holds `l[p[i]]` from `ll l`. So the
  exporter tracks the goal list of every inference (`Script::goals`, set
  when the conclusion is written; the root is the sequent in id order)
  and emits one `ex_perm_r` only before a `⊗` whose goal is not already
  split around it; every other rule acts in place, and a contraction
  leaves its two copies adjacent. Equal ids are equal formulas, so the
  first matching position serves for a repeated occurrence. The
  certificate is classical: a two-sided derivation goes through
  `Rule::classical`, and the checked sequent is the one-sided one. Mix,
  affine `wk` and `Rule::Open` are refused before anything is written
  (`Unsupported`), since the kernel has no such rule; atom names are
  escaped to identifiers and made distinct from `RESERVED` (keywords and
  every kernel name a script mentions), the lemma's name and each other.
  `Options` (D15: `lemma`, `prelude`) is the configuration; no other
  choice is a constant. The snapshots' `.v` files are compiled by the
  flake's `rocq` check against the kernel built from the `nanoyalla`
  input; the kernel needs Rocq 9 with `rocq-stdlib` (its `From Coq
  Require Import Lia`, deprecated but accepted) and nothing of Yalla.

## Layout

`sequents` (arena, printing), `parse`, `serialize`, `fragment`, `occurrences`
(forest, sets, and the intuitionistic `reading`), `proofs` (terms in
`mod.rs`, `check`, `derivation`, the renderer `fmt`, the crate-private
`multiset`, and `interactive` behind the feature of that name), `search`
(the front door in `mod.rs`, the focused engine in
`focus/` with `counts` and `memo`, the net engine in `net`, the additive
path in `additive`, the test-only `generate` with its classical and
intuitionistic proof generators), `nets` (structures and the criterion's front door
in `mod.rs`, the graph and the Yeo test in `graph`, the union-find in
`skeleton`, `sequentialize`), and `export` (the shared `notation`, and
`latex`, `typst`, `svg` and `rocq` behind the features of those names). `lib.rs`
re-exports the public types, so users write
`linlog::Sequent`, `linlog::Proof`, `linlog::prove`, and so on. `hash` is
crate-private.

## Benchmark inputs: LLTP and the families

- **`lltp::read`** (feature `parse`) turns an LLTP file into `axioms ⊢
  conjectures` by assembling text for the crate's own parser: the
  library's connectives and precedences (`*` over `|` over `&` over `+`
  over `-o`, prefix `!`/`?`, postfix `^`) are this crate's, checked on
  every mixed-operator formula of the library. Lines from `%` on are
  comments; the status (`Status::Theorem`, or `NonTheorem` for
  `Non-Theorem` and `CounterSatisfiable`) is the first `Status (intuit.)`
  or `Status (linear)` comment's, else the first plain one's, because the
  translated ILLTP problems carry the classical source's `Status` first
  (39 files, the excluded middle among them, would read as theorems);
  roles other than `axiom`, `hypothesis` and `conjecture`, and an
  annotation after the formula, are refused. A `-` between two name
  characters is part of the name unless it starts `-o` and becomes
  `lltp::HYPHEN` (`‿`), a `.` there becomes `lltp::DOT` (`·`), since the
  Petri nets name places `P-start_1_1` and `merge.s00001061.input` and
  this crate's identifiers hold neither. The mode
  is not in the file: the caller decides (the harness by the `ILL`
  directory). A header's status is the library's claim, not a fact: the
  statuses of translated problems are those of the intuitionistic source.
- **`families`** (feature `parse`): `FAMILIES` lists the benchmark
  families, each a name, a summary, default sizes, instances per size and
  a generator `(size, index) → Instance` (sequent, mode, `provable`,
  `copies`). A family's `provable` comes from the problem it encodes
  (subset sums, QBF evaluation, 3-Partition by construction) or from a
  construction argument written at the generator, never from an engine,
  so that an engine disagreeing is a finding. Random families seed
  SplitMix64 from the size and index. The encodings are public
  (`three_partition`, `three_partition_mll`, `partition`, `qbf`,
  `counter`, `wide`, `mix`), and the engines' tests use them instead of
  private copies. The QBF encoding sequences quantifiers with key atoms:
  `∃x` is `((~tx ⅋ ~kx) ⊕ (~fx ⅋ ~kx)) ⅋ (kx ⊗ S)`, so the rest `S` can
  only be focused once the choice released `~kx`; `∀x` is `(~tx & ~fx) ⅋
  S`; a clause is the `⊕` of `(tx ⊗ ⊤)`/`(fx ⊗ ⊤)`, and the matrix their
  `&`. `mix` wraps each tensor pair in `⊕ 0` because in MLL the count
  equation refutes the bare pairs at once.

## Parsing

`core/src/parse/mod.rs` has two stages:
1. A chumsky Pratt parser produces a borrowed AST (`parse::Tree`/`parse::TwoSided`).
2. That AST is lowered into the arena, which creates one atom entry per occurrence and then calls `optimize()`.

chumsky's API changed wholesale after 0.9, and most examples online and in
memory are for the old one. When a signature is in doubt, ask the
`crate-source-explorer` agent rather than guessing.

Every operator has ASCII and Unicode spellings: `* ⊗`, `| par ⅋`, `&`,
`+ ⊕`, `-o ⊸`, prefix `~ ! ?`, postfix `^`, and `|-`/`⊢`. The constants are
`0`, `1`, `bot ⊥` and `top ⊤`. Precedence, from tightest: `^` > `~ ! ?` > tensor > par > with > plus > lollipop (right-associative).
`core/tests/parse.rs` pins this behaviour through the public API.

Doc examples that parse are fenced with `cfg_attr(feature = "parse", doc =
"```")` and an `ignore` fence otherwise, so `cargo test --no-default-features`
passes; copy that pattern for a new example.

## Serialization

`core/src/serialize/sequents.rs` uses a private serde proxy struct
`{terms, ids, var_dict}` with short tags (`V`, `D`, `⊗`, `⅋`, …) and `u32`
indices. `serialize/proofs.rs` does the same for proofs: `{"sequent": …,
"proof": [node, …]}`, one object per node tagged `ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ?
copy wk mix` with the occurrence ids and premise indices as an array (or
one integer), premises before conclusions and the root last. Deserialization
rebuilds the forest, checks bounds and order and drops unreachable nodes;
whether the proof is correct is `Proof::check`'s question, since the mode
is not in the file. Both are interchange formats for the CLI and the planned
web front end, so a tag or key change is a format break;
`core/tests/serialize.rs` pins the exact strings. A binary format would come
from the same proxies (postcard encodes the variants by index), in the crate
that wants it.

`serialize/search.rs` gives the search's values a wire form, pinned in the
same test file:
- `Fragment` is its name (`"MALL"`, `"MLL with units"`), not its flags, so a
  human reads it. Deserializing a name gives the named fragment, which
  contains every fragment of that name: the trip is lossy towards larger,
  which as an assertion only switches off prunes, never refuses a sequent
  it came from.
- `Mode` is `{"intuitionistic": …, "affine": …, "mix": …}`.
- `Outcome` serializes only (it is output): `verdict` (`proved`,
  `unprovable`, `unknown`), `reason` for `unknown` (a snake_case tag,
  `{"context_too_wide": n}`), `fragment`, `mode`, `engine`, `statistics`,
  and for `proved` the proof's own `sequent` and `proof` keys, flattened, so
  that the whole outcome deserializes as a `Proof` (serde ignores the other
  keys) and `linlog check` reads the output of `linlog prove --format json`.
  A new `Reason` variant or `Statistics` field needs its line in the proxy;
  `Outcome::net` is not serialized (the proof's keys are, and the net is
  `from_proof` of them).
- `Forest` has no serde; it is rebuilt from the sequent.

`serialize/interactive.rs` writes an `Interactive` as `{"sequent": …,
"mode": …, "inferences": [{"sequent": [0, 1, 4], "rule": "⊸L",
"principal": 1, "premises": [1, 2]}, {"sequent": [3, 4]}, …], "history":
[0]}`: the inferences in the state's own top-down order, an open goal as
its sequent alone (`rule`, `principal` and `premises` absent), rule names
as `Rule::name` (`Rule` itself serializes as its name, in
`serialize/proofs.rs`), and the history as the inferences the steps
closed. Reading it back goes through `Interactive::from_parts`, which
replays every closed inference. It is the form a web client holds between
requests, so it is pinned in `core/tests/serialize.rs`.

`serialize/nets.rs` writes a `ProofStructure` as `{"sequent": …, "mix":
false, "links": [[0, 2], [3, 4]]}`, the links as occurrence id pairs in
the order they were made; reading validates the links as `from_links`
does and accepts a partial or incorrect structure, since whether it is a
net is `is_correct`'s question.

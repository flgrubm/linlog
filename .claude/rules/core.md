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

**Nothing recurses over a formula.** A sequent read from JSON can be nested
as deep as it is long, and a recursion per level ends the process where
an error is owed. So every walk is a pass over the arena or over the
forest in index order (`fragment`, `optimize`, `Forest`, `Reading::new`,
the engines' `Classes` and `Counts`), or, where a formula is written, a
loop over `sequents::fmt::Walk`: the stops of a formula in the order it is
written (`Enter` a subformula, `Between` the two subformulas of a binary
one, `Exit` a compound one), from a stack of its own whose first sixteen
steps are inline, so that an ordinary formula costs no allocation. The
`Display` of `Sequent` and `Formula`, `Reading`'s printing, the exports'
`Notation::term` and `Notation::ill` and the Rocq printer are each one
`match` over those stops, and a new printer is another: never a function
that calls itself. `core/tests/depth.rs` runs every walk the public API
offers on formulas nested 100 000 deep, on a thread with a stack of
256 KiB.

`optimize()` runs after parsing. It deduplicates atom names, hash-conses
identical terms, drops unreachable ones and sorts `roots`. Nothing else
guarantees that every arena term is reachable: a deserialized sequent may
carry junk terms, and `fragment()` and `Forest` walk from the roots for that
reason. `Sequent::add` merges two sequents by offsetting atom and term indices.

**A name is an atom**: the atom table of every `Sequent` a caller can hold
has distinct names. `optimize` merges equal names (and drops unused ones),
`add` identifies the added sequent's atoms with those of the same name
(`merge_atoms`), and deserialization does the same after the integrity
check, so a JSON dictionary that repeats a name reads as the sequent with
the name once; before, such a file gave two atoms that printed alike, and
`⊢ ~A, A` was answered "unprovable". `merge_atoms` leaves a table of
distinct names untouched (unused entries and their order included), so a
JSON sequent is written back as it was read, and it never moves a term or
a root, so the occurrence ids a stored proof names are those of the file.
The parser interns the names as it reads them, so its table has distinct
names before its `optimize` already.

The public surface is read-only accessors (`terms`, `term`, `roots`,
`atom_names`, `atom_name`, `atom`, `formula`, and `occurrences`, the size
of the unfolding) plus `optimize`, `add` and `verify_integrity`;
construction goes through the parser or serde. Tests inside the crate
build arenas as struct literals.

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
  returns `Option<OccId>`. That is why no forest has `u32::MAX` or more
  occurrences (`Forest::MOST`, crate-private, is the most).
- **A forest is refused before it is built** when the sequent unfolds to
  more occurrences than a limit (`Error::TooManyOccurrences { occurrences,
  limit }`): `Forest::new` and `TryFrom<Sequent>` within
  `Forest::DEFAULT_LIMIT` (50 million: the largest problem of the LLTP
  library has 27.8 million, and a forest takes about 25 bytes per
  occurrence), `Forest::within(&sequent, limit)` within another, which
  `Forest::MOST` caps. `Sequent::occurrences()` is the count: one pass
  over the arena (`Sequent::sizes`, the occurrences below every term),
  saturating, since an arena that shares its subterms unfolds
  exponentially: a JSON sequent of a few hundred bytes whose subterm is
  shared 25 times over has 67 million occurrences, a forest of 1.9 GiB.
  The check comes before the sequent is cloned or any per-occurrence
  array is reserved. Every path that makes a forest of a sequent it was
  handed (the search's front door, the proofs, nets and interactive
  states read from JSON) goes through one of the two, so the default
  limit holds wherever no caller names another.
- **Atom bias** (`Forest::bias`, computed once in `Forest::new`, a
  function of the sequent alone, so a run stays deterministic; the
  focused engine is its only reader, through `polarity(o)`). Focusing is
  complete for every assignment of polarities to atoms, so the bias is
  chosen for speed and can never change what is provable. Without an
  exponential in the sequent: per atom, the literal that is more often a
  direct factor of a `⊗` is positive, each occurrence weighted by ½ per
  `&` or `⊕` above it (a proof takes one side of a choice, so the two
  heads `~d` of a clause `(… ⊗ ~d) ⊕ (… ⊗ ~d)` count as one); a `⊗`
  with a positive literal factor has its split forced. On a tie, and
  whenever the sequent has a `!` or `?`, the old rule: the literal with
  fewer occurrences is positive, a tie makes `Var` positive, so an atom
  with one sign only has all its literals negative. The engine takes
  the old rule in affine mode as well (`focus::plan`): nothing forces a
  split there, so the factors have nothing to say, and a review measured
  up to 700 times the stable sequents on generated affine sequents with
  the factor rule. With exponentials the bias decides the shape of the
  focused proofs, hence the copies a branch needs, and neither rule
  wins: the counter family chains forward under the factor rule and
  needs `n − 1` copies on its one branch where the rarer-literal rule
  needs `log₂ n`, and at a bound of `n − 1` the factor rule decides the
  counter with 16 tokens in 19 stable sequents instead of 473 232. So
  `Forest::bias` is the rarer-literal rule there, and the engine's
  default runs a search under each ("The default bias with exponentials
  is two searches", below). `Options::bias` names the rules:
  `Bias::Rarer` and `Bias::Factors` for any sequent
  (`Forest::bias_under`; the engine reads its polarities off
  `Counts::positive`, never off `Forest::polarity`, so that the option
  reaches every place a literal's polarity matters, `literal_tensor`
  included). Rules tried on the
  exponential-free targets and not taken: `Var` always (as good on the
  families written two-sided, where it is forward chaining, but it
  depends on how the atoms happen to be written and loses the gains on
  Partition and the wide sequents), `DualVar` always (30 to 300 times
  more stable sequents on the Horn families), the factor count without
  the ½ (the heads of the 3-Partition clauses outvote the goal: 317 138
  stable sequents against 923 at bins of four). Measured, old rule and
  new (stable sequents, splits): unsolvable 3-Partition with bins of
  five 3 373 and 41 160 against 971 and 36 072; QBF 20 #2 105 667 and
  1 037 858 against 60 883 and 97 394; Partition with seven items 179
  and 6 008 against 178 and 2 078; `wide-m3` at 30 91 and 768 against 31
  and 688.
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
runs, no OS randomness, works on wasm).

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

- One bottom-up pass in arena order, no recursion (`Pass`). Each node
  gets a `State { theta, gamma, any, … }`: `gamma` is the linear zone as
  a multiset (`Bag`, a table of counts: copies can repeat an occurrence
  and every subformula of a copied formula), `theta` the **least**
  unrestricted zone the subproof needs (`Copy` adds the occurrence,
  `Quest` removes its subformula), and `any` says a `⊤` above absorbs any
  further linear context. The claim the tests and the review rest on:
  `State` characterises exactly the set of dyadic sequents the subterm
  proves. At the root, `theta` must be empty (every copy has its `?` step
  below it) and `gamma` must equal the roots, or be a sub-multiset of them
  under `any`.
- **The pass holds only the sequents a later node still reads.**
  `readers` counts, per node, the premise references to it; a node's
  `State` is built from its premises' own, the last reader *taking* a
  premise's state and changing it in place, an earlier reader a clone,
  and a state with no reader left is gone. Both zones are hash tables of
  their members (the crate's fixed-seed hasher), so a state costs what it
  holds and nothing for the forest's width, an insertion or a removal
  costs the same whatever the zone's size, and the two-premise rules pour
  the smaller table into the larger. The number of members in output
  position is kept in the state, never recounted (a recount per node made
  the intuitionistic check quadratic in a zone of thousands). What this
  buys, and what it does not: on a proof without shared subproofs the
  live states are those of disjoint subtrees, each no larger than twice
  its subtree, so the memory is linear in the proof whatever the order of
  the arena; a node read by several others is cloned for all but the
  last, so a proof whose shared nodes have large zones can still take
  nodes × zone (the weakenings of one large sequent under a tower of `&`
  is the shape), which no engine's proofs were seen to do. The first
  implementation kept every node's sequent with a bitset as wide as the
  forest (6 GiB on a net of 65 000 clauses whose proof the search finds
  in 43 ms; the new pass checks it in a few milliseconds within the
  search's own memory). It is kept as `proofs/oracle.rs`, compiled for
  tests only, and `agrees_with_the_first_implementation` requires the
  same verdict *and the same error* from both on the engines' proofs of
  the generated sequents and the families, in every mode, and on mutants
  of them: a change to a rule is made in both, or the test says where
  they part.
- **An error costs a second pass.** The states a failing node read are
  gone or changed by the time it fails, so `examine` runs the pass again
  up to that node with its premises pinned (one reader more) and reports
  their sequents from there. `Problem` is found by the first pass; the
  second cannot fail before the node.
- **`examine(proof, goal, mode, reading, observer)` is the one pass
  behind everything**: `check` is it on the roots with no observer; the
  derivation view and the size estimate are observers (`Observer`: every
  node's `State` in arena order and the `Facts` of how its rule applied:
  `used`, `shared`, `absent`, `needs`, `left_goal`), so what they know of
  a proof is what the checker derived, never a second reading of the
  rules. An observer may give occurrences a `weight`, which a state adds
  up over its zones as they change (`State::weight`, `goal_weight`); the
  checker's own observer is `()` and the sums are zeros. A goal other
  than the roots is checked the same way (`Derivation::of_goal`).
- **A zone the rest of the proof cannot consume is refused where it
  arises** (`Pass::within`, `Problem::Surplus`): a rule takes two members
  of a premise's zone at most and passes the others on, and the root's
  zone lies within the goal, so node `i` of `n` may derive at most
  `|goal| + 2·(n − 1 − i)` members, and never more than `Bag::MOST`.
  This is a soundness rule, not a convenience: a term may double a zone
  at every node (`Mix(p, p)`, or a `⊗` on a `⊤` premise taken twice), a
  proof file can hold such a term though no engine builds one, and the
  zones are counters of a fixed width. Before the rule the counters
  wrapped in a release build: a file of 131 nodes that mixes 2⁶⁴ copies
  of `⊢ 1`, promotes `⊥` over them (the zone's length read zero) and
  mixes one more in was a "valid proof" of `⊢ !⊥, 1`, which has none
  (`refuses_a_zone_too_large_to_conclude`). Now the counters saturate
  and a saturated one is always over the bound, so every count a rule
  reads is the true one; the first implementation has the same rule, so
  the two still agree on the error. The rule also bounds what a
  malformed term costs: every zone, and so every error report, is within
  the goal plus twice the nodes. The weights an observer adds up are
  not part of the argument: they are sums of at most that many formula
  sizes.
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
and fails as the checker would. What the builder knows of the term comes
from the checker's pass through an observer (`Record`): a flag per node
for `absorbs` and for `used` (a `?` step whose formula a copy above uses,
a copy whose occurrence is copied again above), the shared unrestricted
occurrences of a `⊗` or Mix, and the standard sequent `?Θ, Γ` only where
the builder splits or pads a context, which is at every `⊗` and Mix and
at the premises of `⊗`, Mix and `&`. Those sequents appear in the
derivation anyway, so the record is never larger than what is built; a
table of every node's sequent would be, by any factor, on a chain of `?`
steps (which are no inferences).

**The size of a derivation is computed from the term, without building
it** (`proofs/size.rs`, `Proof::derivation_size(two_sided)`, a `Size`):
one pass of the checker with an observer that keeps a few numbers per
node, all saturating, since a term with shared subproofs unfolds
exponentially. What `Size` promises: `inferences` and `height` (the
inferences on the longest branch; the text tree has two lines for each)
are exact; `characters` is the sum over the inferences of their sequents
written one-sided, each formula with two characters for its separator,
exact when `exact` is set and an upper bound otherwise; `width` is the
widest sequent the pass saw, a *lower* bound of the text tree's width
(the tree's own width needs the layout: `Derivation::text_size`). The
one place the sum is a bound: a `&` with several `?` formulas that only
one premise uses, whose weakenings above the other premise are each
counted with the sequent of the last (their order is not kept). How it
is exact elsewhere, which is what a change to the builder must keep in
step: per node the inferences and characters of its subtree under the
sequent it derives itself, and `reached`/`reached_goal`, the number of
its inferences whose sequent holds a formula that a `⊤` in the subtree
absorbs (a hypothesis, or two-sided the goal, which the builder sends to
the premise without one at a `⊗` whose premises both absorb). An
absorbed formula adds its characters times that number: at the root for
what the conclusion holds beyond the root's sequent, at every rule for a
subformula its premise lacked (`Facts::absent`), at a `&` for what the
conclusion's context holds beyond an absorbing premise's. A `⊗` or Mix
with `k` shared `?` formulas is `k + 1` inferences, the rule's sequent
holding each formula twice and each contraction below taking one away in
ascending order. `is_the_size_of_the_derivation_built` compares all of
it with the derivation actually built, on the samples of the checker's
differential test, one-sided and two-sided. `Size::bytes()` turns the
count into the estimate that a bound is compared with
(`BYTES_PER_CHARACTER` 8, `BYTES_PER_INFERENCE` 128): measured on a
derivation of 5 119 inferences and 17.0 million characters (`wide-m1` at
1 024), the text tree is 76 MB written and 160 MB at its peak, LaTeX 49
and 106, Typst 59 and 126, the Rocq script 62 and 132, the SVG 212 MB
written and 855 MB at its peak, against an estimate of 130 MiB: the
order of magnitude for every format but the SVG's memory.

**Nothing builds a derivation it was not allowed to** (`ViewOptions`,
the one options value of every path that builds one, plain data with
serde: `limit`, the most bytes of `Size::bytes()` a derivation may be
estimated at, `DEFAULT_LIMIT` 64 MiB, `None` or `UNBOUNDED` for no
bound). `unfold` is the one place derivations are made, for
`Derivation::new`, `two_sided` and `of_goal` alike: the size first (a
pass of the checker), `ViewError::TooLarge { size, limit }` past the
bound with nothing built, then the pass that records what the builder
reads, then the builder, which polls the caller's `stop` once per node
and answers `ViewError::Stopped`. So the text tree, the four exports
(which take a `Derivation`), the graft of `Interactive::close` and a
front end's check output are all under the bound by construction, and a
new path that needs a derivation gets it from there or not at all.
`Proof::derivation()` and `two_sided_derivation()` are the default
options with no stop; `…_with(&view, stop)` take both. A proof whose
derivation is refused for its size has passed the checker (the size's
pass is one). `Interactive::close(goal, options, view, stop)` leaves a
goal open whose graft is refused (`Error::View`), though the search
proved it; what it then tells the user is the front end's to say. The
builder still recurses to the derivation's height, and the bound does
not limit that: a caller on a small stack (the web front end) needs
`Size::height` to decide.

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
pinned in tests, so a layout change is a test change. The layout is two
passes over the inferences in their own order, which needs no walk since
premises precede conclusions (`layout`: box sizes and the premises'
offsets ascending, then absolute columns and depths descending), and
`Display` writes the pieces (a conclusion, a bar with its rule name) row
by row in column order, each sequent written straight into the formatter
and only counted in the first pass. So a tree costs time linear in its
text and memory linear in its inferences; the first renderer built every
subtree as a block of padded lines and copied it at every level (24 s and
333 MiB for a tree of 76 MB). A bar that would start left of its box
moves the premises right (`shift`), exactly as the block renderer padded
them. `Derivation::text_size` is the width and height from the first pass
alone, which is how a front end learns whether a tree fits before it
draws it.

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
  sequent, and grafts `Derivation::of_goal` of the proof found, within
  the `ViewOptions` it is given and until its stop fires; the goal's
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
position (`Error::GoalOutputs`). **Every proof of the roots has passed
the checker when it is returned, in every build** (`Options::check`,
`DEFAULT_CHECK` true; the check is at the end of `prove_goal`, one place
for every engine, and a proof it rejects is `Error::Rejected`, an error
and never a verdict). The engines' own `debug_assert!`s on their proofs
stay, and the flake's `test-debug-assertions` check is what runs them,
since crane tests in the release profile. The harness switches the check
off to time the search alone and checks the proof itself. The proof of a
goal other than the roots
has a root that concludes the goal, so `Proof::check` rejects it; only
`Interactive` consumes such proofs, by grafting their derivation, and
`Derivation::of_goal` checks them against the goal on the way. Where
`Outcome` carries the `Verdict` (`Proved(Box<Proof>)`, `Unprovable` only
after an exhaustive search, which with exponentials means a deepening
level that never hit the copy bound, `Unknown(Reason)`, with
`Reason::CopyBound` when every level hit it), the `Fragment` searched in,
the `Mode`, the `Engine` that ran, the `Statistics`, and `net`, the
`ProofStructure` the net engine found (`None` from the focused engine).
`Options` has private fields and setters (`memo_limit`, `recursion_limit`,
`engine`, `fragment`, `test_period`, `copies`, `jobs`,
`bias`, `forward_copies`, `check`), the
constants `DEFAULT_MEMO_LIMIT`, `DEFAULT_RECURSION_LIMIT`,
`DEFAULT_COPIES` and `DEFAULT_FORWARD_COPIES`, which the CLI shows as
its defaults, `MAX_JOBS` (256: `jobs` takes more as that many, and zero
as one), and `stack_size()`,
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
  multiplicity 4). Its wins were measured against the focused engine
  that enumerated its splits (five orders of magnitude on literals
  repeated three or four times across different conclusions, `wide-m3`
  and `wide-m4`); since the focused engine searches its splits by their
  counts it proves `wide-m3` at 30 and `wide-m4` at 28 in 0.15 ms, as
  fast as the net engine (0.23 and 0.38 ms), and `wide-m1` at 256 in
  7 ms against 11 ms. What still needs the net engine is width at the
  default recursion limit: a free split costs the focused engine a
  level per link, so `wide-m1` at 2 048 ends at the limit where the net
  engine proves it in 0.8 s. Whether `net` stays the default anywhere
  else is for the second baseline's `engines` runs to say; the feature
  that hurts it is equal literals under one pure tree, which the leaf
  symmetry break below would take from its weaknesses. Every other
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
  `prove_until`, and it answers `Unknown (Reason::Stopped)`. **Where it
  is polled**, which is every place a search can spend time without
  reaching another of them:
  - *The focused engine*: once per stable sequent (`prove_stable`); once
    every `SPLITS_PER_POLL` (4096) steps of its searches for the splits
    of a `⊗` or a Mix (`poll_splits`, on a counter of its own,
    `Engine::steps`: a split search whose splits fail in focus visits no
    stable sequent and can run for minutes); once every
    `FORCED_PER_POLL` (4096) forced splits and literals of tensors
    closed in place (`poll_forced`, counter `Engine::forced`: a chain of
    forced splits visits no stable sequent either, and a marking of a
    Petri net is a tensor of thousands of literals); and on a pool at
    every `&` (`with_parallel`, below). The first two pass the work
    done since the last poll (`Stop::fired(work)`), which only the two
    searches of the default bias count (`Stop::Slice`, `Stop::Turn`);
    the chain's poll passes none, so that the slices of those two
    searches are what they were before it existed and the counters of a
    decided run did not move.
  - *The net engine*: once per literal chosen (`decide`) and once per
    exact test that fails (`explore`): a run of failures chooses no
    literal.
  - *The set-up*, on a forest of `SET_UP_POLL` (65 536) occurrences or
    more (`set_up_stopped`): in `prove_goal` once the fragment, the
    reading and the dispatch are done, in `focus::search_goal` (and the
    pool's) after the classes and after the plan, and inside
    `Counts::new_until` every 65 536 occurrences visited, which is the
    longest pass. On the library's largest problem (`SYJ212+1.020` in
    its cbv translation, 27.8 million occurrences) the first poll comes
    after 0.12 s and no two are more than 0.2 s apart; without them the
    first came after 1.27 s. A smaller forest gets none of these polls:
    a pass takes under a millisecond there, and a condition that counts
    its polls (a test, a front end that counts work) sees the engine's
    own and no others.
  - *Not polled*: `Forest::new` (0.43 s on that problem; a caller with
    a deadline builds the forest itself, as the CLI does, and calls
    `prove_goal`), a single pass over the forest, the check of the
    proof at the end of `prove_goal` and the size pass of a derivation
    (below), `sequentialize`, and **the freeing of a full memo**, which
    is what a stop is late by in practice. A memo at its default cap of
    2²⁰ entries is two allocations per entry, and freeing them takes
    0.15 to 0.35 s on the machine's middle cores and 0.55 s on its
    slowest: once when the search returns, and once in the middle of a
    search every time `Memo::insert` empties a full table (`qbf/40#1`:
    a gap of 0.51 s between two polls, every two seconds). A stop that
    falls into the second and then pays the first is about a second
    late on the slowest cores. Keys that live in one allocation, or in
    an arena that is dropped whole, would remove both; that is the
    memo's own design and was left to the step that bounds its bytes.
  - *The check and the size pass are not polled because they are
    short*: on the largest proof the engines find in the LLTP library
    (`SYJ202+1.005` in its cbv translation, 566 490 inferences) the
    check takes 22 ms and `Proof::derivation_size`, the same pass with
    an observer, 39 ms; on the largest Petri nets proved 3 to 6 ms and
    5 to 8 ms. Poll them when a proof a hundred times that size is in
    reach.
  A condition must be cheap, because it is asked at every poll, and it
  must not ration its own work by counting polls: polls come millions
  of times a second on a small problem and 30 ms apart on a forest of
  millions of occurrences, so "look at the clock every 1 024 polls" was
  exact on the first and half a minute late on the second. The CLI and
  the harness read a flag that a timer thread raises. The crate docs in
  `lib.rs` show the common path (parse, fragment, prove, derivation, JSON)
  as a doc test; keep it the shortest correct program when the API moves.
  The focused engine recurses on the caller's stack, bounded by
  `Options::recursion_limit`; a caller that raises the limit runs the
  search on a thread with a larger stack (`Options::stack_size`). The net
  engine and its sequentialization keep stacks of their own.

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
  So `split`, in its search for the free splits, fixes the one output
  member of `Γ` on the consequent's side (`Reading::implication`) and
  assigns the rest; the forced splits need no change (the dual of an output positive
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
  repeats, and a split search assigns positions, so it reads a member's
  side off its own trail, never off `contains`.
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
  derivation. `run` deepens the budget from 0 to the search's bound
  (`Options::copies`; the forward search of the default bias may have
  a larger one, below). A
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
- **Complete failures are keyed up to interchangeable members of `Γ`**
  (`Context::canonical_from`: every member replaced by its class, the
  canonical key; `Θ` stays as it is). By the lemma on interchangeable
  occurrences a `Complete` failure holds for every sequent with that
  canonical key, so it is recorded there and `Memo::refuted` reads it
  for any of them. A proof names occurrences, so `Proved` stays under
  the sequent's own key; renaming a proof on a hit was not taken, since
  an occurrence of the proof may lie below a member of `Γ` and below a
  member of `Θ` at once, and which replacement applies depends on the
  path it came by. **`Exhausted` stays under the sequent's own key
  too, and must**: it is as true of a relative as of the sequent (a
  replacement keeps a proof's shape and copies), but shared it defeats
  the deepening. A sequent whose search reaches a relative of itself
  one copy lower was answered by its own entry of the level before, so
  it was cut again and recorded `Exhausted` one higher, at every level,
  where the search of the relative itself would have ended in a repeat
  and a complete failure; a review found 167 generated sequents that the
  engine refuted before and that stayed at the copy bound, and putting
  the canonical keys on the branch stack as well cured only those whose
  relative lies below them (`repeats_up_to_equal_members` pins one of
  each kind). So among relatives only facts that no budget qualifies
  are shared, and the loop check compares the sequents' own keys, as it
  always did. One table holds everything: a key that is not canonical
  holds a proof or an `Exhausted`, a canonical key may also hold a
  `Complete`, which is all a relative reads there; when the two keys
  are equal, or no two occurrences of the forest are interchangeable
  (`Classes::distinct`, which skips the canonical key altogether), the
  one `get` of before. Keying `Θ` by class too would merge more (a `?`
  below interchangeable members adds different ids) and is not done: it
  would cost a pass over `Θ` per stable sequent. Measured (stable
  sequents, memo entries): the unsolvable 3-Partition with bins of four
  4 761 and 509 before, 2 991 and 296 after; nothing on the families
  with exponentials, whose failures within a level are mostly cut ones
  (the counter with 16 tokens 473 232 and 1 049 either way).
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
  Every stack entry has its hash beside it (`hashes`), compared before
  the sequents: the ancestors of a branch mostly share `Θ` and the set
  of `Γ`, so a comparison of sequents ran over both bitsets before it
  met the difference (the `growing` family at a bound of 1 024 took
  530 ms of CPU for 392 961 stable sequents, 260 ms with the hashes).
- **The default bias with exponentials is two searches** (`focus::plan`,
  `search_goal`). `Bias::Auto` on a goal with exponentials in linear
  mode, classical or intuitionistic (the fragment searched and the
  forest both have a `!` or `?`, the mode is not affine), is decided by
  the *backward* search, `Bias::Rarer` within `Options::copies`, which
  is what `Auto` was alone before, and by the *forward* search,
  `Bias::Factors`, each an unchanged search of the engine (`Rule`: a
  bias and a copy bound) with a memo, an arena, a branch stack and
  counts of its own. The first to decide answers. What the code relies
  on, and what it promises:
  - **The contract, never less than the backward search.** With no stop
    firing, `Auto` answers `Proved` or `Unprovable` wherever
    `Bias::Rarer` does under the same options, and wherever
    `Bias::Factors` does, with the same verdict. The argument is an
    identity, not an estimate: each of the two is the explicit search
    itself. Where they alternate in slices, a search is never
    restarted, only made to wait, so its run is the explicit one
    counter for counter (`default_bias_takes_turns` pins the sum);
    where they take turns from their start, every turn begins with a
    fresh `Engine`, memo and arena, so a turn is a prefix of the
    explicit run and the turn that is not cut *is* that run. `Auto`
    ends only on a decided result or when both searches ended. The
    forward search's levels up to `copies` are those of `Bias::Factors`
    under the same options (the deepening is level by level, so a
    larger bound continues the same run).
    `default_bias_decides_what_either_rule_does` pins it on generated
    sequents, classical and intuitionistic, with and without the memo.
    On a pool the same holds up to the pool's own caveat (decisiveness
    within the bound depends on the interleaving).
  - **Nothing is shared between the two searches but the verdict**, and
    the forest, the reading and the `Classes`, which are not the
    search's. A proof and a complete failure are facts under either
    bias, since provability does not depend on it, and sharing them
    would be sound; it is not done because it would break the identity
    above: an entry from the other search changes which entries this one
    makes, and with them its decisiveness at the bound (the engine's own
    "the memo can change decisiveness within the bound", which a review
    of the first pass saw on real cases when complete failures were
    shared among relatives). A failure cut by the budget is a statement
    about one rule's search space under one budget, and the loop check
    about one branch of one search: neither means anything to the
    other search. The same reason keeps a search's own memo out of its
    next turn where turns restart it. The price is memory: two memos of
    at most `Options::memo_limit` entries each where the searches run
    at once.
  - **`Unprovable` keeps its meaning**: a level of either search that
    ended without a cut, which refutes the sequent because focusing is
    complete for every bias. `Unknown` needs both searches to have ended
    undecided; its reason is the backward search's, and a copy bound is
    reported as `CopyBound(Options::copies)`, which is true of both
    searches (the forward one was cut at every level up to its own
    bound, which is at least that).
  - **The forward bound** is `Options::copies`, and the larger of that
    and `Options::forward_copies` (`DEFAULT_FORWARD_COPIES`) where the
    sequent is a Horn program (`chains`) and the mode has no Mix. A
    program: every root under a `?` is a clause, a tensor of body
    literals, all of one sign throughout the sequent, with at most one
    factor a head instead, a literal of the other sign or a `⅋` of
    such (what `!(a ⊗ b ⊸ c ⊗ d)` lowers to); every other root is a
    marking, a `⅋` of head literals, or a goal, a tensor of body
    literals; `1` and `⊥` stand for an empty body, goal or head. A
    Petri net with a marking to reach is exactly that. Why a bound of its own: a forward chain takes
    one copy per step on one branch, where the same derivation
    backward takes as many as its tree is deep, so no multiple of
    `copies` converts one into the other. Why only on Horn clauses: a
    copy of a clause rewrites `Γ` and opens no branch for further
    copies, so a level costs the markings reachable within it, which
    the memo holds once each; on arbitrary formulas a deeper bound
    multiplies the search by the copies' alternatives per level, and
    with the bound applied everywhere the generated tests no longer
    finished. The test is on the whole sequent and on the signs because
    a first version that looked only at the shape of the formulas under
    `?` let through `(c ⊸ c), !((c ⊸ b) ⊸ c), 1 ⊢ 1 ⊸ 1 ⊗ c`, which
    answered "unknown" in 0.02 s before and took sevenfold per copy
    with the bound (a review's finding). What remains is the price of
    the bound on a real program whose markings grow: `!(a ⊸ a ⊗ b),
    !(1 ⊸ b), !(b ⊸ a ⊗ b), !(a ⊗ a ⊗ b ⊸ a), a ⊢ 1` answers
    "unknown" after 0.6 s where the backward search alone took 0.02 s,
    and of a review's 2 000 random programs 45 took over a second to
    an "unknown" that took under 0.1 s before, against 471 that are
    decided now and were not (3 and 453 at a bound of 10) (the LLTP translations of intuitionistic problems that
    `--copies 10` decides are decided by the bound, under either bias,
    and stay the user's `--copies`). Why not under Mix: every stable
    sequent a chain leaves unproved is tried in every partition, and a
    chain that grows them (`⊢ !?(~a ⅋ c)`) did not finish at a bound of
    10. `Options::copies` keeps its documented meaning for the backward
    search and for either bias named explicitly; with `forward_copies`
    at 0 the forward search runs within `copies` everywhere.
  - **When the two rules agree on every atom** the forward search is the
    backward one continued, and runs alone (its levels up to `copies`
    are the backward search's).
  - **The unit of work** is the engine's own, since the crate has no
    clock: a step of a split search is one, a stable sequent
    `NODE_WORK` plus what grows with its size (the forest's width for
    the zones, the members, the copies and their comparisons with the
    members in `meets`), a split whose premises are tried the forest's
    width again. The engine adds these up in `Engine::work` and hands
    them to the stop condition at its two polls (`Stop::fired(work)`);
    `Stop::Closure` and `Stop::Flags` ignore them, so nothing changes
    for a search that runs alone. So a run is a function of the input.
    The unit follows the time only roughly: what a stable sequent and a
    split step cost varies by two orders of magnitude between problems
    with the sizes of `Γ` and `Θ` (a forced chain's lookups of duals
    are not counted at all), so in seconds one search's share can be
    several times the other's. Counting a poll as the unit made the
    backward search's share a hundred times too long on Petri nets (a
    poll in a split search is 4 096 steps), and a flat cost per stable
    sequent starved a net's backward proof.
  - **On one core with threads** (`focus::parallel::alternate`, feature
    `parallel`, whatever `Options::jobs` says below two): the forward
    search runs on the calling thread and the backward one on a scoped
    thread of `Options::stack_size()`, and a `Baton` lets one of them
    run at a time: a search gives way after a slice of work
    (`Stop::Slice`, `SLICE`; the backward search gets `BACKWARD_SHARE`
    = 2 slices' worth) and waits for its turn. Nothing is restarted and
    nothing depends on the scheduler, so the statistics are the two
    searches' own, added up, and a function of the input. A search that
    decides stops the other at the end of its slice; one that ended
    undecided leaves the other to run on, and the calling thread then
    wakes once a millisecond. The caller's stop is not `Send`, so it
    lives on the calling thread, and it is polled once for every poll
    of either search: at the forward search's own polls, and for the
    backward search's (counted in `Baton::polls`) within a millisecond
    of each, since the calling thread wakes that often while the
    backward search has its turn (`Baton::pass_polling`) and after the
    forward one has ended (`Baton::caught_up`). It used to wait for the
    whole slice on the condition variable, and a slice is counted in
    work, not in time. A condition that counts its polls or reads a
    clock every `n` of them, as the CLI does every 1 024 on one thread,
    therefore sees what it sees of one search; polled only once per
    wake-up, it was a second late (a review's finding). The backward
    search reads `Baton::halt` at every poll. `StopOnPanic` stops it
    when the caller's condition panics. `Ended`, dropped on return and
    on a panic, hands the baton on so that nobody waits for a thread
    that is gone. What it costs against the better rule alone, in units
    of work `W`: `1.5 W` when the backward search decides and `3 W`
    when the forward one does, plus a slice. Why the backward search
    gets twice the work: it is what the default was before, so under a
    time limit everything it decides alone in two thirds of the limit
    stays decided; at equal shares two nets that it proves in 2.1 s and
    1.9 s (`NeighborGrid_z_2d_3n_1m_t_1_2_10_1`, `UtahNoC_5_1`
    classically) were not proved within 5 s, the forward search's units
    being slower there.
  - **On one thread without threads** (`focus::turns`, the fallback when
    the feature is off or the thread cannot start; `FIRST_TURN`,
    `TURN_GROWTH`, `Stop::Turn`): round `i` gives the forward search
    `FIRST_TURN · 4^i` units and the backward one twice that, each
    turn from the search's start; a search that ended undecided takes
    no further turn, and the other then runs without one. `Statistics`
    adds up every turn (the memo's entries are the most of one turn).
    The turns that end early are a geometric series, so the run takes
    less than `5 W` whichever search decides after `W` (its own cut
    turns, under a third of its last, and the other's up to that
    round); a search that decides within its first turn costs what it
    costs alone, plus one first turn of the forward search when it is
    the backward one. This scheme was the first built and measured: on
    the LLTP sample it decided what the alternating one does, at three
    times the time of the better rule in the sum and up to twenty times
    on single nets, and it lost eight of the 1 890 LLTP rows the first
    baseline decided to the 5 s limit, which is why threads are used
    where they exist.
  - **On a pool** (`focus::parallel::search_goal`, `search::parallel::
    race`, `Rule::search_on`) the two searches run side by side, each
    with its own shared memo and arena, the forward one on a pool of
    `jobs / 2` threads and the backward one on a pool of the rest, and
    a decided result raises the other's root flag. Two pools and not
    one, because a pool thread that waits at a scope runs stolen tasks:
    on one pool a thread of the search that has just decided can be
    deep inside a task of the other, which nothing stops, and the
    verdict waits for it. A pool of one thread runs the sequential
    engine (`Rule::search_on` leaves `runtime` unset). The merge
    (`merged`, shared with `alternate`): a verdict of either; else
    `Stopped` when either was stopped, which without a verdict can only
    be the caller's stop; else the backward search's reason.
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
  cleared (`Options::memo_limit`; zero switches it off; the kept arena is
  append-only, so a `Proved` id never dangles).
- **The proof arena has two parts** (`Arena`): a node is *pending* in the
  engine's own stack (`push`, an id with the `PENDING` bit) until the
  stable sequent it helps to prove is proved and memoized, when
  `prove_stable` *keeps* the nodes pushed since its `mark` (`keep` moves
  them to the kept arena, premises renamed, and returns the root's kept
  id); a failed step *releases* them (`release`, a truncation). The
  release points are `prove_stable` on a failure and the four places
  where a first premise is proved and the second fails (`premises`, the
  forced split, `with`, `parts`); every other failure pushes nothing. The
  argument that a release is safe: node ids travel only upwards as return
  values, so nothing outside the failed call holds an id pushed after its
  mark, and a memo entry holds kept ids only. The argument for `keep`:
  the nodes pending above a `prove_stable`'s mark were pushed by its own
  decision, which succeeded, so they rest on each other and on kept nodes
  (memo hits) alone, never on a pending node below the mark; that is
  debug-asserted in `append`. So the kept arena holds the proofs of
  memoized stable sequents and the final proof, and nothing of a failed
  branch: before this, one stable sequent of a Petri net pushed 110 MB of
  nodes a second for left premises whose splits then failed, and nine
  LLTP runs aborted at 16 GiB. What is not reclaimed: the proofs of
  entries the memo dropped when it was cleared, and, with the memo off
  (`memoizes` false), nothing is kept before the root, so the pending
  stack is the partial proof alone. The kept arena holds at most 2³¹
  nodes (`keep` panics beyond, 32 GiB of nodes). A proof's node order is
  the order of keeping, which `core/tests/serialize.rs` pins on one
  small proof.
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
  names. A `Tally` keeps a set's sums incrementally; a `Split` keeps those
  of the two sides of a split in the making and what the members not yet
  assigned can still add (below).
- **Interchangeable occurrences** (`focus/classes.rs`, `Classes`): two
  occurrences of the same term, and under a reading in the same
  position, share a class, named by its first occurrence. The lemma
  everything below rests on: *a sequent stays provable, by a proof of
  the same shape (hence with the same copies on every branch), when
  members of `Γ` are replaced by interchangeable occurrences one for
  one, and `Θ` by any set of occurrences with the same classes.* By
  induction on the proof: every rule reads off an occurrence its kind,
  its atom and sign, its subformulas and (two-sided) its position and
  whether it is an implication; equal terms give equal kinds, atoms and
  subterms, and the position below an occurrence is a function of its
  term and its own position (the reading's choice of the antecedent is
  made per term), so the premises of the rule on the replaced occurrence
  are again replacements of the premises. No rule looks at an
  occurrence's id, its parent or its root. What was checked and is *not*
  part of the definition: the zone (a replacement keeps each member in
  its zone, and `Θ` counts as a set of classes because `?` adds a formula
  that may already be there under another id); the branch stack (the
  loop check prunes a sequent equal to an ancestor, which stays sound
  when fewer proofs are searched: of the proofs with canonical choices a
  smallest one has no repeat either, since the choices are a function of
  the sequent they are made in and a subproof of a canonical proof is
  canonical); the copy bookkeeping (the rule that skips a `Θ` member
  with an unconsumed copy in `Γ` is by id and skips less than the same
  rule by class would, and by class it is the same normal form). The
  count rows, weights and `absorbs` are functions of the term and the
  problem's exponential atoms, so interchangeable occurrences have equal
  counts.
- **One of each kind** (`one_of_each`): of interchangeable focus
  candidates, and of interchangeable members of `Θ` to copy (among those
  the unconsumed-copy rule leaves), only the lowest id is tried; the
  focus on another leaves the same sequent up to a replacement. A
  repeated occurrence of `Γ` is one candidate for the same reason.
- **Canonical splits**: in `search_splits` the left side takes, of each
  class, the members with the lowest ids, so only the number taken
  varies (`C(n, k)` splits of `n` equal hypotheses become one per `k`).
  Sound and complete by the lemma: a split with another choice of as
  many gives the same two premises up to a replacement. In the order of
  `Engine::open` a class is a run with descending ids, so the rule is
  "a member goes left at once when the one before it is of its class
  and went left", which the trail decides; on the pool the patterns
  that break it are not spawned (`split_parallel`), so the chunks still
  partition the splits searched. Under Mix the first member, which is
  fixed on the left, has the lowest id of all, which agrees with the
  rule; a partition both of whose parts hold members of that first
  member's class still comes up twice, which costs time only. Measured (stable sequents, splits): the unsolvable 3-Partition
  with bins of four 1 834 321 and 13 015 869 before, 4 761 and 54 193
  after; the counter with 8 tokens 2 316 421 and 4 644 336 before,
  14 228 and 42 105 after, and with 16 tokens, which no run had
  finished, 473 232 and 2 004 517.
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
- **A factor that is a tensor of positive literals forces its side too**
  (`Forced::Duals`, `Counts::literal_tensor`): one dual per literal, each
  the first left in `Γ`, and the candidate fails when one is missing;
  the factor's proof is then built in place (`literal_tensor`: the
  axioms and the `⊗` nodes from the last occurrence back), with no focus
  on it, so a chain of such tensors costs no recursion either. When both
  factors force, the one closed in place goes first
  (`forced_factor`): `a ⊗ b ⊗ c` is nested to the left, and taking the
  tensor on the left as the forcing factor cost a focus per link (2 500
  tokens ran into the recursion limit; `limits` pins the chain).
  The argument: in focus the factor is decomposed by `⊗` rules down to
  its literals, each of which stays in focus and closes by an initial
  rule alone, that is on exactly its dual, from `Γ` or by a copy from
  `Θ`; the rule applies only to tensors none of whose literals has a dual
  directly under a `?` anywhere in the forest (the only way a literal
  gets into `Θ`), so every dual comes from `Γ` and the factor's side is
  one dual per literal and nothing else. Which occurrences is immaterial
  by the lemma on interchangeable occurrences. Without that proviso the
  rule would be the single literal's "dual from `Γ` first" applied per
  literal, which is complete but moves copies between branches, and
  would leave fewer proofs within a copy bound than the search of every
  split finds; so with a dual in `Θ` the split stays searched. Two-sided
  it needs no change, for the reason the single literal needs none: the
  premise it forces is the only classically provable one. Affine mode
  forces nothing, as before. This is what a Horn clause's body is when
  its atoms are positive, so `(b ⊗ t) ⊗ ~d` costs no split search at
  all. Measured (splits, the stable sequents unchanged): the unsolvable
  3-Partition with bins of five 36 072 before and 3 074 after, Partition
  with seven items 2 078 and 766, the unsolvable one with five 11 433
  and 2 106; of the sampled LLTP nets, `RwMutex_rwmutex-r2000w10_1_1` is
  proved in 1.1 s and `Diffusion2D_2D8_gradient_40x40_50_5_1` in 0.27 s
  where 4.9 s were needed before it.
- **`split_passes`** is the count test of a split as a function (the
  engine's `Rules::new` and a `Split` with every member placed), for the
  interactive state's helper. It is `Split::feasible`, the very test the
  engine's search ends on, so the two cannot drift apart.
- **Free splits are searched, not enumerated** (`search_splits`, for `⊗`
  and Mix alike, `Join` saying which). The statement: the splits whose
  premises are searched are exactly those whose two sides pass the counts
  (the interval check, and the equation where it is on), each once; a
  split that fails the counts is never visited, and no other is skipped.
  The members are assigned one at a time, in a fixed order, to the right
  first and then to the left (a depth-first search with an explicit
  trail, so a context of any width costs no recursion and no mask: there
  is no width limit, and `Reason::ContextTooWide` is gone). A partial
  assignment is cut when `Split::feasible` fails, which is a necessary
  condition for some completion to pass, per side and per atom on its
  own: with `lo`/`hi` the side's sums so far and `below`/`above` the sums
  of the negative parts of the open members' `lo` and of the positive
  parts of their `hi`, a side without an absorbing member needs
  `lo + below ≤ 0 ≤ hi + above` for every atom (any completion adds a
  subset of the open rows, and a subset's sum is bounded by those parts),
  unless an open member absorbs and can still join it, each open
  absorber serving one side (`needy ≤ open_absorbers`); the equation
  likewise with the slack `c − weight − 2` of a side and the open
  members' contributions `1 − weight`. With no member open the bounds
  are the sides' own sums, so the test at a leaf is the old `sides_pass`
  exactly; the argument for the cut is that it only removes subtrees
  whose every leaf fails that test. `Split::bad` counts the excluded
  atoms incrementally (`update` compares before and after, both sides,
  on the atoms of the member's row), so a step costs the member's row.
  The order (`Engine::open`): longer rows first (a compound member
  bears on several atoms; once the compounds are placed the literals of
  an atom are settled by its counts), then by the row's first atom, so
  that an atom's literals are neighbours, then descending id, so that
  the lowest ids change sides fastest, as they did in the Gray-code
  enumeration this replaces; members without a row (units, exponential
  atoms only) last, where only the equation can cut. The order changes
  which proof is found first and nothing else. `Statistics::splits`
  counts the steps of these searches (one feasibility test each) and the
  forced splits. Where no prune can cut (`Split::set_inert`, decided in
  `Engine::open`: the equation off and no member, placed or open, with a
  row entry that excludes zero by itself, which is every split under
  weakening and every Mix of formulas like `(a ⊗ b) ⊕ 0`), `feasible` is
  true whatever the assignment, so the counts are left alone and the
  members not even opened; the steps and their count are the same.
  `Tally::clear` and `Split::clear` zero the atoms their members touched
  (`touched`), not every atom of the sequent: a Petri net has thousands
  of atoms, none with a row, and cleared 24 KB per stable sequent.
  Measured on the first baseline's instances: Partition
  with six items 946 564 520 splits before and 3 337 after at the same
  94 stable sequents, the unsolvable one with four items 8 192 777 and
  6 845, QBF over 12 variables 9 389 062 and 37 980.
- **Mix** is tried last on a stable sequent, after the copies, with the
  first member fixed on the left so each partition comes up once, the
  trivial partition skipped (a leaf with an empty right side), the
  partitions searched by `search_splits` like the splits of a `⊗`, and
  each part decided by `prove` with the same `Θ` and budget, so the memo
  shares parts between partitions. Refuting a
  wide sequent with Mix costs about `3^k` stable sequents for `k` members.
- **Recursion.** `prove`, `focus` and `asynchronous` count one level each;
  `Options::recursion_limit` stops the search with
  `Reason::RecursionLimit`. Two chains cost no level per link, since the
  LLTP Petri nets have them by the thousand: the `?` rules of an
  asynchronous phase are applied in `decompose`'s loop on a growing copy
  of `Θ` (a net's transitions are `!` hypotheses, and one level per `?`
  stopped every net with more than about two thousand of them before its
  first stable sequent, which is what 929 of the first baseline's 983
  recursion-limit rows were), and a chain of forced splits runs in
  `forced_splits`' loop on one context that every forcing factor takes
  its own from, the `⊗` nodes built afterwards from `links` in the order
  the recursion pushed them (a marking is a tensor of thousands of
  literals). A positive literal that forces a split is closed in place
  (`Ax`, or `Ax` under `Copy` from `Θ`, exactly what `initial` does on
  one or two members), and a dual literal is looked up through the
  forest's list of that literal's occurrences, in id order
  as before, not by a pass over the zone: in a chain by `dual_from`,
  which starts where the chain's last lookup of that literal ended
  (`Cursors`, one position per list, reset through the list of those
  that moved; a chain's context only loses members, so what a lookup
  passed over is gone for the rest of the chain), and elsewhere, for
  the unrestricted zone, by `dual_in` from the head. From the head
  every time, a chain over a marking of thousands of equal tokens was
  quadratic in them. A nested chain (below a `!` that a forced split
  promotes) has another context and takes cursors of its own from the
  pool. None of this changes a counter
  on the sequential targets; what changes is which sequents reach the
  limit. Free splits still cost a level per link. Measured stack per
  level, on a chain of tensors whose splits are searched, which is the
  deepest set of frames (`focus`, `split`, `free_split`,
  `search_splits`, `premises`): 3.6 KiB in debug builds, 0.9 KiB in
  release, with the split's counts boxed in their pool (1.5 KiB with
  them in the frame, which overflowed the stack `Options::stack_size`
  gives at a raised limit; it now allows twice the measured). So the
  default of 2048 fits an 8 MiB main-thread stack, in a debug build only
  just; it stays, since of
  the 24 sampled problems that ended at the limit only two (ILLTP-SYJ
  problems whose search is that deep) still do.
- **No allocation per node once warm**: sets, contexts, keys, member lists,
  tallies, split counts, trails and the links of forced chains come from
  pools on the engine (`take_*`/`give_*`); a leaked buffer on an error
  path only costs an allocation later. The memo insert clones its key,
  and a repeated occurrence grows a context's extra list; those are the
  allocations per stable sequent. The copies are ordered by an unstable
  sort on the id and one pass that asks `meets` once per formula: a
  stable `sort_by_key` allocated its buffer for every stable sequent
  with more than twenty copies and called `meets` at every comparison
  (58 % of the samples on the chain of 256 clauses). `meets` itself
  reads marks: `mark_literals` stamps, once per stable sequent, the
  lists of the literals among its members (`Engine::present`, one
  stamp per list of the forest, `Engine::stamp` the current one, a
  `u64` that cannot wrap in any run), and a formula of `Θ` meets a
  member when a literal below it has its dual's list stamped. Comparing
  every literal below every formula with every member was their
  product: a quarter of a second per stable sequent on
  `GPPP_G-PPP-1000-10_10_1` (a marking of thousands of tokens under
  clauses of thousands of literals), between two polls. The order of
  the copies and the work counted for a slice are what they were.
- **The memo can change decisiveness within the bound, never a verdict.**
  An `Exhausted` entry is a fact about the sequent alone, the loop check
  about the branch, so a run with the memo may answer `Unknown` where a
  memo-free run answers `Unprovable` (or the reverse); the generated tests
  assert only that the two never contradict.
- **`exhausted` and `dependency` are engine-wide flags** saved, cleared and
  restored by hand inside `prove_stable`; an early return between the
  decision and the restore, or a stack push anywhere else, silently
  breaks the level's completeness claim.
- **Every proof passes the checker**: in `prove_goal` for a proof of the
  roots, in every build; `debug_assert!` in `search` besides, and
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
  what its conclusions reach. O(n²) in the net's size. It does not
  recurse: a stage leaves what proves its sub-net as steps on a stack of
  the sequentialization's own (`Step`: a sub-net to prove, a Mix, a `⊗`,
  the `⅋` rules of the stage), taken in the order a recursion would make
  its calls, so the nodes come in that order too and a derivation of any
  height costs the caller's stack nothing
  (`a_high_derivation_needs_no_stack`). The proof is checked in a
  `debug_assert!` and the round trip derivation → net → derivation is a
  test.
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
- **The stack** (`Frame { literal, next, forced }`): every frame but the top has
  its current link made, the top is looking for one; `next_partner`
  moves `next` past the partner it returns, so a linking is tried at most
  once; a dead end, a failed test or an exhausted frame takes the last
  link back, always in stack order, which the structure's undo log
  requires. `remaining[atom]` (unlinked pairs per atom) follows every
  link and unlink. The stop condition is polled once per node, in
  `decide`, and once per exact test that fails: a run of failed
  candidates and exhausted frames chooses no literal, and each failure
  costs a test over the structure. A frame is `forced` when its literal
  had exactly one admissible partner when it was chosen (`choose`
  counts the best literal's partners to the end, so the count is exact
  and is what `next_partner` will find); `Engine::choices` counts the
  frames that are not, which is what a pool's cubes are cut by (below).
  Neither changes the search on one thread.
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
- **Every proof passes the checker** (in `prove_goal`, in every build;
  `debug_assert!` in `search` besides; every test), and every net is the
  net of its proof (`from_proof` in the tests' `run`). The differential test against the focused engine
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
  it does on the caller's), which the search drops with the outcome
  (the net engine's in `prove_goal`, the focused engine's in
  `focus::parallel::search_goal`, which builds two for the two searches
  of the default bias with exponentials and splits `jobs` between them);
  `Error::ThreadPool` when the threads cannot start. **A search starts
  no more threads than the machine runs at once**: `prove_goal` takes
  `Options::jobs` through `parallel::threads`, the smaller of it and
  `std::thread::available_parallelism()` (which on Linux follows the
  process's CPU set and quota; where the platform does not tell, the
  options' own bound `MAX_JOBS` is all there is), and one thread left
  is the sequential path. So the tests' `jobs(4)` is two threads on a
  machine of two, and a harness row never names more threads than its
  process may run (`linlog-bench run` refuses the count). Never touch rayon's
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
- **The split searches poll too** (`poll_splits`, every
  `SPLITS_PER_POLL` steps, through the same `Stop`): before, the poll
  there was tied to `Statistics::splits` being a multiple of 4 096, and a
  loop that adds two to the counter per round (its own split and a forced
  one below) from an odd value never hit one. That is how 90 one-thread
  LLTP runs of the first baseline ran past their kill, 166 more at 16
  threads, and two Petri nets were proved minutes after their limit.
  `focus::tests::stops_inside_a_split_search` and the second half of
  `focus::parallel::tests::stops` pin it on a sequent whose 2⁴² splits
  all fail in focus (affine mode, so no count cuts them).
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
  (the first usually fails fast). **`with_parallel` polls the engine's
  flags before it starts anything.** The asynchronous phase polls
  nowhere else (its stable sequents do), and the `&` rule on the pool,
  unlike the sequential one, starts its right premise without waiting
  for the left: a worker that was cancelled or stopped therefore went
  on to start both premises of every `&` below it, each of which did
  the same, and only their stable sequents ended them. With `n` `&` in
  one asynchronous phase that is `2ⁿ` workers. `SYJ202+1.008` in its
  cbv translation (19 KB) has such a tower: under a 5 s limit two
  threads ended after 46 s and four not within 150 s, with 578 stable
  sequents really searched and 268 million stopped at their poll; the
  first failed premise alone set it off, long before the limit.
  `a_cancelled_premise_starts_no_other` pins it on forty roots `a & b`
  (2⁴⁰ premises without the poll). A new rule that fans out on the pool
  polls before it spawns. Mix stays sequential after the
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
  or-node, failure or error at the `&`. A premise that the other's
  error cancelled returns `Stopped`, which gives way to that error in
  the `&`'s result as it does in `Collected::take`: the match took the
  left premise's reason first, so a right premise at the recursion
  limit made the pool answer `Unknown (Stopped)` with no stop fired
  (a review's finding; `a_cancelled_premise_is_no_stop`). What stays:
  an error of one premise cancels the other, so the pool answers
  `RecursionLimit` where the left premise would have failed and one
  thread, which never starts the right one, answers `Unprovable`; and
  a premise's result raises the flag only once its worker has left its
  nested scopes, where a waiting thread may have stolen a task of the
  sibling premise that nothing cancels until then (seen once: an answer
  that came only with the caller's stop, five seconds late). A worker inserts into the memo
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
- **The kept arena is shared behind one `Mutex<Vec<Node>>`**
  (`Kept::Shared`), and every engine of a parallel search has a pending
  stack of its own. Truncating a shared arena would be unsafe (another
  worker's nodes lie above one's mark), so nothing pending is shared: a
  `keep` takes the lock once and appends the whole segment, so a kept id
  is a position in the one arena every `Proved` entry refers to, a
  segment's premises are in it or were kept before it, and the order
  `Proof::new` needs holds across threads; the memo insert happens after
  the keep, so a hit always finds a complete subtree. A pending id means
  nothing outside its engine, so a result that leaves a worker (an
  alternative's, a `&` premise's, the root's) is kept first
  (`Engine::exported`); the spawning engine wraps kept ids in pending
  nodes of its own. Failed branches therefore cost the shared arena
  nothing on the pool either.
- **Levels never overlap**: `run` deepens the copy bound on the root
  engine, which spawns nothing until its first choice and reads
  `exhausted` after every task of the level has ended (the scope waits),
  so the or-reduction over the workers is the merge above and a level is
  `Unprovable` only with every worker's flag clear.
- **The net engine's cubes are what is left of the search, split at
  its choices** (`net::parallel::search`). A cube is the links of a
  branch nobody has followed yet, and the list of cubes is kept in the
  order in which the sequential search would reach them. The root
  engine starts from the one cube without a link and, while there are
  fewer than `CUBES_PER_THREAD` (16) cubes per thread, makes a pass
  over the list that replaces every cube in place (`reset`, `seed`) by
  the branches of its next choice (`explore(Some(1), …)`: the search
  below the seed with every branch recorded and taken back at its first
  link of a literal with more than one admissible partner; the forced
  links on the way to that choice are made and stay in the cube). So
  after `d` passes the cubes are the branches of the first `d` choices
  that the tests do not reject. A branch
  that dies leaves no cube, a proof net found on the way ends
  everything, and an empty list is `Unprovable`. So nothing is searched twice: the cubes
  partition the remaining search at every moment, and a sequent whose
  links are all forced is decided by the first `explore`, which is the
  sequential search, link for link (`forced_links_are_made_once` pins
  equal statistics on `wide(64, 1)`). Before, the cubes were the
  branches of the first `d` links found by a search from the root for
  `d = 1, 2, …`; with forced links there is one branch at every depth,
  the count was never reached, and `wide-m1` at 512 pairs made
  1 + 2 + … + 1 024 = 524 800 links (9.5 s on two threads against
  33 ms on one). What it costs now where the queue stays short: the
  seed's links again per cube taken, without their tests, which is of
  the order of the `choose` the sequential search pays per node.
  Workers pull cubes from an atomic counter with one engine each, so
  the per-worker state is allocated once; a worker that finds a net
  stores it and raises the flag; `Unprovable` needs every cube to have
  ended `Ok(false)`, and any error or a real stop makes the verdict
  `Unknown`. **The order is what keeps a pool from being slower than
  one thread on a provable sequent**: the workers take cubes in the
  sequential search's order, so the cube with the proof one thread
  finds is taken no later than one thread reaches it. A first version
  kept the cubes in a queue and appended a cube's branches at its end;
  the workers then searched cubes that one thread never enters before
  the one with the proof, and Partition with the items 1, 1, 2, 4 took
  two threads 2.2 s and 253 852 literals against 1.3 s and 109 627 on
  one. **A pass is always whole.** Stopping in the middle of one, as
  soon as the count is reached, left the unsplit cubes at the end of
  the list one choice coarser than the rest, and the refutations of
  3-Partition were 3 to 4.5 % slower on two and four threads than with
  cubes of one depth; the count may therefore overshoot by a level's
  branching, as it always could.
  No state is shared beyond the flags: the structure and the scratch
  are per worker. A cube's seed must reproduce the root engine's state
  at the record: the links in order, which the structure's undo log and
  the test cadence (a multiple of the period in links) both follow.
- **A parallel run may return another proof, never another verdict**:
  every level is searched to its end by some worker with no cube
  abandoned unless a proof or an error ends it, so `Proved` and
  `Unprovable` agree with the sequential engine; only decisiveness within
  the copy bound may differ (as it does between memo and no memo), since
  the memo's contents depend on the interleaving. A parallel run may
  answer `Proved` where the sequential one answers `Unknown
  (RecursionLimit)` on another alternative. `Unknown (Stopped)` is the
  caller's stop, never a cancellation. **What a pool promises about
  time**: a stop is honoured by every worker at its next poll (the
  list under "Proof search: the front door"), the driver asks the
  caller's condition once a millisecond, and a pool costs its start
  (some tens of microseconds per thread) plus, on the net engine, the
  seeds above; it promises no speedup, and on the focused engine no
  bound on the work relative to one thread (and-parallel `&` premises
  and cubes search what one thread might have skipped). The tests
  (`focus::parallel::tests`, `net::parallel_tests`) assert exactly this
  on the generated samples with two and four threads; every proof is
  checked. For the focused engine that is `agree`: the two verdicts
  never contradict each other, and nothing is asserted about which of
  them decides within the bound (`b, ((a * 1) -o !a), !(b -o 1),
  !(1 -o ((1 * 1) -o a)), b |- (!!a * b)` with a copy bound of 2 is at
  its bound on one thread and proved on four, every time).
- **There is no portfolio of worker orders.** An option gave every
  worker a seed that ordered the candidates and copies within their
  classes; two baselines showed no gain (the second: 627 problems
  decided against 623, 10 gained and 6 lost, at the same time), and it
  was removed. The two searches of the default bias are the portfolio
  that pays: two orders that differ in the one choice that changes the
  search. Candidates and copies are ordered by id on every engine.
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
memo's. The memo holds at most `Options::memo_limit` pairs and is emptied
when full, like the focused memo (zero switches it off); the product
bound on the time then no longer holds in theory, but the identity of
depth 16 (7.9 million pairs without a cap) is decided with the default
limit of 2²⁰ in 11.7 million visits instead of 10.7 million, in less
time (a table that fits the cache) and in 0.1 GB instead of 0.46 GB.
What took gigabytes on such a proof was not the search but the checker's
first implementation, which kept a `Θ` bitset of the forest's width for
every node (262 141 nodes of 32 KB at depth 16, 7.6 GB): the first
baseline's "8 GB of memo" was this, measured by the peak before and
after the check. The checker no longer keeps one ("The checker").

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
  printers do not recurse either: they are loops over `sequents::fmt::Walk`,
  as `Display` is.
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

`core/src/parse/mod.rs` is a precedence parser written by hand: one loop
over the characters in two states (`operand`, what a formula starts with,
and `operator`, what follows an operand) with one explicit stack
(`Parser::pending`: the open parentheses, the prefix operators, and the
binary connectives with their left operands, each waiting for the operand
to its right). A term goes into the arena the moment it is complete, so
there is no syntax tree, nothing recurses, and nothing has a drop that
does: a formula nested 100 000 deep, or a chain that long, parses on a
stack of 256 KiB (`depth_costs_no_stack` in `core/tests/parse.rs`), in
time and memory linear in the text. What the code relies on:

- **The arena comes out in the order a recursive lowering gives it**:
  postorder, in the order of the text (an operand when it is read, a
  connective when its right operand is complete), the roots in the order
  of the text, the atoms numbered by first occurrence. `optimize()` at
  the end therefore gives the `Sequent` the first parser gave (a
  recursive descent that built a tree and lowered it), and with it the
  occurrence ids that every stored proof and snapshot names. A change to
  the order of the pushes is a format break.
- **Negation is applied at the end.** A term is pushed as it is written;
  a `~`, a `^`, being the antecedent of a `⊸` (which is pushed as `⅋`)
  and standing left of the turnstile each flip the term's flag in
  `Parser::negated`, and `finish` goes once from the last term to the
  first, dualises a flagged term (`Term::dual`) and passes the flag to
  its subterms. Before `optimize` every term has one parent, which is
  what makes the flag well defined.
- **Tokens depend on the state**: `par` is the connective only where a
  connective can stand and a variable where a formula starts (`|- par par
  par` is `par ⅋ par`); `bot` and `top` are constants only as whole
  identifiers; `|-` is the turnstile only on the left side outside every
  parenthesis, and anywhere else a `|` before a `-` that starts no
  formula. An identifier starts with `_` or `XID_Start` and goes on with
  `XID_Continue` (the `unicode-ident` crate's tables).
- **An error is one `ParseError`**: the byte span of the first character
  that cannot go on a sequent, and that character, or the end of the
  input. Two tokens of two characters make the exceptions the first
  parser made: a `-` that no `o` follows where a connective can stand,
  and a `|` that no `-` follows at the very start (where only the
  turnstile can stand), report the character after them.
  `error_positions` in `core/tests/parse.rs` pins both and the rest.
- A text of more terms than a forest can hold (`Forest::MOST`) is
  `Error::TooManyOccurrences`: every term of a text is an occurrence.
- The first parser was chumsky's Pratt parser. Before it was removed
  the two were compared on 1.1 million generated inputs (token soup,
  random sequents in every spelling, and those with one character
  removed, replaced or added; 481 000 sequents and 619 000 errors): the
  same `Sequent` or the same error span and character on every one. The
  test, `agrees_with_the_first_parser`, is in the history with the
  change that introduced this parser.

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
  `{"copy_bound": n}`), `fragment`, `mode`, `engine`, `statistics`,
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

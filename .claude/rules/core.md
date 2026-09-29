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
`A |- A` as `⊢ ~A, A`. Intuitionistic sequents will use the same model (plan
decision D1): an ILL sequent is a one-sided sequent of a particular shape,
which step 8 of the plan tests on the forest; there is no second data model.

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
- Intuitionistic mode is refused (`Problem::Intuitionistic`) rather than
  checked as classical: the one-succedent condition needs the input/output
  side of every occurrence, which depends on which root is the succedent,
  and the same term can be ILL-valid under one reading of an ambiguous
  sequent and not under another (`⊢ ⊤, ⊤ ⊗ ⊤` read as `⊤ ⊸ 0 ⊢ ⊤` or as
  `0 ⊢ ⊤ ⊗ ⊤`). Step 8 owns that choice and replaces the refusal.
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
Step 8 extends this to two-sided sequents; step 5 reads axiom links off the
`ax` inferences (or the `Ax` nodes).

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

`proofs/fmt.rs` draws the tree: premises side by side, bottom-aligned, three
columns apart; a bar of `─` spanning their conclusions or the conclusion,
whichever is wider, with the rule name after it; the conclusion centred
under the bar. Widths are character counts (every symbol used is one column
in a monospace font), lines are trimmed on the right, and there is no
trailing newline. The renderings are pinned in tests, so a layout change is
a test change.

## Proof search: the front door

`search/mod.rs` is what a front end calls: `prove(&sequent, mode,
&options)` and `prove_until(…, stop)` return `Result<Outcome, Error>`, where
`Outcome` carries the `Verdict` (`Proved(Box<Proof>)`, `Unprovable` only
after an exhaustive search, `Unknown(Reason)`), the `Fragment` searched in,
the `Engine` that ran and the `Statistics`. `Options` has private fields
and setters (`memo_limit`, `recursion_limit`, `engine`, `fragment`);
`Reason`, `Statistics`, `Engine` and `Outcome` are `#[non_exhaustive]` so
later steps add variants and fields without a breaking change.

- The dispatch is plan decision D8. Today every classical input without
  exponentials goes to `focus`; intuitionistic and affine modes and
  exponentials are `Error::NoEngine`, an error and not an `Unknown`, until
  steps 7 and 8 fill the rows. A new engine gets an `Engine` variant, a
  row in `prove_until` and a line in the CLI's output.
- `Options::fragment` asserts a fragment: a sequent outside it is
  `Error::FragmentMismatch`, and the search runs in the asserted fragment,
  which switches off the prunes that only hold in the smaller one.
- The crate has no clock (D11): a time limit is a closure the caller gives
  `prove_until`, polled once per stable sequent; it answers `Unknown
  (Reason::Stopped)`. The engine recurses on the caller's stack, bounded by
  `Options::recursion_limit`; a caller that raises the limit runs the
  search on a thread with a larger stack.

## The focused engine

`search/focus/mod.rs` is the spec's MALL-Seq, one engine for every fragment
up to MALL with units and Mix as rule switches (`Rules`, from `Fragment`
and `Mode`). Its functions are the spec's rules: `asynchronous` (the phase
`⊢ Γ ⇑ L`), `prove` (a stable sequent), `focus` (`⊢ Γ ⇓ F`), `split`
(the `⊗` rule), `mix`. What it relies on:

- **Stable sequents only.** The asynchronous phase runs to completion (`⅋`
  opens, `⊥` drops, `⊤` closes with a `Top` node and the pending `⅋`/`⊥`
  nodes wrapped around it, `&` branches on copies of the state); what
  reaches `prove` is a set of positive formulas and negative literals, and
  only those are memoized. `?` is unreachable until step 7 adds the dyadic
  zone.
- **Memo validity.** An entry is a fact about an occurrence set:
  `Proved(NodeId)` into the engine's arena, which a hit reuses as a shared
  subproof (the arena is append-only, so clearing the memo never dangles),
  or `Failed`. In fragments without exponentials it holds unconditionally,
  because cut-free provability depends on the set alone; step 7 must add
  the copy bound to the entry. When the memo is full it is cleared
  (`Options::memo_limit`; zero switches it off). Never memoize across
  forests.
- **A `0` is fatal only without a `⊤`.** The spec calls a `0` in a stable
  sequent fatal, but `⊢ 0, ⊤ ⊕ b` is provable through the `⊕`; the
  immediate failure applies only when no member has a `⊤` below it
  (`Tally::absorbs`). The other immediate tests: a dual pair succeeds; a
  literal-only sequent fails without Mix; an unbalanced sequent fails.
- **Counts** (`focus/counts.rs`): per occurrence a sparse row of intervals
  per atom (literals `±1`, `⊗`/`⅋` sum, `&`/`⊕` hull, units nothing), an
  `absorbs` flag (a `⊤` at or below it: the row is meaningless and any set
  containing the occurrence passes), and a `weight` `t − p − #1 + #⊥`. The
  interval check is sound in every fragment without exponentials (proof by
  induction on the rules, with `⊤` covered by the flag and `0` as `(0, 0)`).
  The hull for `&` is the spec's choice; the intersection would be sound
  too and stronger, and is a follow-up. The count equation
  `c = t − p − #1 + #⊥ + 2` (`≥` with Mix, and `>` for a Mix to be worth
  trying) is only sound without additives, additive units or
  exponentials, and `Rules::equation` switches it on for exactly those
  fragments; `⊢ a ⊕ b, ~a` is the counterexample the spec names. A
  `Tally` keeps a set's sums incrementally, so a split moves one row per
  flip.
- **Focus candidates.** Every `⊗` and `⊕` of a stable sequent; `1` only
  when alone (it needs an empty context); never a literal (a positive
  literal in focus succeeds only in the dual-pair case). Order: a `⊗` with
  a forced split first, then `⊕`, then a `⊗` whose split is enumerated;
  ascending ids within a class. This order is what makes the run
  deterministic, with the memo, which is only looked up, never iterated.
- **Forced splits.** A factor that is a positive literal takes exactly its
  dual from the context, and the first dual occurrence when there are
  several (they are the same formula, so the residues are equal
  multisets); a factor `1` takes the empty context; a factor `0` fails the
  candidate, not the sequent. `⊤`, `⊥` and negative literals force
  nothing: `⊢ ⊥ ⊗ b, a, ~a, ~b` needs `{a, ~a}` on the `⊥` side.
- **Free splits** enumerate the submasks of the compacted members in
  Gray-code order (`submasks`), the empty submask first, two tallies moved
  per flip, and both sides must pass the counts before either premise is
  searched. More than 63 members is `Reason::ContextTooWide` for the whole
  search, never a silent failure; the spec's lazy contexts (step 14) or a
  branch-and-bound over the members are the ways past it.
- **Mix** is tried last on a stable sequent, with the first member fixed on
  the left so each partition comes up once, the trivial partition skipped,
  and each part decided by `prove`, so the memo shares parts between
  partitions. Refuting a wide sequent with Mix costs about `3^k` stable
  sequents for `k` members.
- **Recursion.** `prove`, `focus` and `asynchronous` count one level each
  (at most three per occurrence); `Options::recursion_limit` stops the
  search with `Reason::RecursionLimit`. Measured stack per level: under
  2 KiB in debug builds, under 512 bytes in release, so the default of
  2048 fits an 8 MiB main-thread stack.
- **No allocation per node once warm**: sets, member lists and tallies come
  from pools on the engine (`take_*`/`give_*`); a leaked buffer on an
  error path only costs an allocation later. The memo insert clones its
  key; that is the one allocation per stable sequent.
- **The atom bias hurts Horn clauses.** The forest makes the rarer literal
  positive, so clause bodies whose atoms also appear as hypotheses are
  usually negative and their `⊗` splits are enumerated instead of forced;
  the 3-Partition refutation in the tests takes about a minute in release
  mode for that reason. A bias override is step 13 material.
- **Every proof passes the checker**: `debug_assert!` in `search`, and
  every test that gets a proof calls `check`. The test-only generator
  `search/generate.rs` builds random provable sequents (and mutants of
  them) for every combination of units, additives and Mix; a new engine or
  rule set extends it rather than writing new positives by hand.

## Layout

`sequents` (arena, printing), `parse`, `serialize`, `fragment`, `occurrences`
(forest and sets), `proofs` (terms in `mod.rs`, `check`, `derivation`, the
renderer `fmt`, the crate-private `multiset`), `search` (the front door in
`mod.rs`, the focused engine in `focus/` with `counts` and `memo`, the
test-only `generate`), and the empty `nets` and `export` modules that the
plan fills in. `lib.rs` re-exports the public types, so users write
`linlog::Sequent`, `linlog::Proof`, `linlog::prove`, and so on. `hash` is
crate-private.

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
  A new `Reason` variant or `Statistics` field needs its line in the proxy.
- `Forest` has no serde; it is rebuilt from the sequent.

# Step 1 report: the core refactor

Session of 2026-09-29, from `plan/01-core-refactor.md`. Realises plan
decisions D1 (the data model stays one-sided NNF arenas), D2, D3, D4 and D5.

## What was built

Six changes, in order (`jj log`):

1. **Drop the Logic and Index type parameters.** `Sequent` is a plain type
   over `Vec<Term>`; `TermId(u32)` indexes the arena, `Atom(u32)` the atom
   dictionary. `LL`/`MLL`, the `Index` trait, `LLExpression`, the
   `Expression` trait and `subenum` are gone. `Term` has `kind()`, `dual()`,
   `subterms()`, `atom()`; `Kind` is the payload-free enum of connectives and
   literals. The parser, `Display` and serde are unchanged in behaviour;
   `core/tests/serialize.rs` pins the JSON strings and round trips
   (serde_json became a dev-dependency of `core` for that, at the version
   already locked for the CLI). `Sequent` gained read-only accessors
   (`terms`, `term`, `roots`, `atom_names`, `atom_name`, `atom(name)`,
   `formula(id)` as a `Display` view) and derives `PartialEq`.
2. **Adopt foldhash for the hash tables of core** (through the `new-tool`
   skill; the crate-source-explorer read the pinned source). `hash.rs`
   defines crate-private `HashMap`/`HashSet` aliases over
   `foldhash::fast::FixedState`; `optimize()` uses them.
3. **Lay out the modules the plan names.** `core/src/linear/` is replaced by
   `search`, `proofs`, `nets`, `export` (empty, with module docs), and later
   `fragment` and `occurrences`. `lib.rs` re-exports every public type;
   `parse` and `serialize` are private modules that contribute trait impls.
4. **Add Fragment and Mode with detection.** `Fragment(u8)` with the five
   class flags, the constants `EMPTY`, `MLL`, `MLL_WITH_UNITS`, `ALL`,
   `MALL`, `MELL`, `LL`, the subset order `contains`, `union`/`|`,
   `intersection`/`&`, `has_*` queries, `name()`/`Display`.
   `Kind::fragment()` maps a connective to its class. `Sequent::fragment()`
   is one descending pass over the arena with reachability from the roots.
   `Mode { intuitionistic, affine, mix }` with `CLASSICAL`,
   `INTUITIONISTIC`, `.affine()`, `.with_mix()` and a `Display` in words.
5. **Add the occurrence forest.** `Forest` per D5, see "For steps 2 and 3".
6. **Add bitsets over occurrence ids.** `OccSet`, `submasks`, and
   `Forest::{empty_set, root_set}`.

Then this change: `.claude/rules/core.md` rewritten for the new model,
CLAUDE.md's crate paragraph, the README's architecture paragraph, the crate
doc, the crate-source-explorer's crate list (foldhash in, subenum out), the
update-deps skill's list of direct dependencies, and this report.

## Decisions where the prompt left room

- **Bitset: hand-written `Box<[u64]>`, not fixedbitset.** The engines want
  exact-width raw words as memo keys and arena-stored keys (spec, MALL
  "Data layout"); that is the entire type, about 150 lines with tests. A
  crate would add SIMD blocks with padding, a growable length, and an API
  shaped around a different use. fixedbitset stays the fallback if a later
  step needs an operation the hand-written type lacks.
- **Hasher: foldhash with `FixedState`.** foldhash was already in the graph
  through chumsky's hashbrown, so no crate was added. The pinned version
  (0.1.5) needs no OS randomness at all; on wasm32-unknown-unknown its
  `RandomState` falls back to address-derived entropy, which is fine but not
  guaranteed reproducible. A fixed seed makes every run hash the same way,
  which benchmarks and bug reports want, and nothing in `core` faces
  adversarial keys. rustc-hash would have done as well; foldhash was chosen
  because it costs nothing to add.
- **Layout.** Directory modules with `mod.rs`, as the existing code does.
  `OccSet` lives in `occurrences::set` and is re-exported from
  `occurrences` and from the crate root. `hash` is crate-private; step 3 may
  make it public if the CLI ever needs the alias.
- **The forest owns a clone of its `Sequent`** rather than borrowing it, so
  that `Proof`, `ProofNet` and `Outcome` carry no lifetime (the wasm and
  serde surface would suffer otherwise). The clone is one arena; the forest
  is built once per problem. The "no `String` inside the forest" constraint
  is read as "no per-occurrence strings": the atom dictionary is the
  interning table, and the forest's own arrays are `u32` or narrower.
- **`Forest::new` returns `Result`.** A DAG with 40 levels of shared
  tensors has 2^41 occurrences; JSON input can express that. The size
  computation is a saturating pass over the arena, and
  `Error::TooManyOccurrences` is returned before anything is allocated. The
  doc example therefore has a `?`.
- **Children are derived, not stored**: `left(o) = o + 1`, `right(o) =
  left + size(left)`, from the preorder invariant. Polarity is stored (one
  byte per occurrence) because every engine reads it in its inner loop;
  the atom is read through the arena term (`sequent.term(term[o]).atom()`),
  which is two loads and needs no extra array.
- **Field names**: `term_arena`/`term_ids`/`variable_dict` became
  `terms`/`roots`/`atoms`. The plan's "root order = `term_ids` order" is
  now "`Sequent::roots` order"; the JSON keys (`terms`, `ids`, `var_dict`)
  are untouched.
- **`Fragment::ALL`** is the spec's name for additive-only linear logic. It
  sits next to `LL`; the doc comment says so explicitly. `Display` prints
  the smallest named fragment containing the value (`MLL`, `MLL with
  units`, `ALL`, `MALL`, `MELL`, `LL`); the empty fragment (atoms only)
  prints `MLL`, and multiplicative units only show in the name of a purely
  multiplicative fragment, where they decide the engine (D8).
- **`Mode` is three public bools** with constants and builder methods, not
  three enums: the CLI flags map onto them directly, and `Display` gives
  the words.
- **Atom bias tie**: `Var` positive. An atom occurring with one sign only
  has that sign negative (zero occurrences of the other sign is "fewer"),
  which is the point of the rule: its literals are never focus candidates.
- **`Sign` is `Var`/`DualVar`**, mirroring the term variants, so that
  "sign" (which literal) and "polarity" (focusing) never share a word; the
  spec uses "positive/negative literal" for both.
- **`submasks` is limited to 63 members** (`1 << 64` overflows); the spec
  switches to lazy contexts above ~20 members anyway.

## Deviations from the spec or the plan, with reasons

- The spec's `Formula { hash, size, dual }` fields are not stored on
  `Term`: hash-consing goes through the `HashMap` in `optimize()`, sizes are
  computed per forest, and no engine needs the arena index of a dual formula
  (the axiom rule compares atom and sign of occurrences). Nothing prevents
  adding them later.
- The spec's per-occurrence "index of the ⅋/⊗/&/⊕ node above" is the
  parent except under `!`/`?`; a parent walk covers it and no engine in the
  spec reads it directly.
- The spec's LCA via Euler tour and sparse table is a parent walk, as the
  prompt allowed; `is_below` (two comparisons) is the O(1) check most
  callers want.
- Interval counts and the count equations (spec "Common infrastructure",
  also listed under `ll-occ` in the spec's crate layout) are not built: the
  prompt scoped this step to the forest and sets, and step 3 owns the prunes.
- The `u64` specialisation for `n ≤ 64` is not done; `OccSet` with one word
  is one heap allocation of 8 bytes. Step 13 measures whether an inline
  variant pays.
- No decision in `plan/README.md` turned out wrong. D3's `Atom` name is
  used for the variable index, as it lists.

## Open questions

- Intuitionistic fragment names (`IMLL`, …) for `Fragment::Display` are
  mode-dependent; step 8 adds a mode-aware name (a method taking `Mode`, or
  a wrapper), leaving the classical `Display` as is.
- `Fragment`, `Mode` and `Forest` have no serde. Step 4 decides the wire
  form (name strings versus flags) when the CLI prints outcomes.
- Whether `Proof` (step 2) owns its `Forest`, shares it through `Rc`, or
  refers to it by id: the forest is cheap to clone but not free, and a
  search produces one proof per problem.
- There is no public constructor for a `Sequent` besides the parser and
  serde. Tests in later steps that must run without the `parse` feature
  build arenas as struct literals inside the crate; a small public builder
  can be added when an external user needs one.

## For steps 2 and 3

Types and paths (all re-exported from the crate root):

| type | module | role |
|---|---|---|
| `Sequent`, `Term`, `TermId`, `Atom`, `Kind`, `Formula` | `sequents` | the arena, a node, its indices, the connective enum, a printable formula |
| `Fragment`, `Mode` | `fragment` | detection value, user options |
| `Forest`, `OccId`, `Polarity`, `Sign` | `occurrences` | the forest, its index, focusing polarity, which literal |
| `OccSet`, `submasks`, `Flip`, `Submasks` | `occurrences::set` (also `occurrences::*`) | sets and split enumeration |
| `Error` (`TooManyOccurrences` is new), `ParseError` | crate root | errors |
| `hash::{HashMap, HashSet, BuildHasher}` | `hash` (crate-private) | the memo tables' hasher |

Building and reading a forest:

```rust
let sequent: Sequent = "A, A -o B |- B".parse()?;   // ⊢ ~A, A ⊗ ~B, B
let forest = Forest::new(&sequent)?;                 // or Forest::try_from(sequent)
let n = forest.len();                                // width of every OccSet
for o in forest.ids() {                              // preorder
    let _ = (forest.kind(o), forest.polarity(o), forest.parent(o), forest.size(o));
    let _ = (forest.left(o), forest.right(o), forest.children(o));
    let _ = forest.subtree(o);                       // o .. o + size(o)
}
for &root in forest.roots() { /* the sequent's formulas */ }
for &l in forest.all_literals() {                    // grouped by atom, then Var, then DualVar
    let (atom, sign) = (forest.atom(l).unwrap(), forest.sign(l).unwrap());
}
let a = sequent.atom("A").unwrap();
let (pos, neg) = (forest.positive_literals(a), forest.negative_literals(a)); // by bias
let vars = forest.literals(a, Sign::Var);            // by sign
let bias: Sign = forest.bias(a);
println!("{}", forest.formula(o));                   // prints the subformula
forest.lca(x, y);                                    // None across roots; forest.same_root(x, y)
forest.is_below(o, ancestor);                        // O(1)
```

Sets and the tensor split:

```rust
let mut gamma = forest.root_set();                   // the sequent as a set
gamma.remove(o); gamma.insert(o); gamma.contains(o); gamma.toggle(o);
let members: Vec<OccId> = gamma.iter().collect();    // compacted, ascending
let (mut left, mut right) = (forest.empty_set(), gamma.clone());
for flip in submasks(members.len()) {                // Gray order, empty submask is the start
    let o = members[flip.position as usize];
    left.toggle(o); right.toggle(o);                 // left ∪ right = gamma, disjoint
}
let key = gamma.clone();                             // Hash over the words; HashMap<OccSet, Entry>
&a | &b; &a & &b; &a - &b; a |= &b; a.is_subset(&b); a.is_disjoint(&b); a.words();
```

Other facts: `Kind::polarity()` is `None` for literals; use
`forest.polarity(o)`. `Kind::sign()`, `Kind::arity()`, `Kind::is_literal()`,
`Kind::dual()`, `Kind::fragment()` exist. `Term::subterms()` yields the left
subterm before the right one. `Sequent::fragment()` and
`Fragment::contains(other)` drive dispatch; `Mode` is `Copy`. Doc examples
that parse use the `cfg_attr(feature = "parse", doc = "```")` fence pattern
(`.claude/rules/core.md`, "Parsing"). The JSON format is pinned in
`core/tests/serialize.rs`; extend that file when proofs get a format.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`, `cargo test
--workspace` (25 unit tests in core, 9 parse and 3 serialize integration
tests, 2 doc tests), `cargo test -p linlog --no-default-features`, `cargo
hack check --feature-powerset -p linlog`, `cargo deny check`, rustdoc with
`--deny warnings`, and `nix flake check`: all pass at the last change.

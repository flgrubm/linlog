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

## Layout

`sequents` (arena, printing), `parse`, `serialize`, `fragment`, `occurrences`
(forest and sets), and the empty `proofs`, `nets`, `search`, `export` modules
that the plan fills in. `lib.rs` re-exports the public types, so users write
`linlog::Sequent`, `linlog::Forest`, and so on. `hash` is crate-private.

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
indices. The JSON is an interchange format for the CLI and the planned web
front end, so a tag or key change is a format break; `core/tests/serialize.rs`
pins the exact strings. `Fragment`, `Mode` and `Forest` have no serde yet.

---
paths:
  - "core/**"
---

# linlog core: data model and invariants

Loaded when a file under `core/` is read. What follows is what the code relies
on but does not say in one place.

## Sequents are arena-allocated DAGs

`Sequent<I: Index, L: Logic<I>>` (`core/src/sequents/mod.rs`) has three fields:
- `term_arena: Vec<L::Expression>`: every subformula. Children are referenced by arena index, never by pointer.
- `term_ids: Vec<I>`: the root formulas that make up the sequent.
- `variable_dict: Vec<String>`: variable names. `Var(n)`/`DualVar(n)` index into this.

**A term only references subterms with a strictly smaller index**, so the
arena is topologically sorted. `verify_integrity()` checks this,
`display_term` debug-asserts it, and deserialization runs the check. Code that
builds or rewrites an arena must preserve it.

`optimize()` runs after parsing. It deduplicates variable names, hash-conses
identical terms, drops unreachable ones and sorts `term_ids`.
`Sequent::add` merges two sequents by offsetting variable and term indices.

## One-sided, negation normal form

When parsing, terms on the left of `⊢` get negative polarity. Negation is
pushed down to atoms with `dualize()`, so there is no general negation node,
only `DualVar`. `A ⊸ B` becomes `A^⊥ ⅋ B`. Printing therefore gives
`A |- A` as `⊢ ~A, A`.

## Index and logic fragments

- `Index` (`core/src/index.rs`) is a sealed trait over unsigned integers whose
  width fits the target's `usize`. It keeps arena indices small. Parsing and
  serde exist only for `Sequent<usize, LL>` so far.
- `LLExpression` (`sequents/expressions.rs`) is the full classical LL connective
  set. `#[subenum(MLLExpression)]` generates the multiplicative subset. The
  `Expression` trait implements every operation (`dualize`, `offset`,
  `check_bounds`) once, on `LLExpression`, and converts back and forth with
  `Into`/`TryFrom`. Anything that matches on expressions converts to
  `LLExpression` first, as the existing code does.
- `logics.rs` connects a marker type (`LL`, `MLL`) to its expression type. A
  new fragment needs a `#[subenum(...)]` marker on each variant it keeps and a
  `Logic` impl.
- `core/src/linear/{ll,mll}` are empty placeholders for per-fragment algorithms
  such as proof search.

## Parsing

`core/src/parse/mod.rs` has two stages:
1. A chumsky Pratt parser produces a borrowed AST (`parse::Term`/`parse::Sequent`).
2. That AST is lowered into the arena, which creates one variable entry per occurrence and then calls `optimize()`.

chumsky's API changed wholesale after 0.9, and most examples online and in
memory are for the old one. When a signature is in doubt, ask the
`crate-source-explorer` agent rather than guessing.

Every operator has ASCII and Unicode spellings: `* ⊗`, `| par ⅋`, `&`,
`+ ⊕`, `-o ⊸`, prefix `~ ! ?`, postfix `^`, and `|-`/`⊢`. Precedence, from
tightest: `^` > `~ ! ?` > tensor > par > with > plus > lollipop (right-associative).

## Serialization

`core/src/serialize/sequents.rs` uses a private serde proxy struct
`{terms, ids, var_dict}` with short tags (`V`, `D`, `⊗`, `⅋`, …). The JSON is
an interchange format for the CLI and the planned web front end, so a tag change
is a format break.

# Step 2 report: proof terms, the checker, derivations and serialization

Session of 2026-09-29, from `plan/02-proofs.md`. Realises plan decision D6:
proofs are terms over occurrence ids, checked independently of any engine,
with a derivation view that rendering and export consume.

## What was built

Five changes, in order (`jj log`):

1. **Add proof terms.** `Proof` (`core/src/proofs/mod.rs`) owns a `Forest`
   and an arena of `Node`s, one per rule instance; `NodeId(u32)` indexes it.
   `Node` is a 16-byte `Copy` enum covering every engine's rules: `Ax`,
   `Tensor`, `Par`, `One`, `Bot`, `With`, `Plus` (with a `Side`), `Top`,
   `Bang`, `Quest`, `Copy`, `Weaken` and `Mix`. `Proof::new(forest, nodes,
   root)` verifies bounds and order and keeps the nodes the root reaches.
2. **Add the proof checker.** `proofs::check::check(&proof, mode)`, also
   `Proof::check(mode)`, with `CheckError { node, rule, premises, problem }`,
   `Dyadic` (a derived sequent as ids) and `Problem` (what the rule
   required). Tests apply every rule once correctly and once wrongly in
   MLL, MALL, MELL, affine mode and with Mix.
3. **Add the derivation view and its text rendering.** `Derivation`
   (`proofs/derivation.rs`) with `Inference`, `InfId` and `Rule`, built by
   `Proof::derivation()`, and `Display` (`proofs/fmt.rs`) drawing the tree.
   Tests pin the rendering of one derivation per fragment and the
   inference data of a MELL proof.
4. **Serialize proofs.** serde for `Proof` behind `serialize`
   (`serialize/proofs.rs`), pinned in `core/tests/serialize.rs` with round
   trips and rejection of broken files.
5. **Document the proof model**: `.claude/rules/core.md` (three new
   sections and the serialization note), CLAUDE.md's API paragraph, the
   README's architecture paragraph, `.claude/rules/claude-infra.md`'s index
   line, and this report.

## Types and paths

All re-exported from the crate root.

| type | module | role |
|---|---|---|
| `Proof`, `Node`, `NodeId`, `Side` | `proofs` | the arena, a rule instance, its index, which `⊕` side |
| `check::check`, `CheckError`, `Problem`, `Dyadic` | `proofs::check` | the checker, its error, what went wrong, a derived sequent as ids |
| `Derivation`, `Inference`, `InfId`, `Rule` | `proofs::derivation` | the view, one inference, its index, the rule names with `Display` |
| `Error::{NodeIndexOutOfBounds, PremiseIndexNotDecreasing, OccurrenceIndexOutOfBounds, InvalidProof}` | crate root | what `Proof::new` and deserialization refuse; `InvalidProof` wraps a `CheckError` |

## How an engine constructs a proof

There is no builder type: the arena is a `Vec<Node>` the engine owns during
search. It pushes a node after the nodes of its premises, so every premise
index is smaller than the conclusion's, and keeps the `NodeId` of a proved
sequent in its memo, so a memo hit reuses the subproof (the arena is a DAG;
the derivation view unfolds it). Subproofs of branches that failed stay in
the arena; `Proof::new(forest.clone(), nodes, root)` drops what the root
does not reach, renumbers, and verifies bounds and order, but does not
check the proof. The forest is cloned once per problem, at the end (the
prompt's decision: `Proof` owns its `Forest` by value; no lifetime or
`Rc` reaches the API).

```rust
let mut nodes: Vec<Node> = Vec::new();
let l = nodes.len() as u32; nodes.push(Node::Ax(a, na));      // ⊢ a, ~a
let r = nodes.len() as u32; nodes.push(Node::Ax(b, nb));      // ⊢ b, ~b
nodes.push(Node::Tensor(t, NodeId::new(l), NodeId::new(r)));  // ⊢ a ⊗ b, ~a, ~b
let proof = Proof::new(forest.clone(), nodes, NodeId::new(2))?;
debug_assert_eq!(proof.check(mode), Ok(()));
```

What the checker expects of an engine's terms:

- The rules are the dyadic ones of the spec, unfocused: a `Copy` for every
  D2 step (on the occurrence of the formula in `Θ`, which is the subformula
  of the `?`), a `Quest` where the asynchronous phase moves a `?A` into
  `Θ`, `Bang` with an empty linear zone. The initial rule with `p⊥ ∈ Θ`
  (`⊢ Θ, p⊥ ; · ⇓ p`) is `Copy(p⊥)` above `Ax(p, p⊥)`. Focusing leaves no
  trace: a focused phase is just a run of nodes.
- `Top(o)` records no context; the checker treats a `⊤` leaf as absorbing
  whatever context reaches it, so the engine emits `Top` with nothing else.
- Affine mode: the spec's relaxed rules (`Γ ⊋ {p⊥}` in the initial rule,
  `Γ ≠ ∅` under `!` and `1`) are not rules of the term. The engine emits
  one `Weaken` node per surplus formula *below* the `ax`, `!` or `1`
  (weakening above a promotion would put the formula into the promoted
  context, which is not allowed). `Weaken` is refused outside affine mode
  unless the formula is a `?` formula (that is the standard `?w`, which a
  dyadic engine writes as a `Quest` whose formula goes unused); `Mix` is
  refused without Mix.
- Every occurrence id is the engine's own forest's; nothing is memoized
  across forests (D5).

## The dyadic-to-standard translation

The derivation view shows one-sided sequents of the standard calculus, as
ascending lists of occurrence ids with repeats. The standard sequent of a
subproof is `⊢ ?Θ, Γ` where `Θ` is the *least* unrestricted zone the
subproof needs, the one the checker derives bottom-up (`Copy` adds, `Quest`
removes). This is not Andreoli's translation, which would contract all of
`Θ` at every `⊗` and weaken all of it at every leaf; ours puts structural
rules only where they are needed:

| term node | derivation |
|---|---|
| `Copy(A)` | `?d` on the copy; below it a `?c` if `A` is used again above |
| `Quest(?A)` | nothing if `A` is used above (the sequents coincide), else `?w` |
| `Tensor`, `Mix` | the rule, then one `?c` per `?` formula both premises use |
| `With` | above each premise, one `?w` per `?` formula only the other uses, unless a `⊤` in it absorbs them |
| `Bang` | `!` with the `?` context as is |
| `Top` | `⊤` with the context it absorbs, which flows down to it through every rule; a `⊗` split gives the absorbed part to the absorbing premise |
| `Weaken` | `wk`, or `?w` when the formula is a `?` formula |
| the rest | themselves; `Plus` becomes `⊕₁` or `⊕₂` |

`Rule` is `Ax Tensor Par One Bot With PlusLeft PlusRight Top Promotion
Dereliction Contraction Weakening Mix AffineWeakening`, printing as `ax ⊗ ⅋
1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ?d ?c ?w mix wk`. An `Inference` carries the sequent, the
rule, the principal formula's position (`None` for `ax`, whose sequent is
its two literals, and Mix) and the premises. The `ax` inferences, or the
`Ax` nodes directly, give step 5 the axiom links.

The renderer draws premises side by side, bottom-aligned, three columns
apart, a bar spanning their conclusions (or the conclusion, if wider) with
the rule after it, and the conclusion centred under the bar:

```
─────── ax    ─────── ax
⊢ ~A, A       ⊢ ~A, A
──────── ?d   ──────── ?d
⊢ ?~A, A      ⊢ ?~A, A
────────────────────── ⊗
  ⊢ ?~A, ?~A, A ⊗ A
  ───────────────── ?c
    ⊢ ?~A, A ⊗ A
```

## Serialization

`{"sequent": <the sequent's JSON>, "proof": [node, …]}`, one object per node
with the rule's tag and its numbers as an array (one integer for `1` and
`⊤`): `{"ax":[0,2]}`, `{"⊗":[1,0,1]}` (occurrence, left premise, right
premise), `{"⊕₁":[3,0]}`, `{"⊤":2}`, `{"copy":[1,0]}`, `{"wk":[1,0]}`,
`{"mix":[1,3]}`, and `⅋ 1 ⊥ & ⊕₂ ! ?` alike. Premises come before
conclusions and the root is last. Deserialization rebuilds the forest from
the sequent, refuses an empty arena, an occurrence outside the forest or a
premise not before its node, and drops unreachable nodes; it does not run
the checker, because the mode is not in the file (step 4 decides the wire
form of `Mode`), and the CLI's `check` command will. The tags are frozen
like the sequent's.

## Decisions where the prompt left room

- **`⊤` through a flag, not a recorded context.** The checker works
  bottom-up because a term does not record how a `⊗` splits its context,
  so it must derive conclusions from premises; `⊤` is the one rule whose
  conclusion the premises do not determine. Rather than storing the
  absorbed context in the node (a heap object per `⊤`, and an engine
  detail), the derived sequent carries an `any` flag meaning "and any
  further linear context". Each rule combines the flags so that the derived
  sequent still characterises exactly what the subterm proves; promotion
  resets the flag. The derivation view instantiates the context top-down.
- **`Θ` as a least requirement.** The checker never tracks the actual
  unrestricted zone; it derives the least one each subproof needs and
  requires it empty at the root, which is equivalent to "every copy has its
  `?` step below it". This also makes the standard translation small.
- **Multisets as sorted vectors**, not bitsets with an overflow map: the
  checker needs sums, unions, differences and inclusion on multisets, which
  a merge over sorted lists gives in a dozen lines; each node costs the size
  of its zone. The prompt suggested bitsets where copies cannot repeat; one
  representation for all fragments was preferred, since the checker is not
  a hot path.
- **`Node` as an enum** rather than a struct of tag, occurrence and child
  array: the same 16 bytes, and every match in the checker and the view
  reads as the rule it implements. `Plus` carries a `Side` rather than
  splitting into two variants, as the spec writes it.
- **No ILL rule tags.** On the lowered sequent (D1) every ILL rule is a
  classical one: `⊸L` is `⊗`, `⊸R` and `⊗L` are `⅋`, `&L₁`/`&L₂` are
  `⊕₁`/`⊕₂`, `⊕L` is `&`, `&R` is `&`, `1L` is `⊥`, `1R` is `1`, `0L` is `⊤`,
  `⊤R` is `⊤`, `!R` is promotion, `!L` is dereliction, `!c`/`!w` are
  `?c`/`?w`, `ax` is `ax`. So the term type needs nothing for step 8; the
  derivation view will need the two-sided sequent and the ILL names.
- **Intuitionistic mode is refused**, not checked as classical. The
  one-succedent condition needs the input/output side of every occurrence,
  which depends on which root is the succedent; when a sequent has several
  output-shaped roots (`⊤`, `0` and formulas built from them are ambiguous),
  the same term is ILL-valid under one reading and not another: `⊢ ⊤, ⊤ ⊗ ⊤`
  read as `⊤ ⊸ 0 ⊢ ⊤` rejects the term that sends the root `⊤` to the left
  premise of the `⊗`, while `0 ⊢ ⊤ ⊗ ⊤` accepts it. Choosing the reading is
  step 8's shape analysis, so a `Problem::Intuitionistic` marks the spot
  rather than a check that is silently incomplete. The prompt's alternative,
  an argument that the classical terms cover the ILL rules, is the table
  above.
- **`Proof::new` compacts** rather than refusing unreachable nodes, in the
  spirit of `Sequent::optimize` dropping unreachable terms, because an
  engine's arena legitimately holds the subproofs of failed branches.
- **`Quest` vanishes from the view** when its formula is used above: the
  dyadic step "this `?A` is now in `Θ`" has no standard counterpart, and
  Yalla has no rule for it either. An unused one becomes `?w`.
- **Errors carry ids, not formulas.** `CheckError` has structured fields
  and a `Display` with ids; a message with formulas needs the forest, which
  the CLI (step 4) has. Keeping strings out of the error keeps it `Eq` and
  cheap to match on in tests.
- **The renderer is the tree**, not the indented one-line form the prompt
  allowed as an alternative; the tree is what Click & coLLecT draws and
  what a teaching tool shows.
- **No postcard.** It would come from the same serde proxies (postcard
  encodes enum variants by index, so the tags cost nothing), but no
  consumer wants a binary format yet; the crate that does adds the
  dependency.

## Deviations from the spec or the plan, with reasons

- The spec's term has `Plus(o, side, p)`, `Bang(o, p)`, `Quest(o, p)` and
  `Copy(o, p)` and nothing for weakening or Mix; `Weaken` and `Mix` were
  added as the prompt required. The `⊕` side is a `Side`, not an integer.
- The checker is linear in the proof size times the zone size, since each
  node copies and merges its premises' zones; the spec's "linear time" is
  read as this. A bitset representation would make it proof size times
  forest width over 64.
- No decision in `plan/README.md` turned out wrong.

## Open questions

- **Intuitionistic checking** (step 8): the reading of an ambiguous sequent,
  and then the one-succedent test on every derived sequent, which is a
  count over a side map once the map exists. The derivation view for ILL
  will need the two-sided sequent (hypotheses and the succedent) and the
  ILL rule names; `Inference` and `Rule` are the extension points.
- **Nullary Mix** (`⊢` from nothing) is not a rule; only binary Mix. Add a
  node if an engine or the Yalla export ever needs it.
- **Canonical node order.** `Proof::new` keeps the engine's order, so two
  engines that find the same proof may serialize it differently. A
  postorder renumbering would make the arena canonical if that matters.
- The derivation builder recurses over the tree; its depth is the
  derivation's height, thousands at most for the inputs the plan targets.
  If a CLI run ever overflows the stack, run it on a thread with a larger
  one, as the spec advises for the search anyway. A proof whose arena
  shares subproofs heavily (memo hits under `&`) unfolds to a tree that is
  exponentially larger than the arena; the view is meant for proofs a
  human reads.
- `CheckError::Display` with formulas: a method taking the forest, when
  the CLI wants it.

## For step 3

- Build proofs as "How an engine constructs a proof" says; `Node` is
  `Copy`, `NodeId::new(nodes.len() as u32)` before the push is the id.
  `proof.check(mode)` in a `debug_assert!` and in every test.
- The forest's `roots()` are ascending ids, and every occurrence set the
  checker compares is ascending, so a `Vec<OccId>` collected from an
  `OccSet` is already in the checker's order.
- The hand-written proofs in `core/src/proofs/check.rs`'s tests are the
  reference for what the terms of the classic examples look like
  (`⊢ ~A, A ⊗ ~B, B`, `⊢ A & B, ~A ⊕ ~B`, `⊢ ⊤ ⊗ A, ~A, B`, `!A ⊢ A ⊗ A`,
  `!A, !(A ⊸ B) ⊢ !B`, Mix on `A ⊗ B ⊢ A ⅋ B`). The random proof generator
  the step 3 prompt asks for produces exactly such node lists and can feed
  them to `Proof::new` and `check` before any engine exists.
- The derivation view and the renderer are what the CLI prints in step 4:
  `proof.derivation()?.to_string()`.

## Review of the checker

A fresh-context reviewer (a sub-agent that had not seen the design being
made) read the spec's dyadic rules and the checker, wrote an independent
top-down reference checker with no `any` flag and no least-`Θ` trick
(`⊤` as membership, `⊗` and Mix by enumerating splits, copies by
membership in a top-down `Θ`) in a throwaway crate outside the
repository, and compared the two on 1255 proofs found by a random search
over 40 sequents across MELL, additives, units, affine mode and Mix, on
75 300 mutants of those proofs and on 600 000 random terms, in all four
modes: no disagreement. It found one bug, a panic in the intuitionistic
refusal when the root has premises (fixed, with the test changed to a root
that has one), and one gap, `Weaken` of a `?` formula refused in linear
mode although `?w` is a rule of MELL (now accepted in every mode, shown as
`?w`). The intricate cases it named, a `⊤` standing in for the formula a
promotion consumes, both sides of a `&` absorbing, a copy taken from an
absorbing premise, one `?` occurrence given its `?` step twice on one
path, and one subproof shared by both premises of a Mix, are pinned in
`accepts_every_rule`. It confirmed the invariant the derivation view rests
on (the sequent passed down equals `?Θ` plus `Γ` plus what a `⊤` absorbs,
the last empty unless the subproof absorbs) through every rule.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`, `cargo test
--workspace` (35 unit tests in core, 9 parse and 6 serialize integration
tests, 3 doc tests), `cargo test -p linlog --no-default-features`, `cargo
hack check --feature-powerset -p linlog`, rustdoc with `--deny warnings`,
and `nix flake check`: all pass at the last change. No dependency changed.

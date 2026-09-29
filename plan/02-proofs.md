# Step 2: proof terms, the checker, derivations and serialization

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (decisions D5 to D9 matter here; this step realises D6)
  and `plan/reports/01-core-refactor.md` (what step 1 built and how to use
  it).
- `proof-search-specifications.md`: "Common infrastructure" (especially
  "Proof output" and "Memoization contract"), the rules listed in each
  engine specification (MLL-Seq, MALL-Seq, MELL-Seq, the affine variant, the
  two-sided engine), and "Cross-cutting engineering notes" ("Certificates
  first", "Testing strategy").
- `plan/notes/export-targets.md`: how Click & coLLecT and Yalla represent
  proofs. Ours must be translatable into both later (steps 9 and 11), which
  fixes what the derivation view has to contain.
- `.claude/rules/core.md` and the `core/src/**` step 1 left.

## Goal

A proof representation that every engine will emit, an independent checker
that decides whether a proof proves a sequent, a derivation view for humans
and exporters, and serialization. No search yet. Proof nets (D6a) are step 5
and build on the derivation view, so design the view with the conversion in
mind: from a derivation of MLL one must be able to read off the axiom links
(which literal occurrence is paired with which).

## What to build

1. **Proof terms (D6).** The spec's term over occurrence ids, one node per
   rule instance, stored in an arena (`Vec` of nodes with `u32` children)
   owned by a `Proof` value that also records the forest it refers to (by
   value or by an id the caller resolves; decide and document). Cover the
   rules of every engine in the spec so no later step has to change the
   type: axiom, `⊗`, `⅋`, `1`, `⊥`, `&`, `⊕` (side), `⊤` (`0` is never a
   rule), `!` (promotion), `?` handling in dyadic form (the `?` step that
   moves into Θ, `Copy` for using a Θ formula, and whatever weakening the
   translation to the standard calculus needs), Mix, weakening for affine
   mode, and the intuitionistic left rules of the two-sided engine
   (`⊸` left, `&` left, `⊕` left, `!` left, `1` left, `0` left) or a
   convincing argument that the classical terms cover them through D1. Keep
   the term small: a rule tag, the principal occurrence, and child indices.
2. **The checker.** A function that takes a sequent (as a forest) and a
   proof and returns `Ok(())` or a precise error (which node, which rule,
   what the premise sequents were and what the rule required). It interprets
   the unfocused rules of the standard calculus and the dyadic bookkeeping
   exactly as the spec states them, re-deriving every node's sequent, in
   linear time in the proof size (bitsets over occurrence ids; count vectors
   where copies can repeat an occurrence). It shares no code with any engine
   (engines do not exist yet; keep it that way by putting the checker in
   `proofs`, and say so in its doc comment). Mode-aware: linear vs affine
   (weakening), Mix on or off, classical vs intuitionistic (one succedent,
   the two-sided rules), so that it rejects a proof that uses a rule the mode
   forbids.
3. **Derivation view.** From a proof and its forest, the tree of explicit
   sequents and rule names of the standard sequent calculus: for classical
   mode one-sided sequents `⊢ Γ` with structural rules explicit (dyadic `?`
   handling expanded into dereliction, contraction and weakening in the
   usual way; document the translation), and a design that step 8 can extend
   to two-sided intuitionistic sequents `Γ ⊢ A` with the ILL rule names.
   Each node carries the sequent as occurrence ids (so a renderer can print
   formulas via the forest), the rule name, and the principal formula's
   position. Rule names as an enum with `Display`, the usual spellings
   (`ax`, `⊗`, `⅋`, `1`, `⊥`, `&`, `⊕₁`, `⊕₂`, `⊤`, `!`, `?d`, `?c`, `?w`,
   `mix`, `wk`, and the ILL ones).
4. **Text rendering.** A plain-text/Unicode rendering of a derivation as a
   tree (Click & coLLecT's `to_ascii_list` draws stacked boxes with the rule
   name on the bar; an indented one-sequent-per-line form is an acceptable
   alternative or addition). This is what the CLI prints by default in
   step 4, so make it readable: formulas printed as `Sequent` prints them.
5. **Serialization.** serde (behind `serialize`) for proofs, compact: the
   term arena with short tags like the sequent format, the occurrence ids as
   integers, and the sequent (or a reference to it) so a file is
   self-contained; deserialization runs the checker or at least the bounds
   checks. Consider a binary format (`postcard`) only if it comes cheaply
   through the same derives and can live behind the same feature; otherwise
   leave a note. Round-trip tests.
6. **Documentation.** Extend `.claude/rules/core.md` with the proof model
   (what a term is, the checker's independence, the translation to the
   derivation view, the format's stability). Update CLAUDE.md if module
   names or commands changed.

## Constraints

- No `unsafe`; no dependency without a reason; `core` stays wasm-friendly.
- The proof format is an interchange format like the sequent JSON: choose
  tags you are willing to keep.
- Hand-written proofs in the tests are the only proofs there are; write
  enough of them: every rule at least once positive and once negative
  (checker rejects), MLL, MALL, MELL, affine and Mix cases, and the
  derivation view of each compared against an expected rendering.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`cargo deny check` if a dependency changed, `nix flake check` at the end
(run `jj st` first).

## Deliverables

- Thematic jj commits ("Add proof terms", "Add the proof checker", "Add the
  derivation view and its text rendering", "Serialize proofs", …).
- `plan/reports/02-proofs.md`: the term and derivation types with their
  module paths, how an engine constructs a proof (the builder API), the
  dyadic-to-standard translation you chose, decisions and deviations, open
  questions, and what step 3 must know. Commit it with the last change.

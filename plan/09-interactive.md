# Step 9: interactive proving in the library

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D5, D6, D7, D9, D11, D13) and every report in
  `plan/reports/`, above all `02-proofs.md` (the term, the checker, the
  derivation view and its translation), `03-focused-engine.md` and
  `07-exponentials.md` (the engines, their entry points, how a goal with
  `?` formulas is searched) and `08-intuitionistic.md` (the two-sided
  reading and rule names).
- `plan/notes/export-targets.md` § "Click & coLLecT": its JSON has a
  `ruleRequest` with a rule name and a `formulaPosition`, and an
  `appliedRule: null` for an open leaf; that is the interaction model to
  match in spirit, not in format.
- `core/src/proofs/**`, `core/src/search/**`, `cli/src/**`.

## What step 8 left you

`plan/reports/08-intuitionistic.md`, "For steps 9 to 12 and 15". The
two-sided engine is not a second engine: `search::focus::search` and
`search_goal` take an `Option<&Reading>` (`Reading::new(&forest)`, the
intuitionistic reading of the whole forest), and with one they keep the
goal on the consequent's side of every `⊸L` split. So `search_goal(forest,
goal, fragment, mode, reading, options, stop)` already decides a goal
two-sided; in intuitionistic mode a goal must hold exactly one occurrence
in output position (`Reading::outputs`), which is what to validate when
the client applies a rule. `search::additive::search` decides a pair of
roots; for a goal of two additive-only occurrences it needs a pair entry,
the rest stays with the focused engine. In intuitionistic mode a goal is
shown two-sided (`Reading::position` picks the side, `Reading::formula`
the ILL spelling, as `proofs/fmt.rs` does for `Derivation::two_sided`),
a rule is named through `Rule::intuitionistic(position)`, and Mix is
refused (`Error::IntuitionisticMix`). The one-succedent conditions the
checker tests are R1 to R3 in `.claude/rules/core.md` § "The checker";
the interactive layer validates the same three at application time and
still runs the checker at the end.

## Goal

A library user (the CLI now, the web client later) can build a proof step
by step: hold a partial derivation with open goals, ask which rules apply
to a formula of a goal, apply one and get the goals that remain or a
precise reason why it does not apply, undo, ask the search to close one
goal or all of them, and at the end obtain a `Proof` that the independent
checker accepts. The state is serializable so that a session can be saved
and resumed and so that a web client can hold it.

## What to build

1. **The state.** A type (say `Interactive`, in `proofs::interactive` or
   its own module; name it for a human) that owns the forest, the mode,
   the inferences made so far in the standard calculus (the same sequent
   representation as `Inference`: occurrence ids in ascending order with
   repeats; the same `Rule` names), and the open goals, each an id and a
   sequent. It starts from a `Sequent` (the whole thing is one open goal),
   or from a partial state read back from JSON. `Clone` is the undo
   mechanism the client can use; also give it an explicit `undo` that
   retracts the last applied step, since every client wants that.
2. **Which rules apply.** Given a goal and a position in its sequent, the
   rules that can act on that formula in the current mode, as values the
   client can present and pass back: the rule, and for rules that need
   more the choices to make (`⊗` and Mix need the split of the context;
   `⊕` a side; contraction nothing; a `!` promotion needs the context to
   be `?` formulas). Selecting the principal formula by position mirrors
   Click & coLLecT; identify it internally by occurrence id and index
   among equal ids, so a client that reorders formulas for display still
   addresses the right one. Do not enumerate the splits of `⊗` for the
   client (exponential); accept the client's choice and validate it,
   and offer one helper that says whether a proposed split passes the
   count prunes of the focused engine (a cheap "this split cannot work"
   before the user tries it), reusing `search::focus::counts`, not
   duplicating it.
3. **Applying a rule.** Validates the rule against the formula's kind, the
   mode (weakening only affine, Mix only with Mix, promotion's context,
   the one-succedent condition in intuitionistic mode), records the
   inference and returns the new goals, or an error saying what the rule
   needed. Structural rules of the standard calculus (`?d`, `?c`, `?w`,
   `wk`, Mix) are rules the user applies explicitly; the dyadic
   bookkeeping of the term is not the user's business.
4. **Search from a goal.** `search` gains a public entry that decides a
   goal given as a multiset of occurrences of an existing forest rather
   than a `Sequent`'s roots. Step 7 already built the crate-private
   `search::focus::search_goal(&Forest, &[OccId], …)` (its report, "The
   API"): the asynchronous phase starts from the given occurrences with
   an empty `Θ`, `?` formulas enter `Θ` there, and it returns the node,
   the arena and the statistics rather than a `Proof`, and since step 8
   it takes the reading, so the two-sided engine is covered; wrap it,
   give the additive path a pair entry, and decide whether the net engine
   accepts subtrees as conclusions or such goals go to the focused engine
   (state it in the report). The interactive state uses it to close one goal or every open
   goal, with the usual `Options` and stop condition, and grafts the found
   proof's derivation view onto the goal. The outcome per goal is the
   three-valued one of D9.
5. **From state to proof.** When no goal is open, produce a `Proof` term
   (the translation from the standard calculus back to the dyadic term:
   `?d` becomes `Copy`, `?c` nothing, `?w` a `Weaken` or an unused `Quest`,
   and every `?` formula gets its `Quest` node where it enters the linear
   zone; the step 2 report's translation table read upwards) and run the
   checker on it; the interactive layer is not trusted more than an
   engine. Also the derivation view of a partial state, with open goals
   as leaves of a distinguished rule (`Rule::Open` or a sibling type), so
   the text renderer and the export steps can draw a proof in progress.
6. **Serialization.** serde for the state behind `serialize`: the sequent,
   the mode, the inferences and the open goals, pinned in
   `core/tests/serialize.rs` like the other formats. The JSON is what a
   web client holds between requests, so it must be stable and small.
7. **The CLI.** A minimal line-based interactive mode, `linlog prove
   --interactive` or `linlog interact`, enough to exercise every operation
   from a terminal: print the goals, list the rules for a position, apply
   one, undo, let the search close a goal or all, save and load the state,
   finish and check. Keep it small and readable; the web client is the
   real front end.
8. **Documentation**: `.claude/rules/core.md` (the state's invariants, the
   standard-to-dyadic translation), `.claude/rules/cli.md`, README (usage
   of the interactive mode, and the feature moved to built).

## Constraints

- The whole layer lives behind the cargo feature `interactive` (decision
  D14), on by default and enabled by the CLI; the engines' goal entry is
  unconditional since it is part of search.
- No `unsafe`; no second representation of sequents or rules: the
  interactive state reuses `Inference`, `Rule`, the forest and the checker.
- The engines are extended, not forked: one entry that takes a goal, with
  the roots as the default goal.
- Determinism and D11 as before; the state must be usable from wasm.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo hack check --feature-powerset -p linlog`,
`nix flake check` at the end (`jj st` first). Tests that pin: every rule
applied through the interface once, the rejections (wrong kind, mode,
promotion context, one succedent), a proof built by hand through the
interface that the checker accepts, a goal closed by the search and the
result checked, undo restoring the previous goals, and a JSON round trip of
a partial state.

## Deliverables

- Thematic jj commits ("Search from a goal instead of the roots", "Add the
  interactive proof state", "Translate derivations back to proof terms",
  "Serialize interactive states", "Add an interactive mode to the CLI", …).
- `plan/reports/09-interactive.md`: the API with one example per
  operation, the translation back to terms, decisions, deviations, open
  questions, and what the export steps need to render open goals and what
  the web front end will call.

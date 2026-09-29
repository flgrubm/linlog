# Step 9 report: interactive proving in the library

Session of 2026-09-29, from `plan/09-interactive.md`. Realises plan decision
D13 (interactive proving as a partial derivation) and the `interactive`
feature of D14.

## Outcome

A library user builds a proof step by step: `Interactive` holds a
derivation of the standard sequent calculus with open goals, over the same
occurrence forest, with the same `Inference` and `Rule` values as the
derivation view; `rules(goal, position)` says which rules act on a formula,
`apply(goal, position, rule, left)` applies one and returns the goals it
opens or a `Refusal` saying what the rule needed; `undo` retracts the last
step (a clone works too); `close(goal, …)` and `close_all` let the search
close goals, grafting the derivation of the proof found; `derivation()` is
the tree so far with open goals as `Rule::Open` leaves, which the text
renderer draws as bare sequents; `proof()` translates a finished derivation
into a `Proof` term and runs the checker on it, so the layer is trusted no
more than an engine; and the whole state has a JSON form that a client
holds between requests. The search gained `prove_goal`, which decides any
multiset of occurrences of a forest, `prove_until` being that on the
roots. The CLI gained `linlog interact`, a line-based session with the
commands `goals`, `rules`, `apply`, `undo`, `close`, `show`, `proof`,
`save`, `load`, `help` and `quit`.

Seven commits, "Search from a goal instead of the roots" to "Document
interactive proving", then this report. A fresh-context reviewer confirmed
the rule validation, the translation and undo on about 106 000 random
rule applications and 2 500 completed derivations in every mode, and found
one rule bug and several defects in reading a state back, all fixed (see
"Review"). All checks pass at the last code change: `cargo clippy
--workspace --all-targets -- --deny warnings`, `cargo test --workspace`
(100 unit tests in core with 6 ignored, 9 parse and 10 serialize
integration tests, 13 doc tests, 2 unit and 7 integration tests in the
CLI), `cargo hack check --feature-powerset -p linlog` (nine feature sets)
and `nix flake check`. No dependency changed.

## The API

Everything is re-exported from the crate root; the interactive layer is
behind the default feature `interactive`, which the CLI enables.

| item | role |
|---|---|
| `prove_goal(&forest, goal, mode, &options, stop) -> Result<Outcome, Error>` | decides a multiset of occurrences of a forest, in any order; the roots are the sequent; the proof of another goal has a root concluding the goal, which `Proof::check` rejects (it is for grafting) |
| `Error::GoalOutputs(n)`, `Error::NetGoal` | an intuitionistic goal without exactly one output; the net engine forced off the roots |
| `Interactive::new(&sequent, mode)` | the state, with the sequent as the one open goal; refuses a sequent without an intuitionistic reading and Mix in intuitionistic mode, as `prove` does |
| `forest()`, `sequent()`, `mode()`, `reading()` | what the state is over; the reading (`Some` in intuitionistic mode) gives every occurrence's side for a two-sided display |
| `inferences()`, `goals()`, `goal(id)`, `is_complete()`, `steps()` | the arena (root first, then the goals each step opened), the open goals in opening order, one goal's sequent, whether none is open, how many steps stand |
| `rules(goal, position) -> Result<Vec<Rule>, Refusal>` | the rules acting on a formula by its connective and the mode; two-sided names in intuitionistic mode |
| `apply(goal, position, rule, left) -> Result<Vec<InfId>, Refusal>` | applies a rule; `left` is the positions of the context formulas going to the left premise of a `⊗` or Mix (the formula itself goes left for Mix), empty otherwise; returns the goals opened in the rule's order of premises |
| `split_passes(goal, position, left) -> Result<bool, Refusal>` | whether a split of the `⊗` at the position (or a Mix) passes the focused engine's count prunes |
| `undo() -> Option<InfId>` | retracts the last step and returns the goal reopened |
| `close(goal, &options, stop) -> Result<Outcome, Error>`, `close_all(&options, stop) -> Result<Vec<(InfId, Outcome)>, Error>` | the search on one goal or every open goal; on `Proved` the derivation is grafted as one step |
| `derivation() -> Derivation` | the tree so far, renumbered premises first, with `Rule::Open` leaves, two-sided in intuitionistic mode |
| `proof() -> Result<Proof, Error>` | the checked term of a finished derivation; `Error::OpenGoals(n)` while goals are open, `Error::InvalidProof` if the checker objects (which it does not for derivations built through the interface) |
| `Refusal::{NoGoal, NoFormula, Rule, Mode, NotAlone, NoDual, NotQuest, Split, NoSplit, Succedents, Output}` | why a rule does not apply, naming the rule and positions; `Error::Refused` wraps it where an `Error` is returned |
| `Rule::Open`, `Rule::classical()`, `Rule: FromStr` | the open leaf; the classical rule behind a two-sided name; parsing a rule from its name or an ASCII spelling (`*`, `par`, `+1`, `-oL`, `&L1`, …) |
| serde for `Interactive` and `Rule` (`serialize`) | the JSON form below; a rule is its name |
| `Error::InconsistentState(&str)` | what deserialization refuses, with the reason |
| CLI `linlog interact SEQUENT` / `--state FILE` | the session, with the mode flags and the search options of `prove` (`--copies`, `--timeout`, `--memo-limit`, `--recursion-limit`) |

One example per operation, on `⊢ ~A, A ⊗ ~B, B` (the parse of
`A, A -o B |- B`; the goal's formulas are `0: ~A`, `1: A ⊗ ~B`, `2: B`):

```rust
let mut state = Interactive::new(&sequent, Mode::CLASSICAL)?;
let root = state.goals().next().unwrap();               // InfId 0
state.rules(root, 1)?;                                   // [Rule::Tensor]
state.split_passes(root, 1, &[2])?;                      // false: ~A, A ⊗ ~B, B split with B left fails the counts
let goals = state.apply(root, 1, Rule::Tensor, &[0])?;   // [1: ⊢ ~A, A, 2: ⊢ ~B, B]
state.apply(goals[0], 0, Rule::Ax, &[])?;                // []: closed
state.apply(goals[1], 0, Rule::Par, &[])                 // Err(Refusal::Rule { rule: Par, position: 0 })
state.undo();                                            // Some(goals[0]): ⊢ ~A, A is open again
let outcome = state.close(goals[0], &Options::default(), || false)?;  // Verdict::Proved, focus engine
state.close_all(&Options::default(), || false)?;         // [(goals[1], Proved)]
println!("{}", state.derivation());                      // the tree, no open leaf left
let proof = state.proof()?;                              // checked: Ax, Ax, Tensor
let json = serde_json::to_string(&state)?;               // the session
let back: Interactive = serde_json::from_str(&json)?;    // replayed and consistent
```

In intuitionistic mode the same sequent reads `A, A ⊸ B ⊢ B`; `rules(root,
1)` is `[Rule::ImpLeft]`, `apply(root, 1, Rule::Tensor, &[0])` is accepted
under the classical name and recorded as `⊸L`, and `apply(root, 1,
Rule::ImpLeft, &[2])` is `Refusal::Succedents(2)`, since the goal `B` would
join the antecedent's side.

The JSON of a state, pinned in `core/tests/serialize.rs`:

```json
{"sequent": …, "mode": {"intuitionistic":true,"affine":false,"mix":false},
 "inferences": [{"sequent":[0,1,4],"rule":"⊸L","principal":1,"premises":[1,2]},
                {"sequent":[0,2],"rule":"ax"},
                {"sequent":[3,4]}],
 "history": [0,1]}
```

The inferences are in the state's own order (the root first, then what
each step opened), an open goal is its sequent alone, and `history` lists
the inferences the steps closed, so that undo survives a save. Reading it
back replays every closed inference and checks that each step's subtree
sits where the step would have put it.

## The state

The arena is top-down: inference 0 concludes the sequent; a step fills in
the rule, the principal position and the premises of one open goal and
appends the goals it opens at the end. So a premise has a larger index
than its conclusion, the reverse of a `Derivation`, and `derivation()`
renumbers into postorder (its ids are not the state's; the state's ids are
what a client addresses goals by, and they are stable until undone). The
inferences a step added are a suffix of the arena until a later step
exists, so `undo` truncates to the smallest index in the closed goal's
subtree and reopens the goal, and the history is only the list of goals
closed. A search graft is one step: its whole subtree is the suffix.

A formula is addressed by its position in the goal's sequent, as Click &
coLLecT's `formulaPosition`; the sequent is the ascending list of
occurrence ids with repeats, so equal ids at different positions are
interchangeable and a client that reorders formulas for display maps its
order back to positions. In intuitionistic mode `Reading::position`
decides the side each formula is shown on, and the CLI prints `0: A, 1: A
⊸ B ⊢ 2: B`.

`apply` validates in `expand`, which also computes the premises: the
connective the rule acts on; the mode (`wk` only affine, Mix only with
Mix); the context (`ax` needs exactly the dual, `1` needs to be alone, `!`
needs a `?`-only context, a split names each other position at most
once); and in intuitionistic mode R1 (every premise has exactly one
occurrence in output position) and R3 (never weaken the output). R2 of the
checker (an absent output child absorbed by a `⊤` only if the premise has
no output already) is implied by R1 on the premise's actual sequent, which
the state knows. `rules` lists by connective and mode only; the context
conditions are `apply`'s, since a split can fail them anyway.

## The translation back to terms

`proof()` walks the finished tree bottom-up (`Terms`) and is the step 2
table read upwards:

| inference | term |
|---|---|
| `ax` | `Ax(x, y)` |
| `⊗`, `⅋`, `1`, `⊥`, `&`, `⊕₁`/`⊕₂`, `⊤`, `!`, `wk`, `mix` | their node |
| `?d` on `?A` | `Copy(A, …)` |
| `?c`, `?w` | nothing |
| a `?` formula entering a premise as the subformula a rule introduces (`⅋`, `&`, `⊕`, `⊗`, `!`, `?d`) | `Quest(?A, …)` below that rule, around the premise's term |
| a `?` formula among the roots | `Quest(?A, …)` below the root |

So a `?` formula is in the unrestricted zone from the point where it
enters the derivation upwards: a contraction's second instance is the same
`Θ` member, a weakened instance is simply never copied, and the dyadic
linear zone never holds a `?` formula above its entry, which is what
`Bang` needs (promotion under `?A, ?A, !B` is a `Bang` under two `Quest`s
of the same `?A`, which the checker accepts). Every `Copy` sits above its
`Quest`, so the least `Θ` at the root is empty. The term's own derivation
view may place structural rules elsewhere than the user did (it contracts
below a `⊗` where the user contracted above the root); the state's
derivation is the user's tree and the term's the checker's, and only the
root sequent has to agree. `⊤` records no context in the term, as for the
engines; the checker's `any` flag absorbs what the user's `⊤` leaf held.

## Decisions where the prompt left room

- **Open goals are inferences with `Rule::Open`**, not a sibling type: the
  arena stays one `Vec<Inference>`, the derivation view needs no second
  leaf kind, and the exporters see one `Rule` to dispatch on. The renderer
  draws an open leaf as its sequent without a bar, which distinguishes it
  from every closed leaf.
- **The goal id is `InfId`**, the index in the state's arena, rather than
  a type of its own; the derivation's renumbered ids are documented as
  different.
- **`apply` takes the split as a slice of positions** and no `Choice`
  type: `⊕` is two rules (`⊕₁`, `⊕₂`) already, so only `⊗` and Mix need
  more, and a client knows which two. `rules` returns `Vec<Rule>`; the
  CLI marks the two with "(with a split)".
- **Mix is applied at a position**: the formula there goes left with the
  further positions, so `apply` has one shape for every rule and `rules`
  can offer Mix on any formula.
- **Two-sided names are recorded**, and the classical name is accepted on
  input: a two-sided state's inferences then equal `Derivation::two_sided`'s,
  and the translation maps back with `Rule::classical`.
- **The reading is recomputed** on each intuitionistic operation (O(n))
  rather than stored: `Reading` borrows the forest the state owns.
- **`close` returns the search's `Outcome`** unchanged, its proof being
  the proof of the goal alone, rather than a new three-valued type: D9's
  verdict is already there, with the engine and the statistics.
- **`prove_goal` computes the goal's own fragment** over the subtrees, so
  a goal without exponentials inside an LL sequent gets the MLL prunes;
  the count invariants are per occurrence and work on any subtree.
- **The net engine stays with the roots.** A `ProofStructure`'s
  conclusions are the forest's roots, and accepting a set of subtrees as
  conclusions would need a structure over a sub-forest; goals off the
  roots go to the focused engine (or the additive path for two
  additive-only occurrences), and `Engine::Net` forced on one is
  `Error::NetGoal`. A sub-forest structure is a follow-up if profiles ask.
- **`?w` is nothing in the term** and the entry `Quest` goes unused,
  rather than a `Weaken` node: a `Weaken` would put the `?` instance into
  the linear zone below it, breaking the invariant that the zone holds no
  `?` formula above its entry, which `Bang` and the root rely on.
- **Deserialization replays every closed inference** through the same
  `expand` that `apply` uses, recovering a split from the left premise,
  so a loaded state is exactly as trustworthy as one built through the
  API; the checker at the end is the final word anyway.
- **The CLI exit status is about the proof**: 0 when the session ends with
  a finished proof that checks, 1 otherwise; a refused command prints
  `error: …` and the session goes on. The sequent must be an argument or
  `--file`, since standard input carries the commands.
- **`Rule: FromStr` accepts ASCII spellings** so that a terminal user types
  `*`, `par`, `+1`, `-oL`; the JSON writes the Unicode names.

## Deviations from the prompt, with reasons

- The prompt's "`?w` becomes a `Weaken` or an unused `Quest`" is realised
  as the unused `Quest` only, for the invariant above.
- No `Choice` value is passed back for `⊕`: it is two rules.
- `rules` does not filter by the context (whether the axiom's dual is
  there, whether promotion's context is `?`-only): it names what acts on
  the connective in the mode, and `apply` says what is missing. A client
  that wants a fully filtered list can try `apply` on a clone.
- No decision in `plan/README.md` turned out wrong. D13 held as written:
  the state is a derivation with open goals over the forest, the same
  representation as the view, and the term is re-derived and checked.

## Review

A fresh-context reviewer (a sub-agent that had not seen the design being
made) read the state, the checker, the derivation view and the reading,
wrote an independent model of the standard rules in a throwaway crate
outside the repository, and drove the public API with random sequents over
the connectives of each fragment in six modes (classical, affine, Mix,
affine with Mix, intuitionistic, intuitionistic affine): 106 145 `apply`
calls (37 965 accepted, 68 180 refused, every refusal with a reason and
without a change of state), compared step by step with its own rules;
2 485 derivations completed, hand rules mixed with 3 683 search grafts,
every one translated by `proof()` and accepted by the checker in its mode;
every accepted step and every graft undone and compared with a clone; and
the named corner cases (a `?c` split across `⊗` and `&`, `?w` of a root
against an introduced `?`, `?d` of `??A`, promotion under `?A, ?A`, `⊤`
absorbing in the middle and on both sides of a `⊗` or `&`, Mix with a
shared `?`, `0L` absorbing the goal at `⊸L`, `!R` under `!c`). It confirmed
the translation (its argument, now in `.claude/rules/core.md`: the linear
zone the term derives is a sub-multiset of the inference's non-`?`
formulas, which is why R1 at application time covers the checker's R2) and
undo, and found:

- **Mix on a goal that repeats the formula it is applied at dropped the
  second copy** (`⊢ ?(a ⅋ ~a)` after `?c`, `?d`, `?d`; 55 of its
  disagreements, all this): the Mix arm removed the principal from the
  right side once more. Not a soundness hole (the empty goal cannot be
  closed), fixed and pinned in `mix_keeps_repeated_formulas`.
- **Reading a state back over-rejected and could panic**, never
  under-rejected: the history check compared each step's *current*
  subtree with the suffix (any two steps on one branch failed, with an
  underflow in debug builds); a Mix could not be replayed (no principal,
  and the Mix formula stayed in the context); a search graft appended its
  derivation premises first, which the shape check refuses; a `⊗` without
  a recorded principal passed and made `proof()` panic; a Mix with an
  empty conclusion indexed an empty vector; a history naming one step
  twice was accepted. All fixed: the check now takes, from the last step
  back, the inferences of the step's subtree that exist at that point;
  `replay` requires a principal exactly for the rules that have one and
  reads a Mix's split from its left premise; `graft` appends in reverse so
  premises keep larger indices; history entries must be distinct. The
  round trip of a state with a chain of steps, a Mix, a graft and a
  repeated formula is pinned in `core/tests/serialize.rs`, with the
  rejections.

The reviewer also noted that the module documentation claimed the premise
order that `graft` had broken; it holds again.

## Open questions and follow-ups

- **Goals and the net engine.** A `ProofStructure` over a sub-forest (the
  goal's subtrees as conclusions) would let the net engine close MLL goals
  off the roots; today they go to the focused engine, which is fine for
  goals a human opens.
- **`Reading` recomputed per operation** is O(n) per `apply` in
  intuitionistic mode; a client with thousands of occurrences and many
  steps may want the positions stored in the state (a `Box<[Position]>`
  next to the forest). Not done, since no profile asks.
- **The history covers only steps taken through the API**; a hand-written
  JSON with closed inferences and an empty history loads but has nothing
  to undo, by design.
- **`close_all` runs the goals in order with one stop closure**; a client
  wanting a budget per goal wraps the closure itself.
- **Nullary Mix** is still not a rule; an empty goal (`⊢`) can be opened
  by a Mix split that takes everything to one side and cannot be closed.

## For steps 10 to 12 and the web front end

- **Rendering open goals**: `Derivation` from `Interactive::derivation`
  has inferences with `rule == Rule::Open`, no principal and no premises;
  the text renderer draws them as the sequent alone. LaTeX and Typst
  exporters should emit a hypothesis leaf (`\hypo` in ebproof, as Click &
  coLLecT's `Hypothesis_proof`), the SVG a sequent without a bar, and the
  Rocq certificate should refuse a derivation with open goals (or emit an
  admit). `Rule::classical` gives the one-sided rule behind a two-sided
  name, which the Yalla mapping wants.
- **What the web front end will call**: `Interactive::new` or
  `serde_json::from_str` on the held state; `goals`/`goal`/`reading` to
  draw the goals (positions are the click targets, `Reading::position`
  the side); `rules` for the menu, with the two split rules needing a
  context selection; `apply` with the selection; `split_passes` to grey
  out a selection before it is tried; `undo`; `close` with a stop closure
  on a node budget (wasm has no clock in core; the closure counts);
  `derivation` to draw the tree (through step 11's SVG); `proof` at the
  end, then the exports; and `serde_json::to_string(&state)` to hold it.
  Everything is `Send`-free, allocation-light and clock-free, as D11 asks.
- **The checker's view of a finished interactive proof** may differ from
  the user's tree in where `?c`/`?w` sit; an exporter of the *user's*
  derivation exports `Interactive::derivation`, and one of the *term's*
  derivation exports `proof.derivation()`; certificates want the term.

## Verification

At the last code change: `cargo clippy --workspace --all-targets -- --deny
warnings`, `cargo test --workspace` (the counts above), `cargo hack check
--feature-powerset -p linlog`, rustdoc with warnings denied, and `nix flake
check` all pass; each of the seven commits was built and its tests run on
its own (the two amended after the review in a temporary jj workspace).
The CLI was exercised by hand in both modes, and its scripted session is
pinned in `cli/tests/cli.rs`. No dependency changed, so `cargo deny check`
was not run.

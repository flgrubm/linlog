# Step 8 report: intuitionistic mode and the additive fast path

Session of 2026-09-29, from `plan/08-intuitionistic.md`. Realises plan
decision D1's intuitionistic reading of the one-sided arena, the two-sided
engine of D7 and the intuitionistic and additive rows of D8.

## Outcome

`Mode::intuitionistic` works end to end. A one-sided sequent is read as an
intuitionistic one by the polarization of its subformula occurrences
(`Reading`, `Position`), which names the goal, recovers `⊸` from `~A ⅋ B`
and prints `Γ ⊢ A`; the dispatch sends unit-free IMLL through the embedding
into the net engine, two additive-only formulas to the new additive path and
every other intuitionistic fragment to the two-sided engine, which is the
classical focused engine with one added constraint; the checker tests the
one-succedent condition on every derived sequent instead of refusing the
mode; the derivation view shows two-sided sequents with the ILL rule names;
and the CLI reaches all of it through `-i`, `--engine two-sided|additive`,
`seq print -i` and `seq fragment -i`, with `check -i` no longer refused.
A fresh-context reviewer compared the two-sided engine, the checker and the
embedding with an independent unfocused two-sided prover on about 60 000
random sequents over every ILL connective, linear and affine, with no
disagreement, and found one bug in the additive path (fixed, see "Review").

Nine commits, "Recognise intuitionistic sequents by shape" to "Add
ILLTP-style problems as a slow test", then this report. All checks pass at
the last code change: `cargo clippy --workspace --all-targets -- --deny
warnings`, `cargo test --workspace` (91 unit tests in core with 6 ignored,
9 parse and 9 serialize integration tests, 11 doc tests, 2 unit and 6
integration tests in the CLI), `cargo hack check --feature-powerset -p
linlog`, and `nix flake check` (see "Verification"). No dependency changed.

## The API

Everything is re-exported from the crate root.

| item | role |
|---|---|
| `Reading::new(&forest) -> Result<Reading, ShapeError>` | the intuitionistic reading: `position(o)`, `goal()`, `hypotheses()`, `implication(o)` (antecedent, consequent), `outputs(ids)`, `formula(o)` (an `IllFormula` that prints with `⊸`, `1`, `⊤`, `0`), `Display` as `Γ ⊢ A` |
| `Position::{Input, Output}` | Lamarche's polarity of an occurrence: hypothesis side or goal side |
| `ShapeError::{NoGoal, SeveralGoals(a, b), Formula(o)}`, `describe(&forest)` | why a sequent has no reading, with ids or with formulas |
| `Fragment::name_in(mode)` | `IMLL`, `IMLL with units`, `IALL`, `IMALL`, `IMELL`, `ILL` in intuitionistic mode, the classical name otherwise; the JSON of an `Outcome` uses it, and a `Fragment` reads back from either spelling |
| `Proof::check(Mode::INTUITIONISTIC…)` | the classical check plus the one-succedent condition; `Problem::Shape(ShapeError)` for a sequent without a reading, `Problem::Succedents(n)` for a sequent of the proof with `n ≠ 1` goals; `Problem::Intuitionistic` is gone |
| `Proof::two_sided_derivation()`, `Derivation::two_sided(&proof)`, `Derivation::reading()` | the two-sided view; `Rule` gains `ImpLeft … BangWeakening` (`⊸L ⊸R ⊗L ⊗R &L₁ &L₂ &R ⊕L ⊕R₁ ⊕R₂ 1L 1R 0L ⊤R !L !R !c !w`) and `Rule::intuitionistic(position)` maps a classical rule to them |
| `Engine::TwoSided`, `Engine::Additive` | names `two-sided` and `additive` in text and JSON |
| `Error::NotIntuitionistic(ShapeError)`, `Error::IntuitionisticMix`, `Error::EngineMode { engine, mode }`, `Error::NotAdditive { fragment, roots }` | the new refusals of the front door |
| `search::focus::search(&forest, fragment, mode, reading: Option<&Reading>, …)`, `search_goal(…, reading, …)` | crate-private: the focused engine takes the reading |
| `search::additive::search(&forest, mode, &options, stop)` | crate-private: the additive path |
| CLI | `-i` everywhere, `--engine two-sided`, `--engine additive`, `seq print -i`, `seq fragment -i`, `--stats` for the additive engine, `--format net -i` |

## The shape rules, as implemented

An occurrence in *output position* (the goal, or the antecedent of a
hypothesis) is an ILL formula: `⊗ ⊕ & ! 1 ⊤ 0`, an atom `a`, or `A ⊸ B`
stored as `A⊥ ⅋ B`. In *input position* (a hypothesis, or the antecedent of
the goal) it is the negation of one: `⅋ & ⊕ ? ⊥ 0 ⊤`, `~a`, or `A ⊗ B⊥` for
a hypothesis `A ⊸ B`. The position flips at the antecedent of an implication
and nowhere else. A bottom-up pass computes which positions every occurrence
can take (`Var`, `1`, `!` output only; `~a`, `⊥`, `?` input only; `⊤` and
`0` both; `&`/`⊕` what both children can; `⊗` output when both children are
output, input when one is output and the other input; `⅋` the other way
round); the first occurrence with neither, in descending id order, is
`ShapeError::Formula`, a minimal offending subformula, and the CLI message
names it with the root it lies in. An intuitionistic sequent has exactly one
output-shaped root: a root that can only be output is the goal (two of them
is `SeveralGoals`, none that can be output is `NoGoal`); otherwise the last
root by id that can be output. Then a top-down pass assigns positions.

**The symmetric reading.** An output `⅋` whose left factor cannot be input
but whose right one can is read with the right factor as antecedent, so
`b ⅋ ~a` is `a ⊸ b` and, dually, the hypothesis `~b ⊗ a` is the same
formula; the left factor is the antecedent whenever that reading works,
which is always the case for parser output. `Reading::implication` returns
the antecedent first, and the printer, the engine and the derivation view
use it, so nothing else needs to know which factor flipped.

**Ambiguity.** Only formulas built from `⊤` and `0` alone can stand on
either side, and for those the written succedent is not recoverable: the
arena keeps its roots sorted by term and hash-conses equal subterms, so a
`⊤`-built succedent that equals a hypothesis subterm gets an early id and
another root becomes the goal (`0, ⊤ ⊢ ⊤` prints as `0, 0 ⊢ 0`; `top |-
top * top` prints as `⊤ ⊢ ⊤ ⊗ ⊤`, and `|- top, top * top` as `0 ⊢ ⊤ ⊗ ⊤`).
The reviewer brute-forced 607 464 such sequents and found no pair of
readings that differ in provability, so the verdict is unaffected; only the
two-sided print and derivation can show the other reading. Engine, checker
and view all compute the reading from the same forest, which keeps them
consistent. Recovering the written side would need the arena to remember
the parser's root order, a question for the planning session (below).

## How the two-sided engine shares the classical one

The two-sided engine is the classical focused engine handed a `Reading`
(`Engine::TwoSided` names that configuration; D7's question is answered
with "a parameter", not a sibling module). Every rule of the spec's
two-sided focused calculus is a rule of the one-sided engine on the lowered
sequent: `⊸R` and `⊗L` are the `⅋` of the asynchronous phase, `&R` and `⊕L`
its `&`, `1L` its `⊥`, `⊤R` and `0L` its `⊤`, `!L` is `quest` plus a copy,
`⊗R`, `⊕R`, `1R`, `!R` and the initial rules are `focus` on the goal, `⊸L`
and `&L` are `focus` on a hypothesis. Starting from a sequent with one
output, every rule keeps exactly one output on each premise by itself,
except the split of a hypothesis `A ⊸ B` (a `⊗` in input position), where
the goal must go with the consequent `B⊥`. So the one change is in `split`:
in the free enumeration the unique output member of `Γ` is fixed on the
consequent's side (which `Reading::implication` names) and left out of the
submask enumeration. The forced splits need no change, since the dual of an
output positive literal is a hypothesis in `Γ` or `Θ`, the dual of an input
positive literal is the goal itself or nothing (`Θ` holds only input
occurrences, the subformulas of `?`), `1` and `!` are output-only and take
the empty side, and a `0` factor fails. Affine leaves never see an output
among the leftovers (debug-asserted in `weakened`), promotion needs `Γ`
empty as before, the count prunes are necessary conditions on the lowered
sequent and hence sound, and the memo, the copy budget, the loop check, the
stack and the pools are indifferent to positions, since the key `(Θ, Γ)`
determines the two-sided sequent. Mix is refused before the engine runs
(`Error::IntuitionisticMix`: a premise of a Mix would have no goal).

Completeness rests on the two-sided focused calculus being complete for ILL
and every one of its derivations being enumerated; the reviewer argued it
against the code and tested it (below). On the counter program of step 7's
report in two-sided form (`!(a ⊗ a ⊸ b), !(b ⊗ b ⊸ c), !(c ⊗ c ⊸ d), a^8 ⊢
d`) the constraint pays: 21 ms and 219 000 stable sequents in release mode
where the classical search needs 126 ms and 1.76 million (the ignored test
`illtp_style_slow`).

## IMLL by embedding

In intuitionistic mode the dispatch sends unit-free IMLL with no literal
more than twice to the net engine on the one-sided sequent unchanged, and
returns its proof as it is. The argument that no essential-net condition is
needed for the verdict: an MLL⁻ sequent whose roots are all input-shaped is
unprovable, since every leaf (`ax`) has an output-shaped literal and every
rule of a cut-free proof keeps at least one output on some premise, so in a
cut-free proof of a one-output sequent the split of a hypothesis `A ⊸ B`
can never leave the `B⊥` side without the goal; every other rule keeps one
output on each premise trivially. Hence every sequent of every cut-free MLL⁻
proof of an IMLL⁻ sequent has exactly one output, every sequentialization of
a classical net of it passes the intuitionistic checker (`R1` below; `R2`
and `R3` are vacuous without `⊤`, weakening and Mix), and conservativity
gives the other direction. With `1` the lowered sequent has units and the
dispatch goes two-sided. `--format net -i` prints the net of the one-sided
sequent. The test `embedding_agrees_with_the_two_sided_engine` compares the
two engines on generated IMLL⁻ sequents and their mutants where the dispatch
picks the net engine (with repeated literals the net engine is exponential
and the dispatch keeps it off them; forcing it there is what stalled a first
version of the test).

## The checker and the derivation view

Step 2's table holds: every ILL rule is a classical node, so the term type is
unchanged. In intuitionistic mode `check` first reads the sequent
(`Problem::Shape` if it has no reading) and then, in the same bottom-up
pass, tests three conditions against the positions: **R1**, every derived
linear zone holds at most one occurrence in output position, and exactly one
unless `any` (a `⊤` above supplies the goal); **R2**, in `take`, an absent
child in output position may be absorbed by `any` only if the premise's zone
has no output already (else the premise's actual sequent would have two
goals); **R3**, `Weaken` never weakens an output; and Mix is `Forbidden`.
The conditions are exactly "some top-down instantiation of the absorbed
contexts is an ILL derivation": at a `⊗` the fixed output counts of the two
premises sum to two, so the side with none must absorb, which R1 guarantees,
and the derivation view builds that instantiation, sending the goal among
the absorbed formulas to the premise that has none (the classical rule gives
everything to the left one). The step 2 example `⊢ ⊤, ⊤ ⊗ ⊤` is now read as
`0 ⊢ ⊤ ⊗ ⊤` and its term accepted with the hypothesis `0` absorbed by one
`⊤R`. The classical proof of Schellinx's kind of sequent (below) is rejected
at the `⅋` whose premise would have two goals, with the message "a sequent
of the rule has 2 formulas on the right of ⊢ instead of one".

`Derivation::two_sided` checks the proof in intuitionistic affine mode (so
`wk` shows where used) and unfolds the same tree with the ILL rule names by
the position of the principal formula; the inferences carry the same
occurrence ids, only the names and the rendering differ, so the exporters
of steps 10 to 12 see one view type and use `Derivation::reading` to print
`Γ ⊢ A`. The renderer prints hypotheses in id order.

## The additive fast path

`search::additive` decides a sequent of exactly two additive-only formulas
by a recursion on pairs of subformula occurrences: `⊤` closes, `&` on either
side needs both subformulas against the other, two dual literals are an
axiom, `⊕` on either side tries one subformula at a time, nothing else
proves anything. `&` is invertible and goes first; which `⊕` to decompose is
a real choice, since a `&` below the other formula's `⊕` may need both sides
of this one (`⊢ ~c ⊕ ~a, b ⊕ (c & a)`), so both formulas' `⊕` are tried and a
memo on the pair bounds the work by `|A|·|B|`. The procedure is the same in
every mode: additive rules keep one output by themselves, and neither
weakening nor Mix can help a two-formula sequent (a proof of one formula
alone ends in `⊤` leaves, which absorb the other). The dispatch row takes
two roots in the additive fragment with at least one additive connective
(atoms alone stay with the net engine); forcing `--engine additive` on any
two additive-only formulas works, on anything else it is
`Error::NotAdditive`. `--stats` prints pairs visited, memo hits and entries.

## Decisions where the prompt left room

- **The reading is a value over the forest** (`Reading<'a>` borrows it),
  computed where needed in O(n), not stored in the forest: the forest is
  mode-independent, and step 9 can query `position(o)` on the reading of
  any goal's forest.
- **The goal among ambiguous roots is the last root by id.** The parser
  numbers the succedent after the hypotheses, so this is the written
  succedent unless hash-consing moved it; the alternatives (first root, or
  a preference by connective) are wrong at least as often and harder to
  state. Documented in the rules file with the example.
- **The symmetric reading is accepted** rather than refusing `b ⅋ ~a`: it
  is the same formula up to commutativity, costs two bits per occurrence
  in the bottom-up pass, and keeps the reading a semantic property of the
  arena rather than of the parser's output order.
- **`IALL`** names the additive-only intuitionistic fragment, by analogy
  with the classical `ALL`, so that the verdict line shows the fragment
  that picked the additive engine; the prompt lists only the four standard
  names.
- **`Engine::TwoSided` and `Engine::Additive`** are the variants the plan
  names; `two-sided` is a configuration of the focused engine, not a
  second implementation, and forcing `--engine focus` in intuitionistic
  mode or `--engine two-sided` in classical mode is `Error::EngineMode`,
  since a classical proof may fail the intuitionistic checker.
- **Mix in intuitionistic mode is an error of its own**
  (`Error::IntuitionisticMix`), not silently ignored: a Mix premise has no
  goal, and the checker rejects Mix nodes in the mode for the same reason.
- **The checker's conditions are bottom-up** (R1 to R3), keeping the
  checker linear and iterative; the alternative, checking the unfolded
  derivation, would be exponential on proofs with shared subproofs.
- **The one-succedent error names a count**, `Problem::Succedents(n)`, for
  all three conditions rather than three variants; the node and its
  premises in the error say where.
- **`Derivation::two_sided` is a second constructor**, and
  `Proof::derivation()` stays the classical view, so nothing changes for
  classical callers; the CLI picks by mode.
- **The ILL generator** (`generate::ill`) builds two-sided proofs bottom-up
  in ILL syntax with a `zero` switch separate from `additives`, so that
  the differential test against the classical engine can ask for agreement
  exactly where conservativity holds (without `0`) and count the
  non-conservative mutants where it does not.
- **The embedding test compares only where the dispatch picks the net
  engine** (no literal more than twice), for the reason above.

## Deviations from the spec or the prompt, with reasons

- **The two-sided printing landed in the reading's commit**, not in a
  commit of its own: the printer is a method of `Reading` and the property
  test (parse, print, parse again) belongs to it. The round trip is checked
  on the two-sided print, since the symmetric reading legitimately
  re-parses `a ⊸ 0` for an arena that held `0 ⅋ ~a`.
- **The spec's "memoize on subformula pairs" is what the additive path
  does, after a detour**: a first version reasoned that a fixed rule order
  makes every pair reachable once and dropped the memo, which was wrong
  because the `⊕` choice is real; the reviewer's reproducers are pinned in
  `additive_sequents`.
- **The spec's IMALL/IMELL data layout** ("hypothesis bitset, goal as a
  `u32`") is not used: the goal is a member of `Γ` like any other and the
  memo key stays `(Θ, Γ)`, which determines the two-sided sequent since
  positions are fixed per occurrence. Nothing is duplicated.
- The spec's "affine ILL: the same with the supermultiset prune, which
  decides" is subject to step 7's erratum: affine intuitionistic mode is the
  bounded search with weakening at the leaves.
- No decision in `plan/README.md` turned out wrong. D1 held: no second
  representation, and the ambiguity it leaves is documented.

## Review

A fresh-context reviewer (a sub-agent that had not seen the design being
made) read the spec's intuitionistic section, the reading, the checker, the
engine and the dispatch, argued each claim against the code, and wrote an
independent unfocused two-sided ILL prover in a throwaway crate outside the
repository (every rule of ILL, `⊸L` over every split, a bound on
derelictions per branch, affine variant, memo on the sequent), parsing the
reading's own two-sided print so that both sides decide the same sequent. It
compared verdicts with `prove` on 20 000 random full-ILL sequents over two
and three atoms in linear and affine mode, 10 000 with the two-sided engine
forced, 20 000 IMALL sequents with units, 6 000 larger ones, 4 000 IMLL⁻
sequents across the net and the two-sided engine, 8 000 classical proofs of
ILL-shaped sequents against the checker's acceptance, and 6 000 additive-only
sequents against the additive path; every proof was checked in
intuitionistic mode and unfolded two-sided with one output per inference.
No disagreement except the additive path's missing `⊕` choice (18 wrong
refutations of 6 000, fixed and pinned). It confirmed the claims: the
reading is total and deterministic and nothing depends on it being the
written one; R1 to R3 are exactly the existence of an ILL instantiation and
the view builds it; the single split constraint suffices, with the forced
splits, `Θ`, the affine leaves, promotion and the count prunes argued one by
one; the embedding's lemma; and it brute-forced the reading ambiguity
(607 464 cases, no provability difference). It also noted a harmless quirk
of the classical affine engine: `focus_on` on a positive literal may close
with an unrelated dual pair and weaken the focus literal, which is sound
and cannot happen two-sided.

## Open questions and follow-ups

- **The written succedent.** Recovering it for `⊤`/`0`-built formulas needs
  the arena to keep the parser's root order or the count of right-hand
  roots, which touches `Sequent`'s canonical form and JSON; the planning
  session should decide whether the display is worth a format change.
- **Intuitionistic goals for step 9.** `search_goal` takes the reading; an
  open goal with exactly one output occurrence is what it expects, and
  `Reading::position` is the query a client uses to show it two-sided.
- **Essential nets (step 15).** The reading gives every occurrence its
  polarity and every implication its antecedent, which is the orientation
  Lamarche's structures need; `Outcome::net` on an IMLL⁻ sequent is a
  classical net whose every sequentialization is intuitionistic, so an
  essential-net engine is a performance alternative, not a correctness
  need.
- **The additive path on more than two roots** (a set of additive formulas
  is decided by the focused engine today) and on `Θ` (a `!` of an additive
  formula) were not asked for.
- **`--copies` in the two-sided counter program** is spent the same way as
  classically; the interchangeable-hypotheses follow-up of step 7 applies
  two-sided as well.

## For steps 9 to 12 and 15

- Step 9: a goal is a multiset of occurrences with, in intuitionistic
  mode, exactly one in output position; `search_goal(forest, goal,
  fragment, mode, reading, options, stop)` searches it, and
  `Reading::position` decides on which side a client draws each formula.
  The two-sided derivation is `Derivation::two_sided`, whose inferences
  have the same shape as the classical ones.
- Steps 10 and 11: `Derivation::reading()` is `Some` for a two-sided
  derivation; print an inference's sequent as the hypotheses (input
  positions, id order), `⊢`, the goal, with `Reading::formula` for the ILL
  spelling, as `proofs/fmt.rs` does; `Rule::name` gives the ILL names.
- Step 12: Yalla's `ill` takes two-sided sequents with the ILL rules; the
  mapping from a classical node to an ILL rule is `Rule::intuitionistic`
  by the principal's position, and `⊸L`'s split is the node's premises.
- Step 15: the reading is the polarization; an essential-net engine would
  add the dominator condition to the net engine's `complete` branch and
  could drop the sequentialization's detour.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`, `cargo test
--workspace`, `cargo hack check --feature-powerset -p linlog` and `nix flake
check` all pass at the last code change, on top of which this report sits;
the timings above are from release builds of the ignored tests. No
dependency changed, so `cargo deny check` was not run.

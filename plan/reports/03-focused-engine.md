# Step 3 report: the focused engine for MLL, MLL with units and MALL

Session of 2026-09-29, from `plan/03-focused-engine.md`. Realises plan
decision D7 for the classical focused engine, the first rows of D8 and the
outcome shape of D9.

## What was built

Six changes, in order (`jj log`), then this one with the documentation:

1. **Add interval counts per occurrence.** `search/focus/counts.rs`: per
   occurrence a sparse row of `(atom, lo, hi)` intervals, an `absorbs` flag
   for a `⊤` at or below it, and the weight `t − p − #1 + #⊥` of its
   subtree; a `Tally` that sums members incrementally and answers the
   interval check, the `MLL` count equation and whether a Mix is admissible.
2. **Add the focused engine for MLL and MALL.** `search/focus/mod.rs` and
   `search/focus/memo.rs`: the MALL-Seq engine of the spec with units and
   Mix as rule switches, the memo of stable sequents with a cap, the
   recursion limit and the stop condition; `Options`, `Verdict`, `Reason`
   and `Statistics` in `search/mod.rs`.
3. **Add the search front door.** `prove`, `prove_until`, `Outcome`,
   `Engine`, the dispatch of D8 and the errors `NoEngine` and
   `FragmentMismatch`; re-exports from the crate root.
4. **Let a `⊤` below a member save a stable sequent with a `0`.** A
   completeness bug the generator found on its first run; see "Deviations".
5. **Generate random proofs for testing.** `search/generate.rs`, test-only.
6. **Add a 3-Partition Horn encoding as a slow test.**

All checks pass at the last change: `cargo clippy --workspace --all-targets
-- --deny warnings`, `cargo test --workspace` (52 unit tests in core, two of
them ignored, 9 parse and 6 serialize integration tests, 6 doc tests),
`cargo test -p linlog --no-default-features`, `cargo hack check
--feature-powerset -p linlog`, rustdoc with `--deny warnings`, and `nix
flake check`. No dependency changed.

## The public API

Everything is re-exported from the crate root; the module is `linlog::search`.

| item | role |
|---|---|
| `prove(&Sequent, Mode, &Options) -> Result<Outcome, Error>` | decides a sequent with the engine its fragment calls for |
| `prove_until(&Sequent, Mode, &Options, impl FnMut() -> bool)` | the same, polling the closure once per stable sequent and stopping with `Reason::Stopped` when it returns true |
| `Options` | private fields, `Default`, setters `memo_limit(usize)` (default 2²⁰, zero switches the memo off), `recursion_limit(u32)` (default 2048), `engine(Option<Engine>)`, `fragment(Option<Fragment>)` |
| `Outcome { verdict, fragment, engine, statistics }` | `#[non_exhaustive]` struct, public fields |
| `Verdict` | `Proved(Box<Proof>)`, `Unprovable`, `Unknown(Reason)`; `proof()` returns the proof if any |
| `Reason` | `#[non_exhaustive]`: `Stopped`, `RecursionLimit`, `ContextTooWide(usize)`; `Display` gives a phrase |
| `Statistics` | `#[non_exhaustive]`, public fields `nodes` (stable sequents visited, memo hits included), `memo_hits`, `memo_entries` (the peak), `splits` (context splits examined for `⊗` and Mix) |
| `Engine` | `#[non_exhaustive]`: `Focus`; `Display` prints `focus` |
| `Error::NoEngine { fragment, mode }` | "no engine for MELL in classical mode yet" |
| `Error::FragmentMismatch { asserted, detected }` | the sequent lies outside the fragment `Options::fragment` asserts |

`search::focus::search(&Forest, Fragment, Mode, &Options, &mut dyn FnMut()
-> bool) -> (Verdict, Statistics)` is the crate-private engine entry the
front door calls; step 7 extends it, step 12 wraps it.

## What the dispatch does

`prove_until` detects the fragment (or takes the asserted one, refusing a
sequent outside it), then applies D8 as far as it exists: intuitionistic
mode, affine mode and any fragment with exponentials are
`Error::NoEngine`; everything else, from atom-only sequents to MALL with
units, with or without Mix, goes to the focused engine. The outcome reports
the fragment searched in and `Engine::Focus`. A fragment asserted larger
than the detected one runs the search with the prunes of the larger
fragment, so `--fragment mall` on an MLL input switches off the count
equation; smaller is refused.

Step 4 wires this as: `prove_until(&s, mode, &options, || Instant::now() >=
deadline)` for `--timeout`, an `AtomicBool` for Ctrl-C, and a thread whose
stack is `recursion_limit × 2 KiB` (debug) or `× 512 B` (release) when
`--recursion-limit` or the input size asks for more than the default.

## The engine, in the spec's terms

`asynchronous` is the phase `⊢ Γ ⇑ L` with `L` a stack: `⅋` pushes both
sides, `⊥` is dropped, `⊤` closes with a `Top` node (the `⅋` and `⊥` nodes
pending on that branch are wrapped around it), `&` runs the left premise on
copies of the state and the right one on the state itself; anything else
moves into `Γ`. When `L` is empty, `Γ` is stable and `prove` takes over:
the memo, then the immediate tests (a `0` with no absorbing member, a dual
pair, the interval check, the count equation where it holds, and without
Mix a literal-only sequent), then `focus` on each candidate in order
(`⊗` with a forced split, `⊕`, `⊗` with an enumerated split, by id), then
Mix. `focus` is `⊢ Γ ⇓ F`: `⊕` tries both sides, `⊗` calls `split`, `1`
needs `Γ = ∅`, `0` fails, a positive literal needs `Γ = {p⊥}`, and a
negative `F` is released into `asynchronous`. `split` handles the forced
cases (a positive literal factor takes its first dual occurrence, `1` the
empty context, `0` fails) and otherwise enumerates the submasks of the
compacted members in Gray-code order with two tallies moved per flip; both
sides must pass the counts before either premise is searched. `mix` fixes
the first member on the left and enumerates the rest, skipping the trivial
partition.

Every proved stable sequent is a `Proved(NodeId)` in the memo and a hit
reuses the subproof, so the arena is a DAG; `Proof::new` at the end drops
the failed branches. The proof is checked in a `debug_assert!` and in every
test.

## Performance observations

- The generator sample in release mode (`cargo test -p linlog --release
  generated_large_sample -- --ignored --nocapture`): 4000 generated sequents
  of up to 24 rules over 3 atoms, in all eight combinations of units,
  additives and Mix, proved with checked proofs, and 3340 mutants decided
  the same way with and without the memo (773 of them provable), in 2.0 s;
  the largest search visited 6435 stable sequents.
- A larger sample (budget 30) ran a total of 21 s of which 21 s was one
  unprovable mutant with 17 top-level formulas under Mix: 129 million
  stable sequents, 714 million splits, about 6 million stable sequents per
  second. Refuting a wide sequent with Mix costs about `3^k` for `k`
  members (every subset is decided once, and each enumerates its own
  partitions), which the memo cannot reduce; the other 2914 sequents took
  170 ms together.
- The 3-Partition encoding (bins offer units and slots, an item is a `&`
  over the bins of a Horn clause, the goal is the tensor of the outputs):
  solvable instances are proved at once (25 stable sequents, 42 thousand
  splits for six items and two bins of size 6); refuting an unsolvable
  instance with bins of size 4 takes 55 s in release mode (1.8 million
  stable sequents, 3.9 billion splits), and one with bins of size 6 did
  not finish in five minutes. The cause is the atom bias: the clause
  bodies' atoms occur more often in bodies than as hypotheses, so the
  forest makes the hypotheses positive and the bodies negative, and every
  clause's `⊗` split is enumerated instead of forced. The solvable
  instance is a normal test; the refutation is ignored.
- Stack: one engine level (a `prove`, `focus` or `asynchronous` call) costs
  under 2 KiB in debug builds and under 512 bytes in release, measured by
  running chains of 1000 to 4000 nested `⊗`, `&` and `⊕` on threads of
  known stack size. The default recursion limit of 2048 therefore fits an
  8 MiB main-thread stack in both profiles. The depth never exceeds three
  levels per occurrence.
- Nothing allocates per node once the pools are warm, except the memo
  insert, which clones its key.

## Decisions where the prompt left room

- **Recursion with a counted depth, not an explicit stack.** The engine's
  functions are the spec's rules and read as such; an explicit stack would
  turn `split`'s enumeration, `&`'s branching and `⊕`'s choice into frame
  types. `Options::recursion_limit` bounds the nesting and the search
  answers `Reason::RecursionLimit`; the CLI (step 4) sizes a thread from it.
- **`Outcome` is a struct, `Verdict` the three values.** The prompt's
  "`Outcome` (D9) with `Statistics`" reads as an outcome that carries the
  statistics, and step 4 wants the fragment and the engine next to the
  verdict for its output line. `Proved` boxes the proof because a proof
  carries a forest (clippy's `large_enum_variant`).
- **`Mode` by value.** The prompt writes `prove(&Sequent, &Mode,
  &Options)`; `Mode` is a three-byte `Copy` type that `Proof::check` takes
  by value, so `prove` does too.
- **The stop condition is a parameter, not a field.** `Options` stays a
  plain value (`Clone`, `Debug`, `PartialEq`), which the CLI can print and
  a later wire format can carry; `prove_until` takes the closure, and
  `prove` is `prove_until` with `|| false`. It is polled once per stable
  sequent, the spec's cadence for a stop flag, so step 12's flag is the
  same closure.
- **Which knobs exist now.** `memo_limit`, `recursion_limit`, `engine` and
  `fragment` have a meaning today; the copy bound, thread count and
  determinism switch do not, and would be dead fields. `Options` has
  private fields and setters, so adding them later breaks nothing.
- **Memo eviction is a clear.** "Evict arbitrarily" with the least
  machinery: when the table reaches the cap it is emptied. Entries are
  facts the search finds again; the arena is append-only, so a `Proved`
  node id never dangles. `memo_limit(0)` is the memo-free engine the
  differential tests compare against.
- **The hull for `&`**, as the spec writes it. The intersection of the two
  sides' intervals would also be sound (both premises must balance the same
  context) and prunes more, at the cost of an "empty interval" state; the
  spec's form was kept, and the stronger one is listed below.
- **A context of more than 63 members is `Reason::ContextTooWide`** for the
  whole search: the honest answer, never a silent failure, and cheap to
  detect. The spec's lazy contexts are step 14.
- **Mix in the multiplicative fragments only when `c > weight + 2`**, the
  prompt's "only when the count equation admits it"; in MALL the interval
  check on each side is the only prune.
- **Candidate order**: forced `⊗` first (their split is one call), then
  `⊕` (two continuations), then free `⊗` (up to `2^k`), ids ascending in
  each class. Deterministic by construction, and the memo is only ever
  looked up, never iterated.
- **The generator carries its own SplitMix64.** A dependency for five lines
  of test code would not earn its place (D10). It builds proofs bottom-up as
  the spec's testing strategy says and keeps only the conclusion as text
  for the parser; `&` gets a "twin" of the other side (the same formula, `⊤`,
  a `⊕` with anything, `A ⅋ ⊥`, `1 ⊗ A`, nested) so both premises share
  the context, and `⊤` gets random junk formulas as its context.
- **The 3-Partition encoding is Horn with `&`**, that is MALL, because the
  bin choice needs a choice and the pure `⊗`/`⊸` Horn fragment cannot
  express "one of these clauses" without further machinery; Kanovich's
  Horn programs with `&`-choice are the model.

## Deviations from the spec or the prompt, with reasons

- **A `0` in a stable sequent is not fatal when a member has a `⊤` below
  it.** The spec's pitfall "0 in a stable sequent is fatal" is wrong:
  `⊢ 0, ⊤ ⊕ b` is provable by focusing on the `⊕` and letting the `⊤`
  absorb the `0`, and so is `⊢ 0 ⊗ a, ~a, ⊤ ⊕ b`. The generator produced
  such a sequent on its first run. The immediate failure now requires that
  no member absorbs; focusing on `0` itself still fails, as the focused
  calculus says, and completeness follows from the other decisions.
- **`⊤` in the interval counts.** The spec gives units the row `(0, 0)`,
  which would prune `⊢ ⊤, a`. A `⊤` anywhere below a member makes the
  member absorb, and any set with an absorbing member passes; `0` keeps
  `(0, 0)`, which is sound and weaker than the truth.
- **A literal-only stable sequent fails only without Mix.** The spec's
  immediate failure "no positive non-literal formula and not a dual pair"
  is wrong with Mix (`⊢ a, ~a, b, ~b` is provable); the engine applies it
  only when Mix is off and otherwise lets the Mix step partition the
  literals.
- **Forced splits only for a positive literal, `1` and `0`.** The prompt
  lists `⊥` and `⊤` as forcing too, but neither does: `⊢ ⊥ ⊗ b, a, ~a, ~b`
  needs `{a, ~a}` on the `⊥` side, and a `⊤` side accepts any subset while
  the other side's provability is not monotone in what it gets. Negative
  literals force nothing either. Both go through the enumeration.
- **One commit for the engine, Mix and the count prunes** rather than the
  two the prompt sketches: Mix and the equation are switches in the same
  `Rules` value and the same functions, and were built together.
- No decision in `plan/README.md` turned out wrong. D11 held without
  effort: the crate has no clock and spawns nothing.

## Review

A fresh-context reviewer (a sub-agent that had not seen the design being
made) read the spec's focused skeleton, count invariants, memoization
contract and MALL-Seq sections and the engine, and did two things. First,
it argued each completeness and soundness question against the code:
every `⊗` and `⊕` of a stable sequent is a candidate and skipping `1` (unless
alone) and literals is exact; the forced splits are exact, the first dual
occurrence sufficing because the residues are the same multiset; the
interval check is sound by induction on cut-free proofs, with an absorbing
member passing unconditionally; the count equation is
`c = t − p − #1 + #⊥ + 2 + 2·#Mix` by the same induction, so `≥` with Mix
and `>` for a Mix to be worth trying are exact, and the switch is on for
exactly the fragments where it holds; Mix permutes to stable sequents, so
trying it last there is complete; the early `Top` with the pending `⅋`/`⊥`
wrapped around it is a valid term; memo entries are facts; the run is
deterministic (it ran every call twice and compared verdicts, statistics
and node arrays). Second, it wrote an independent unfocused, prune-free
reference prover over formula multisets (every rule of MALL, `⊗` over every
submask, Mix over every partition, no `0` rule, memo by multiset) in a
throwaway crate outside the repository and compared verdicts on about 1.43
million random sequents: uniform, atom-balanced, generated-provable and
mutated ones, over MLL, MLL with units, MALL without and with `⊤`/`0`, each
with and without Mix, with the default options, with a memo of two entries,
with no memo, and with the fragment asserted as MALL on MLL inputs; plus 51
hand-picked edge cases (several dual occurrences for a forced split, `⊤`
under pending `⅋` and `&`, `0` beside absorbing members, units under Mix,
one-signed atoms, duals hidden inside `⊕`). No disagreement, no `Unknown`,
every proof checked. It confirmed the two spec errors above (the `0` rule,
and "no positive non-literal and not a dual pair → fail", which is wrong
with Mix: `⊢ a, ~a, b, ~b`; the engine falls through to Mix instead), and
found no bug. Its minor notes are follow-ups below: the Mix parity, the
`&` row with one absorbing side, and the empty interval for `0`.

## Open questions and follow-ups

- **A branch-and-bound split instead of Gray code.** Assigning members one
  at a time and pruning a partial assignment when the remaining members'
  intervals cannot bring either side to balance would cut the 3.9 billion
  splits of the 3-Partition refutation by orders of magnitude and would
  remove the 63-member limit. The prompt fixed Gray order; step 13 should
  measure this first.
- **Tighter counts, all sound and all unmeasured.** The intersection for
  `&` as above; the non-absorbing side's row for a `&` with one absorbing
  side (both premises must be provable) instead of dropping the row; the
  empty interval for `0` so that `0 ⊗ a` is refuted by counts; and with Mix
  the parity `c − w` even, since `c = w + 2 + 2·#Mix`.
- **Atom bias per problem.** The forest's "rarer literal is positive" is
  the wrong bias for Horn clauses; a `Forest` option or an engine-side
  override (both engines and the checker must agree within a problem) is
  step 13 material, with the 3-Partition refutation as the benchmark.
- **A memo for focus results.** `focus(Γ, F)` is a fact about `(Γ, F)` as
  much as `prove(Γ)` is about `Γ`; the spec memoizes stable sequents only,
  and whether a second table pays is a measurement.
- **Memo keys in an arena** with `u32` offsets (spec "Data layout") would
  halve the memo's memory; the map holds cloned `OccSet`s today.
- **Mix costs `3^k`.** In MLL every Mix can be pushed to the root, so a
  partition DP over Mix-free provability would replace the nested search;
  in MALL it cannot (a `&` premise may need the Mix). Not pursued.
- **Nullary Mix** is still not a rule: `⊢` is unprovable with Mix too.

## For step 4

- `prove_until` with a deadline closure; `Reason: Display` and `Engine:
  Display` for the output line; `Outcome.fragment` is what to print as the
  detected fragment (or the asserted one).
- `Options::recursion_limit` and the stack figures above decide the search
  thread's stack; the derivation builder recurses too (step 2's report).
- `Error::NoEngine` and `Error::FragmentMismatch` are the user-facing
  refusals; `Fragment` and `Mode` print in words.
- `--engine focus` is `Options::engine(Some(Engine::Focus))`; any other
  engine name has no variant yet.

## For step 7

- `Rules` gets the exponential switches; `asynchronous` handles `Kind::Quest`
  (now `unreachable!`) by moving the subformula into `Θ`; `decide` sees
  `Kind::Bang` as a candidate; `focus_on` gets the `!` rule.
- `memo::Entry` gets the remaining bound and `Memo::get` the comparison
  the memoization contract requires; the key becomes both zones.
- `Counts::new` gives `!A`/`?A` the row of `A`; skip the atoms below a `?`
  as the spec says, or drop the interval check when exponentials occur.
- The generator's `Rules` gets `exponentials`, and `twin` a `?`-aware case
  if needed.

## For step 12

- `Memo` is one type with `get`/`insert`; a sharded map replaces it. The
  stop closure is the stop flag. `Engine` holds all state; there is no
  global. The pools are per engine, so per worker.

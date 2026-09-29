# Step 7 report: exponentials (MELL, full LL) and affine mode

Session of 2026-09-29, from `plan/07-exponentials.md`. Realises the MELL,
LL and affine rows of plan decision D8 on the focused engine of D7, with
the three-valued outcome of D9 now meaning what it says for a fragment
without a decision procedure.

## Outcome

The focused engine searches dyadic sequents `⊢ Θ ; Γ` and handles every
classical fragment up to full LL: `?` moves a formula into the unrestricted
zone, `!` promotes with an empty linear zone, a formula of `Θ` is copied
under a per-branch budget that deepens iteratively from 0 to
`Options::copies` (default 3, as llprover), memo entries carry the budget a
failure was cut at, a stable sequent that repeats an ancestor on its branch
is pruned, and the verdict is `Proved`, `Unprovable` only after a level
that never hit its bound, or `Unknown(Reason::CopyBound(n))`. Affine mode
adds weakening at the leaves and is otherwise the same bounded search: the
spec's supermultiset-ancestor prune, which was to make it a decision
procedure, is unsound and is not implemented (see "Deviations"). Every
proof is a dyadic term the step 2 checker accepts, and the derivation view
shows the standard rules: dereliction, contraction, weakening and promotion
where the translation table puts them. The dispatch routes exponentials
and affine mode to the engine; `--copies` is live in the CLI, and `unknown
(…): the copy bound of 3 was reached; raise it with --copies` is its
verdict line. A fresh-context reviewer argued the soundness conditions
against the code and ran a differential comparison on 5 200 random
sequents; it found the affine prune, which is now gone, and confirmed the
rest (see "Review").

Thirteen changes (`jj log`), then this report:

1. **Add exponentials and affine mode to the focused engine**: the
   `Context` multiset (`search/focus/context.rs`), the memo with bounds
   (`memo.rs`), the count prunes with exponentials (`counts.rs`), the
   engine (`mod.rs`), `Options::copies`, `Reason::CopyBound`,
   `Error::NetMode`, the dispatch rows, the JSON tag `{"copy_bound": n}`,
   and `--copies` in the CLI.
2. **Test the exponentials, the copy bound and affine mode.**
3. **Generate random proofs with exponentials**: `search/generate.rs`.
4. **Search affine mode without a copy bound** (undone by change 11).
5. **Add Horn programs as tests.**
6. **Document the exponentials and affine mode**: `.claude/rules/core.md`,
   `.claude/rules/cli.md`, CLAUDE.md, README.
7. **Let a ⊤ in the unrestricted zone save a stable sequent with a 0**, a
   bug the generated sample found.
8. **Bound the generated copies by every dereliction**, a correction of
   the generator's bound.
9. **Trim the slow Horn test to the counter that finishes.**
10. **Sample fewer sequents with exponentials in the large generated run**,
    then none (change 13): the timed sample of the exponential rule sets
    is the CLI run above.
11. **Drop the ancestor-inclusion prune and bound affine mode like linear
    mode**, the reviewer's finding.
12. **Document the affine correction**, including an errata entry in the
    spec.
13. **Keep the large generated sample to the fragments without
    exponentials.**

All checks pass at the last change: `cargo clippy --workspace --all-targets
-- --deny warnings`, `cargo test --workspace` (78 unit tests in core with 5
ignored, 9 parse and 9 serialize integration tests, 2 unit and 5
integration tests in the CLI, the doc tests), `cargo hack check
--feature-powerset -p linlog`, and `nix flake check` (see "Verification").
No dependency changed.

## The API and option changes

| item | change |
|---|---|
| `Options::copies(u32)`, `Options::DEFAULT_COPIES` (3) | the per-branch copy bound the search deepens up to; no effect without exponentials |
| `Reason::CopyBound(u32)` | every level up to this bound hit it; `Display` "the copy bound of 3 was reached"; JSON `{"copy_bound": 3}` |
| `Error::NetMode(Mode)` | `Engine::Net` forced in affine mode ("proof nets exist in classical mode only, with or without Mix, not in classical affine mode") |
| `prove`, `prove_until` | MELL, LL and every fragment in affine mode go to `Engine::Focus`; only intuitionistic mode is still `Error::NoEngine` |
| `search::focus::search_goal(&Forest, &[OccId], …)` | crate-private: the search from any multiset of occurrences with an empty `Θ`, returning the node, the arena and the statistics; `search` is `search_goal` on the roots, for step 9 |
| CLI `--copies N` | unhidden, default 3, help text with the three-valued meaning; the verdict line names the flag |

`Outcome`, `Statistics` and the JSON forms are otherwise unchanged; the
proof file of a MELL proof carries `?`, `!` and `copy` nodes as step 2
froze them.

## The engine, in the spec's terms

`Θ` is an `OccSet` passed down the branch; it only grows (`quest` clones it
into a larger set from the pool when the subformula is new, and does
nothing when it is already there, since `Θ` is a set). `Γ` is a `Context`:
a bitset plus a sorted list of extra copies, empty until a copy repeats an
occurrence (releasing a copied `~a ⅋ ~a` puts the same `~a` twice into
`Γ`), which is the one allocation on the hot path the prompt allowed. The
memo key is both zones. The branch stack holds the stable sequents of the
current branch for the loop check.

The rules, on top of step 3's: the asynchronous phase handles `?`;
`decide` collects the D1 candidates of `Γ` (`⊗`, `⊕`; `1` and `!` only
when alone, or in affine mode always), then the copies from `Θ` (D2),
skipping a member with an unconsumed copy in `Γ`, ordering those with a
literal whose dual is in `Γ` first, then Mix; `focus` on `!A` needs `Γ`
empty (weakened in affine mode) and releases `A` into an empty zone under a
`Bang`; `initial` implements both initial rules, `⊢ Θ ; p⊥ ⇓ p` as `Ax`
and `⊢ Θ, p⊥ ; · ⇓ p` as `Copy(p⊥)` above `Ax(p, p⊥)`, in a stable sequent
(a dual pair, or a lone literal whose dual is in `Θ`) and in focus on a
positive literal. A positive-literal factor of a `⊗` takes its dual from
`Γ` when there is one and otherwise leaves its side empty for the `Θ`
initial rule; `1` and `!` factors force the empty side. The `&` rule
duplicates `Γ`, keeps `Θ` and gives each premise the whole budget. A `0`
in a stable sequent is fatal only when neither `Γ` nor `Θ` has a member
with a `⊤` below it.

The bound: `run` loops the budget from 0 to `Options::copies`, with the
memo kept across levels. Every D2 and every `Θ` initial rule costs one
unit; when the budget is 0 and a copy would have been tried (there is a
member of `Θ` without an unconsumed copy in `Γ`, or a literal whose dual
lies in `Θ`), the engine sets `exhausted`. A level that ends with the flag
clear and no proof is `Unprovable`; the last level with the flag set is
`Reason::CopyBound`. Memo entries: `Proved(node)`, `Failed(Complete)` and
`Failed(Exhausted(r))`, the last a hit only when at most `r` copies are
left now.

## Soundness arguments, in a few sentences each

- **`Unprovable`.** The flag `exhausted` is set on every path where the
  budget refuses a copy that the rules would otherwise try: in
  `decide_with` when the list of eligible `Θ` members is non-empty at
  budget 0, in `initial` when the `Θ` initial rule needs a copy at budget
  0 (the other literal pairs are still tried), and on a memo hit of
  `Failed(Exhausted)`. It is saved and cleared around each stable
  sequent's decision, so a subtree's flag is its own, and restored as the
  disjunction. A level whose root ends with the flag clear explored every
  branch to a genuine failure: nothing a larger budget adds was reachable,
  so the sequent is unprovable at every budget, which is unprovable. The
  reviewer's argument for the skipped copies (a member with an unconsumed
  copy in `Γ`): a D2 on such a member permutes to the point where the `Γ`
  copy is consumed, with no more copies on any branch, so at budget 0 an
  empty list is genuinely nothing to cut.
- **The memo with the bound.** `Proved` is a proof at any budget; the
  bound is a search device, not part of the answer, so reusing a proof
  that took more copies than the current level allows is fine, and the
  docs of `Options::copies` say so (`Proved` at level `k` does not mean a
  proof with at most `k` copies per branch). `Complete` is a failure
  recorded when the subtree never hit the budget and no prune in it
  depended on an ancestor (below), so it holds at every budget.
  `Exhausted(r)` holds for any remaining budget ≤ r because a smaller
  budget proves less; with more budget left the entry is ignored, as the
  spec's contract requires. A later `Exhausted` only raises `r`; `Proved`
  and `Complete` replace it and are never replaced by it.
- **The loop check.** A stable sequent equal to an ancestor on its branch
  is pruned: any proof of it is a proof of the ancestor, so a smallest
  proof of the ancestor never passes through it, at any budget. The
  failure it causes is a fact about the branch, not the sequent:
  `dependency` records the depth of the ancestor, a failing sequent that
  carries a dependency on an ancestor is not memoized, and the dependency
  is discharged at the ancestor itself. Pruned branches never set
  `exhausted`, since they are redundant at every budget. The memo is
  consulted first, but only a `Proved` or `Complete` entry answers before
  the loop check; an `Exhausted` entry waits, so that a repeated sequent
  is pruned rather than counted as cut by the budget (found by a trace).
- **Affine mode** changes no fact the memo records: `Proved` and
  `Complete` are facts about a sequent in affine logic as in linear logic,
  and the leaves weaken what is left over, which is complete because
  weakening permutes upwards through every rule except promotion, where
  it stays below the `!` (the term has the `Weaken` nodes there). Nothing
  but `0` forces a split, because every leaf takes any context.
- **Count prunes with exponentials.** An atom with a literal below any `?`
  or `!` in the problem gets no interval entries anywhere; a `⊤` below any
  `?` or `!` disables the check (`absorbs_from_copies`): a copy of it
  absorbs any imbalance, which the tally of `Γ` cannot see because `Θ` is
  not in the tally. With those two rules every atom the check still sees
  is produced and consumed only by linear rules, for which step 3's
  induction holds. The count equation is off with exponentials and in
  affine mode; the interval check is off in affine mode (weakening
  discards any imbalance).

## Timings

Release builds on the development machine, from the CLI with `--stats`
and the ignored tests.

The counter program `!(a ⊗ a ⊸ b), !(b ⊗ b ⊸ c), !(c ⊗ c ⊸ d), a^8 ⊢ d`
(Petri-net reachability in the ILLTP style, `horn_programs_slow`):

| bound | verdict | stable sequents | memo hits | memo entries at most | time |
|---|---|---|---|---|---|
| 1 | unknown | 19 685 | 19 171 | 513 | 1.7 ms |
| 2 | unknown | 845 484 | 843 433 | 1 537 | 67 ms |
| 3 | proved | 1 764 046 | 1 759 672 | 2 815 | 126 ms |
| 4 to 7 | proved | the same | the same | the same | the same |

A branch to one token of the goal fires one clause per level of the
counter, so the per-branch bound is 3 for eight tokens; the earlier levels
are re-explored in full, and their entries are all `Exhausted`, so the
memo across levels recovers little here. The memo size stays in the
thousands while the hits are in the millions: the eight `a` hypotheses are
distinct occurrences, so every choice of which two `~a` a clause consumes
gives a different stable sequent that is the same sequent up to renaming
(see the follow-ups). The unreachable marking `a^8 ⊢ d ⊗ a` is unknown at
bound 3 after 6.9 million stable sequents (437 ms). Sixteen tokens (bound
4) did not finish within a quarter of an hour, so the ignored test keeps
the eight-token instance only.

The growing context `!(a ⊸ a ⊗ a), a ⊢ ?b`: unknown at every bound, with
4, 10, 19, 31, 46 and 64 stable sequents and 3, 5, 7, 9, 11 and 13 memo
entries at bounds 1 to 6, quadratic in the bound as the levels re-explore
a linear chain.

The generated sample: the 4 000 provable sequents of up to 24 rules over
the eight rule sets with exponentials (through the release CLI, one
process per sequent, bound 6, 2 s each) are all proved in 28 s in total,
except 9 that hit the 2 s limit; their 3 386 mutants at bound 3 are 1 117
provable, 1 883 unprovable and 386 unknown, in 55 s. A few sequents of
the sample take minutes at the bound their derelictions give (a run of 100
per rule set in the test harness did not finish in twenty minutes), which
is why the ignored `generated_large_sample` keeps to the eight rule sets
without exponentials: 4 000 proved, 3 340 mutants decided consistently, in
2.1 s, as in step 3.

The quick tests: `generated_sequents` (640 sequents, each also in affine
mode, and their mutants, 3.4 s in a debug build), `horn_programs`,
`classic_exponentials`, `full_ll`, `affine`, `copy_bound`,
`memo_across_levels` and `exponential_derivations` together under a
second.

## Decisions where the prompt left room

- **The `Θ` initial rule counts as a copy.** Andreoli's `⊢ Θ, p⊥ ; · ⇓ p`
  is an initial rule, not a D2, but it is a `Copy` node and a `?d` in the
  derivation, and `--copies` promises "how often `?` formulas may be
  copied on one branch"; counting it keeps the bound equal to the most
  derelictions on a branch of the focused proof. So `!a ⊢ a` needs bound
  1, and bound 0 is exactly "no `?d` at all".
- **The bound is per branch of the focused proof**, as the spec and the
  prompt say; `!a ⊢ a ⊗ a` therefore needs bound 1, not 2 as the prompt's
  list counts copies in total, and `!a, !(a ⊸ a ⊸ a ⊸ b) ⊢ b` needs 2.
  Focusing can need more copies per branch than the standard proof has
  derelictions per branch, because a copy is made at the stable sequent
  below a positive phase and is then on every branch of that phase; the
  generator therefore bounds by all derelictions of its proof (change 8).
  The forced split's preference for the dual in `Γ` can cost one more
  level for the same reason (`⊢ ?~p, ?p, ~p, p ⊗ ⊥` is proved at bound 2),
  never a verdict.
- **`Complete` failures are valid at every budget**, a strengthening of
  the spec's contract (which stores every failure with a bound): a subtree
  that never hit the budget explored everything a larger budget would.
- **Conditional failures are not memoized** rather than memoized with a
  weaker validity, through the `dependency` depth; the ancestor discharges
  them. This is the standard treatment of loop checks with memoization and
  costs nothing on inputs without repeats.
- **The memo may change decisiveness, never a verdict**: an `Exhausted`
  entry is a fact about the sequent alone and the loop check a fact about
  the branch, so a run with the memo can be `Unknown` where the memo-free
  run is `Unprovable` (the generator found one within the first sample)
  or the reverse; the generated tests assert that the two never
  contradict. Making the two agree would mean re-searching every
  `Exhausted` hit under the current stack, which is the memo-free search.
- **`weakened` emits one `Weaken` per leftover copy below the leaf**, as
  step 2 asked, and never above a promotion: `Bang` is applied to the
  released empty zone and the leftover is weakened under it.
- **`search_goal` takes a slice of occurrences** with `Θ` empty; step 9's
  goal "with `?` formulas in the linear zone and repeats where a
  contraction was applied by hand" is exactly such a multiset, since the
  asynchronous phase moves the `?` formulas into `Θ` itself. The `Proof`
  is still built from the roots by `search`; step 9 grafts the node of
  `search_goal`.
- **The generator's contraction** tensors a premise with itself, so every
  `?` formula of the premise appears twice and contracts; promotion
  derelicts whatever of the context is not a `?` formula first.
- **`Error::NetMode`** rather than reusing `NetFragment`, whose message
  would name the wrong reason.

## Deviations from the spec or the prompt, with reasons

- **The affine prune is not implemented, and affine mode is not a
  decision procedure.** The spec's argument ("a premise that is a
  supermultiset of a sequent below it can be shortened by weakening away
  the surplus") runs the wrong way: weakening turns a proof of the smaller
  sequent into a proof of the larger one, so a proof of the descendant
  says nothing about the ancestor, and `⊢ ?(a ⅋ ~a)` (also `⊢ ?!1`,
  `!(a ⊗ ~a) ⊢`) is provable only through a stable sequent that contains
  the root. The prune was built as the prompt asked, together with the
  unbounded affine level it was to justify (change 4), and the reviewer's
  differential run showed 426 wrong `Unprovable` verdicts out of 5 200
  random affine sequents; change 11 removed it and change 12 recorded the
  erratum in the spec. The prune in the other direction (a descendant
  contained in an ancestor) is sound but prunes exactly the useful
  branches. Affine MELL is decidable (Kopylov), but by an argument this
  step cannot turn into a prune; affine mode is the bounded search with
  weakening, and `CopyBound` is a possible answer there too. The prompt's
  "termination test" is therefore the bounded one.
- The count prunes needed one rule the spec does not state: a `⊤` below a
  `?` or `!` makes the interval check unsound for every atom (found by the
  test `!a, !(a ⊸ 0) ⊢ b`, provable through the copied `⊤`, refuted by the
  balance of `b` until the rule was added); and the spec's "a `0` in a
  stable sequent is fatal" is wrong once more when a `Θ` member has a `⊤`
  below it (found by the generated sample, change 7).
- The spec's "immediate failure: no positive non-literal formula and not a
  dual pair" is also wrong with a non-empty `Θ` (a copy can bring a
  positive formula): the engine has no such test, it falls through to the
  copies.
- The spec memoizes every failure with a bound; `Complete` failures are
  kept unconditionally (above).
- The prompt lists `!a ⊢ a ⊗ a` and `!a ⊢ !a ⊗ !a` as "needing two
  copies"; per branch they need one, and the tests say so.
- No decision in `plan/README.md` turned out wrong. D11 holds: the engine
  has no clock, thread or global.

## Review

A fresh-context reviewer (a sub-agent that had not seen the design being
made) read the spec's memoization contract, MALL-Seq and MELL sections
and the engine, and argued each condition against the code. It found the
affine prune unsound (above), with the counterexamples `⊢ ?(a ⅋ ~a)`,
`⊢ ?!1`, `⊢ ?(~a ⊕ a)`, `⊢ ?(a ⅋ ~a), b` and `!(a ⊗ ~a) ⊢`, and observed
that every stable-to-stable path whose end contains its start contains a
copy, so the prune forbade exactly the derelictions that grow `Γ`; it also
confirmed the `0`-with-`Θ` bug of change 7 while that fix was landing. It
argued sound: `Unprovable` in linear mode (every budget cut sets
`exhausted`, `Complete` entries are budget-free by induction, the skipped
copies permute away), the memo contract (`get` filters `Exhausted` by the
remaining budget, `insert` only strengthens), the loop check and the order
of memo, stack and `Exhausted` hit in `prove_stable`, the completeness of
the `Θ` rules (both initial rules in both places, `!` in focus, `quest` as
a set, the forced literal side with the `Θ` fallback), the count prunes
(every `Θ` member and copy is below a `?`, so has an empty row, and the
`Θ` initial rule only consumes exponential atoms), and determinism and the
pools. It noted the level semantics of `Proved` entries (a proof at level
`k` may carry more than `k` derelictions on a branch; the docs now say so)
and the one-level cost of the forced split's `Γ`-first choice, and one
latent bug that the affine bound made live: `initial` returned at the
first literal whose dual was only in `Θ` at budget 0, abandoning the other
pairs; it now goes on (change 11).

Its harness, an independent unfocused dyadic prover over formula
multisets with all rules, no heuristics and a copies-per-branch bound, in
a throwaway crate outside the repository, compared verdicts on 5 200
random sequents over `⊗ ⅋ 1 ⊥ & ⊕ ⊤ 0 ! ?` and two or three atoms, each
at bounds 0 to 3 in classical mode and once in affine mode (about 26 000
engine calls): in linear mode 6 090 `Proved` (every proof passed the
checker), 11 495 `Unprovable`, no disagreement at reference bounds up to
7; in affine mode 2 685 `Proved` (all checked), 2 515 `Unprovable`, of
which 426 were provable, all through the prune. After change 11 the
generated tests prove every generated sequent in affine mode as well, and
the reviewer's five counterexamples are pinned in `affine`. The
fragilities it named are in the rules file: `exhausted` and `dependency`
are engine-wide flags restored by hand in `prove_stable`, and `Proved`
entries are budget-free.

## Open questions and follow-ups

- **A decision procedure for affine fragments.** The spec's prune is
  wrong; Kopylov's procedure and Lazić–Schmitz's bounds exist, and the
  contractive analogue (a descendant *containing* an ancestor is
  redundant when contraction is available) is the spec's other prune,
  which is the sound one for contractive fragments. Whether an affine
  prune exists that is both sound and useful is open; until then affine
  mode is bounded.
- **Interchangeable occurrences.** Hypotheses that are the same formula
  (`a, a, a, …`, or several copies of one clause) are distinct occurrence
  ids, so the memo and the loop check see `C(8, 2)` different stable
  sequents where there is one up to renaming; the counter program's
  millions of memo hits are that. A canonical choice among identical
  members (take the first, as the forced split already does for dual
  literals; a prefix rule in the free enumeration; keys up to the
  `TermId`s of the members) is the fix, and step 14's benchmarks the place
  to measure it.
- **A factor that is a tensor of positive literals** (`a ⊗ a` in
  `(a ⊗ a) ⊗ ~b`) is not forced, so the free enumeration runs over every
  subset of `Γ`, and with exponential atoms the intervals reject none of
  them; a nested-forced rule would cut the counter program's search by
  orders of magnitude.
- **Levels re-explore from scratch** when every entry of the previous
  level is `Exhausted`, as on the counter program; the spec's remark that
  the memo recovers most of the deepening cost holds only for subtrees
  that fail completely. A per-level restart from the frontier of exhausted
  sequents is the classic remedy and is step 14b material.
- **A few generated sequents take minutes** at the bound their
  derelictions give, mostly with Mix and many members (the `3^k` partition
  cost of step 3 multiplied by the copies); the large sample is trimmed
  for that reason.
- **The loop check's cost** is `O(depth · W)` per stable sequent; a hash
  per stack entry would make it cheap on deep branches.
- **The 63-member limit** now counts copies; a wide `Γ` with many copies
  hits `ContextTooWide` sooner than before.
- **A budget-aware memo** would be needed to report the least copies a
  proof takes, since a `Proved` entry may be reused at a smaller budget.

## For step 8

- The two-sided engine reuses the dyadic machinery as it is: `Θ` and `Γ`
  are occurrence sets of the lowered sequent, the copy budget, the memo
  entries, the stack and the loop check are indifferent to the side of an
  occurrence. What step 8 adds is the one-succedent condition on stable
  sequents and the input/output reading of `!` (a `!` hypothesis is a `?`
  formula of the lowered sequent, so it lands in `Θ` through `quest` with
  no new rule) and the intuitionistic checker; `Rules` is where a
  `one_sided: false` switch goes, and `initial` and `decide_with` are the
  two places that look at the shape of a stable sequent.
- `search_goal` is the entry for a goal that is not the roots.

## For step 13

- The memo now stores three kinds of entry and `insert` merges them (a
  larger `Exhausted` budget wins, `Complete` and `Proved` win over
  `Exhausted`); under concurrency this is the compare-and-swap the spec
  asks for, and the merge rule is the invariant to keep: an entry's
  validity only ever grows.
- The branch stack and `dependency` are per branch, hence per worker; a
  worker that starts from a cube must start with the stack of that cube's
  ancestors, or the loop check loses those prunes (never soundness).
- `exhausted` is per level and must be or-reduced over workers before a
  level is called `Failed`; a level is `Unprovable` only if every worker's
  flag is clear.
- Levels must not run concurrently (the spec says so): an `Exhausted(r)`
  entry from a lower level is what makes the next level cheap only when
  the memo is shared in order.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`, `cargo test
--workspace`, `cargo hack check --feature-powerset -p linlog` and `nix
flake check` ("all checks passed!"): all pass at the last code change, on
top of which this report sits. No dependency changed, so `cargo deny check` was not
run. The timings above come from release builds of the ignored tests and
of the CLI.

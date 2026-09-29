# Step 6 report: proof-net search for MLL

Session of 2026-09-29, from `plan/06-net-search.md`. Realises the net-search
row of plan decision D8 and the `search::net` engine of D7 on top of the
proof structures of step 5.

## Outcome

`linlog::search::net` exists and is the engine that unit-free MLL, with or
without Mix, is routed to by default. It rejects a sequent by the count
equation and the per-atom balance before any link, then searches the axiom
linkings over a `ProofStructure` by backtracking with an explicit stack:
the unlinked literal with the fewest admissible partners is chosen at every
node, a partner is rejected in constant time when the two literals hang
under a common `⊗` of one conclusion, when the `⅋`-free skeleton joins them
already, or when it would put the partners of equal literal conclusions out
of order, the exact acyclicity test of step 5 runs after every link on a
structure of at most 200 occurrences and after every fourth link on a
larger one, and a complete linking that passes it is a proof net, which is
sequentialized into the proof term returned; the outcome also carries the
net. `--engine net` and `--engine focus` force an engine in the CLI,
`--stats` prints the counters of the engine that ran, and `--format net`
prints the net the search found without the round trip through the proof.
The engine agrees with the focused engine on every sequent of the
differential sample, and a fresh-context reviewer compared it with a
brute-force enumeration of all linkings and with the focused engine on
about 54 000 cases without a disagreement.

What the numbers say, in short: on inputs with distinct atoms and wide
contexts the net engine is linear where the focused engine is exponential
(a 2000-conjunct sequent of 13 999 occurrences in half a second, where the
focused engine gives up at 64 members), but on Horn encodings with few
atoms of high multiplicity, which is what the known hard families are, the
focused engine is faster by two to four orders of magnitude, because the
equal literals inside `b ⊗ b ⊗ b` and `~b ⅋ ~b` are interchangeable and the
spec forbids breaking that symmetry inside formulas. The default routing
follows the plan; the planning session should weigh the numbers below.

Five commits: "Search axiom linkings with incremental pruning", "Expose
the net engine in the CLI", "Add a Partition encoding as a slow net test",
"Document the net engine", then this report. All checks pass at the last
code change: `cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace` (73 unit tests in core with 4 ignored, 9 parse
and 9 serialize integration tests, 9 doc tests, 2 unit and 5 integration
tests in the CLI), `cargo test -p linlog --no-default-features`,
`cargo hack check --feature-powerset -p linlog`, rustdoc with warnings
denied, and `nix flake check`. No dependency was added.

## The API

| item | role |
|---|---|
| `Engine::Net` | the new variant; `Display` and JSON name `net` |
| `prove`, `prove_until` | route unit-free MLL (the empty fragment included) to `net`, everything else up to MALL to `focus`; `Options::engine(Some(Engine::Net))` outside unit-free MLL, detected or asserted, is `Error::NetFragment` |
| `Options::test_period(Option<u32>)` | how many links go between two exact tests; `None` is the default rule (every link up to 200 occurrences, every fourth above); zero counts as one |
| `Outcome::net: Option<ProofStructure>` | the net the net engine found; `None` from the focused engine and for any other verdict; not serialized |
| `Statistics::links`, `Statistics::tests` | links tried (made, and taken back unless kept) and exact tests run; `nodes` is literals chosen for the net engine; the memo and split counters stay zero |
| `search::net::search(&Forest, Mode, &Options, &mut dyn FnMut() -> bool)` | the crate-private entry, returning the verdict, the statistics and the net |
| CLI | `--engine net`, `--stats` per engine, `--format net` prints `Outcome::net` when present |

The `Outcome` and `Statistics` additions are backwards compatible through
`#[non_exhaustive]`; the JSON of an outcome gains the keys `links` and
`tests` in `statistics`, which `check` ignores. The engine cannot answer
`RecursionLimit` or `ContextTooWide`; its only `Unknown` is `Stopped`.

## The search, in the spec's terms

Preprocessing counts `c`, `t`, `p` and the literals per atom and sign, and
answers `Unprovable` unless `c = t − p + 2` (`≥` with Mix) and every atom is
balanced. The engine then owns one `ProofStructure` (a clone of the forest
inside), one `Scratch`, `remaining[atom]` (the unlinked pairs per atom,
kept on every link and unlink), the chains `copy_before`/`copy_after` of
equal literal conclusions, and the stack of `Frame { literal, next }`.
`choose` looks at every unlinked literal, counts its admissible partners
(stopping at the best count so far) and picks the fewest, ties to the
first in atom order, `a` before `~a`, then id; a count of zero is a dead
end and the branch is abandoned at once. `next_partner` walks the literals
of the other sign of the atom from `next` and returns the first unlinked
admissible one. After a link the exact test runs at its cadence or on
completion; a failed test, a dead end or an exhausted frame takes the last
link back, in stack order. Every frame but the top has its current link
made, so a linking is tried at most once. The stop condition is polled in
`decide`, once per node, like the focused engine's once per stable
sequent. On completion `sequentialize` (which re-runs `is_correct`,
allocating a scratch once) gives the proof, checked in a `debug_assert!`.

## The pitfalls checklist

- *The counts are necessary conditions for MLL only.* The engine runs only
  on unit-free MLL: the dispatch refuses `Engine::Net` on any other
  fragment, asserted or detected, and `ProofStructure::new` would refuse
  the forest anyway.
- *Atomic axioms only.* Links join literal occurrences, which the structure
  validates in debug builds; the search never considers anything else.
- *The LCA rejection within one conclusion only.* `Forest::lca` is `None`
  across roots, so the rejection never fires for two conclusions.
- *`⊗` premise edges get distinct colours.* Step 5's structure; the search
  adds no colouring of its own.
- *Connectedness after acyclicity.* The equation is tested before the
  search as a necessary condition, and it is read as connectedness only
  for a complete linking that passed the exact test, where it is
  equivalent to it (`k = t + 1`, see the rules file).
- *Contexts as sets of ids.* The linking is over occurrence ids; two
  occurrences of one formula are distinct literals, and the symmetry break
  is exactly about equal ids of equal literal conclusions.
- *Units break the linking model.* `Error::NetFragment` for MLL with units,
  from the dispatch and from `ProofStructure::new`.
- *Recursion depth equals the number of links.* The search has an explicit
  stack; only sequentialization recurses, to the derivation's height, and
  the CLI runs the whole search on its large-stack thread.

## Timings

Release builds on the development machine; the differential sample and
the symmetry example are the ignored tests `net_versus_focus_timing` and
`symmetry_of_equal_conclusions` (the latter with a temporary switch), the
families come from a scratch program outside the repository. Time limits
were 60 s (30 s for the last 3-Partition row).

The differential sample (`sample(150, 16, 10)`: generated provable
sequents, their mutants, doubled sequents and random balanced sequents,
with and without Mix, 1200 sequents of which 674 provable): both engines
agree; net 44.7 ms in total, focus 12.5 ms, net with the exact test every
fourth link 42.0 ms. The sequents are small, so per-problem setup (the
structure, its graph and skeleton, the scratch, the sequentialization)
dominates the net engine's time; the sample measures agreement, not
search power.

The symmetry example of the tests (four `a` conclusions and a switching
cycle the exact test alone sees, test postponed to the complete linking):
68 links and 24 exact tests without the break, 19 links and 1 test with
it.

A wide family, `(a_1 ⅋ b_1) ⊗ … ⊗ (a_k ⅋ b_k)` with the conclusions
`~a_i ⊗ ~b_i`, every atom distinct, provable:

| k | occurrences | net | focus |
|---|---|---|---|
| 12 | 83 | 214 µs, 24 links | 124 µs, 2 759 splits |
| 16 | 111 | 181 µs | 757 µs, 43 729 splits |
| 20 | 139 | 225 µs | 12.0 ms, 699 099 splits |
| 24 | 167 | 314 µs | 147 ms, 11.2 million splits |
| 200 | 1 399 | 7.6 ms, 400 links, 100 tests | context too wide |
| 2000 | 13 999 | 521 ms, 4000 links, 1000 tests | context too wide |

Matsuoka's Partition encoding (`partition` in the tests: item `a_i` buys
`s_i` units `b` or `s_i` units `c`, half of each buys the items back, half
of each buys the goal), yes and no instances:

| sizes | occurrences | verdict | net | focus |
|---|---|---|---|---|
| 1, 1 | 28 | proved | 134 µs, 15 nodes | 74 µs |
| 1, 3 | 44 | unprovable | 527 µs, 124 nodes | 164 µs |
| 2, 1, 1 | 50 | proved | 844 µs | 183 µs |
| 1, 1, 4 | 66 | unprovable | 50.5 ms, 5 658 nodes | 2.2 ms |
| 2, 2, 1, 1 | 72 | proved | 62 ms, 6 635 nodes | 1.9 ms |
| 1, 2, 5 | 82 | unprovable | 3.74 s, 293 870 nodes, 1.39 million links | 5.4 ms |
| 1, 1, 2, 4 | 88 | proved | 1.09 s, 109 627 nodes | 0.7 ms |
| 1, 1, 1, 5 | 88 | unprovable | 7.60 s, 560 182 nodes | 57 ms |
| 2, 3, 2, 1 | 88 | proved | 4.86 s, 344 946 nodes, 1.67 million links | 5.5 ms |
| 3, 3, 3, 1 | 104 | unprovable | unknown after 60 s (4.1 million nodes) | 97 ms |
| 1, 2, 3, 4, 5, 5 | 196 | proved | unknown after 60 s | 20.6 s |
| 1, 1, 1, 1, 1, 7 | 132 | unprovable | unknown after 60 s | 51.8 s, 4.4 billion splits |
| 2, 2, 2, 2, 2, 2, 9, 1 | 224 | unprovable | unknown after 60 s | unknown after 99 s |

Lincoln's two-literal 3-Partition encoding, `⊢ k ⊗ (~c)^{⅋ S_1}, …,
((~k ⅋ ~k ⅋ ~k) ⅋ c^{⊗ B})^{⊗ m}`:

| instance | occurrences | verdict | net | focus |
|---|---|---|---|---|
| m = 2, B = 4, sizes 1 1 2 1 1 2 | 49 | proved | 1.2 ms, 155 nodes | 78 µs |
| m = 2, B = 4, sizes 1 1 1 1 1 3 | 49 | unprovable | 1.89 s, 729 157 nodes | 0.3 ms, 1 node |
| m = 2, B = 6, sizes 1 2 3 2 2 2 | 65 | proved | 42 ms, 2 966 nodes | 185 µs |
| m = 2, B = 6, sizes 1 1 1 3 3 3 | 65 | unprovable | unknown after 30 s (8.5 million nodes) | 0.19 ms, 1 node |

Why the Horn encodings are lost: the `b` literals of `b ⊗ b ⊗ b ⊗ b` and
the `~b` of `~b ⅋ ~b ⅋ ~b` are interchangeable (permuting the partners of
the leaves of a pure `⊗` tree or a pure `⅋` tree of equal literals maps
nets to nets), so once an early choice (which `~a_i` an item's clause
takes) is wrong, every one of the `(t/2)!² · Π s_i!` equivalent
assignments of the units is explored before a cycle shows; on the
`2, 3, 2, 1` instance that is about 330 000, which is the node count. The
focused engine sees the same wrongness through the counts of a `⊗` split
at once: Lincoln's refutations cost it one stable sequent and 70 splits.
No MRV order can help, since the constraint that fails is a count over a
region, not a local cycle.

## Decisions where the prompt left room

- **Exact minimum-remaining-values with forward checking.** The prompt's
  "candidate counts kept in a small array updated on link/unlink" is
  `remaining[atom]`; the choice itself counts the admissible partners of
  every unlinked literal with both rejections and the ordering constraint
  applied, and abandons the branch when some literal has none. The scan
  costs the sum over atoms of the square of their multiplicity per node,
  which is within a constant of the exact tests the spec already mandates
  per node, and a dead end found now saves a subtree. The count of one
  literal stops once it cannot beat the best, and no atom is skipped, so
  a zero is never missed.
- **Both signs are candidates for the choice**, not only the positive
  literals: a `~a` without admissible `a` is a dead end as much as the
  other way round, and it is found one node earlier.
- **Symmetry breaking for equal literal conclusions only.** Sound by the
  lexicographic argument in the rules file, for any number of groups and
  wherever the partners lie. The first-literal key of the spec for equal
  compound conclusions is unsound across groups: two copies of `F` and two
  of `G` whose first literals link into each other's copies at non-first
  positions form an orbit in which no member sorts both groups (worked
  out during the session, and the reason the spec's "lexicographically
  smaller linking on the first copy" was not implemented for compound
  copies). A sound compound variant needs keys under roots that no
  symmetry moves; it is a follow-up. Inside formulas nothing is broken,
  as the spec says, at the cost above.
- **The count equation is the final connectedness test.** For a complete
  acyclic linking it is the same equation `is_correct` tests, and the
  quantities are fixed by the forest, so the test runs once, before the
  search; `sequentialize` re-runs the full criterion and its success is
  `expect`ed.
- **The exact test's cadence is `Options::test_period`**, with the spec's
  default of every link up to 200 occurrences and every fourth link above,
  and a test on every complete linking regardless. The CLI does not expose
  it; step 14 can, if the benchmarks want to tune it.
- **`Outcome::net`** carries the net found, so `--format net` prints the
  structure itself and not `from_proof` of the term; it is not serialized,
  since the proof's keys are and the net is `from_proof` of them.
- **`nodes` counts literals chosen** (calls of `choose`), the stop
  condition is polled there, and `links` and `tests` are the net engine's
  own counters; the JSON keeps one flat `statistics` object for both
  engines, with the other engine's counters at zero. The CLI's `--stats`
  prints per engine.
- **`--engine net` outside MLL is `Error::NetFragment`**, the error the
  CLI already prints for `--format net`, with the asserted fragment when
  one is asserted.
- **The random balanced generator** (`generate::balanced`) builds a
  sequent from `k` dual pairs by `k − 1` tensor joins (at most that many
  with Mix) and any number of par joins that leaves a conclusion, in random
  order, so every sample passes the counts; most are unprovable. The
  doubled sequents `Γ, Γ` of the sample give equal compound conclusions,
  provable with Mix, mostly not without.
- **The hard family is Matsuoka's Partition** (the encoding was taken from
  his HCVS 2017 paper through a research sub-agent, with Lincoln and
  Winkler's two-literal 3-Partition as the second family measured but not
  committed); the quick test decides four small instances with both
  engines, the ignored test times `2, 3, 2, 1` and `1, 2, 5`, a few
  seconds each in release mode.
- **The per-node work is not all O(1).** The spec makes the rejections
  O(1) and the choice unspecified; `lca` walks the depth of a literal and
  `same_component` two union-find paths, both amortised over a node's
  candidates, and the exact test is O(V + E) per round. Nothing allocates
  in the loop.

## Deviations from the spec or the prompt, with reasons

- The dispatch default follows D8, but the Horn measurements above say
  that for few atoms of high multiplicity the focused engine is the better
  default by orders of magnitude; the plan's step 14 has the harness to
  decide, and the rules file records both facts. A multiplicity-based
  routing heuristic (route to `focus` when some atom has more than a
  handful of occurrences of one sign) is the cheap fix if the numbers
  hold on the standard libraries.
- The `--stats` text was changed to print per engine; the prompt asked
  only for the fragment/engine line, but the focused engine's counters
  would have printed as zeros.
- No decision in `plan/README.md` turned out wrong. D11 holds: the engine
  has no clock, thread or global.

## Review

A fresh-context reviewer read the spec's MLL section, the rules file and
the engine and argued every claim: the necessity of the counts and the
sufficiency of the equation for connectedness after acyclicity; the
soundness of both rejections under every partial linking (with the
correction that the LCA cycle is kept by some switching, not every one,
which the comments now say); the lexicographic argument for the symmetry
break, for all groups at once and with partners inside other groups, and
the consistency of the check between `choose` and `next_partner`; the
stack invariant and the unlink order; that the early stop in counting
never hides a zero; and determinism. Its harness in a throwaway crate
enumerated every per-atom bijection of about 26 800 random sequents (trees
over one to three atoms with one to seven pairs, and "grouped" sequents
with several equal-literal conclusion groups whose partners lie inside
other groups) plus the classics and the empty sequent, in both modes, about
54 000 cases and four million linkings, and compared "some linking is a
net" with the engine at the default cadence, at a period of 2 and of 100,
and with the focused engine: no disagreement, no `Unknown`, no panic,
every proof checked, every net equal to its proof's net, and in every
provable case some brute-force net was ordered and the engine's net was.
Its fragilities are recorded in the rules file: the completion shortcut
depends on the preprocessing running first; the symmetry break is tied to
literal roots; the stop condition is polled per node, not per link.

## Open questions and follow-ups

- **Leaf symmetry breaking** for pure `⊗` and `⅋` trees of equal literals,
  which the Horn families need; sound by the argument in the rules file,
  and against the spec's instruction, so a plan decision.
- **A region count**: the focused engine's split counts have no analogue
  here, and they are what refutes the Horn encodings at once. A partial
  structure's `⊗` skeleton components could carry per-atom balances, which
  would be a new prune worth measuring against the leaf break.
- **The default engine** for unit-free MLL, as above; also a `--engine`
  portfolio (run both, first answer wins) once step 13 has threads.
- **Compound equal conclusions**: the sound variant with keys under
  unmoved roots, if inputs with repeated compound conclusions show up.
- **The stop condition per link** rather than per node: a frame with many
  candidates runs them all between two polls; the CLI's deadline is
  therefore late by at most one frame, which is fine today.
- **Setup cost per problem**: the structure clones the forest and the
  proof clones it again; on the tiny sequents of the differential sample
  this is most of the time. A borrowed structure would remove one clone.
- **`sequentialize` recurses**; unchanged from step 5, and the CLI's
  search thread covers it.
- **`test_period` in the CLI**, for step 14's tuning.

## For step 8

- The engine runs on the forest of the lowered sequent as it is: IMLL over
  `⊗ ⊸` without `1` is unit-free MLL after lowering, so the embedding is
  the dispatch row plus the intuitionistic reading. What the engine offers
  is a net and a term; whether a term must also satisfy the one-succedent
  condition of the checker's intuitionistic mode, and whether the net
  engine needs an extra condition on complete linkings (the essential-net
  criterion) or the classical net always sequentializes into an
  intuitionistic proof, is step 8's question. The one place to add a
  condition is the `complete` branch of `run`, next to the exact test.
- `Outcome::net` and `--format net` work unchanged for the embedding; the
  text form prints the lowered sequent.

## For step 13

- The engine has no global: everything is in `Engine` (the structure, the
  scratch, the counts, the stack, the statistics), the stop closure is the
  shared flag, and `ProofStructure` and `Scratch` are `Clone`, so a worker
  is a clone of the state after a prefix of links.
- Cubes are the first `d` frames: the choice order is deterministic, so
  the cube tree is reproducible, and a worker starts by making the cube's
  links (through `link`) and running the loop from there; a `seed` entry
  that takes a list of links is the API to add.
- Per-node cost is dominated by `choose` (the sum of squared
  multiplicities) and the exact test (`V + E` per round); both are per
  worker, no sharing needed.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo test -p linlog --no-default-features`,
`cargo hack check --feature-powerset -p linlog`, rustdoc with warnings
denied, and `nix flake check`: all pass at the last code change, on top of
which this report sits. The timings above come from release builds: the
ignored tests `net_versus_focus_timing` and `partition_instances_slow`,
and a scratch program outside the repository for the families.

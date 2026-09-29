# Step 13 report: parallel search

Session of 2026-09-30, from the step-13 prompt. Realises the `parallel`
feature of D14 (off by default, on in the CLI, never on wasm, D11) and the
spec's "Parallel runtime" as far as the engines of steps 3, 6, 7 and 8
allow.

## Outcome

`linlog` gains the cargo feature `parallel` (rayon 1.12, the only new
dependency; `core` without the feature has no rayon in its tree), and
`Options::jobs(n)` runs the focused engine, the two-sided engine and the
net engine on a pool of `n` threads built for the search and dropped with
it. The focused engine runs cube-and-conquer as nested fork-join over the
first two choices of every branch (the focus on a stable sequent with its
candidates and copies together, the side of a `⊕`, the free splits of a
`⊗` chunked by the assignment of the first few members) and the `&`
premises and-parallel, every worker a copy of the branch it starts from
with pools of its own, all workers sharing one sharded memo and one proof
arena; the copy bound deepens one level at a time on the root engine. The
net engine enumerates the branches of the first `d` links as cubes, `d`
growing until there are sixteen cubes per thread, and the workers take
cubes from a counter with one engine each. The caller's stop closure
stays what it was (`FnMut`, no `Send`): the driver polls it on the calling
thread once a millisecond and raises an atomic flag that every worker
polls at every stable sequent or literal chosen, with a cancel flag per
parallel choice chained above it. `Options::portfolio(true)` gives every
worker an order of its own for the alternatives below the parallel
levels. The CLI has `--jobs N` (`-j`, default: the machine's parallelism)
and `--deterministic` (the sequential engines, whose proof and counts are
a function of the input) on `prove` and `interact`. Every proof a parallel
run returns passes the checker; a parallel run may return another proof
than the sequential engine, never another decided verdict; the tests
assert this on the generated samples with two and four threads, with and
without the portfolio, classical and intuitionistic, linear and affine,
and on the net engine's sample.

Commits: "Add the parallel runtime behind a feature", "Run the focused
engine as cube-and-conquer", "Parallelize proof-net search", "Add --jobs
and --deterministic to the CLI", "Document the parallel search", "Measure
parallel speedups".

## The API and the flags

- `Options::jobs(n)`: the threads the search may use; one (the default)
  runs the sequential engines, zero counts as one. Without the feature the
  value is kept and ignored, so a front end can hold one options value for
  both builds. `Options::portfolio(bool)`, off by default.
  `Options::stack_size()` returns the stack a thread needs at the
  recursion limit (the formula the CLI used to keep to itself); the pool's
  workers and the CLI's search thread are sized by it.
- `Error::ThreadPool(threads, reason)` (feature `parallel`) when the pool
  cannot start.
- `prove_goal` serves the parallel path too: any goal, not only the
  roots, since the parallel layer wraps `focus::search_goal`'s engine and
  the fork-join happens wherever the branch meets its first choices.
  `Interactive::close` therefore runs on the pool when its options say so,
  its client's stop closure polled by the driver. The net engine works on
  the roots only, as before.
- The additive path stays sequential in every case: it is linear-time in
  the product of the two formulas' sizes and has nothing to share out.
- D15 for the front ends: `jobs` and `portfolio` are fields of the one
  `Options` value; the CLI maps `--jobs`/`--deterministic` onto `jobs`
  (`--deterministic` wins), the web front end leaves the feature off and
  `jobs` at one, a wrapper on a server sets `jobs` from its own budget.
  `Options` has no serde (it never had; it is not output), so a JSON
  settings object maps onto the setters as the CLI does.
- `Outcome::statistics` on a parallel run adds every worker's counters,
  so `nodes` is the work done by all threads, not what one thread would
  have done; the memo's `hits` and `peak` are read off the shared table
  once (`peak` summed over the shards, an upper bound). The CLI test that
  pins `--stats` counts passes `--deterministic`.

## The runtime

`search/parallel.rs`: `Runtime::new(threads, stack_size)` builds a
`rayon::ThreadPool` (`num_threads` set, so `RAYON_NUM_THREADS` is never
consulted; `stack_size` set, so a worker recurses as far as the caller's
thread would); `Runtime::drive(stop, work)` runs `work` in the pool from
an `in_place_scope` on the calling thread and, while it runs, waits on a
channel with a one-millisecond timeout, polling the caller's closure at
each timeout and raising the root `AtomicBool`. A panic in a worker
propagates through the scope to the caller. `Flags` is the chain of stop
flags a worker polls: its choice's cancel flag, then the ancestors', then
the root; `Stop` (in `search/mod.rs`) is the engines' stop condition,
`Closure` for the sequential path and `Flags` for a worker, so the engines
have one field and one poll site each.

Why polling on the calling thread rather than an `Arc<AtomicBool>` in
`Options` or a `Send` bound on the closure: `prove_goal`'s signature and
`Interactive::close`'s stay as they are for every caller (the CLI's
closure holds a clock and a counter, the interactive layer's client's
whatever it likes), the atomic flag is reachable from that closure without
a global, and one poll a millisecond costs nothing. The one consequence
is in the CLI: its closure looked at the clock every 1024 polls to keep
the clock off the hot path, which on the parallel path would have made a
time limit late by a second, so it looks every poll when `jobs > 1`
(`polls_per_clock`).

## The focused engine on the pool

The engine keeps its shape; the parallel layer (`focus/parallel.rs`) is
a second `impl Engine` block plus four hooks in the sequential code, each
one `if self.cubes() { return self.…_parallel(…) }` behind the feature:
in `with` (the `&` rule), in `decide_with` after the candidates and the
copies are listed (the copies are now listed before the candidates are
tried, which changes nothing sequentially: the two loops were
independent), in `focus_on`'s `⊕` arm, and in `split` after the sides and
tallies of the free enumeration are set up, whose loop became
`enumerate_split(start)` so that a task can enumerate the members from
`start` on.

- **A worker** (`Spawn::worker`) is an `Engine` over the same forest,
  reading, counts and rules by reference, the shared memo and arena, the
  spawning engine's live branch stack copied, its `depth`, `copies` and
  `or_depth`, fresh pools and counters, `exhausted` clear and
  `dependency` none. The stack copy is what the step-7 report asked for:
  the loop check keeps the prunes of the cube's ancestors. `Engine::new`
  now takes the counts, the rules, the memo (`Table::Own` or
  `Table::Shared`) and the arena (`Arena::Own` or `Arena::Shared`) from
  outside, and `search_goal` builds them; the sequential path is
  otherwise unchanged (one enum match per memo access and per push).
- **A choice** (`choose_parallel`): the engine that meets it spawns a
  worker per alternative but the first into an `in_place_scope` of the
  pool and runs the first alternative on one more worker on its own
  thread; a proof or an error raises the choice's cancel flag at once,
  so the siblings return `Stopped` at their next stable sequent. Every
  alternative runs on a worker because only a worker's stop chain holds
  the choice's flag: the first version ran the first alternative on the
  spawning engine itself, whose chain does not, and the review below
  measured a whole refutation spent on an alternative a sibling had
  already settled. Only the
  first `LEVELS = 2` choices of a branch do this; below, a worker is the
  sequential engine, so the tasks are few and large (Karp–Zhang's
  "steal near the root"), and rayon's work stealing supplies "when the
  pool has idle workers": a task nobody steals runs on the spawning
  thread after its own alternative, so a choice met on a busy pool costs
  a worker's construction and nothing else.
- **The merge** (`Collected::take`, the match at the end of
  `choose_parallel`) is the sequential rule's: a proof from any
  alternative wins (also over another's error, so the pool may decide
  where one thread gives up with `RecursionLimit`; the report of step 3
  had that as the sequential behaviour, and it stays so on one thread),
  else the first error that is not a `Stopped` caused by the choice's own
  cancellation, else a failure with `exhausted` or-ed and `dependency`
  min-ed over the alternatives that ran to their end. A cancelled
  alternative's flags are dropped, as the sequential engine would never
  have run it. `prove_stable` around the choice is unchanged: it records
  `Failed(Complete)` only if no dependency survived, exactly as before.
- **The `&` rule** (`with_parallel`) runs the right premise on a worker
  of the pool and the left one on a worker on the spawning thread, the
  first to fail or err cancelling the other; a failed premise decides and the other's flags are dropped, both
  count when both ran to the end. The `⊗` premises stay sequential: the
  first premise usually fails at once on the counts, and and-parallelism
  there would spend a worker per split.
- **The splits** (`split_parallel`): `fixed = min(members, clamp(log2(2 ×
  threads), 1, 6))` members are assigned by the task's pattern, the rest
  enumerated by the task in Gray-code order from the pattern's state; the
  union over the patterns is the sequential enumeration's set of splits,
  the empty submask included (pattern zero starts from the sides the
  sequential code starts from, the goal fixed on the consequent's side in
  intuitionistic mode).
- **Mix** stays sequential and last (`last_resort`), after the parallel
  alternatives failed.
- **The copy bound**: `run` deepens on the root engine, which is alone
  until its first choice; every task of a level has ended when the level's
  `exhausted` is read, so the or-reduction over the workers is the merge
  above, and a level is `Unprovable` only with every worker's flag clear.
  Levels never overlap.
- **The portfolio**: a worker's `seed` (from the spawning engine's seed
  and the alternative's index, never zero) replaces the id in the order
  of candidates and copies within their classes (`rank`), so workers
  below the levels explore in different orders; the first alternative of
  a choice keeps the spawning engine's order, so a run with one thread is
  unchanged.

## The memo under concurrency

`memo::Shared` is 64 shards of the sequential `Memo` behind one
`Mutex` each, a key's top hash bits choosing the shard (foldhash with the
crate's fixed seed, as the map itself), the cap split among the shards
(a full shard is cleared as a full memo was). `Memo::insert`'s merge
rule, run under the shard's lock, is the compare-and-swap the spec asks
for: a larger `Exhausted` budget wins, `Complete` and `Proved` win over
`Exhausted`, a proof stays; so an entry's validity only grows whatever
the interleaving, `bound_remaining` never shrinks, and two workers
deciding one sequent cost duplicated work, never a weaker entry. A lock
is held for one map operation, never across a call into the engine.

The arena is shared the same way (`Arena::Shared`, one `Mutex<Vec<Node>>`,
the lock held for one push): a node's id is its position in the one arena
every `Proved` entry refers to, a node's premises are pushed before it by
whichever worker built them, so the order `Proof::new` needs holds across
threads, and the memo insert follows the push, so a hit always finds a
complete subtree; the arena is append-only, so a cleared shard never
leaves a dangling id. Pushes are as many as rule instances on successful
branches, far fewer than nodes visited, and the table below shows no
contention worth a per-worker arena with a relocation pass.

`dashmap` was not taken: the notes record its maintenance as thin, the
shards are twenty lines over the existing type, and the speedups measured
leave no contention to buy back with a dependency. Sharding by
`Mutex<Memo>` also keeps `hits` and `peak` per shard for free.

## The net engine on the pool

`Engine::explore(limit, cubes)` is the sequential loop with one extra
branch: a branch whose links reach the limit is recorded as a cube (its
links, in order) and taken back instead of followed; `run` is
`explore(None)`. `seed(links)` makes a cube's links on an empty
structure, `reset` takes every link back and clears the frames, keeping
the counters. The driver (`net::parallel::search`) enumerates on a root
engine at depth 1, 2, … until there are `CUBES_PER_THREAD = 16` cubes per
thread or the enumeration decides the sequent by itself (a proof within
the limit, or no branch surviving the tests, which is `Unprovable`); the
enumeration repeats the shallower depths, a geometric cost next to the
cubes. Then `threads` tasks pull cubes from an atomic counter with one
engine each (the per-worker structure and scratch allocated once, reset
per cube); a worker that finds a net stores it and raises the flag,
`Unprovable` needs every cube to have ended `Ok(false)`, and any error or
a real stop is `Unknown`. The choice order (fewest admissible partners
first) is inherited, so the cubes are the small-multiplicity atoms first
as the spec asks; the exact test's cadence counts the seeded links, so a
cube's worker tests exactly where the sequential search would. No state
is shared beyond the flags.

## Speedups

Release build, `cargo test -p linlog --release --features parallel
speedups -- --ignored --nocapture`, on the machine of this session
(sixteen hardware threads). Wall-clock time and, in brackets, the nodes
visited by all threads together; the sequential column is the reference.

Focused engine (`Options::copies` 3; the counter program is the Horn
test's, eight tokens, reusable clauses):

| instance | verdict | 1 thread | 2 | 4 | 8 | 4, portfolio | 8, portfolio |
|---|---|---|---|---|---|---|---|
| 3-Partition, solved (`[1,2,3,1,2,3]`, 2 bins of 6) | proved | 0.69 ms (25) | 2.2 ms (59) | 2.7 ms (125) | 2.3 ms (172) | 2.5 ms (74) | 2.3 ms (118) |
| 3-Partition, refuted (`[1,1,1,3,1,1]`, 2 bins of 4) | unprovable | 45.3 s (1.83 M) | 24.4 s (1.83 M) | 12.8 s (1.93 M) | 7.59 s (1.92 M) | 12.9 s (1.84 M) | 7.43 s (1.85 M) |
| counter 8 ⊢ d | proved | 134 ms (1.76 M) | 154 ms (2.64 M) | 113 ms (2.89 M) | 99 ms (3.22 M) | 124 ms (3.11 M) | 97 ms (3.22 M) |
| counter 8 ⊢ d ⊗ a | unknown (copy bound 3) | 487 ms (6.90 M) | 466 ms (7.24 M) | 367 ms (7.35 M) | 335 ms (7.79 M) | 388 ms (7.35 M) | 334 ms (7.76 M) |
| counter 8 ⊢ d ⊗ a, affine | unknown (copy bound 3) | 85 ms (0.66 M) | 68 ms (0.68 M) | 66 ms (0.73 M) | 95 ms (0.83 M) | 46 ms (0.81 M) | 73 ms (0.96 M) |

Net engine (`--engine net`, Matsuoka's Partition encoding of the net
tests):

| instance | verdict | 1 thread | 2 | 4 | 8 |
|---|---|---|---|---|---|
| Partition `[2,3,2,1]` | proved | 4.55 s (345 k) | 2.14 s (337 k) | 1.06 s (324 k) | 0.61 s (339 k) |
| Partition `[1,2,5]` | unprovable | 3.55 s (294 k) | 1.79 s (294 k) | 0.92 s (294 k) | 0.53 s (294 k) |

Where it pays and where it does not:

- **Wide or-trees with little sharing scale almost linearly**: the
  3-Partition refutation (a `⊗` split per clause, every branch its own)
  gains 1.9× on 2 threads, 3.5× on 4 and 6.0× on 8, with the nodes
  visited up by 5 %, which is the duplicated work of racing workers and
  of alternatives cancellation reaches late; the net engine's Partition
  instances gain 2.1×, 4.3× and 7.4× (proved) and 2.0×, 3.9× and 6.7×
  (refuted), with the nodes unchanged on the refutation, since cubes
  share nothing and a cube is never doubled.
- **Memo-bound searches gain little or nothing**: the counter programs
  spend their time in stable sequents most of which the memo answers,
  and the and-parallel `&` premises and the parallel copies visit up to
  twice the nodes (1.76 M to 3.22 M on the proved instance) for a gain of
  1.35× at 8 threads and a loss at 2 (1.45× on the undecided one); the
  shared memo is not the limit (the shards' locks are uncontended at
  these rates), the duplicated exploration is. The affine instance is
  too small to measure past the pool's start-up (a few milliseconds).
- **Small instances lose**: a solved 3-Partition takes 0.7 ms on one
  thread and 2 ms on any pool, the pool's threads and the workers'
  construction costing more than the search. The CLI's default of every
  core is still right, since the difference is milliseconds; a caller
  that proves many small goals should keep `jobs` at one.
- **The portfolio brings nothing consistent** (12.9 s against 12.8 s at
  4 threads, 7.43 s against 7.59 s at 8 on the refutation; 124 ms
  against 113 ms and 97 ms against 99 ms on the proved counter): with
  cube-and-conquer the workers already explore disjoint regions, and a
  reordering below the levels only reshuffles the order within a region.
  It stays available and off.
- **The cancellation fix changed none of these numbers** beyond noise
  (the same families measured before it were within 5 %): none of them
  has a proof on one alternative while a sibling refutes; the review's
  `⊢ H ⊕ (a ⅋ ~a)` is the case it changes, from the whole refutation of
  `H` (26 s on 2 threads) to under a millisecond.

## Decisions where the prompt left room

- **Nested fork-join over a static cube list for the focused engine.**
  The prompt's "cube-and-conquer over the first two levels of or-choices"
  could be a static enumeration of (stable sequent, alternative) pairs
  followed by a worker pool and a re-run of the sequential engine on the
  primed memo. Fork-join at the choices was chosen because the `&` rule
  reaches several stable sequents before any choice, so a static
  enumeration would need a mode of the engine that records choices
  without deciding, and because it gives the and-parallel `&` and the
  proof assembly for free through the shared arena; the cost is one
  engine construction per task, which is a few allocations.
- **The first alternative runs on the spawning thread.** Every closure a
  pool task runs must be `Send`, and the engine's stop condition may be
  the caller's non-`Send` closure, so the spawning engine cannot be moved
  into a task; `in_place_scope` lets its thread run one alternative
  (on a worker, so that the choice's cancel flag reaches it) while the
  tasks run, which keeps the thread busy without a task of its own.
- **A shared arena rather than per-worker arenas.** `Proved(NodeId)`
  entries in a shared memo must mean the same node to every worker;
  a shared `Vec` behind a mutex gives that with no relocation, and pushes
  are rare next to nodes visited.
- **`Options::jobs` rather than a runtime value in the API.** D9 lists the
  thread count among the options; a pool per call keeps `Options` plain
  data with no global, at the price of starting the threads per call
  (tens of microseconds per thread), which an interactive session pays
  per `close`.
- **The pool's stack is `Options::stack_size()`**, moved from the CLI into
  core so that the two thread kinds cannot disagree.
- **`--deterministic` takes precedence over `--jobs`** rather than
  conflicting with it, so a script can add it to any command line.
- **The default of `--jobs` is the machine's parallelism**, as the prompt
  asks ("on by default in the CLI").
- **The portfolio is off by default**: see the table.

## Deviations from the prompt, with reasons

- **Per-worker scratch is the worker's pools, not a bump arena.** The
  engines already allocate nothing per node once their pools are warm;
  a worker's pools warm in its first few nodes. `bumpalo` would add a
  dependency for the allocations of the first nodes of each task.
- **No thread sanitizer run.** The toolchain is stable Rust
  (`rust-toolchain.toml`), and `-Zsanitizer=thread` needs nightly and a
  rebuilt standard library; the code has no `unsafe`, every shared value
  is behind a `Mutex` or an atomic, and rayon's scopes give the
  happens-before edges the merges rely on.
- **Timing harness rather than criterion.** Two ignored tests print a
  table in release mode, as the earlier steps' timings did; criterion
  would add a dev-dependency for numbers the plan's step 14 harness will
  supersede.
- **`Unknown(RecursionLimit)` may become `Proved` on the pool** (another
  alternative found a proof while one hit the limit). This is a
  different `Unknown`-to-decided outcome, not a different decided
  verdict; the tests treat it as allowed.

## The review

A fresh-context reviewer read the parallel layer against the rules and
the sequential engines, ran the in-repo parallel tests, and built a
differential fuzzer outside the repository (random sequents over eight
classical rule sets in linear, Mix and affine mode and four
intuitionistic ones in linear and affine mode, identity seeds and
mutants, copy bounds 1 to 3, memo limits from 0 to 2²⁰, one case in four
with a recursion limit of 24 to 63, 2, 3, 4 and 8 threads with and
without the portfolio, unit-free MLL also with each engine forced; every
proof checked, every net through `is_correct`): one seed complete, about
3 000 sequential cases against 12 000 parallel runs, and two more seeds
cut off by the reviewer's own timeout after 1 000 and 500 cases (8 300
parallel runs more, the second seed drawing the net engine's test period
from 1 to 5, so the cubes met the exact test at every cadence), no
verdict mismatch and no check failure anywhere. It found the seven points it was asked about sound (the merge
in `Collected::take` and the two matches, the stack and depth copies,
the split chunking, the copies' budget, the shared memo and arena, the
net cubes and the cadence, the portfolio as an order only), and two
things to fix or note:

- **The alternative run in place was never cancelled** (medium, a cost
  and not a verdict): the spawning engine's stop chain does not hold the
  choice's flag, so on `⊢ H ⊕ (a ⅋ ~a)` with `H` the refuted 3-Partition
  instance the whole refutation of `H` ran on every thread count (51 s,
  26 s, 15 s on 1, 2, 4) while the `⊕`'s other side had proved the
  sequent in microseconds. Fixed as described above: every alternative
  and every `&` premise runs on a worker; the tests pass, and the
  measurement is repeated below.
- **The recursion counter bounds the frames of one task, not of a
  thread** (low): a pool thread waiting at a scope runs stolen tasks on
  its own stack, so its frames are the scope's (a choice near the root)
  plus the stolen task's, nested waits compounding. The margin of
  `Options::stack_size` (twice the measured cost per level, at least
  8 MiB) covers the default limit; a raised limit is where an overflow
  would first show. Recorded in `.claude/rules/core.md`; no code change.

## Open questions and follow-ups

- **A per-worker arena with a relocation pass** if a future profile shows
  the arena's lock: `Proof::new` already renumbers, so a merge of
  arenas by `(worker, index)` ids is a bounded change.
- **The additive path** stays sequential by design.
- **The pool per call**: a caller that closes many small goals could
  keep a pool; that needs a `Runtime` in the public API, which D9's
  plain-data options do not offer. Revisit with the web front end's
  server, if any.
- **`nets/mod.rs` has two unused test imports** under
  `--no-default-features --all-targets` (`cargo clippy -p linlog
  --no-default-features --all-targets`), which no documented check runs;
  pre-existing, untouched.
- **The parallel tests take about a minute in debug builds** (five
  configurations per sample); trim the samples if the suite's time
  matters more than the coverage.

## Verification

At the last code change, on top of which this report sits: `cargo clippy
--workspace --all-targets -- --deny warnings`, `cargo test --workspace`
(which builds core with the feature, since the CLI depends on it, so the
five parallel tests run there: 111 core tests), `cargo test -p linlog
--no-default-features`, `cargo hack check --each-feature -p linlog`,
`cargo hack check --feature-powerset --depth 2 -p linlog`, `cargo deny
check` (advisories, bans, licenses, sources), and `nix flake check` ("all
checks passed!", the export and Rocq checks included) all pass; the
in-repo parallel tests were also run in release mode, and the reviewer's
fuzzer as described above. The first `nix flake check` failed on the net
engine's stop test, whose first sequent was provable and could be decided
before the driver's first poll on a fast build; the test now stops a
Partition refutation that takes seconds. No thread sanitizer was run (see
the deviations). The timings come from the two ignored `speedups` tests
in release mode on an otherwise idle machine.

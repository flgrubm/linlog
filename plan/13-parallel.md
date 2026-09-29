# Step 13: parallel search

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` (D7, D9, D10, D11) and every report in `plan/reports/`,
  especially 03, 06, 07 and 08 (the engines' state and memo designs).
- `proof-search-specifications.md`: every "Parallelization" subsection
  (MLL, MALL, MELL, intuitionistic) and "Cross-cutting engineering notes" §
  "Parallel runtime".
- `plan/notes/export-targets.md` § Crates (rayon, dashmap and the wasm
  caveat).
- `.claude/rules/core.md`, `core/src/search/**`.

## Goal

A `parallel` feature of `linlog` (on by default in the CLI, off for wasm)
that runs the engines on several cores as the spec describes: cube-and-
conquer at the root for the or-choices, and-parallel `&` premises sharing
the memo, an atomic stop flag polled at every `prove` entry, a shared memo
(sharded, with the bound-carrying entries of step 7 updated by compare-and-
swap so `bound_remaining` only grows), per-worker scratch allocated once.
`--jobs N` and `--deterministic` in the CLI; the sequential engines stay
the default under `--deterministic` and the reference in tests.

## What step 3 left you

`plan/reports/03-focused-engine.md`, "For step 12" (now this step): `Memo` is one type
with `get`/`insert` that a sharded map replaces; the stop closure polled per
stable sequent is where the stop flag goes; `Engine` holds all state and its
pools, so one engine per worker; there is no global.

## What step 8 left you

The two-sided engine is the focused engine given a `Reading`
(`Engine::TwoSided` names the configuration), so whatever cube-and-conquer
does to the focused engine covers intuitionistic mode; the reading is
read-only and shared. The additive path (`search::additive`, a memoized
recursion on subformula pairs) is small enough to stay sequential; say so
rather than parallelizing it.

## What to build

1. **Runtime**: one rayon pool sized by `--jobs`; the stop flag; the sharded
   memo (`dashmap` or 64 shards of `Mutex<HashMap>`; choose by measurement
   and by the maintenance status in the notes); per-worker scratch.
2. **Focused engine**: cube-and-conquer over the first two levels of
   or-choices (decide, `⊕`, `⊗`-split), and-parallel `&` premises when the
   pool has idle workers, a portfolio option (different focus orderings and
   atom biases per worker) as the cheapest first win.
3. **Net engine** (its report, "For step 13": everything is in `Engine`,
   `ProofStructure` and `Scratch` are `Clone`, the choice order is
   deterministic, so a worker is a clone after a prefix of links; add a
   `seed` entry that takes a list of links): cubes from the first `d` link choices with `d` chosen for
   8–32× more cubes than cores, smallest-multiplicity atoms first, no shared
   state beyond the stop flag.
4. **Exponentials** (step 7's report, "For step 13"): one deepening
   level at a time, never concurrently; parallelism within a level; the
   memo's three kinds of entry merged so that validity only grows (a
   larger `Exhausted` budget wins, `Complete` and `Proved` win over
   `Exhausted`), which is the compare-and-swap; the `exhausted` flag
   or-reduced over the workers before a level is called failed; a worker
   that starts from a cube starts with the branch stack of the cube's
   ancestors, or the loop check loses those prunes.
5. **Correctness**: every result still passes the checker; `Unprovable`
   still requires the whole space to have been searched by some worker with
   no cube abandoned; document how cancellation and `Unknown` interact.
6. **Determinism**: `--deterministic` selects the sequential engines; the
   parallel path may return a different proof but never a different verdict.
   Test both statements on the generator samples with several thread counts.
7. **Measurement**: a benchmark (criterion as a dev-dependency through the
   `new-tool` skill, or a simple timing harness) on the hard families from
   the earlier steps, sequential versus 2, 4, 8 threads; report the numbers
   and where parallelism does not pay.
8. **Documentation**: `.claude/rules/core.md` (the runtime's invariants,
   what must be polled and where), CLAUDE.md (feature, flags, the
   `cargo hack` run now covering `parallel`), README.

## Constraints

- No `unsafe`; no lock held across a recursive call; no global state other
  than what the pool owns.
- `cargo hack check --feature-powerset -p linlog` must pass with and without
  `parallel`; `core` without the feature has no rayon in its tree.
- Keep the sequential code paths intact and readable; the parallel layer
  wraps them rather than forking them.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace` (with and without the feature),
`cargo hack check --feature-powerset -p linlog`, `cargo deny check`,
`nix flake check` at the end (`jj st` first). Run the tests under a thread
sanitizer if the toolchain makes it easy; else say so.

## Deliverables

- Thematic jj commits ("Add the parallel runtime behind a feature",
  "Run the focused engine as cube-and-conquer", "Parallelize proof-net
  search", "Measure parallel speedups", …).
- `plan/reports/13-parallel.md`: API and flags, the memo design under
  concurrency, the speedup table, decisions, deviations, open questions.

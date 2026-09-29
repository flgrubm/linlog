# Step 14: benchmarks, LLTP input and the hard families

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/README.md` and every report in `plan/reports/`.
- `proof-search-specifications.md` § "Cross-cutting engineering notes":
  "Testing strategy" (known-hard families) and "Benchmark harness", and the
  References for LLTP/ILLTP (github.com/meta-logic/lltp) and the hard-family
  papers.
- `.claude/rules/ci.md` if CI changes.

## Goal

A benchmark harness that reads the LLTP/ILLTP problem format and linlog's
own, runs each problem with a timeout on a fresh engine, records the
outcome, wall time, node count and memo size as CSV, and reports
solved-within-timeout counts per family; the hard families from the spec
generated on demand; and a place where the numbers are tracked so a later
change shows its effect.

## What to build

1. **LLTP reader**: the `fof(...)` syntax used by LLTP/ILLTP (read the
   repository's format description and a few files; WebFetch is allowed),
   parsed into `Sequent` with mode flags (ILL problems are two-sided). Put
   it behind the `parse` feature in `core` or in the bench crate; decide by
   whether the web front end could want it.
2. **Family generators**: Kanovich's Horn encoding of 3-Partition and
   Matsuoka's encodings for MLL, the LMSS QBF encoding for MALL (Chaudhuri's
   qbf suites are the model), and a Petri-net style !-Horn family for MELL;
   each as a function from a size to a `Sequent` with a known verdict.
3. **The harness**: `linlog bench` or a `bench/` workspace crate (D4 allows
   it; choose by whether the CLI should carry the extra dependencies), with
   a per-problem timeout (a fresh thread or process per problem so a hung
   engine cannot take the run down), CSV output, a summary table, and the
   options of `prove` (engine, jobs, copies). Follow the spec's harness
   description.
4. **Problem sets**: a small curated set committed under `bench/problems/`
   (license permitting; LLTP's license must allow redistribution, else
   fetch on demand and document), the generated families at a few sizes,
   and the sequents from the earlier steps' slow tests.
5. **The known slow cases as benchmark families.** Step 3's report
   ("Performance observations", "Open questions and follow-ups") names
   them: refuting a wide sequent under Mix costs about `3^k`; refuting an
   unsolvable 3-Partition instance takes 55 s for bins of size 4 (3.9
   billion splits) because the atom bias makes the Horn clauses' bodies
   negative and every `⊗` split is enumerated. Make both families part of
   the harness at several sizes so that the performance pass after this
   step (branch-and-bound splitting instead of Gray-code enumeration, a
   per-problem atom bias, the tighter counts the report lists, memo keys in
   an arena) has numbers to beat. Measure, do not fix here.
6. **Tracking**: a `bench/RESULTS.md` (or CSV) with the current numbers per
   family and engine, and the command that regenerates it; not a CI job
   (timings on shared runners are noise), but a `nix flake check` entry
   that runs the harness on a tiny set to keep it building.
7. **Documentation**: README (how to run the benchmarks), CLAUDE.md
   (commands, the bench crate), `.claude/rules/core.md` if formats were
   added.

## Constraints

- No `unsafe`; dependencies scoped to the bench crate where only it needs
  them.
- Deterministic problem generation (seeded).
- Report numbers faithfully, including families where the engines time out.

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, `cargo deny check` if a dependency changed,
`nix flake check` at the end (`jj st` first), and one full benchmark run in
release mode with the table pasted into the report.

## Deliverables

- Thematic jj commits ("Read LLTP problems", "Generate the hard families",
  "Add the benchmark harness", "Record baseline results", …).
- `plan/reports/14-benchmarks.md`: how to run, the baseline table, what the
  numbers say about each engine, decisions, deviations, open questions, and
  what step 15's candidates would have to beat.

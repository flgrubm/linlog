# Step 25: a batch mode, and LLTP input for the command

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. This prompt is finished
at the review of step 24. Read before you start:

- `plan/later.md`: "A batch mode for the CLI" (its six headings are the
  requirement), and under "Follow-ups: the benchmarks" the wrong
  headers.
- `plan/reports/17-assessment.md`: 2.2, 3.12, and the author's answers 1
  and 9.
- `plan/reports/19-time-limits.md`, `20-memory-and-boundaries.md`,
  `21-defaults.md`, `23-api.md`, `24-command-harness-docs.md`.
- `plan/README.md`: D15, D16.
- `core/src/lltp.rs`, `cli/**`, `bench/src/problems.rs`, `bench/src/run.rs`.

## Goal

`linlog prove` decides many sequents in one call: a file of them, several
files, a directory, a stream on standard input answered line by line, in
the command's own syntax, in the harness's problem files and in the LLTP
library's. It is what research use looks like, what an editor or a
script talks to, and what every later measurement by day runs through.

## What to build

1. **The batch as a library notion** (D15): problems in, results out,
   as iterators, with one options value; the command is its first
   caller.
2. **Input**: one sequent per line with comments and optional names;
   several `--file` arguments; a directory; the harness's problem files;
   LLTP files, which `lltp::read` reads and the command cannot take
   today, also for a single `prove`.
3. **Output**: one result per sequent in input order, as it is decided,
   as a line of text or a JSON Lines record with what `--format json`
   and `--stats` carry; for the drawing formats a directory. A malformed
   line is that line's error. The exit status is the worst verdict, by
   the order error, unknown, unprovable, proved.
4. **Limits**: the time limit per sequent and one for the whole batch;
   the memory bound of step 20 per sequent, which is what lets one
   process hold a batch; `--isolate` for a child per sequent where that
   is not enough.
5. **Cores** go across the sequents by default, one sequent per worker
   on the sequential engines; within one sequent when the batch is short
   or the user says so (D16: the default is sensible and both are
   flags). A pool is kept across sequents rather than built per call.
6. **A stream**: standard input read line by line and each answer
   flushed, so that the same command serves a program that asks, waits
   and asks again.
7. **The draft of the header report.** With LLTP input in the command,
   write `plan/notes/lltp-headers.md`: the 28 files whose headers
   contradict them, each with linlog's verdict, the checked proof as
   JSON or the classical countermodel, the bound it needed, and where
   the Maude prover's own result file agrees. The author sends it, not
   a session; the release step reminds the author of it.

## Constraints

- A batch's results are those of the single calls, verdict for verdict.
- The harness keeps its child per run: it measures, and a measurement
  wants isolation.
- No engine change.

## Verification

The checks of CLAUDE.md's table and `nix flake check`. The batch against
single calls on `bench/problems/slow-tests.txt` and on a sample of two
hundred LLTP problems, in both orders of cores; a timing of what a shell
loop pays against the batch (named here: two pinned cores, under ten
minutes, detached).

## Deliverables

- Thematic jj commits.
- `plan/notes/lltp-headers.md`.
- `plan/reports/25-batch.md`: the options and how each front end sets
  them, the timing, decisions, deviations, open questions.

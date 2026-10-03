# Step 31: the third baseline, and the first release

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step needs the
machine for a night, which the author gives; this prompt is finished at
the review of step 30. Read before you start:

- `plan/reports/14-benchmarks.md` ("What step 16 must repeat"),
  `16-baseline.md`, `17-assessment.md` (5.6, the author's answer 9 and
  what follows it), `21-defaults.md`, `24`, `30`.
- `plan/notes/distribution.md` (how and where to publish, what each
  registry asks, the repositories) and `plan/notes/lltp-headers.md`.
- `.claude/rules/bench.md`, `bench/baseline.sh`.

## Goal

The third baseline, comparable with the second under the flags the
second ran under and with a pass under the new defaults; and everything
a release needs prepared, so that the author's own acts (the tag, the
publication) are the last and smallest part.

## What is fixed now

1. **The baseline**, by `.claude/rules/bench.md` and the lessons of
   steps 14 and 16: the estimate by arithmetic, a word to the author
   before the night, no probes by day beyond those named.
   `bench/COMPARISON.md` gains the third column.
2. **The release, prepared**: version 0.1.0 in the manifests, a
   changelog, `CITATION.cff`, the crates' metadata and what docs.rs
   builds, a dry run of publishing, the release workflow, the Rocq
   library's opam file, as `plan/notes/distribution.md` lays out. The
   owner is the organization `linlog-prover` (D22): every address
   written is `github.com/linlog-prover/linlog`, and if the repository
   has not been transferred yet, the step says so first and lists what
   the note says a transfer touches. Pushing, tagging, transferring and
   publishing are the author's.
3. **The reminder** (the author, 2026-10-03): the final message of this
   step tells the author that `plan/notes/lltp-headers.md` is ready to
   send to the LLTP library's maintainers, and asks whether to send it.
4. **README** as the face of a released tool: installation, a first
   proof, where the documentation and the benchmarks are.

## Deliverables

Thematic jj commits; `bench/results/DAY/`; `plan/reports/31-baseline-release.md`.

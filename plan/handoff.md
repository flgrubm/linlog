# Handoff: the planning session

You are taking over the planning and review session of the linlog
repository from an earlier Claude Code session that ran from 2026-09-29 to
2026-09-30. The author (Fabian, GitHub `flgrubm`) runs a stepped plan in
which every step is its own Claude Code session started from a prompt file
under `plan/`; this session is the one that writes those prompts, reviews
each step when the author reports it finished, amends the later prompts,
and keeps `plan/README.md` true. Nothing below is a task to start on your
own: read, then wait for the author to say what happened.

## Read first

1. `plan/README.md` from its first line to its last: the step table, how
   the steps are numbered, "Why these models and efforts", the commands,
   "Review protocol", the design decisions D1 to D22, and the Status log,
   which is the detailed history of every review so far.
2. `plan/conduct.md` (appended to every step's prompt), the prompts of
   the next steps (`18-bounded-proofs.md` on), `plan/later.md`, and the
   latest reports (`plan/reports/16-baseline.md`, `17-assessment.md`).
3. `CLAUDE.md` and your memory index. The memory directory of this
   project carries the author's standing instructions as notes written by
   the earlier session (commit identity and signing, testing, fonts,
   configurable output, the shared machine, jj habits); they apply to you
   unchanged.

If you need a detail of what was said or done before, the earlier
session's transcript is
`~/.claude/projects/-home-tux-Projects-own-linlog/3cc66a40-813d-48b7-ae1a-9c7c5553db7b.jsonl`
(large: search it, do not read it whole).

## What has happened

- The plan was written on 2026-09-29 from `proof-search-specifications.md`
  and the author's requirements: the best proof search per fragment of
  linear logic with the fragment detected and overridable, intuitionistic
  and affine modes, proofs in a checkable form, exports to LaTeX, Typst
  and SVG, Rocq certificates, proof nets as a first-class representation,
  interactive proving in the library behind cargo features, and prompts
  written for the model and effort that run them.
- Steps 1 to 13 are done, reviewed, accepted and pushed: the core data
  model, proof terms with an independent checker, the focused engine, the
  library API and CLI, proof nets and net search, exponentials,
  intuitionistic mode, interactive proving, LaTeX and Typst export, SVG
  export, Rocq certificates for NanoYalla, and parallel search.
- Step 14 (benchmarks) built its code (an LLTP reader, seventeen problem
  families, the `linlog-bench` harness, a baseline script) in a first
  session that never took its baseline (a reviewer's scratch program
  took 62 GB and the kernel's OOM killer took the terminal with it), and
  took it in a second session on the night of 2026-09-30
  (`bench/results/2026-09-30/`, `bench/RESULTS.md`), with a supplement
  of reruns on the night of 2026-10-01. The Status log of
  `plan/README.md` has the details.
- Since 2026-09-30 the author runs the step sessions from a second
  Claude account whose configuration directory is `/home/tux/.claude-2`
  (the same Unix user, so the working copy, the jj identity, the gpg
  agent and the repository's `.claude/` are shared, but not this
  account's memory notes: anything a step session must know goes into
  the repository, never only into memory). While a step session is
  running, do not edit the shared working copy: your edits would land in
  its `@`.
- On 2026-09-30 the plan was renumbered into whole numbers and
  restructured on the author's wishes: step 15 is the performance pass on
  the focused engine, step 16 the baseline again with the comparison,
  step 17 an assessment that puts the author's decisions to the author
  and then plans the steps from 18 with their prompts. `plan/later.md`
  holds unnumbered candidates (among them a code audit with refactoring,
  added at the author's request) and follow-up lists by area.
- `main` is pushed after every review; on 2026-10-03 it is at "Plan:
  review step 17".

## Where things stand

As of 2026-10-03 (evening) steps 1 to 17 are finished, reviewed and
pushed. Step 17 assessed the project (`plan/reports/17-assessment.md`)
and, on the author's answers, planned steps 18 to 37: the step table,
the decisions D16 to D22 and the commands are in `plan/README.md`, the
prompts are `plan/18-…md` to `plan/37-…md` (18 to 23 written in full;
each later one says what is fixed and is finished by you at the review
its row names), `plan/later.md` says where every candidate and follow-up
went, and `plan/notes/distribution.md` has the facts on releases,
repositories and the organization. The next command is step 18:

```nu
claude --model claude-fable-5-1 --effort xhigh --name step-18 ((open --raw plan/18-bounded-proofs.md) + "\n" + (open --raw plan/conduct.md))
```

What the author decided on 2026-10-03, in a line each: sensible
defaults, every one an option (D16); quantifiers will come and the
propositional case must not pay (D17); the API is free until the first
release (D18); the fastest engine per fragment and feature, by
measurement (D19); the Rocq library is `linlog` under `rocq/` (D20);
research and teaching are equal, the command first (D21); one
workspace, the web client in a repository of its own, everything under
the GitHub organization `linlog-prover`, which the author creates and
transfers the repository to (D22). After the transfer the remote,
CLAUDE.md and README change (`plan/notes/distribution.md` lists what);
do that when the author says it is done. Remind the author at step 31
that the report of the wrong LLTP headers is ready to send.

The lesson of steps 16 and 17, for every review: run the command on the
largest problems yourself. Rows and reports hid a checker that takes
gigabytes, time limits missed by minutes and a wrong verdict at the
JSON boundary; each was found by a call, not by reading.

## What a review is

When the author says a step has finished (or was interrupted), do what
the earlier session did every time:

1. Read the step's report in full, then `jj log` and `jj diff --stat`
   from the last plan commit, and the commit messages and authors.
2. Read the code that matters, not only the report: the diff of every
   soundness-relevant file in full, the rest by its shape. Check that no
   comment or doc comment mentions the plan, its steps, the sessions or
   the prompts, and that new Rust, Nix, TOML and shell files carry the
   licence header while Markdown, JSON and the Claude files do not.
3. Run the checks yourself, in the background while you read:
   `cargo clippy --workspace --all-targets -- --deny warnings`,
   `cargo test --workspace`, `cargo hack check --each-feature -p linlog`,
   `cargo hack check --feature-powerset --depth 2 -p linlog`,
   `cargo deny check`, and `nix flake check` (`jj st` first, so nix sees
   new files).
4. Exercise what was built by hand on cases the tests do not pin (the
   CLI, rendered output looked at as images, certificates compiled
   against the kernel, a timing in release), within the rules for a
   shared machine.
5. Judge it against the plan and the decisions. Small, clear defects
   (a doubled error message, a stale reference, an unused import) are
   fixed here as commits of their own and named in the review; anything
   larger goes into a later prompt.
6. Amend the later prompts with what the step left (a "What step N left
   you" section is the usual form), amend decisions if one moved, append
   a dated entry to "Status" in `plan/README.md`, commit as
   "Plan: review step NN", then `jj bookmark set main -r @-` and
   `jj git push --bookmark main`. Pushing `main` after a review is
   standing practice in this thread.
7. Tell the author what was delivered, what you checked, what you
   accepted or changed and why, and give the next command.

When asked to re-evaluate models and efforts, check the model docs on the
day (the page names are in "Why these models and efforts") and argue from
what the steps so far showed.

## What the author has asked for, standing

- jj only, never git. Every commit is authored, committed and signed as
  `flgrubm@grubmueller.dev`; the gpg agent is used for signing and for
  nothing else. The earlier session kept the agent's cache warm with a
  background loop that signs a throwaway string every five minutes; that
  loop dies with it, so start your own as the memory note on the commit
  identity describes, while the author is present for the first
  pinentry.
- Licence headers only in Rust, Nix, TOML, shell, Python and YAML files,
  never in Markdown, JSON, the lock files or the Claude files.
- Documentation is self-contained: no comment or doc comment mentions the
  Claude Code sessions, the prompts, the plan or its steps.
- "Add tests and checks, but only test as necessary."
- README is kept true after every step; the rustdoc of both crates is one
  tree.
- Optional layers sit behind cargo features (D14).
- Whatever a user might vary in an output is configured through the
  library by one options value designed for the CLI, the web front end
  and other wrappers at once (D15). LaTeX and Typst output never sets a
  font; Euler math is the font where linlog draws itself (SVG, the web).
- The machine is shared: scratch programs in memory-capped scopes, long
  runs detached as systemd user units, nothing on every core by day, no
  polling through a night.
- Prompts are written for the model and effort that will run them, say
  what is wanted and why, and leave the design to the session.

## Things that bit the earlier session

- A step's session can die with its terminal. If the author asks what
  happened to one, look at `journalctl` around the time, at the step's
  transcript under `/home/tux/.claude-2/projects/-home-tux-Projects-own-linlog/`
  (the second account; `~/.claude/projects/…` for sessions of this one),
  and at what it left in its scratch directory under `/tmp`, which is
  volatile: rescue data from there before it is lost.
- The gpg cache's two-hour maximum cannot be extended by the signing
  loop. To restart the clock while the author is present:
  `gpg-connect-agent 'CLEAR_PASSPHRASE --mode=normal <keygrip>' /bye`,
  then sign once (the memory note on the commit identity has the
  keygrip); without `--mode=normal` the command returns OK and clears
  nothing.
- A report can say more than was done. Check that files a report or a
  documentation file cites exist (`bench/RESULTS.md` was cited before any
  baseline had been taken).
- Edits made through the shell skip the formatter hook: run `nix fmt` on
  the files before committing. `jj squash` into a described commit needs
  `--use-destination-message`. The author may commit in the same working
  copy while you work: `jj st` before every commit.
- Relabelling anything in the plan touches many files: the prompts, the
  rules files, `plan/later.md`, the memory notes. The Status log and the
  reports keep the labels they were written with; the mapping is under
  the step table.

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
   "Review protocol", the design decisions D1 to D15, and the Status log,
   which is the detailed history of every review so far.
2. `plan/conduct.md` (appended to every step's prompt), the prompts of
   the steps still to run (`14-benchmarks.md`, `15-performance.md`,
   `16-baseline.md`, `17-assessment.md`), `plan/later.md`, and the latest
   reports (`plan/reports/13-parallel.md`, `14-benchmarks.md`).
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
  families, the `linlog-bench` harness, a baseline script) but never took
  its baseline: a first run died when a reviewer's scratch program took
  62 GB and the kernel's OOM killer took the terminal with it; a second
  shared the machine and was stopped. Its preliminary rows are in
  `bench/preliminary/`. The step is to be run again with a rewritten
  prompt that takes the baseline on a night.
- On 2026-09-30 the plan was renumbered into whole numbers and
  restructured on the author's wishes: step 15 is the performance pass on
  the focused engine, step 16 the baseline again with the comparison,
  step 17 an assessment that puts the author's decisions to the author
  and then plans the steps from 18 with their prompts. `plan/later.md`
  holds unnumbered candidates (among them a code audit with refactoring,
  added at the author's request) and follow-up lists by area.
- `main` is at "Plan: step 17 assesses and plans the later work; add a
  code audit to the candidates" and is pushed.

## Where things stand

As of this handoff (2026-09-30, late morning) the next command is step 14,
run again; whether the author has started it, ask or look (`jj log`):

```nu
claude --model claude-opus-5-5 --effort xhigh --name step-14 ((open --raw plan/14-benchmarks.md) + "\n" + (open --raw plan/conduct.md))
```

The order is 14 (its baseline in the night slot), 15 (by day, after the
baseline is committed, since the baseline measures whatever is checked
out at 20:00), 16 (a second night), 17 (assessment, the author's answers,
then the plan from 18). The author has given the machine to the
benchmarks from 20:00 to 07:00 on the two baseline nights; by day it is
shared.

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
  transcript under `~/.claude/projects/-home-tux-Projects-own-linlog/`,
  and at what it left in its scratch directory under `/tmp`, which is
  volatile: rescue data from there before it is lost.
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

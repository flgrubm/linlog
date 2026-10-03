# Step 24: the command, the harness, the flake and the documents in order

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. This prompt is finished
at the review of step 23, whose names it must use. Read before you start:

- `plan/reports/17-assessment.md`: section 1.1 in full (every stale
  claim and every limit nothing mentions is this step's to correct or
  document), "The command, the harness and the flake as a maintainer
  finds them", and section 1.4.
- `plan/reports/23-api.md` and `plan/notes/api.md`.
- `plan/README.md`: D15, D16, D18.
- `.claude/rules/cli.md`, `bench.md`, `ci.md`, `claude-infra.md`.
- `cli/**`, `bench/**`, `modules/**`, `.github/**`, `.claude/**`,
  README.md, CLAUDE.md.

## Goal

The command, the harness, the flake and every document say and do the
same thing, once. A maintainer who adds a format, a flag, a reason or a
check edits one place and the compiler or a test names the others.

## What to build

1. **The command.** `prove` and `interact` share their search flags, the
   building of `Options`, the stop closure and the verdict line; a new
   output format is one place, not four matches and three silent skips;
   the help texts that are wrong are right; `interact`'s words are read
   with quoting, its ASCII rule names are in its help, and `--file -` is
   refused; a closed pipe keeps the verdict; `--output` into a missing
   directory fails before the search. Exit statuses beyond 0 to 3 that a
   user can meet are documented or removed.
2. **Tests for the stated behaviours** that have none (`--timeout`,
   Ctrl-C, `--jobs`, `--bias`, `--memo-limit`, `--output`), sized like
   their neighbours, and **README's invocations pinned**: a test or a
   check runs every console block against the binary, so that no step
   needs a script of its own again.
3. **The harness.** Tests for `summary` and the comparison; the `bench`
   check fails on an `error` or `crash` row as well as on a mismatch;
   one configuration label; a table of the counters in `summary`, which
   step 29's oracle reads; a late verdict kept apart; the columns read
   by name. `bench/baseline.sh` takes what is hard-coded for the second
   baseline as parameters, `bench/reruns.txt` and the repeated
   experiments (`period-*`, `long-1`) are retired or regenerated from
   the last baseline's rows.
4. **The flake and CI.** The fragments the command prints by default
   compile in the `export` check; the systems `modules/systems.nix`
   declares are those CI builds, or fewer are declared; the constants
   that name pinned versions are checked against the pins.
5. **Every stale claim of section 1.1** corrected, in README, CLAUDE.md,
   the rules files, the devshell's menu, the skills and the help texts;
   the spec's two wrong statements moved to its errata.

## Constraints

- The command's output for a valid call does not change, except where an
  item above says so; each such change is a commit of its own and listed
  in the report.
- The CSV columns of the two baselines stay readable.
- No engine change; `core/` only where the command needs a function the
  library should have had.

## Verification

The checks of CLAUDE.md's table, `linlog-bench run --all-families
--timeout 5`, `nix flake check`, and `summary --before` on the two
committed baselines reproducing `bench/COMPARISON.md`.

## Deliverables

- Thematic jj commits.
- `plan/reports/24-command-harness-docs.md`: what changed for a user,
  what for a maintainer, decisions, deviations, open questions, what
  step 25 must know.

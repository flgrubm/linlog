# Step 16: the baseline again, and what the performance pass changed

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/14-benchmarks.md` (the first baseline: how it was taken,
  on what hardware and settings, what disturbed it, and "what step 16
  must repeat exactly") and `plan/reports/15-performance.md` (what
  changed in the engines, the target set's before and after, and which
  rows of the baseline to look at first).
- `.claude/rules/bench.md`, `bench/baseline.sh`, `bench/RESULTS.md` and
  the first baseline's directory under `bench/results/`.
- `plan/README.md`: the step table and the Status entries for steps 14
  and 15.

## Goal

The same baseline as step 14's, taken the same way on the engines as step
15 left them, and a comparison of the two that says, per family and per
LLTP collection, what the performance pass gained, what it cost, and what
it left as it was.

## The slot

The machine is the benchmark's from 20:00 to 07:00 on the night the
author names. Outside that slot it is shared, and the rules in "How to
work on this step" below apply.

## What to do

1. **By day: make the second run comparable with the first.** Read how
   the first was taken and check that the machine is as it was: the same
   power profile, governor and turbo state, the same pinning, the same
   timers and services in the slot (`systemctl list-timers --all`, with
   and without `--user`), the same library commit for LLTP, the same
   script and sizes. Where something differs, say so in the report rather
   than hide it; change the script only if step 15 added an option the
   baseline must exercise (an atom-bias knob is a configuration axis, a
   context wider than 63 members is now a run and no longer a refusal)
   or made a family's sizes too easy, in which case the old sizes stay
   and the new ones are added, so every row of the first baseline has its
   counterpart.
2. **The night.** Arm the unattended start as step 14 did, give the
   author the two blocks to paste (before leaving, and in the morning),
   the time the run should end, and the word to come back with
   ("continue"), and end your turn. Do not poll overnight.
3. **In the morning.** Read the journal: duration, the load at the start,
   the jobs that fired, the throttling, the rows that waited for a CPU.
   If the run did not finish, leave the rest for another night. Commit
   the new results directory and `bench/RESULTS.md` as "Record the
   baseline after the performance pass".
4. **The comparison.** `linlog-bench summary` over both directories, and
   from it a section of `bench/RESULTS.md` (or a file beside it) that a
   reader can take in at a glance: per family, the largest size decided
   before and after and the times of the sizes both decide; per LLTP
   collection and mode, the problems decided before and after and where
   the others ended (time limit, copy bound, recursion limit, width);
   the parallel columns and the portfolio before and after; and every
   row that got worse, with the reason if the counters give one. If the
   harness needs a comparison mode to print this, add it to `summary`
   rather than computing the tables by hand. Every verdict that differs
   between the two baselines on the same problem in the same mode is
   looked at: a decided verdict that changed is a bug in step 15 and is
   reported as such, at the top.
5. **Documentation.** README and the rules files where they quote
   numbers; `plan/later.md` where a candidate's premise changed (the
   net engine's targets, what is left for the inverse method and for the
   Petri-net route), since step 17 assesses the candidates from there.

## Constraints

- The engines are not touched in this step. A regression is reported,
  with its instance and both rows, not fixed.
- No run by day beyond one-minute checks of the script.
- Report faithfully: what did not improve, what got slower, which rows
  of either night were disturbed.

## Verification

`nix flake check` after the script or the harness changed, and again
after the results and the report are in (`jj st` first).

## Deliverables

- Thematic jj commits ("Record the baseline after the performance pass",
  "Compare two baselines in the summary", …).
- `plan/reports/16-baseline.md`: how the run was taken and how comparable
  it is with the first, the comparison tables, what the performance pass
  gained and cost, any verdict that differs, and what the numbers say
  the next engine step should be.

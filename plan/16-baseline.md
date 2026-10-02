# Step 16: the baseline again, and what the performance pass changed

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/14-benchmarks.md` (the first baseline: how it was taken,
  on what hardware and settings, what disturbed it, and "what step 16
  must repeat exactly") and `plan/reports/15-performance.md` (what
  changed in the engines, the target set's before and after, and which
  rows of the baseline to look at first).
- `.claude/rules/bench.md`, `bench/baseline.sh`, `bench/reruns.txt`,
  `bench/RESULTS.md` and the first baseline's directory,
  `bench/results/2026-09-30/`, whose `starts.txt` lists its two starts.
- `plan/README.md`: the step table and the Status entries for steps 14
  and 15.

## Goal

The same baseline as step 14's, taken the same way on the engines as step
15 left them, and a comparison of the two that says, per family and per
LLTP collection, what the performance pass gained, what it cost, and what
it left as it was.

## The slot

The author names the night and agrees the slot with you. The first
baseline had 20:00 to 07:00; this time the author expects to be able to
give the machine earlier in the evening, and the end is negotiable (the
author, 2026-10-02). So the slot follows from the run, not the run from
the slot: once your estimate is ready (item 1), tell the author how long
the whole run needs, propose a start and an end with a margin of half an
hour or so, and arm with what the author agrees to
(`bench/baseline.sh --arm --fresh --slot=HH:MM-HH:MM`). One slot that
holds the whole run is worth more than a tidy hour: a baseline finished
in one night has one start on one commit under one set of conditions.
An earlier start does not hurt the comparison, since the unit waits for
an idle machine on mains either way and the first baseline started at
21:23; what must hold is that the machine is the benchmark's for the
whole slot. Outside the slot it is shared, and the rules in "How to work
on this step" below apply.

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
   counterpart. Know how the first baseline came about: its stages 1 to
   3 ran on the night of 2026-09-30 on commit `b53cb17c6831`; a second
   night, 2026-10-01, added what the first had missed (the net engine's
   cubes on MLL 3-Partition and its one-thread Partition table) and the
   fourth stage, the killed or crashed runs of `bench/reruns.txt` again
   with more grace and memory into `*-generous.csv`, from a later commit
   of the harness alone (`core/`, `cli/`, the Cargo files and the
   toolchain identical, as `jj diff --from b53cb17c6831 --to <commit>
   --stat -- core cli Cargo.toml Cargo.lock rust-toolchain.toml` shows).
   Your one night runs the whole script, stage 4 included, with
   `bench/reruns.txt` as it stands, even where step 15's engines no
   longer need the room, so that the `-generous` files compare too.
   Have the author paste the evening block before 20:00, not after the
   run has started as on the first night.

   Check that the night still fits. The first baseline took 9 h 21 min
   for stages 1 to 3 and 1 h 28 min for its supplement, 10 h 50 min of
   an 11-hour slot, while the script's `estimate` says 10 h 30 min, so a
   run that starts at its latest start would be cut in stage 4. Step 15
   moves the durations in both directions: targets that timed out may
   be decided, and the 845 LLTP problems that were refused at once for
   a context too wide to split are now searched, each for up to its 5 s
   in both passes and again on every core with and without the
   portfolio. Estimate the night by day from step 15's target rows and
   the first baseline's (arithmetic, not a run), set `estimate` from
   it, and ask the author for a slot that holds it ("The slot" above).
   Only if no slot the author can give holds the run: do not drop rows
   the first baseline has, since the stages are ordered so that the
   stop cuts the parallel ones, and the script run again without
   `--fresh` finishes the baseline on a second night.
2. **The night.** Arm the unattended start as step 14 did, with the
   slot agreed, give the
   author the two blocks to paste (before leaving, and in the morning),
   the time the run should end, and the word to come back with
   ("continue"), and end your turn. Do not poll overnight. The timers
   are transient and a reboot drops them, as happened before the first
   baseline's second night: arm after the machine's last reboot, and
   have the author look at `systemctl --user list-timers` before
   leaving.
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
   reported as such, at the top. The rows that were `killed` or
   `crash` on the first night (a search that missed its stop inside a
   split enumeration, a proof arena that outgrew its cap; the first
   report explains both) are where step 15's fixes should show as clean
   `timeout` rows or decisions; count them. `summary` counts a verdict
   found after the limit as solved (two Petri nets of the first
   baseline's `lltp-recursion-generous.csv` are proved 21.6 s and 552 s
   into a 5 s limit, which the grace of stage 4 allows): the comparison
   keeps such late verdicts apart from those within the limit, in both
   baselines, and compares the `*-generous.csv` files by verdict, reason
   and time.
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

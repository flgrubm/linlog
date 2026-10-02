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

## What step 15 left you

`plan/reports/15-performance.md`, above all "The default bias" and "For
step 16". The focused engine now searches its splits by their counts,
chooses among equal members canonically, shares complete failures among
interchangeable sequents and picks its atom bias from the sequent's
shape; on a sequent with exponentials the default runs two searches, a
backward one (the rarer-literal rule) and a forward one (the factor
rule, on a Horn program within a copy bound of its own, 30), and
answers with the first that decides. The options are `Options::bias`
and `Options::forward_copies` (`linlog-bench run --bias
rarer|factors`, `--forward-copies`; columns `bias` and `forward_copies`
at the end of the rows). For this step that means:

- **The counters changed their meaning.** `splits` counts steps of the
  split searches where it counted submasks, and `memo_hits` counts a
  stable sequent once. Compare the two baselines' sequential rows by
  verdict, reason and time; the counters compare only within one
  baseline. What replaces them as the check on your set-up:
  `bench/targets.sh LABEL` by day (two pinned cores, about twenty
  minutes; named here, so it is a run you may make) must reproduce the
  decided rows of `bench/targets/after-bias.csv` exactly at the commit
  you measure.
- **The limits moved, and the night grows with them.** In the first
  baseline's intuitionistic pass 845 problems stopped at the width limit
  and 983 at the recursion limit, within milliseconds. The width limit
  is gone and the recursion limit rarely binds, and by step 15's sample
  most of those problems ran into their 5 s under the rarer-literal
  rule alone (21 of 24 and 15 of 24): about 2 300 timeouts in a
  sequential pass over the library where there were 984, three and a
  half hours for a pass that took 97 minutes. That is what the pass
  under `--bias rarer` will cost. Under the default the forward search
  decides many of those nets (on the sample 38 of 113 end at the time
  limit where 69 did), so expect something over a thousand timeouts and
  about two hours for each default pass.
- **The reruns take the first baseline's problems.** The script derives
  the problems of `lltp-copies-10`, `lltp-recursion`, `lltp-all-cores`
  and `lltp-portfolio` from the pass it has just run (`ended`). In a
  fresh directory that would rerun other problems than the first
  baseline did, and on every core far more of them: the set that timed
  out or took 50 ms would grow from 1 102 problems to about 2 800, four
  hours with the portfolio and four without. Make those four runs take
  their problems from the first baseline's rows
  (`bench/results/2026-09-30/lltp-intuitionistic.csv`), so that every
  row has its counterpart and the stages keep their length; what is
  slow only now is in the sequential passes already.
- **No loss is expected.** The planning session ran the engine on every
  LLTP row the first baseline decided (1 890 rows of its three
  sequential passes, 5 s): no verdict contradicts, and on an idle core
  none is lost. (Before the default ran both searches one was, the
  Petri net `IBM5964_1_1`: 1.8 s in the first baseline, 42 s under the
  backward search alone, 0.2 ms now; it will show in the `rarer` pass.)
  Your comparison lists every row decided before and not now, and any
  entry in that list is news.
- **The night measures the default users get, and its two components
  beside it.** The script's default passes are the combined default as
  they stand. Add two sequential passes over the intuitionistic library
  as the record of what the combination is made of: `--bias rarer`
  (`lltp-rarer.csv`: the backward search alone, which is the engine the
  first baseline's pass compares with row by row, and what the default
  gains over it) and `--bias factors --copies 30` (`lltp-forward.csv`:
  the forward search alone at the default's forward bound, and what the
  alternation costs on the rows it decides; 1.7 times the better rule
  in the median on the sample). They fit into stage 1's four streams,
  two of which are free after the first hour. On the sample the default
  proves 50 of 109 where the backward search alone proves 11; the
  contract the night checks at scale is that the default decides
  everything the `rarer` pass decides, short of the time limit.
- **Where the default is closest to its limit.** With no stop the
  default never decides less than the backward search; under a time
  limit it can lose what that search decides in more than two thirds of
  the limit. Of the rows the first baseline decided none is lost at
  5 s on an idle core, but the margin is thin on a few: the
  `NeighborGrid_z_2d_3n_1m_t_1_2_*` nets take 3.9 s (1.4 s in the first
  baseline, 2.2 s under `--bias rarer`), and on a loaded machine the
  planning session saw one of them at 5.2 s. In four streams with a
  throttling package they are the first rows to go; if they do, say so
  as a loss at the limit and give their `rarer` rows beside them.
- **`lltp-copies-10` bears on the default copy bound.** Its rows that
  are not Petri nets are what a bound of 3 still leaves undecided under
  either bias (five of the sample); report how many a bound of 10
  decides, for step 17's question whether `--copies` should rise.
- **The portfolio is measured once more** (`lltp-portfolio`), beside a
  default that already runs two searches side by side on a pool; step
  15 recommends removing the option if it shows no gain again. On every
  core the two searches get half the threads each, which nobody has
  measured: say what `lltp-all-cores` shows against one thread.
- **The families got easy.** Every size the first baseline timed out
  on is decided in milliseconds now, except `mix` at eleven pairs. The
  old sizes stay, for their counterparts; add larger ones until the
  largest of each family times out at 300 s again, found by arithmetic
  from the growth in `bench/TARGETS.md` and one-minute checks, not by
  long runs by day, and say in the report which sizes are new. The
  thread table of stage 3 needs the larger sizes to say anything: a
  refutation of a millisecond leaves a pool nothing to share out.
- **`engines.csv` bears on the routing.** The focused engine proves
  `wide-m3` and `wide-m4` as fast as the net engine and loses only
  where a free split per link meets the recursion limit (`wide-m1` at
  2 048). Report focus against net per family, so that step 17 can
  decide whether `net` stays a default route.

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
   than hide it; change the script only as "What step 15 left you" says
   (the reruns' sets, the two bias passes, the added sizes), so that
   every row of the first baseline has its counterpart. Know how the
   first baseline came about: its stages 1 to
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

   Work out what the night needs. The first baseline took 9 h 21 min
   for stages 1 to 3 and 1 h 28 min for its supplement, 10 h 50 min in
   all, while the script's `estimate` says 10 h 30 min. Step 15 moves
   the durations in both directions: the sequential LLTP passes more
   than double (above), the two bias passes come in beside them, the
   family runs shrink to nothing at the old sizes and take their 300 s
   again at the new ones, and the reruns keep their length if their
   sets are the first baseline's. Estimate the night by day from step
   15's target rows and the first baseline's (arithmetic, not a run),
   set `estimate` from it, and ask the author for a slot that holds it
   ("The slot" above).
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
   reported as such, at the top. A row decided before and not now is
   not a wrong verdict but a loss, and every one is listed with both
   rows and with what the two passes under an explicit bias make of the
   same problem. The bias gets a table of its own: per LLTP collection,
   the problems the default decides, those the backward search alone
   and the forward search alone decide, and the overlaps: it says
   whether the combined default keeps its contract at scale (never less
   than `rarer`, short of the limit), what the alternation costs on the
   rows both decide, and what it leaves on the table. The
   rows that were `killed` or
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
- No run by day beyond one-minute checks of the script and the target
  set once (`bench/targets.sh`, two cores, twenty minutes).
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

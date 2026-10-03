# Step 18 report: the proof check and the derivation within bounds

Session of 2026-10-03, by day on the shared machine, in one session (the
seam after item 3 was not needed). Every proof the search returns is now
checked, in every build, in memory proportional to the proof; no front
end builds a derivation past a bound on its estimated size; and the
command gives the verdict at once, the tree where it fits, and one line
that says what it left out and how to get it. The Petri net of 65 643
clauses whose proof took 6 GiB to check, to print, to draw, to certify
and to graft now ends in 0.35 s within 72 MB on every one of those paths,
with exit status 0. The JSON sequent with a repeated atom name is proved.

## Outcome, item by item

1. **The wrong verdict.** A name is an atom: `Sequent::optimize` merges
   atoms by name and `Sequent::add` concatenates tables that `optimize`
   then merges, so equal names were always meant as one atom, and a file
   that repeats one is read as the sequent with the name once rather than
   refused. `Sequent::merge_atoms` does it after the integrity check of
   deserialization and at the end of the public `add`; the parser's
   lowering, the one place that holds repeats on purpose, uses the
   crate-private `append`. A table of distinct names is left untouched,
   so no JSON form changes and no occurrence id of a stored proof moves.
   The assessment's file is answered `provable (MLL, classical, net
   engine)`. First commit, with its test.
2. **The checker in linear memory** (`core/src/proofs/check.rs`). One
   bottom-up pass in arena order as before, without recursion and without
   any code of an engine. A node's sequent (`State`) is kept only while a
   later node reads it: the pass counts every node's readers, the last
   reader takes its premise's state and changes it in place, an earlier
   one a clone. Both zones are hash tables of their members, so nothing
   is as wide as the forest, and the number of formulas in output
   position is kept in the state instead of recounted per node. The first
   implementation is `core/src/proofs/oracle.rs`, compiled for tests
   only. `agrees_with_the_first_implementation` requires the same result
   and the same `CheckError` from both on 1 300 proofs the engines find
   (generated sequents of every rule set, classical and intuitionistic,
   linear and affine, and the smallest instance of every family), in the
   mode found and in four others, and on over 5 000 mutants. A
   fresh-context reviewer read both rule by rule and compared them on
   27 000 random proof terms (random nodes, heavy sharing, `⊤`, `&`,
   copies, Mix) in six modes, about 162 000 checks, tens of thousands of
   them rejections with their error reports: no difference, no defect.
3. **Every proof is checked, in every build.** `prove_goal` runs the
   checker on a proof of the roots before it returns
   (`search::Options::check`, `DEFAULT_CHECK` true), for every engine in
   one place; a proof it rejects is `Error::Rejected`, an error and never
   a verdict. A proof of a goal off the roots is checked against its goal
   by `Derivation::of_goal`, which grafts it (before, that the root
   concludes the goal was a `debug_assert!`). The flake has
   `test-debug-assertions`: the tests once more with debug assertions and
   overflow checks on, since crane tests in the release profile.
4. **The size of a derivation** (`core/src/proofs/size.rs`,
   `Proof::derivation_size(two_sided)`, a `Size`): inferences, characters
   of the sequents, height and a lower bound of the width, from one pass
   of the checker, saturating. Exact but for one case, which `Size::exact`
   reports (below).
5. **The bound** (`ViewOptions`, `ViewError`, in
   `core/src/proofs/derivation.rs`): one options value, honoured where
   derivations are made (`unfold`), so by the text tree, the four
   exports, the graft of `Interactive::close` and `check`'s output alike.
6. **The terminal** (`--tree auto|always|never` in the command).
7. **The text tree** (`core/src/proofs/fmt.rs`): two passes and a
   row-by-row writer; the pinned renderings pass unchanged.
8. **The time limit and Ctrl-C** reach the building of the derivation
   and the writing of the text tree; an output file is written whole or
   not at all.
9. **The harness**: the verdict before the check, the kill from the end
   of the load, the crash reason; and a new last column `check_ms`.
10. **Documentation**: `.claude/rules/core.md` (a name is an atom; the
    checker's memory invariant, its second pass for errors, the observer;
    what `Size` promises; the bound; the text layout), `cli.md`,
    `bench.md`, `CLAUDE.md`, `README.md` (whose "returning a checked
    proof" is now true), `bench/TARGETS.md`.

## The options, and how each front end sets them (D15, D16)

| option | type and default | the command | the web front end, another wrapper |
|---|---|---|---|
| whether a proof is checked before it is returned | `search::Options::check(bool)`, `Options::DEFAULT_CHECK` = true | `--no-check` on `prove` | a field of the search options once they have serde (step 23); the harness sets it false and checks itself |
| the bound on a derivation's estimated size | `ViewOptions { limit: Option<u64> }`, `ViewOptions::DEFAULT_LIMIT` = 64 MiB, `ViewOptions::UNBOUNDED`; `Default`, `Clone`, serde with `#[serde(default)]` | `--derivation-limit SIZE\|none` on `prove`, `check` and `interact` | held as JSON (`{"limit": 67108864}`, `{"limit": null}`) and passed to `Proof::derivation_with`, `two_sided_derivation_with` and `Interactive::close` |
| whether a text tree is printed on a terminal | the command's own (`Tree`, default `Auto`) | `--tree auto\|always\|never` | not applicable: whether a tree fits a terminal is the command's question; a front end with a viewport asks `Proof::derivation_size` and `Derivation::text_size` |
| how long a child may load | the harness's `DEFAULT_LOAD_LIMIT` = 120 s | `linlog-bench run --load-limit SECONDS` | – |

`Size::BYTES_PER_CHARACTER` (8) and `Size::BYTES_PER_INFERENCE` (128) are
public constants, not options: they define the unit the limit is in.
`SCREENS` (3, the screens a tree may fill on a terminal) is a named
constant of the command without a flag of its own; the step allowed no
output option beyond the bound and the switch, and the switch overrides
it. `STEPS_PER_CLOCK` (256) is a polling cadence, not a choice a user
makes.

## Measurements

All in scopes capped at 1 GiB without swap, pinned to one efficiency
core, with the release build of this step's head; "before" is the
release build of the commit the step started from, in the same scopes.

**The assessment's calls** on `TokenRing-40-unfolded_1_1` (65 643
clauses, 4.5 MB of sequent), each of which took 6 GiB before:

| call | now |
|---|---|
| R1 `prove -i --jobs 1 --timeout 5s`, default output | 0.37 s, 71 MB, exit 0: the verdict, and "the derivation is not written: its 65767 inferences with 133975099833 characters of sequents are estimated at 998.2 GiB, over the limit of 64.0 MiB; …" |
| R2 the same with `--format rocq`, `svg`, `latex`, `typst` | 0.34 to 0.36 s, 71 MB, exit 0: the verdict as the format's comment, the line on standard error |
| R3 `--format json`, then `check -i` on the 5.5 MB file | written in 0.36 s within 78 MB; `check -i` 0.08 s, 50 MB, exit 0 (`--quiet` 0.07 s) |
| R4 `interact -i`, command `close` | 0.36 s, 61 MB: "the search proved the goal, but the derivation to graft is too large: …; the goal stays open" |
| R5 `prove --quiet --stats` | checked now: 67 ms with the check and 62 ms with `--no-check` on the command's own clock, 71 MB |
| R14 `prove --format text --timeout 10s` on `wide-m1` at 1 024 | 0.20 s, 11 MB, exit 0: the verdict and the line (130.1 MiB estimated); with `--derivation-limit none` the tree of 76 381 765 bytes in 0.56 s within 160 MB, where it took 23 s into a limit of 10 s |

With the limit lifted and `--timeout 1s` on `wide-m1` at 2 048, whose
search takes 0.74 s, the command ends after 1.03 s with the verdict, the
statistics and "the derivation is not written: the time limit of 1s was
reached", exit 0, and no file left beside the output.

**The 14 crash rows** of `bench/results/2026-10-02/lltp-intuitionistic.csv`
(the Petri nets of tens of thousands of transitions), through
`linlog-bench run --lltp … --only … --timeout 5`: all `proved` with
`checked` `ok`, the checks between 4.3 and 14.3 ms.

**`wide-m1` in every format**, time and peak memory, with the size of
the output. "After" is with `--derivation-limit none`; with the default
limit the five derivation formats write the verdict alone in 0.2 s
within 11 MB at 1 024 and in 0.8 s within 22 MB at 2 048 (the search).

| format | 1 024 before | 1 024 after | 2 048 before | 2 048 after |
|---|---|---|---|---|
| text | 31.9 s, 415 MB (76 MB) | 0.55 s, 160 MB | killed at 1 GiB | 2.3 s, 641 MB (317 MB) |
| latex | 0.33 s, 106 MB (49 MB) | 0.30 s, 106 MB | 1.3 s, 407 MB (197 MB) | 1.2 s, 407 MB |
| typst | 0.35 s, 126 MB (59 MB) | 0.31 s, 127 MB | 1.4 s, 489 MB (239 MB) | 1.3 s, 489 MB |
| svg | 1.2 s, 856 MB (212 MB) | 1.1 s, 855 MB | killed at 1 GiB | killed at 1 GiB |
| rocq | 0.46 s, 132 MB (62 MB) | 0.45 s, 132 MB | 2.1 s, 515 MB (253 MB) | 2.0 s, 515 MB |
| json | 0.19 s, 11 MB | 0.20 s, 11 MB | 0.81 s, 22 MB | 0.75 s, 22 MB |
| net | 0.20 s, 11 MB | 0.18 s, 11 MB | 0.76 s, 22 MB | 0.83 s, 22 MB |
| net-svg | 0.20 s, 14 MB | 0.21 s, 14 MB | 0.80 s, 28 MB | 0.79 s, 28 MB |

What changed is the text tree (58 times faster at 1 024, and it exists at
2 048) and that nothing past the limit is built by default. The exports'
own cost is as it was: they were linear already, and the derivation they
read is no smaller. The SVG takes 50 bytes of memory per character of
sequent, six times the other formats, and is the one format whose memory
at the default limit (about 400 MB at 64 MiB of estimate) is not of the
limit's order; that is the SVG layout's, and a follow-up.

**The estimate against what is built** (`wide-m1` at 1 024: 5 119
inferences, 16 965 792 characters, 130 MiB estimated): text 76 MB written
and 160 MB at its peak, LaTeX 49 and 106, Typst 59 and 126, Rocq 62 and
132, SVG 212 and 855. The constants were chosen from these.

**The target set** (`bench/targets.sh after-check`, once, 165 runs on its
two pinned cores): all 99 rows that `bench/targets/after-bias.csv` decides
have its verdict, `nodes`, `splits`, `memo_hits` and `memo_entries`; the
undecided rows have its verdict and reason. All 131 proved runs are
`checked` `ok`. The checks take 39.4 ms together against 12 847 ms of
search, 0.31 %; per run the median is 8.5 % of the search time and the
most 44 % (0.87 ms on a search of 1.98 ms); the longest check is 6.3 ms.
The file is committed with `bench/TARGETS.md`.

**The families' verdicts** (`linlog-bench run --all-families --timeout
5`, 123 runs, as the verification table asks for a change under
`bench/`): no mismatch; 59 proofs, all `checked` `ok`. `additive/16`,
whose check took 7.6 GB, is checked in 10.3 ms.

**The flake.** Built alone on four cores from this step's sources,
`test-debug-assertions` took 26 s (`test` 30 s, with the rebuild of the
dependencies that the new crate caused): it compiles the dependencies
and the workspace a second time, since debug assertions change every
crate's build, so it adds about what `test` itself costs to
`nix flake check`.

## Decisions

- **Merge, not refuse** (item 1), for the reason above; and `merge_atoms`
  rather than `optimize` on reading, because `optimize` sorts roots and
  hash-conses terms, which would renumber the occurrences a proof file
  names.
- **Tables, not sorted lists, for the zones.** A sorted list makes an
  insertion cost the zone's length, which on a chain of 65 000 `?` steps
  is the quadratic the rewrite was to remove, in time if not in memory.
  The tables use the crate's fixed-seed hasher; everything reported is
  sorted first, so the output does not depend on their order.
- **An error report costs a second pass.** The states a failing node
  read are consumed by then, so the pass runs again up to the node with
  its premises pinned. It keeps the rule code written once and the
  success path free of copies; a failure is the rare case.
- **One pass behind the checker, the view and the estimate**
  (`examine` with an `Observer`). The derivation builder and the size
  read what the checker derived, never the rules a second time. The
  builder's record keeps a sequent only where it splits or pads a
  context, which the derivation shows anyway; a table of every node's
  sequent would again be quadratic on a chain of `?` steps.
- **A rejected proof is an error, not a panic and not a verdict.** The
  check sits at the end of `prove_goal`, after the search, and is not
  under the caller's stop: it is one pass, 14 ms on the largest net
  measured. `Options::check(false)` exists for the harness, which times
  the search alone, and for a caller that checks itself.
- **The estimate's unit is characters of one-sided text**, two per
  formula for its separator, computed per term in one pass. It is exact
  for the one-sided text tree's conclusions and a proxy for the other
  formats and for the two-sided notation (which writes `A ⊸ B` where the
  count has `~A ⅋ B`). `Size::bytes()` turns it into the quantity the
  limit bounds. The one inexact case: above a premise of `&`, the
  weakenings of several `?` formulas that only the other premise uses
  are each counted with the sequent of the last, an upper bound;
  `Size::exact` says when. `is_the_size_of_the_derivation_built` compares
  inferences, height and characters with the derivation built, one-sided
  and two-sided, on the checker's samples: equal on every one, and on a
  sequent made for the inexact case never less.
- **The width in `Size` is a lower bound** (the widest sequent). The
  text tree's width depends on the layout, and on what a `⊤` absorbs
  along each branch; the command decides the fit in two stages, from the
  size before anything is built (too wide or too long already: no
  derivation) and from `Derivation::text_size` once it is.
- **The bound lives in `unfold`**, the one place derivations are made, so
  a new path cannot forget it. `Proof::derivation()` keeps its name and
  means the default options; its error type is now `ViewError`
  (`Invalid`, `TooLarge { size, limit }`, `Stopped`).
  `Interactive::close` and `close_all` take the `ViewOptions`; a goal
  whose graft is refused stays open and the call is `Error::View`.
- **Where the line goes.** After the verdict when the output is a
  terminal; on standard error otherwise, so that a file or a pipe holds
  exactly the verdict (and the statistics) it would hold for any proof.
  The exit status is the verdict's in every case.
- **A few screens is three**, and a terminal that does not report its
  size counts as 80 by 24. `--tree never` leaves the derivation out of
  every format without a line; `--quiet` is unchanged.
- **`terminal_size`** (MIT or Apache-2.0, through the `new-tool` skill)
  for the terminal's size: the crate clap uses for the same question. It
  is asked about standard output alone (`terminal_size_of`), since its
  `terminal_size()` falls back to standard error and standard input.
- **The harness's protocol**: the child prints `loaded`, its row before
  the check, and its row again after it; the parent takes the last row,
  and for a dead child the row it has, with `checked` saying how the
  child died. `check_ms` is a new last column, which is how the cost on
  the target set was measured.
- **`test-debug-assertions` also turns overflow checks on**, as a
  development build has them, and runs `--all-targets` (the doc tests
  ran in `test`).

## Deviations and assumptions

- More commits than the prompt lists: the harness, the flake check, the
  whole-file write and the target set's record are changes of their own.
- **The exports cannot be stopped inside.** The time limit and Ctrl-C
  are polled while the derivation is built (per inference) and while the
  text tree is written; `latex`, `typst`, `svg` and `rocq` return a
  `String` and run to their end, linear in an output the limit bounds.
  Their signatures are step 22's to change.
- **The command still assembles its output as one string** before it
  writes it, so the text tree's peak memory is about twice its size
  (160 MB for 76 MB). Linear, but a writer handed down to the renderer
  would halve it.
- **`SCREENS` has no flag** (above).
- **The check is not under the time limit** (above).
- "Before" has no number for the text tree and the SVG at 2 048: both
  were killed at the scope's 1 GiB.
- A second Ctrl-C during the write of an output file ends the process
  at once and leaves the `.partial` file beside the untouched output.

## Open questions and follow-ups

- **A malformed proof term can still take memory beyond its size**:
  `Mix(p, p)` repeated doubles a zone per node, in this pass as in the
  first implementation, before the root rejects it. No engine builds
  one; a proof file can. A zone longer than the roots plus the nodes
  still to come can never be consumed, which is the test to add with the
  input boundaries (step 20).
- **A node read by several others is cloned for all but the last**, so a
  proof whose shared nodes have large zones can take nodes × zone. The
  shape is known (the weakenings of one large sequent under a tower of
  `&`); no proof of the engines, the families or the LLTP nets run here
  showed it. Persistent zones would remove it.
- **The derivation builder recurses to the derivation's height.** The
  command runs it on the search's large stack; a front end on a small
  stack must look at `Size::height` first. An explicit stack is the fix.
- **The SVG layout's memory** (50 bytes per character of sequent).
- **`close_all` drops the outcomes it collected** when a goal's graft is
  refused, as it does on any error (the assessment's D9, steps 21 and
  24).
- **`check` prints the whole sequent** in its verdict line, 4 MB for the
  large net; a compact view is step 22's.
- Whether `bench/reruns.txt` is still needed for the large SYJ files now
  that the kill counts from the load's end was not measured (no run on
  them here).

## What steps 19 to 22 must know

- **19 (time limits).** `prove_until` now ends with the check of the
  proof, outside any poll; the command's stop condition is polled again
  after the search, by `derivation` in `cli/src/prove.rs` (`halt`,
  `why`). The harness's child prints `loaded` before its search and runs
  with `Options::check(false)`; the parent's kill counts from that line.
  `bench/targets/after-check.csv` is the file with `check_ms`; the
  counters to compare with are still those of `after-bias.csv`.
- **20 (memory and inputs).** The JSON boundary merges atom names
  (`Sequent::merge_atoms`); the forest's refusal of deep sharing is
  untouched. The checker's two remaining ways to take memory are the two
  follow-ups above. `Size` is how a caller learns what a derivation
  costs before it pays.
- **21 (defaults).** The defaults this step set: every proof checked;
  64 MiB for a derivation; on a terminal a tree of at most the
  terminal's width and three screens. A default time limit will hold
  through the derivation as the explicit one does.
- **22 (configurable output).** `ViewOptions` is the options value of
  the derivation view and has one field; the compact view and whatever
  else varies in how a derivation is shown belong there. The command's
  `Show` and `Shown` are where an output format gets its derivation, and
  `Show::left_out` decides where the line goes. The text renderer's
  `GAP`, the command's `SCREENS`, and the exports' `String` results are
  the constants and signatures left for that step.
- **Quantifiers (D17).** The checker's zones are keyed by occurrence;
  with terms and witnesses a member is an occurrence under a
  substitution, so `Bag`'s key and `State`'s tables are where instances
  go, and `Observer::weight` (per occurrence today, from a table per
  term) becomes a function of the instance. `ViewOptions` and `Size`
  need no change of shape.

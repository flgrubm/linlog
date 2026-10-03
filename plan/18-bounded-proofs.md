# Step 18: the proof check and the derivation within bounds

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: the summary, section 1.2 (the defects
  D1 to D4 and D12), section 2.6 (the calls R1 to R5 and R14) and the
  author's answers at its end.
- `plan/README.md`: D6, D9, D13, D15 and the decisions from D16 on, and
  the Status entry of step 17.
- `plan/later.md`: "Follow-ups: the command's output" and, under
  "Follow-ups: the benchmarks", what is said of `derive`, of the verdict
  written before the check and of the kill.
- `.claude/rules/core.md`: "The checker", "The derivation view",
  "Interactive proving", "Serialization"; `.claude/rules/cli.md` and
  `.claude/rules/bench.md`.
- `core/src/proofs/check.rs`, `derivation.rs`, `fmt.rs`, `multiset.rs`,
  `interactive.rs` (`close`, `graft`), `core/src/export/svg/tree.rs`,
  `core/src/serialize/sequents.rs`, `core/src/sequents/mod.rs`,
  `core/src/search/mod.rs` (`prove_goal`), `cli/src/prove.rs`,
  `cli/src/interact.rs`, `cli/src/io.rs`, `bench/src/run.rs`,
  `modules/checks.nix`.

## What step 17 left you

The engines are sound and fast; what hurts a user comes after the search.
A proof of a Petri net with 65 000 clauses is found in 43 ms within 68 MB,
and then the check of it, the text tree, the LaTeX, Typst, SVG and Rocq
output, the graft of `interact`'s `close` and `linlog check` on its JSON
each take 6 GiB in three seconds, with no cap and after the point where
`--timeout` applies. A proof of 7 000 occurrences costs the text renderer
333 MiB and 24 s and the SVG 835 MiB. In a release build no proof is
checked unless a derivation is drawn: the engines check under
`debug_assert!` only, and `nix flake check` runs its tests in the release
profile, so neither the flake nor CI ever runs those assertions. And one
wrong verdict exists at the JSON boundary: a sequent whose atom table
repeats a name is read as two atoms, prints as `⊢ ~A, A` and is answered
"unprovable".

## Goal

Whatever proof the search can find is checked, in every build, in memory
proportional to the proof; and no front end builds, draws or writes a
derivation it cannot afford. The user gets the verdict at once, the tree
where it fits, and one line that says how to get it where it does not.

## What to build

1. **First, the wrong verdict.** Reading a sequent from JSON gives equal
   names one atom, or refuses the file; decide which from what
   `Sequent::optimize` and `Sequent::add` already promise, and make the
   same hold for every way a `Sequent` comes into being. A commit of its
   own, with its test, before anything else.
2. **The checker in linear memory.** `check` keeps for a node only what a
   later node still reads: no set as wide as the forest per node, a
   premise's state moved rather than cloned, dropped after its last use.
   It stays one bottom-up pass without recursion and shares no code with
   any engine (D6). The old `derive` is the oracle: a differential test
   over every proof the tests and the families produce, with mutants, and
   a fresh-context reviewer for the rewrite before you call it done.
3. **Every proof is checked, in every build.** `prove`, `prove_until`
   and `prove_goal` on the roots return a proof that passed the checker,
   in release builds too, once item 2 makes that cheap; say what it
   costs on the target set. A proof of a goal off the roots keeps its
   own rule (its root concludes the goal), checked by what consumes it.
   The flake gains a run of the tests with debug assertions, so that CI
   runs the engines' own assertions; say what it adds to the check's
   time.
4. **The size of a derivation, without building it.** A function of the
   proof that gives the unfolded tree's inferences and the sum of its
   sequents' sizes (and what the text tree's width and height would be),
   in saturating arithmetic, exact where it can be and a stated bound
   elsewhere. A proof with shared nodes unfolds exponentially; the
   estimate is what says so in time.
5. **A bound on what is shown, as an option of the library** (D15, D16).
   One options value for showing a derivation, with a safety bound on
   the estimated output (default 64 MiB, settable, liftable), that every
   path honours: the text tree, the four exports, the graft of a
   search's result in `Interactive`, and `check`'s output. Past the
   bound nothing is built; the verdict stands, the caller is told the
   size and the ways to get the derivation (`--format json`, the option
   that lifts the bound), and the exit status stays the verdict's.
6. **A tree that does not fit the terminal is not printed** (the
   author, 2026-10-03). When standard output is a terminal and the
   format is the text tree, the tree is shown if it fits the terminal's
   columns and a few screens; otherwise the verdict is followed by one
   line with the tree's size and how to get it. A switch of the kind
   `--color` is (`auto`, `always`, `never`) overrides; `--quiet` stays
   what it is. Into a file or a pipe only the safety bound applies.
7. **The text tree in time and memory linear in its output**: the layout
   `export::svg::tree` computes in two passes, at character widths,
   written row by row. The pinned renderings do not change.
8. **The time limit and Ctrl-C reach the derivation.** What is built and
   written after the search stops when the limit passes or the user
   interrupts, and nothing half-written is left.
9. **The harness** writes the search's verdict before it checks, so
   that a check that dies leaves the row its verdict; counts its kill
   from the end of the load; and gives a crashed child's row the reason
   and not a hint about backtraces.
10. **Documentation**: the rules files (the checker's new invariant,
    what the estimate promises), CLAUDE.md, and README, whose "returns a
    checked proof" becomes true.

## Constraints

- No change to what any engine searches: the counters of
  `bench/targets.sh` stay those of `bench/targets/after-bias.csv` (run
  it once; twenty minutes on its two pinned cores).
- The JSON forms and the snapshots do not change, beyond what item 1
  refuses.
- No new view of a derivation and no new output option beyond the bound
  and the switch: the configurable output is step 22's.
- Every default is a named constant of an options type and has a flag
  (D16).

## Verification

`cargo clippy --workspace --all-targets -- --deny warnings`,
`cargo test --workspace`, both `cargo hack` runs, `nix flake check` at the
end. Then the calls of the assessment, each in a scope capped at 1 GiB:
R1 to R5 and R14 (the sequents through `linlog-bench run --lltp` with a
name filter and through the command on `axioms |- conjecture`), the 14
crash rows of `bench/results/2026-10-02/lltp-intuitionistic.csv` decided
with `checked` ok, `wide-m1` at 1 024 and 2 048 in every format with time
and peak memory before and after, and D1's file refused or proved.

## Deliverables

- Thematic jj commits ("Give equal atom names of a JSON sequent one
  atom", "Check a proof in memory linear in its size", "Check every proof
  the search returns", "Estimate a derivation's size without building
  it", "Bound the derivation a front end builds", "Leave out a tree that
  does not fit the terminal", "Lay the text tree out in two passes", …).
- `plan/reports/18-bounded-proofs.md`: what was built, the options and
  how each front end sets them (D15), the measurements, decisions,
  deviations, open questions, and what steps 19 to 22 must know.

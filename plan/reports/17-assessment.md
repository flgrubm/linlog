# Step 17 report: where the project stands, and the work ahead

Session of 2026-10-03, in two parts: the assessment with its questions
to the author (sections 1 to 8, as written before the answers), and the
answers with what they changed (section 9), after which the steps from
18 were planned. No code was changed. Sections 6 and 7 are the proposal;
where they differ from section 9 and from `plan/README.md`, those hold.

## Summary

**Where the project stands.** The suite does what its README says: it
parses, proves, checks, draws and certifies sequents of every
propositional fragment, and its engines are fast. The two baselines
agree on every verdict, the second loses nothing the first decided, and
against the one prover whose results the LLTP library records (a Maude
prover, 1 342 problems, five minutes each) there is no disagreement in
any pass. What separates linlog's default from that prover is one
number: at its copy bound of 3 the default leaves 196 theorems that the
Maude prover finds; with the bound lifted it leaves three and decides
158 that the Maude prover does not (814 against 659).

**What is wrong is at the edges, not in the verdicts.** This session
ran the command on the baseline's largest problems and read the code
behind what it saw:

- *Memory after the search.* A proof the search finds in 43 ms takes
  6 GiB in three seconds to check, to print, to draw, to certify, to
  graft in `interact`, and in `linlog check` on its JSON. Only `--quiet`
  and `--format json` survive, and those two do not check the proof at
  all: in a release build no proof is checked unless a derivation is
  drawn. Middle-sized proofs suffer too (the text tree of a proof of
  7 000 occurrences: 333 MiB and 24 s).
- *Time limits that are not kept, and a pool that is slower.* Under
  `--timeout 5s`, one thread runs 140 s on a Petri net, and a pool of
  four threads had not stopped after 150 s on a sequent of 19 KB. On the
  wide MLL sequents that are the net engine's case, two threads take
  9.5 s where one takes 33 ms. The command's default is every core.
- *No limit by default.* Without flags the command has no time limit and
  no memory limit; the memo's cap counts entries, which on a large
  forest is 16 GiB a search.
- *The JSON boundary.* A sequent whose atom table repeats a name prints
  as `⊢ ~A, A` and is answered "unprovable".

**The code** is sound where the reviews looked and carries its debt in
few places: the focused engine's hand-kept flags and its second copy for
the pool, rule and symbol tables written many times over, and the
decision that every output is configured through the library (D15),
which holds for one export of four. Section 1.2 is the ranked list.

**The candidates.** Worth doing, in this order: the check and the
derivation within bounds (18), the search within its limits (19), the
defaults a user meets (20), configurable output with no font (21), the
library's surface in order (22), a batch mode with LLTP input (23), the
web front end (24), the Rocq library (25), the focused engine in order
(26), an engine for Horn programs with coverability and the
coverability suite from practice (27, in place of an external
reachability tool, which is dead upstream), a third baseline and a
release (28). Worth a session each when the author wants them: ordinary
logic through its embeddings, MELL nets with boxes. Only on the author's
word: cut elimination, the Lambek calculus, quantifiers. Not worth doing
on what the baselines show: net-engine pruning, essential nets as an
engine, the inverse method, MALL nets.

**Decided here, for the author to overrule.** The net engine keeps its
route (for width, not for speed), with its cubes repaired.
`Options::portfolio` goes. The command checks every proof it reports
once the check is cheap.

**Open.** Nine questions in section 7; the plan from step 18 waits for
the answers. Not verified: the cause of the pool's missed stop; whether
a Horn engine beats the forward search (step 27 measures it before it
commits); anything on every core.

## 1. The state of the repository

How it was read: the plan, its reports, the rules files and the spec by
this session; `core/src/search/`, the rest of `core/src` with
`core/tests`, and `cli/`, `bench/`, the flake and the documentation by
three sub-agents that read the files in full and reported with file and
line (what each did not read is at the end of 1.4); the claims taken
from them that matter most were tried again here. At the head,
`cargo clippy --workspace --all-targets -- --deny warnings` is clean and
`cargo test --workspace` passes (122 tests in the core crate in 5.1 s,
2 ignored).

### 1.1 What the documents say against what exists

**README.md.** Its usage section is true: all 45 `linlog` invocations of
its console blocks reproduce byte for byte with their exit statuses (a
sub-agent ran them, with `--jobs 2 --timeout 10s` added), and the three
`linlog-bench` examples give the same tables. What is stale or wrong:

- "returning a checked proof" (twice, and in the crate documentation of
  `core/src/lib.rs`): the engines check under `debug_assert!` only, and
  the released command checks a proof only by drawing its derivation
  (D4 below).
- "The search runs on every core by default" is true and, on the wide
  MLL sequents that go to the net engine, the slow path by a factor of
  hundreds (D11).
- "Planned, in roughly this order" heads a single bullet that names five
  things; the candidates since are absent. Part two rewrites it.
- Standalone documents "set in the Euler math font" matches the code and
  contradicts D12; step 21 changes both.
- The exit statuses: the causes of 3 omit the recursion limit, and the
  statuses a user can meet outside the scheme are nowhere: 130 (a second
  Ctrl-C), 101 (a panic: a parse error past column 65 535), 134 (a stack
  overflow on a deep or very wide input), and a closed pipe
  (`linlog prove … | head -1` is exit 2 and loses the verdict).
- Present and not mentioned: `--file` and standard input,
  `--memo-limit`, `--recursion-limit`, `interact --state`,
  `seq json --optimize`, the values of `--fragment`; of the harness
  `--only`, `--reverse`, `--grace`, `--repeat`, `--resume`, `--modes`,
  and `baseline.sh --into`.

**CLAUDE.md and the rules files.**

- CLAUDE.md, "`nix flake check`, which runs all of the above": its
  `test` check is crane's, which runs `cargo test --release`
  (`modules/checks.nix`), so neither the flake nor CI ever runs a debug
  assertion, the engines' own check of their proofs among them. The
  tests that call `check` themselves still run.
- CLAUDE.md and `bench.md`, "or of `after.csv` when both runs name a
  `--bias`": `bench/targets.sh` takes a label and passes no bias.
  CLAUDE.md also has `bench/TARGETS.md` comparing three labels (five),
  "eventually prove", a "README roadmap" (none), `cleanCargoSource` (the
  code uses `commonCargoSources`), and the lint allowance "while things
  are scaffolded".
- `cli.md`: the Ctrl-C handler "is installed by `prove` only"
  (`interact` installs it too); a new format is "its arm in `prove`'s
  and `check_text`'s match", where it is four matches that force it and
  three places that skip it silently.
- `bench.md`: "the 16-field tail" (`TAIL` is 21); the reason
  `context_too_wide` (gone; a new reason becomes `other`, which is not
  listed).
- `core.md`: "its only reader, through `polarity(o)`" against "never
  off `Forest::polarity`"; "Euler everywhere"; "SVG text, say" as a
  target to come.
- The devshell's menu and the `update-deps` skill list the checks
  without `export`, `rocq`, `bench` and `claude-hooks`.
- `--help`: `check --format json` is described as `prove`'s object and
  writes `{"error":…,"mode":…,"valid":…}`; `--standalone` says "latex
  and typst" and Rocq takes it; `net` says "in classical mode" and
  `prove -i --format net` works; `interact`'s sequent is "read from
  standard input when absent", which it refuses; `linlog-bench run`'s
  help for `--bias auto` describes the default before step 15's second
  session.
- The spec: "NEXPTIME membership is not established" (it is: Lincoln and
  Shankar 1994); KReach as the tool to call. `plan/later.md`: eleven
  facts corrected in section 3.
- Doc comments in `search/`: "the affine prune" (none exists), "the time
  limit was reached" (it writes "the search was stopped"), `splits` as
  submasks.

**Limits a user meets that nothing mentions**, beyond 2.6: `--jobs` has
no bound (10 000 on `A |- A` takes 12 s and 183 CPU-seconds past a
`--timeout 10s`; a sub-agent's probe with 100 000 loaded every core of
this machine for about five minutes, against this step's rule, which is
how the finding was made); `A * B` without a turnstile is "unexpected
end of input"; `check` on the JSON of an unprovable outcome is "missing
field `sequent`"; `--output` into a missing directory fails after the
search and loses the result; `interact` splits its words on blanks, so
`save my file.json` writes `my` and `proof latex` writes a file named
`latex`; one Ctrl-C during rendering is swallowed; the atom names
`lltp::read` makes (`‿`, `·`) do not compile under pdfLaTeX;
`rocq::Options::lemma` is not checked to be an identifier.

**The command, the harness and the flake as a maintainer finds them**
(these join the refactoring's list for step 22):

- `prove` and `interact` duplicate each other: eight search flags
  declared twice (`argument_parsing.rs`), `Options` built twice, the
  stop closure twice, the verdict line twice with different words
  ("provable" and "proved").
- The harness has one test (clap's); `summary.rs` and `compare.rs` have
  none; the `bench` check fails on `MISMATCH` only, so `error` and
  `crash` rows pass it; a crash row's reason is the last line of the
  child's error output, which for all 76 of the second baseline is the
  hint about `RUST_BACKTRACE`; the configuration label exists twice and
  the copy in `compare.rs` lacks the bias.
- `bench/baseline.sh` is written for the second baseline:
  `reference=…/2026-09-30`, the cores, the estimate, `--copies 30`
  restating a default. `bench/reruns.txt` is spent: 57 of its 95 entries
  no longer end killed or crashed, and 94 rows that do are not in it.
- The `export` check compiles standalone documents only; the fragments
  the command prints by default are never compiled. CI builds
  `x86_64-linux` while `modules/systems.nix` declares two more systems.
- Of the command's stated behaviours, no test passes `--timeout`,
  Ctrl-C, `--jobs` (all tests but one run on every core), `--bias`,
  `--memo-limit` or `--output`; nothing in the repository pins README's
  invocations (each step ran a script of its own and threw it away).

### 1.2 The code as a maintainer finds it

Two lists, each ranked by what an item costs to leave alone. The first
is defects: they change behaviour, so they are work for steps 18 and 19
and not for a refactoring. The second is the refactoring's list, with
the hot spots step 15 left merged in.

**Defects** (D1, D2 and D6 were reproduced by this session's own runs;
D3 and D5 are a sub-agent's measurements in capped scopes; D4 was read
in the code and shows in R5):

| # | what | where | step |
|---|---|---|---|
| D1 | A JSON sequent whose atom table repeats a name is taken as two atoms: `{"terms":[{"D":0},{"V":1}],"ids":[0,1],"var_dict":["A","A"]}` prints as `⊢ ~A, A` and `prove` answers "unprovable: the search was exhaustive". The JSON form is what the web front end will send | `serialize/sequents.rs` (deserialization checks integrity, never names; `optimize` is not run), `Sequent::add` | 19 |
| D2 | The check keeps, for every node, a set as wide as the forest and the node's whole linear zone, and clones a premise per use: quadratic, gigabytes from about 100 000 occurrences (R1 to R4) | `check.rs`: `Derived`, `derive` | 18 |
| D3 | The text tree pads every line of every subtree to full width at every level: 333 MiB and 24 s for a proof of 7 167 occurrences (`wide-m1` at 1 024), eight times the time per doubling. The SVG of the same proof takes 835 MiB, at 2 048 3.3 GiB; LaTeX 103 MiB, 396 MiB, 1.6 GiB at 4 096 | `proofs/fmt.rs::block`; `Derivation::build`, which stores a full sequent per inference and unfolds shared nodes without a bound | 18 |
| D4 | No proof is checked in a release build unless a derivation is drawn (R5) | `debug_assert!` in `search/mod.rs::prove_goal`, `search/net.rs`, `nets/sequentialize.rs` | 18 |
| D5 | A JSON sequent of 433 bytes with a subterm shared 24 times over takes 1.9 GiB and outlives its time limit; the forest refuses only at 2³² occurrences | `occurrences/mod.rs` (`TooManyOccurrences`) | 19 |
| D6 | The missed stops (R6, R7): `forced_splits` and `literal_tensor` count splits and never poll, and `dual_in` rescans a literal's occurrences from the head, quadratic in equal tokens; the calling thread waits in `alternate` without polling; the net engine's backtrack cascade and every set-up (`Counts::new`, `Classes`) run unpolled. The pool's miss on the SYJ problem has no cause yet | `focus/mod.rs`, `focus/parallel.rs`, `net.rs` | 19 |
| D7 | Memory without a bound (R8): the memo's cap in entries; the kept arena, which never shrinks after the memo is cleared and panics at 2³¹ nodes instead of answering "unknown"; the rows of `Counts::new`, quadratic in a tensor of distinct atoms | `focus/memo.rs`, `focus/mod.rs::Arena`, `focus/counts.rs` | 19 |
| D8 | `prove_goal` takes "the roots in any order" by its documentation and compares with `==`: permuted roots lose the net engine, or get `Error::NetGoal` when it is forced | `search/mod.rs` (`is_roots`) | 26 |
| D9 | A drawing's ids are the derivation's, the session's goals are numbered otherwise, and the map between them is dropped: a client cannot tell which goal was clicked. `close_all` drops the outcomes it collected when a later goal errors, while its grafts stay | `Interactive::derivation`, `close_all` | 21, 24 |
| D10 | Aborts instead of errors: a formula nested 100 000 deep (R11); a parse error past column 65 535 panics in the caret's formatting; `OccSet::is_subset` and `is_disjoint` on sets of different widths truncate silently; the public `ProofStructure::link` only debug-asserts its arguments | `parse/mod.rs`, `cli/src/lib.rs`, `occurrences/set.rs`, `nets/mod.rs` | 19, 22 |
| D11 | The net engine on a pool is quadratic on the sequents it is the default for: `wide-m1` at 512 pairs is proved with 1 024 links in 33 ms on one thread and with 524 800 links in 9.5 s on two (at 256: 8 ms against 1.2 s). Reproduced here through the harness. The command's default is every core, so this is what a user gets | the cube enumeration of `net::parallel::search` | 19 |
| D12 | The time limit and the first Ctrl-C end with the search: the derivation is built and rendered after `prove_until` returns, so a text tree of 76 MB is written 23 s into a limit of 10 s | `cli/src/prove.rs::prove` | 18 |
| D13 | `--jobs` has no bound: 10 000 threads on `A |- A` cost 183 CPU-seconds | `cli/src/argument_parsing.rs`, `search/parallel.rs::Runtime::new` | 19 |

**The refactoring's list.**

1. *The cut and dependency flags of the focused engine are engine-wide
   state, saved, cleared and restored by hand* (`Engine::prove_stable`,
   and three hand-written merges in `focus/parallel.rs`). Nothing but a
   sentence of the rules file guards them, and an early return between
   save and restore lets a level answer `Unprovable` wrongly. Returned
   as part of a step's result, they stop being a trap and each rule can
   be written once. Step 26.
2. *The pool's engine is a second copy of the sequential one.* The 33
   fields of `Engine` are written out three times (`Engine::new`,
   `Spawn` with `Engine::spawn`, `Spawn::worker`), and each rule body
   exists twice (the loops of `decide_with`, `focus` and `with` against
   `alternative` and `with_parallel`). The copies already differ: on one
   thread the flags of failed siblings survive a proved alternative, on
   the pool they are dropped. Both are sound; they are not the same
   search. Step 26.
3. *The default bias has three schedulers and two merges* (`turns`,
   `alternate`, the pool's race), which count `memo_entries` differently
   (a maximum, a sum, a sum of shard peaks), and the one-core scheduler
   lives in `focus/parallel.rs`. Step 26.
4. *There is no engine interface.* Four entry points with four
   signatures, the conversion of a search's result into a `Verdict`
   written five times, and options that an engine ignores in silence
   (the net engine `recursion_limit`, `memo_limit`, `copies`, `bias`;
   the additive path `jobs`; the focused engine `test_period`). Every
   new engine adds a special case to `prove_goal`. Step 26, before 27.
5. *D15 holds for one export of four.* `rocq::Options` has serde;
   `svg::Style` has none; LaTeX, Typst and the text tree have no options
   value, and their choices are constants (`PREAMBLE`, `PAGE`, the label
   tables, the open goal's shape, the text tree's gap, the SVG font
   family). `search::Options` has no serde either. Four signatures write
   a derivation. Step 21.
6. *One rule is nine edits.* `Rule` has 34 variants, 19 of them a
   classical rule times a position, and its names live in
   `Rule::name`/`from_str`, two label tables, the Rocq script, the
   interactive `rules` and `acts`, and `Node::name`. The interactive
   JSON depends on `from_str(name)` being the inverse of `name`, which
   no test states. Steps 21 and 22.
7. *The same walk several times.* Five formula printers and three
   assemblies of a two-sided sequent, which the rules file says "must
   stay in step"; two tree layouts, of which the text one is D3. Step
   22 (the text tree itself in 18).
8. *Files past one reader.* `focus/mod.rs` (3 219 lines; seams: the
   scheduling of the two searches, the arena, the split search, the
   pools of scratch buffers, the tests), `proofs/interactive.rs` (1 514:
   state and undo, rule validation, reading a state back, the
   translation to terms, the search), `proofs/derivation.rs` (the `Rule`
   type and its tables are a file of their own), and
   `.claude/rules/core.md` itself: 1 740 lines that load on any read
   under `core/`, 600 of them the focused engine's. One rules file per
   module path would cost a session a tenth of that. Steps 22 and 26.
9. *The forest carries the focused engine's atom bias*
   (`occurrences/mod.rs`: `Bias`, `signs`, `bias_under`), which only
   that engine reads. Step 26.
10. *Ownership.* Every owner clones the forest (`Proof`,
    `ProofStructure`, `Interactive`), `Reading` and `Derivation` borrow
    it, and `Interactive` recomputes the reading per call. Step 22
    decides whether a shared forest is worth it before the web front end
    multiplies the callers.
11. *Errors.* `ShapeError`, `Unsupported` and `UnknownRule` stand
    outside `Error`; there are three `describe` conventions; `NetError`
    writes each message twice; none is serializable; nets take a `bool`
    where everything else takes a `Mode`. Step 22.
12. *The lint allowances of `core/src/lib.rs`.* `unused_variables`
    hides nothing in any feature set and can go. `dead_code` hides four
    dead items (`additive::search`, `Context::set`, the alias `Cube`,
    `Flags::own`), six used by tests only, and about a dozen that are
    live under one feature and need a `cfg` (`Stop::Slice`, the `Shared`
    memo and arena, `Derivation::from_parts` and `of_goal`, `Notation`
    under `rocq` alone). CLAUDE.md's "while things are scaffolded" is no
    longer the reason. Step 22.
13. *Names.* Three structs `Engine` beside the enum `search::Engine`;
    `focus::Rules`, `focus::Rule`, `generate::Rules`, `proofs::Rule`.
    Step 26.
14. *The hot spots step 15 left*, checked against the code: the memo key
    is hashed up to four times per stable sequent, both zones in full
    (`prove_stable`), and once more for the shard; an insert clones its
    key; the canonical key is built for every stable sequent. These
    three are worth taking in step 26 at identical counters. The member
    list and tally, `OccSet` as a boxed slice and link-time optimisation
    are not: no target asks.

### 1.3 Invariants that only a rules file holds

Each is stated in `.claude/rules/core.md` and enforced by no type,
assertion or test.

- The flags of item 1 above: what breaks is a verdict.
- The stack a level of recursion may take (`Options::stack_size`,
  measured by hand): no test recurses to the limit through searched
  splits, and on a pool a thread that waits runs stolen tasks on the
  same stack. What breaks is an abort at a raised limit.
- Atom names are distinct (D1), and a reading belongs to its forest
  (`Derivation::from_parts` and `check::derive` take any): what breaks is
  a verdict or a drawing.
- "The engine reads polarities off `Counts::positive`, never off
  `Forest::polarity`", while `Forest::polarity` stays public (and two
  sentences of the rules file contradict each other on it).
- "Every failure outside the release points pushes nothing" (the arena):
  what breaks is memory, silently.
- "Interchangeable occurrences have equal counts": what breaks is
  pruning only.
- That the printers over `Notation` stay in step with `Display`.

### 1.4 Tests

*Missing for a stated behaviour.* Nothing large goes through `check`,
`derivation` or a renderer (no test has more than about twenty formulas
there), which is how D2 and D3 went unseen. The focused memo with a
small limit that is not zero (the clearing when full, the cap per
shard). `chains`, the Horn test, directly and on its negative cases. A
stop inside a forced chain. `prove_goal` off the roots with
exponentials, repeated occurrences or weakening. Of the 33 rule labels
per target, 19 appear in no snapshot, so the `export` check never
compiles them (`mix`, `wk`, every two-sided exponential and unit rule
among them). Parse errors are tested by `is_err()` only. The round trip
`Rule::from_str(rule.name())`.

*In excess.* The tree of `A, A -o B |- B` is pinned four times; the net
engine's `classic_sequents` repeats twelve sequents beside the
differential test; `search::tests::stop` equals `net::tests::stop`; the
second assertion of `focus::parallel::tests::agree` pins a contract the
rules file calls false.

*The suite's time* is no concern: 5.1 s for the core crate.

*What the three reviews of step 15 had not covered.* `prove_goal` off
the roots: no wrong verdict found by reading (counts, classes and bias
are the forest's and conservative); the conclusion of such a proof is
checked by a debug assertion only. `Interactive::close`: inherits D2 and
D4 through `Derivation::of_goal`. `Options::portfolio`: its seed is read
in one ordering function after the canonical choices are made; no risk
seen, no use either.

*Not read by anyone in this step:* the values of the SVG advance table
and 17 of the 21 snapshots beyond their labels.

### 1.5 The follow-up lists of `plan/later.md`

Every entry, with where it stands. The step numbers are section 6's.

**The focused engine.**

| entry | status |
|---|---|
| the unit of work of the default bias; the `NeighborGrid` rows that pay most | stands; step 20 looks at the share, and it loses its weight if step 27 takes the nets |
| the forward bound applies to Horn programs only | stands; superseded for nets by step 27, for the rest by step 20's deepening |
| the deepening restarts every level from the root; the restart from the frontier | stands; dropped if step 27 is built (it only pays on long chains) |
| five sampled problems that `--copies 10` decides | step 20 |
| `Options::portfolio` has no use | confirmed by the second baseline; removed in step 19 |
| the threads of a pool split evenly between the two searches | stands, unmeasured; step 28's baseline |
| the two searches alternate on two threads; a search that can be suspended | stands; decided with step 24 (the web front end is its only pressing user) |
| the Horn test reads the roots, not the goal | stands; step 26 |
| Mix costs `3^n` | stands; step 26 |
| free splits where the counts have no rows | stands; moot for nets after step 27 |
| constant factors not taken | the first three (key hashing, the insert's allocation, the canonical key) step 26; the member list, `OccSet` and link-time optimisation dropped for lack of a target |
| sharing more among interchangeable sequents; the order of a split search's members; the intersection for `&` | stand; deferred, no target asks |
| a free split costs a level of recursion per link | stands; step 26 |
| a sound affine prune | open research; step 27 answers it for Horn programs by coverability |
| the check and the derivation of a large net | step 18 |
| the search's own memory; the forward search's missed stop | step 19 |
| the default's contract at the limit; the copy bound | step 20 |

**Intuitionistic mode.** The written succedent among `⊤`/`0`-built
roots stands and is taken up by step 25, whose two-sided statement
carries it. The additive path on more than two roots stands, with no
need shown. The canonical choice two-sided is done (step 15).

**Interactive proving.** The net engine on a sub-forest: deferred. The
reading recomputed per operation, a filtered `rules` list and a per-goal
budget for `close_all`: step 24, which is the client that would feel
them. The empty goal a one-sided Mix opens: step 22 (a one-line refusal).
The rule names accepted on reading a state: harmless, stays.

**The exports.** Reporting the curryst limit upstream is obsolete (it is
reported and closed); the own Typst layout, Greek atom names under
pdfLaTeX, a format for `interact`'s `proof`, the label table as an
option, per-formula ids, anchors at the atom, the nesting-safe arc cap,
disconnection colouring, Greek letters as italic code points and the row
height are step 21.

**The command's output.** Step 18, with question 8.

**Parallel search.** The false assertion of `agree`, the missed stop on
the SYJ problems, the pool's memory on the largest files and the
portfolio: step 19. A `Runtime` kept across calls: step 23, where a
batch pays a pool per sequent otherwise. The default thread count:
question 3. A per-worker arena, the duplicated exploration of `&`
premises, a thread sanitizer: deferred. "The parallel tests take about a
minute" is obsolete: the core crate's 122 tests ran in 5.1 s in this
session's debug build.

**The benchmarks.** The wrong headers upstream: question 9. LLTP input
for the command: step 23. The kill counted from the end of the load, the
verdict written before the check, `derive`'s memory: step 18. `summary`
counting a late verdict as solved, and a table of counters in `summary`:
step 22, before step 26 needs the oracle. The net engine's test period
and Matsuoka's 3D-Matching: dropped with 3.3. Problems from practice:
3.15 and step 27.

## 2. What the baselines say

Sources: `bench/results/2026-10-02/` (the second baseline) and
`bench/results/2026-09-30/` (the first), `bench/COMPARISON.md`,
`plan/reports/14-benchmarks.md`, `15-performance.md` and `16-baseline.md`
with the review's corrections, and this session's own probes of the
command (2.6). Counts that this session recomputed from the rows say so.

### 2.1 Where each engine stands

**The focused engine** is the suite. After the performance pass it
decides every generated family at the sizes the first baseline timed out
on, in microseconds to milliseconds, with one exception. What it does not
decide, and why:

| what | where it ends | the cause |
|---|---|---|
| `mix` at eleven pairs | time (over 1 200 s; ten pairs 146 s) | the algorithm: `3^n` memo lookups, no prune separates the parts |
| `partition-yes` at 28, `partition-no` at 14 and 15, `qbf` at 48 (#3) | time at 300 s | the problem's nature: NP- and PSPACE-hard families at sizes chosen to time out |
| `qbf/48#0` | memory after 290 s | a limit that is missing: the memo's cap counts entries, not bytes |
| `wide-m3`, `wide-m4` at 2 048 literals | recursion limit | a limit: a free split costs a level of recursion per link |
| `counter-over` at 32 and 64, `growing` | copy bound | a limit, as the families intend |
| 1 594 of 3 137 LLTP Petri nets | time at 5 s | the algorithm: the forward search deepens level by level from the root on forests of tens of thousands of occurrences; the backward search has free splits that no count cuts |
| 727 intuitionistic LLTP problems outside the nets | copy bound of 3 | a limit: 369 of the first baseline's 832 are decided at a bound of 10, 288 by the forward search at 30, each within 5 s |
| 54 ILLTP-SYJ problems | recursion limit | a limit, and the search is that deep |
| 42 ILLTP-SYJ problems | time | the problem's size (up to 12 million occurrences) |

Outside the nets an answer is immediate either way (recomputed from
`lltp-intuitionistic.csv`): the 428 proofs and 101 refutations take
0.03 ms in the median and 269 ms at most, and the 725 answers "copy
bound" take 0.18 ms in the median, 601 of them under 10 ms and 17 over a
second. On the nets the 1 520 proofs take 1.7 ms in the median, 944 under
10 ms and 81 over a second. So the engine is fast where it decides, and
its "unknown" is fast where the copy bound of 3 ends it; what is slow is
an undecided net, which runs until something stops it (2.6).

**The two-sided engine** is the focused engine given the reading; both
baselines show it never deciding less than the classical search of the
same sequent and the nets 1.36 times faster in the median (first
baseline). The counter with 64 tokens takes 81 s classically and 268 ms
two-sided.

**The net engine** is unchanged since step 6 (its times in the second
baseline are the first's within 1 to 4 %). It is linear on distinct
literals (14 335 occurrences in 0.64 s) and exponential on equal literals
inside one tree: every Horn encoding takes it seconds or a timeout where
the focused engine now takes microseconds (Partition with six items over
60 s against 130 µs). Its cubes scale almost linearly (13× at sixteen
threads on MLL 3-Partition), which no longer buys a verdict the suite
lacks.

**The additive path** decides the identity of depth 16 in 210 ms; nothing
about it is open except that the proof it returns is too large to check
(2.6).

**In parallel.** With refutations in microseconds the families have
little left for a pool: `partition-no/12` 4.7× on sixteen threads, the
counter 3.3 to 3.5× on eight (two searches, a pool each), QBF 1.3×, Mix
slower with every thread, and `partition-yes/24` proved in 106 s on one
thread and not within 120 s on any pool. On the 1 102 LLTP problems of
the all-cores run sixteen threads decide 623 against 586 (37 gained, none
lost). In time, the 586 both decide take 1.1 ms in the median on one
thread and 4.6 ms on sixteen; 106 are more than 100 ms faster on sixteen
threads and 15 more than 100 ms slower. The net engine was measured on a
pool only on the Horn encodings, where its cubes scale; on the wide
sequents it is the default for, a pool is hundreds of times slower than
one thread (R13 in 2.6), which no row of either baseline shows.

### 2.2 Against the provers LLTP records

The library ships the results of one prover only, for 1 342 of its 4 512
problems: thirteen text files under `bench/lltp/ILL/` (`PATH ; TIME ;
VERDICT`), produced by Olarte's focused ILL prover in Maude
(`bench/lltp/README.md`, line 44; the ILLTP paper, arXiv:1904.06850,
Table 1, has the same counts and says: times in milliseconds, a QEMU
virtual CPU at 2 GHz with 6 GB, a timeout of five minutes). No prover is
recorded for the Petri nets, for `ILL/misc`, `ILL/Non-theorems` or
anything under `CLL/`. A sub-agent matched the files by path against the
second baseline's rows (its scripts are in the session's scratch
directory; nothing was run).

| on the 1 342 recorded problems | decided |
|---|--:|
| the Maude prover, 300 s | 659 (620 theorems, 39 non-theorems) |
| linlog's default, 5 s, one thread | 526 |
| decided by both | 463, with the same verdict on every one |
| only linlog | 63 (62 refutations where Maude timed out, one proof) |
| only Maude | 196, every one a theorem that linlog leaves at the copy bound of 3 |
| linlog's four passes together (default, `--bias rarer`, `--copies 10`, forward at 30 copies; 5 s each) | 814 |
| only Maude, against those four | 3 (`SYJ201+1.003` and `SYJ203+1.009` in the 01 translation, `SYJ203+1.009` in cbn; Maude took 51 to 277 s) |

What is comparable: the verdicts on files matched by path, both provers
reading the same file as `axioms ⊢ conjecture` in ILL. No verdict
disagrees, in any pass of either baseline, and on 12 of the 28 files
whose headers are wrong the Maude prover contradicts the header with
linlog's verdict. What is not comparable: the times (linlog is 1 500 to
4 300 times faster in the quartiles on the problems both decide, but the
machines differ, the Maude times have a floor of 16 ms that looks like
start-up, and linlog's time is the search without the process); the
limits (300 s against 5 s); and an "unknown" at the copy bound, which is
a refusal to search further after 0.18 ms, against a timeout, which is
300 s of search. The honest reading is the one the table gives: under its
default bound linlog loses 196 theorems to a prover from 2019 that is
"very basic" by its authors' own word, and with the bound lifted it loses
three and decides 158 that prover does not (814 against 659). The copy bound is the whole
difference.

### 2.3 What is an algorithm, what a limit, what the problem

- **Limits that a default sets** (the cheapest to move): the copy bound of
  3 (727 problems, and the 196 above); the recursion limit on free-split
  chains (the wide sequents at 2 048, two SYJ families); the missing
  time limit and the missing memory limit (2.6).
- **The engine's algorithm**: Mix (`3^n`); the nets at the time limit
  (the deepening from the root, the per-step cost on wide forests, the
  free splits of the backward search); the unit of work by which the two
  searches of the default share one core.
- **The problem's nature**: the hard families at their largest sizes, and
  the SYJ files of millions of occurrences, which take 16 s and more to
  load before any search.

### 2.4 The dispatch between the engines

The focused engine is within a factor of three of the net engine on the
wide sequents from 8 to 1 024 literals and faster from 256 on, so speed
no longer argues for the net route. Width does: at 2 048 literals the
focused engine meets the recursion limit and the net engine takes 0.64 s.
The route `prefers_net` (`core/src/search/mod.rs`: unit-free MLL with no
literal more than twice) therefore costs nothing where it applies on one
thread, and is the only engine that is not bounded by the recursion
limit there. On a pool it costs a great deal (R13): the baseline's
`engines` runs are on one thread and did not see that the command's
default, every core, makes exactly these sequents quadratic. My reading:
leave D8's row as it is, write down that it is kept for width and for
the net as a result, not for speed, and repair the cubes in step 19. Pruning the net engine
(leaf symmetry, a per-atom balance) would make that engine faster on
inputs the dispatch never sends it; it buys no verdict. A loop for chains
of free splits in the focused engine would remove the last case and
belongs with that engine's follow-ups, not before them.

### 2.5 The atom bias, the copy bound and the portfolio

- **The combination keeps its contract** across the library with two
  exceptions at the limit: `ResAllocation_RAS-C-100_5_1` (the backward
  search alone 2.71 s, the default over 5 s) and
  `Diffusion2D_2D8_gradient_40x40_100_5_1` (4.50 s). The first is inside
  the two thirds of the limit that the scheme was argued to keep, so the
  argument does not hold in seconds: the shares are counted in work.
- **What it costs against the better rule**: 1.29 times the backward
  search alone and 1.64 times the forward search alone, in the median on
  the nets both decide.
- **What it gains**: 1 520 nets against 442 under the backward search
  alone.
- **What is left on the table**: 67 nets that the forward search alone
  decides within 5 s and the default does not, and 288 problems outside
  the nets that the forward search decides at 30 copies and the default
  leaves at its bound of 3, because they are no Horn programs. The second
  is the copy bound's question, and by 2.2 it is the larger one: a default
  that deepens while its time lasts, instead of stopping at 3, takes the
  288 and the 196 without a flag. What a larger fixed bound costs is also
  in the rows: at a bound of 10, 293 of the 832 problems that answered
  "copy bound" in a fraction of a millisecond run into the 5 s limit
  instead. So the bound cannot simply rise; it has to be tied to a budget
  of time. This is question 2.
- **The portfolio** (`Options::portfolio`) decides 627 against 623 with
  10 gained and 6 lost at the same time (1.01×). It has shown no gain in
  two baselines and on step 13's families. It should go.

### 2.6 Robustness: what a call can do to the machine

Found by running the command (`target/release/linlog`, built from the
head) on the baseline's largest problems, each in a scope capped at 6 or
8 GiB without swap, pinned to one to four cores. The sequents were
written from the LLTP files by a script that assembles `axioms |-
conjecture` as `lltp::read` does, since the command cannot read an LLTP
file.

| # | the call | what happens | where the cause is |
|---|---|---|---|
| R1 | `prove -i --jobs 1` on `TokenRing-40-unfolded_1_1` (65 643 clauses), default output, with `--timeout 5s` | the search answers in 43 ms within 68 MB; the command then takes 6 GiB in 3.5 s and is killed. The time limit does not apply any more | `proofs::check::derive`, reached through `Proof::derivation` in `cli/src/prove.rs::derivation` |
| R2 | the same with `--format rocq` and with `--format svg` | the same, 6 GiB in 3.4 s | the same |
| R3 | the same with `--format json`, then `linlog check -i` on the 5.5 MB file | JSON is written in 0.3 s within 73 MB; **`check` then takes 6 GiB in 3.3 s** | `proofs::check::check` itself. New: the review had JSON as unaffected, and it is until someone checks the file |
| R4 | `interact -i` on the same sequent, command `close` | 6 GiB in 3.5 s | the graft of the proof found (`Derivation::of_goal`) |
| R5 | `prove --quiet` on any of these | fine, because **`--quiet` and `--format json` never check the proof**: `prove_until` checks only in debug builds (`debug_assert!`), and the command checks only by building the derivation | `core/src/search/mod.rs::prove_goal`, `cli/src/prove.rs::prove`. README's "returning a checked proof" is true of the default output only |
| R6 | `prove -i --jobs 1 --timeout 5s` on `GPPP_G-PPP-1000-10_10_1` | not ended after 45 s (the review measured 140 s). With `--jobs 2` it ends after 5.46 s; under `--bias rarer` in 54 ms at the recursion limit | the forward search between two polls |
| R7 | `prove -i --jobs 4 --timeout 5s` on `SYJ202+1.008` in cbv, a sequent of 19 KB | **not ended after 150 s. With `--jobs 2` it ends after 46 s** and reports 268 million stable sequents; with `--jobs 1` after 5.17 s. New in its extent: the baselines saw the kill at 10.5 s on sixteen threads only, and the command's default is every core | a pool's stop, on a small problem |
| R8 | the same problem, `--jobs 1`, 5 s | 918 MB, the memo at its cap of 2²⁰ entries after 5 s | `Options::DEFAULT_MEMO_LIMIT` counts entries. On a forest of 65 000 occurrences an entry is two bitsets of 8 KB, so the cap is 16 GiB a search and the default runs two |
| R9 | `prove -i --jobs 1` on `TokenRing-40-unfolded_100_1`, no `--timeout` (the command's default) | still running after 124 s, 851 MB and growing | no default time limit, no memory limit: 1 594 nets of the library behave like this |
| R10 | `prove -i --jobs 1 --timeout 1s` on `SYJ212+1.020` in cbv (95 MB) | ends after 88 s with 7.3 GiB; the search itself reports 32.8 s under its limit of 1 s | the limit starts after the parse (`prove` reads the sequent before `on_large_stack`), and the search's set-up on 12 million occurrences comes before its first poll |
| R11 | a formula nested 100 000 deep (`a * (a * (…))`) | `seq fragment` and `prove` abort with a stack overflow on the main thread (exit by signal, not status 2); 30 000 deep works but takes 2.4 s to name the fragment and 8.9 s to prove, so something before the search is quadratic in the depth | parsing or lowering on the main thread's stack |
| R12 | `--recursion-limit 4000000000` | a clean error (the thread's stack cannot be reserved) | fine |
| R13 | `wide-m1` at 256 and 512 pairs through the harness, `--jobs 1` and `--jobs 2` | 8 ms and 33 ms on one thread; **1.2 s and 9.5 s on two**, with 131 328 and 524 800 links tried against 512 and 1 024 | the net engine's cubes: on a sequent whose links are forced the enumeration never reaches its count of cubes and starts again at every depth (D11) |
| R14 | `prove --format text --timeout 10s` on `wide-m1` at 1 024 (a sub-agent's run) | exit 0 after 23.4 s with 76 MB of tree and 404 MiB; the SVG of the same proof is 212 MB | the limit ends with the search (D12); the renderer (D3) |

Ranked by what a user meets: R7, R13 and R9 first (the default command,
which runs on every core: on a small problem with a time limit, on a
wide MLL sequent, and on a large net without a limit), then R1 to R4 and
R14 (any proof of some size, through every format but two), then R8, R6,
R10 and R11. R5 is not a hazard but a gap between what the README promises and
what the default library call and two of the command's paths do.

Not probed: `qbf/48#0` to its end (290 s; the memo's growth is the
review's 15 MB a second) and anything on every core.

## 3. Every candidate, one by one

The candidates of `plan/later.md` in its order. "Outside" facts were
checked on 2026-10-03 by a sub-agent against primary sources (the
repositories' APIs and files, package indexes, Crossref and arXiv); the
source is named with each. A session is one of the kind the plan has run.
Model and effort follow "Why these models and efforts" in
`plan/README.md` and today's model documentation
(platform.claude.com/docs/en/about-claude/models/overview and
`…/choosing-a-model`, `…/build-with-claude/effort`): the lineup and the
prices are those of 2026-09-29 (Fable 5.1, Opus 5.5, Sonnet 5.5, Haiku
4.5), and the advice is unchanged: start with Opus 5.5, move to Fable 5.1
where `xhigh` falls short on demanding reasoning.

### 3.1 Code audit and refactoring

*For.* Whoever maintains the code, which is the author and every later
session. *Premise.* Holds, and section 1 is its audit: the ranked list is
there, so the candidate is now the refactoring alone. *Shape.* Two
pieces that differ in kind. The library's surface, the exports, the
command, the harness and the documentation: breadth, no soundness
argument, Opus 5.5 at `xhigh`, one to two sessions. The focused engine:
its file split, one engine interface, the cut and dependency flags as
part of a step's result instead of engine-wide state, one worker
constructor, each rule written once for one thread and for the pool, and
the three hot spots worth taking (hashing the memo keys, the allocation
of an insert, the canonical key built per stable sequent); Fable 5.1 at
`xhigh`, one to two sessions, every commit under the counter oracle of
`bench/targets.sh`. *Risk.* The flags: an early return between the save
and the restore lets a level claim `Unprovable` wrongly, and it is the
one place where a refactoring changes a verdict in silence. *Checked by.*
Tests and flake checks after every commit; JSON forms and snapshots
unchanged; identical counters on the target set. *Verdict.* Do it, after
the robustness steps (so that defects are fixed once, in code people have
read) and before the web front end, the batch mode and any new engine.

### 3.2 Configurable output, and no font in LaTeX and Typst

*For.* Everyone who pastes linlog's output into a document; the web front
end's settings. Decided by the author (D12 as amended, D15). *Premise.*
Holds: `export::latex` still loads `eulervm` and `export::typst` still
sets Euler Math in standalone documents, and of the four exports only SVG
(`Style`) and Rocq (`rocq::Options`) take an options value; the command
passes the defaults and has no flag for either (`cli/src/prove.rs`).
*Outside.* The Typst limit the export follow-ups wanted reported is
reported and closed: curryst issue 19, "not planned", the maintainer
pointing at Typst, whose 0.15.1 still has `MAX_SHOW_RULE_DEPTH = 64`
(github.com/pauladam94/curryst/issues/19; typst v0.15.1,
`crates/typst-library/src/engine.rs`). curryst is still 0.6.0 and ebproof
2.1.1. So the sketch's second branch is what remains: a Typst tree from
linlog's own layout, as an option beside curryst. *Size.* Two sessions,
Opus 5.5 at `xhigh` (it wrote these exports): the font removal, the
options types and the `--style` surface; then the Typst layout and the
compact view of structural runs, which the bounded derivation needs for
large proofs to be showable at all. *Checked by.* The `export` check with
no font in its closure for LaTeX and Typst; snapshots; a deep tree
compiling under Typst. *Verdict.* Do it, early: it is wanted, cheap, and
the web front end's settings are its JSON.

### 3.3 Net-engine pruning and routing

*Premise.* Gone for verdicts (2.4): the focused engine decides in
microseconds everything the pruning was to fix, and the dispatch never
sends the net engine a sequent of the kind it loses on. The routing
question is answered: keep the row, for width. *Verdict.* Not worth
doing now. It returns only if nets as a search procedure become a
teaching subject of their own (question 7). Deferred.

### 3.4 MELL proof nets with exponential boxes

*For.* Teaching and papers: nets with boxes are how exponentials are
drawn, and the suite draws nets only for MLL. *Builds on.* `nets/`,
`export::svg::net`, the proof term's `Bang`, `Quest` and `Copy`.
*Outside.* Guerrini and Masini, "Parsing MELL proof nets", TCS 254
(2001), exists as cited. *Size.* Two sessions, Fable 5.1 at `xhigh`: the
structure with boxes, the criterion per box depth and both conversions
(soundness work, with a fresh-context reviewer against switching
enumeration as step 5 had); then the drawing. *Risk.* The choice of `?`
nodes decides whether desequentialization is canonical; weakening makes
correctness of nets with `?w` and `⊥` delicate. *Checked by.* Round trips
proof → net → proof through the checker; the criterion against brute
force on small structures. *Verdict.* Worth doing if the author teaches
with nets (question 7); after the refactoring, independent of the
engines.

### 3.5 Essential nets for IMLL

*Premise.* Step 8 proved the verdict needs no essential-net condition,
and the baselines show no IMLL input on which the embedding is slow.
*Outside.* Lamarche's report is on HAL (inria-00347336, 2008); Murawski
and Ong, LICS 2000 and TOCL 2006; Moot's paper is from 2004 (the arXiv
posting is 2008, which the sketch cites). *Verdict.* Not worth doing as
an engine. As an object to show (a polarised net with its dominator
tree) it is a variant of 3.4's drawing; deferred with it.

### 3.6 The focused inverse method

*Premise.* Two cases were left to it. Mix: the cheaper answer is the
prune the follow-ups name (`n·2^n` instead of `3^n`), a change inside the
focused engine. The nets beyond the forward search: 5.4 serves them
better than a database of sequents with subsumption would. *Verdict.*
Deferred; nothing the baselines show calls for it.

### 3.7 The !-Horn fragment through Petri-net reachability

*Premise.* Changed twice. The forward search already decides 1 520 of
the 3 137 nets; and the tool the sketch names is dead. *Outside.* KReach
(`dixonary/kosaraju`, BSD-3) was last touched on 2020-02-16; the
maintained tools are TAPAAL's verifypn and Mist, both GPL-3.0
(github.com/TAPAAL/verifypn, pushed 2026-09-30;
github.com/pierreganty/mist, pushed 2026-09-11), so they could only be
called as programs, never linked. Reachability is Ackermann-complete
(Leroux and Schmitz, LICS 2019; Czerwiński and Orlikowski, and Leroux,
FOCS 2021). *Verdict.* Not as sketched. Replaced by 5.4, a native engine,
which also gives the affine case a decision procedure.

### 3.8 Cyclic MLL and the Lambek calculus

*For.* Categorial grammar. *Outside.* The embedding of the Lambek
calculus into first-order MILL is Moot and Piazza, JoLLI 10 (2001); Moot's
LinearOne (first-order MILL, Prolog, LGPL-2.1) was pushed on 2026-06-11
and is the one maintained prover in that area. *Size.* Two sessions,
Fable 5.1 at `xhigh` (a planar linking search, a non-commutative mode
through the derivation view and every export). *Verdict.* Only if the
author's research needs non-commutative logic (question 7); deferred
otherwise.

### 3.9 First-order linear logic

*For.* Research that needs quantifiers: linear logic programs, parametric
nets, categorial grammar through MILL1. *Outside.* The sketch's facts
hold and one sharpens: first-order MLL is in NP and first-order MALL in
NEXPTIME (Lincoln and Shankar, LICS 1994) and NEXPTIME-hard (Lincoln and
Scedrov, TCS 135, 1994), so MALL1 is NEXPTIME-complete; the spec's
"membership is not established" is wrong. No first-order problem library
for linear logic exists; LinearOne is the only maintained first-order
prover; Lean's FormalizedFormalLogic has a first-order LL¹, NanoYalla
has none. *Size.* Six sessions or more, the data model alone first
(Fable 5.1 at `xhigh` throughout): terms and binders in the arena, the
parser, the printer and the JSON; unification and a trail in both
engines, with every prune of step 15 gone through again; witnesses in
proofs, the checker, the views, the interactive rules, four exports.
*Risk.* It is the one candidate that can slow or break the propositional
case, which is everything the suite does today, and it has no benchmark
to be measured by. *Verdict.* Not now, unless the author's work needs it
(question 4). If it does, the data model comes before the refactoring
and the certificate library, which both fix types it changes.

### 3.10 A Rocq library of linlog's own; NanoYalla kept for compatibility

*For.* Research: a paper that cites the tool expects a certificate for
what it proves, and today intuitionistic proofs are certified
classically and Mix and affine proofs not at all. The shape is the
author's. *Outside.* Rocq is at 9.3.0 (2026-09-19); nixpkgs defaults
`rocqPackages` to 9.1 on unstable and on 26.05 and has neither Yalla nor
OLlibs; the standard library is the package `rocq-stdlib` with the
logical path `Stdlib`, and the reference manual names `_RocqProject`
with `rocq makefile` and calls dune's support experimental
(rocq-prover.org/doc/V9.3.0/refman/practical-tools/utilities.html).
`microyalla/nanoill.v` is still there and still imports nothing. Three
corrections to the sketch: the Mix theorem it calls "sketched on paper,
not machine-checked" is machine-checked in Yalla (`mix2_to_ll` in
`ll_fragments.v`, without cut), so the bridge can follow that proof;
Yalla's general Mix (`pmix`) is on its untagged master (2.1.0, tested
with Rocq 9.2), not in the opam release 2.0.7; and a Lean target exists
today, since `leanprover/cslib` has one-sided classical linear logic with
all four units, exponentials and cut elimination
(`Cslib/Logics/LinearLogic/CLL`), which FormalizedFormalLogic still
lacks. Yalla's `lj.v` also has LJ with machine-checked translations into
ILL, which bears on 3.13. *Size.* Three to four sessions as the sketch
stages them, Fable 5.1 at `xhigh` for the checker and its soundness
proof and at `high` for the rest. *Risk.* The soundness proof over the
least unrestricted zone and the absorbing `⊤`; the Rust checker changing
under it (5.1 rewrites `derive`, so the library is written after that).
*Checked by.* A flake check that builds the library and every
certificate and prints the assumptions of the main theorem (none).
*Verdict.* Do it; when is question 6.

### 3.11 MALL proof nets

*Outside.* Hughes and van Glabbeek (TOCL 2005) and Hughes and Heijltjes
(LICS 2016) exist as cited; nothing changes the sketch's judgement.
*Verdict.* Dropped until a use appears.

### 3.12 A batch mode for the command

*For.* Research use (a file of sequents, a directory of problems), a
script or an editor that asks many questions. *Builds on.*
`cli/src/prove.rs`, `lltp::read`, the harness's problem files; needs 5.1
(isolation rests on a memory bound) and 5.2 (cores across sequents or
within one is the default thread count's question). *Premise.* Holds: the
command takes one sequent and cannot read an LLTP file; this session had
to write a converter to probe the library's problems at all. *Size.* One
session, Opus 5.5 at `xhigh`. *Checked by.* The batch's results equal
the single calls' on the problem file and an LLTP sample; a timing of
what the loop paid. *Verdict.* Do it, after the defaults.

### 3.13 Ordinary logic through its embeddings

*For.* Teaching (how classical and intuitionistic logic sit inside
linear logic) and the user with an ordinary formula in hand. *Premise.*
The obstacle the sketch names, the copy bound, is real and 5.2 removes
most of it in practice: with larger bounds linlog decides all but three
of the translated problems the Maude prover decides. A decision
procedure it is not. *Outside.* No result bounding the copies on the
image of a translation was found. The terminating calculi of the
literature work on the intuitionistic side: Dyckhoff's contraction-free
LJT (JSL 57, 1992, corrected 2018) and loop-checked LJ (Heuerding,
Seyfried and Zimmermann 1996; Howe 1997). The ILTP library's
propositional part is 274 problems (128 theorems, 112 non-theorems, 34
open) in TPTP syntax with no licence stated (iltp.de/formulae.html), so
it can be fetched by the flake as LLTP is, not committed. *Shape.* The
layer first (the syntax, the three translations as functions, the image
printable, classical logic decided through affine MALL, intuitionistic
and minimal logic through ILL under the deepening default, "unknown"
where it is unknown): one session, Opus 5.5 at `xhigh`. Then, only if
wanted, termination (a loop check on dyadic sequents, which is engine
research with a risk of not paying, Fable 5.1 at `xhigh`) and the
read-back to LK and LJ. *Verdict.* The layer is worth a session for
teaching, after 5.2; the termination work is not, until the layer has
users. Question 7.

### 3.14 The web front end

*For.* Teaching: a student proves a sequent by clicking, without
installing anything. README names it as the crate to come. *Builds on.*
`Interactive` and its JSON, `export::svg`, the options of 3.2; the
library compiled without `parallel`. *Outside*
(doc.rust-lang.org/stable/rustc/platform-support/wasm32-unknown-unknown.html
and the crates' pages): on `wasm32-unknown-unknown` `std::thread::spawn`
panics and `Instant::now()` panics (`web-time` 1.1.0 replaces it);
rustc links with a stack of 1 MiB, which a link argument raises;
threads need nightly, `build-std` and cross-origin isolation, which
GitHub Pages cannot set; wasm-bindgen is 0.2.129, wasm-pack lives on in
the wasm-bindgen organisation (0.15.0, 2026-05-15), trunk is 0.21.14.
*What that means here.* A single-threaded client on GitHub Pages is
unproblematic. The focused engine recurses to 2 048 levels of about
1 KiB, over the default stack, so the build raises it or lowers the
limit; no rewrite is needed to start. Without threads `Bias::Auto` runs
its two searches in restarting turns (up to five times the better
search, other counters than the command's): acceptable for a first
version, and the reason an explicit-stack engine is the eventual fix.
What must exist first: a serde form of `search::Options`; the bounded
derivation (a tab must never build a tree of gigabytes); a stop that
counts work; per-formula ids in the SVG for clicks (the export
follow-ups). *Size.* Three sessions with a plan of their own: the wasm
build as a flake check and the bindings (JSON in, JSON and SVG out);
the client; hosting beside the documentation site. Opus 5.5 at `xhigh`.
*Risk.* Scope: a web application grows without end; its plan needs a
first version that is `interact` with a mouse and no more. *Verdict.* Do
it, after 3.1's surface and 3.2; where in the order is question 1.

### 3.15 Problems from practice

*Outside*, source by source. Coverability: `blondimi/qcover`
(Apache-2.0, last pushed 2021) holds 176 instances in five suites (Mist
27, the concurrent C programs of BFC 46, Erlang programs of Soter 50,
medical 12, bug tracking 41), each in Mist's `.spec` format, about
470 MB, with the expected result in the file; the benchmark files carry
no licence statement of their own. The literature counts 61 unsafe and
115 safe instances (FastForward, arXiv:2010.07912; read from a summary,
not the paper). These are real non-theorems, and coverability is affine
mode. Model Checking Contest: 137 models now, of which LLTP used 76;
submitted models are public domain. Planning: no collection encoded in
linear logic exists; problems would have to be translated from PDDL
here. Granule's synthesis benchmarks are graded signatures with data
types and polymorphism; only a few are propositional ILL. llprover's
"collection" is one file of 70 lines of textbook examples, partly
first-order, without a licence. *Verdict.* The coverability suite is the
one set worth taking, fetched at a pinned commit by the flake as LLTP is.
It is of use only with something that can refute: affine mode today is
a bounded search, so the 115 safe instances would answer "unknown".
Hence it goes with 5.4. Granule and llprover's file are dropped as
benchmarks; planning is deferred.

### 3.16 The follow-up lists

Brought up to date in section 1.5; each standing entry is assigned to a
step in section 6.

## 4. How the candidates interact

**What must come first, and why.**

- *The bounded check and derivation (5.1) before everything that shows a
  proof*: the batch mode, the web front end, configurable output, MELL
  nets and the certificates all hand large proofs to `derive` or to
  `Derivation::build`. A browser tab that takes 6 GiB is worse than a
  terminal that does.
- *The stops and the memory bound (5.1) before the defaults (5.2), and
  both before the batch mode*: a default that deepens while its time
  lasts relies on the time limit being kept (R6, R7), and a batch in one
  process relies on one sequent not taking the others down (R8).
- *The defaults (5.2) before ordinary logic*: its sketch says so; an
  embedding that answers "unknown" at three copies is of no use.
- *The refactoring of the library's surface before the web front end,
  the batch mode and the certificate library's exporter*: all three are
  new callers of the API (search options with a serde form, the output
  options of D15, a size estimate, a stop that counts work), and each
  would otherwise invent its own.
- *The answer on quantifiers before the refactoring settles the types
  and before the certificate library is designed.* If quantifiers are
  wanted, `Term`, `Forest` and the proof term change under every other
  candidate, and the Rocq inductives have to be planned with binders
  from the first file.

**What competes for the same code.**

- *Configurable output and the refactoring of the exports* both rewrite
  the label and symbol tables of `export/`. Doing D15 first and the
  refactoring second means the refactoring tidies the final shape;
  the other order writes the tables twice. They are cheaper as one step.
- *A compact derivation view* (a run of structural rules as one
  inference; "Follow-ups: the command's output") changes the type every
  export reads. It belongs in the step that bounds the derivation, or in
  the export refactoring, not in between.
- *The focused engine's follow-ups, a suspended search (explicit stack)
  and a native engine for Horn programs (5.4)* all aim at the nets. The
  Horn engine would take the nets from the focused engine's forward
  search, after which the unit of work, the restart from the frontier and
  the per-step cost on wide forests matter much less. So the Horn engine
  is decided first, and the focused follow-ups that only serve nets are
  dropped if it is built.
- *First-order logic and everything else*: it goes through every layer,
  so whatever is built before it is built twice in part.

**What is cheaper together.**

- The command's LLTP input, the batch mode and the practice problem sets:
  one reader and one loop serve all three, and the batch mode is what the
  problem sets are run with.
- The font removal, the output options and the `interact` certify command
  (one step, as the sketch has it).
- MELL nets with boxes and their SVG drawing; cut elimination on nets, if
  wanted, with either.

**What closes off or opens up.**

- The web front end opens up nothing in the engines but fixes the API:
  after it, a change of the JSON forms or of `Interactive` is a change of
  two code bases.
- The certificate library opens intuitionistic, Mix and affine
  certificates, and a certificate for ordinary logic's read-back.
- Ordinary logic opens the ILTP propositional library as a benchmark
  whose statuses are right, which the LLTP translations are not (28
  files).
- A release (5.6) closes off free renaming: from then on the Rust API
  and the command's flags are promises.

**What the refactoring should settle before new work, and leave alone.**
Settle: the public surface (what is `pub`, `#[non_exhaustive]`, error
types), a serde form for `search::Options` and one options value per
output (D15), the duplicated tables and walks of the exports, the
crate-wide lint allowances, the documentation's drift. Leave alone until
the engine work is decided: the control flow of `focus/mod.rs` (a split
into files along its existing seams is safe under the counter oracle; a
rewrite to an explicit stack is its own step and only worth it for the
web front end or if the thread per call hurts), and the net engine, which
nothing is planned on.

## 5. What is missing

Candidates that the list does not have and that the repository, the
baselines or the project's purpose call for.

### 5.1 A call that keeps its limits (robustness)

*What.* The defects of 2.6, as one piece of work in two sessions.
(a) The check and the derivation: `check` in memory linear in the proof
(what a node derives shared along a branch or freed when its last
premise has read it); a size estimate of a derivation that does not build
it; the command's rule for a tree that does not fit (the author's wish of
2026-10-03: not printed, with a manual override) and the safety bound
everywhere else, as an option of the library (D15); `interact`'s `close`
under the same bound; the harness writing the search's verdict before it
checks; and a decision on R5, whether `prove` checks what it returns in
release builds once the check is cheap. (b) The search: the forward
search's stop on one thread (R6), a pool's stop (R7), a memory bound in
bytes for the memo and the arena with `Reason::MemoryLimit` (R8), the
time limit counted from the command's start (R10), an error instead of
an abort on a formula nested too deep (R11).
*For whom.* Everyone; it is what "usable in practice" means first.
*Builds on.* `proofs/check.rs`, `proofs/derivation.rs`, `cli/src/prove.rs`,
`focus/mod.rs` and `focus/parallel.rs`, `search/parallel.rs`.
*Size and model.* Two sessions, Fable 5.1 at `xhigh`: the checker anchors
soundness, so its rewrite needs a differential test against the old one
on every proof the tests and the families produce, and the stops are in
the engine's hot loops.
*Risk.* A checker that is cheaper and wrong; mitigated by keeping the old
`derive` as the test oracle. *Checked by.* The twelve calls of 2.6 with
their caps, each ending within its limit and its memory; the target set's
counters unchanged; the 14 crash rows of the baseline decided.

### 5.2 The defaults a user meets

*What.* The default copy bound as a deepening that goes on while a
default time budget lasts, reporting the bound it reached, with
`--copies N` as a cap for whoever wants one and `--timeout` for the
budget; a default time limit at all (R9); what "unknown" says to the user
(which bound, how long, what to try). The default thread count is part of
it (question 3).
*For whom.* The user without flags, which is every student and most
first uses.
*Builds on.* The deepening loop of `focus::run`, `Options`, the command's
flags; needs 5.1(b).
*Size and model.* One session, Fable 5.1 at `high`: little code, but the
meaning of `Unprovable` and `CopyBound` under a budget, and the forward
bound's relation to it, are the engine's contract.
*Checked by.* `lltp-copies-10`'s 1 003 problems and the 1 342 that the
Maude prover has results for, by day on two pinned cores (about an hour
and a half; the step would name the run): the default should decide what
`--copies 10` and the forward pass decide, and lose none of what it
decides today.

### 5.3 Why a sequent is unprovable

*What.* For a refuted sequent, a reason a student can read: the atom
whose counts do not balance, the count equation that fails, for MLL the
switching cycle of every attempted linking where there is one cheap to
state, otherwise "the search was exhaustive" as today. The focused engine
has the first two at the root already (`Counts`, `Tally`).
*For whom.* Teaching. A refutation without a reason teaches nothing.
*Size.* Small; folded into the step on defaults or on configurable
output. *Checked by.* Snapshot tests of the messages.

### 5.4 A native engine for Horn programs (the Petri-net candidate, reshaped)

*What.* The sketch calls an external reachability tool. The baselines
suggest something else: the nets are the one real-world set, 1 594 of
them end at the time limit, every one is a theorem whose proof is a
firing sequence of at most 150 steps, and the forward search that finds
such sequences pays for full-width bitsets and a deepening from the root
at every step. A small explicit-state engine for sequents that `chains`
recognises (markings as count vectors, transitions indexed by their input
places, a visited set, the proof term built from the firing sequence)
does the same search at the cost of a transition per step. In affine mode
the same data gives the backward coverability algorithm, which decides:
the first decision procedure for an affine fragment in the suite, on the
fragment where coverability problems from practice live.
*For whom.* Research use on nets; the efficiency the author asked for,
on problems people bring.
*Builds on.* `focus::chains` (the Horn test), `Proof`, the dispatch; no
change to the focused engine. *Size and model.* Two sessions (reachability
with proofs; coverability with its termination argument), Fable 5.1 at
`xhigh`. *Risk.* Reachability by plain forward search is still
exponential on nets with large markings; the measure says whether it
beats the forward focused search, and if it does not the step stops
after its first session. *Checked by.* The 1 594 nets and the coverability
suites of "Problems from practice", against the forward pass.

### 5.5 Cut, and cut elimination

*What.* A cut rule in interactive proofs (the user names the cut
formula), proof terms with cuts, the checker's rule for them, and cut
elimination step by step: on terms, and on MLL nets, where it is the
reason nets exist. Every proof the suite produces today is cut-free, and
the forest holds only the sequent's subformulas, so cut formulas need
roots of their own beside the sequent's.
*For whom.* Teaching above all: cut elimination is what a course on
linear logic shows, and no drawing of it exists in the suite.
*Size and model.* Three sessions (the data model and checker; elimination
on terms; on nets with the drawing), Fable 5.1 at `xhigh`. *Risk.* It
changes `Forest` and `Proof` under everything; it interacts with
first-order logic and with MELL nets. Only if the author wants it
(question 7).

### 5.6 A release

*What.* A version a paper can cite: a tagged 0.1.0, the crates published
(the names `linlog` and `linlog-cli` on crates.io were not checked), a
changelog, `CITATION.cff`, the documentation site per version, and the
baseline's tables as the reproducible record of what the release decides.
*For whom.* The reader of a paper that cites the tool. *Size.* One
session, Opus 5.5; publishing itself is the author's act. *When.* After
5.1 and 5.2 at the earliest, since a release fixes the defaults.

### 5.7 The wrong LLTP headers, upstream

Twenty-eight files, each with a checked proof or a classical
countermodel; the Maude prover's own result files contradict twelve of
the headers. A session can prepare the report (the list, the proofs as
JSON, the countermodels, the one malformed file); sending it is the
author's.

## 6. The order I propose

A step keeps the plan's form: a whole number, one prompt, as many
sessions as it takes. The order below is the proposal before the
author's answers; questions 1, 4, 6 and 7 can move or strike steps.

| # | Step | Sessions | Model, effort | Needs |
|---|---|--:|---|---|
| 18 | The proof check and the derivation within bounds | 1 | Fable 5.1, `xhigh` | – |
| 19 | The search within its limits | 1 | Fable 5.1, `xhigh` | – |
| 20 | The defaults a user meets | 1 | Fable 5.1, `high` | 19 |
| 21 | Configurable output, no font, a Typst layout of linlog's own | 2 | Opus 5.5, `xhigh` | 18 |
| 22 | The library's surface, the command, the harness and the documentation in order | 1–2 | Opus 5.5, `xhigh` | 21 |
| 23 | A batch mode, and LLTP input for the command | 1 | Opus 5.5, `xhigh` | 19, 20, 22 |
| 24 | The web front end | 3 | Opus 5.5, `xhigh` | 18, 21, 22 |
| 25 | A Rocq library of linlog's own | 3–4 | Fable 5.1, `xhigh` and `high` | 18 |
| 26 | The focused engine in order | 1–2 | Fable 5.1, `xhigh` | 19 |
| 27 | An engine for Horn programs, coverability, and the coverability suite | 2 | Fable 5.1, `xhigh` | 23, 26 |
| 28 | The third baseline, and a release | 1 and a night | Opus 5.5, `xhigh` | 27 |

**18. The proof check and the derivation within bounds.** *Goal.* A
proof of any size the search can find is checked in memory linear in the
proof, and no front end builds a derivation it cannot afford: the size is
estimated first, the command leaves out a tree that does not fit the
terminal and, anywhere, one past a safety bound, saying so and how to get
it. *Boundary.* No engine change; no new view of a derivation (21). *Folds
in.* The checker's `derive` (benchmark follow-ups), "A proof tree that
does not fit is not printed" (the command's output), the harness writing
the verdict before it checks and counting its kill from the end of the
load, `interact`'s `close` under the bound, the text tree laid out in
two passes as the SVG is (D3), the time limit and Ctrl-C reaching the
building and writing of a derivation (D12), and the gap R5: the command
checks every proof it reports, `--quiet` and `--format json` included,
once that is cheap. *Why Fable at `xhigh`.* The checker is what soundness
rests on; its rewrite needs an argument and a differential test against
the old one. *Verified by.* R1 to R5 of 2.6 within 1 GiB; the 14 crash
rows decided with `checked` ok; old and new `derive` agreeing on every
proof of the tests and the families; snapshots unchanged.

**19. The search within its limits.** *Goal.* A time limit is kept to
within a fraction of a second on one thread and on a pool, counted from
the command's start, and a search answers "unknown" with a reason
instead of exhausting memory. *What.* A poll in the forced chains
(`forced_splits`, `literal_tensor`, the quadratic `dual_in`), the pool's
stop on `SYJ202+1.008` (cause not yet known), the wait in `alternate`,
the set-up before the first poll; the net engine's cubes on sequents
whose links are forced (D11) and a bound on `--jobs` (D13); a bound in bytes on the memo, the
arena and the per-level counts, with a new `Reason`; the boundaries: a
JSON sequent with a repeated atom name is canonicalised or refused (D1),
a forest past a bound on its occurrences is refused (D5), a formula
nested too deep is an error and not an abort; `Options::portfolio`
removed;
the false assertion of `focus::parallel::tests::agree` corrected.
*Boundary.* No change of what is searched: the target set's counters
stay identical. *Folds in.* "The search's own memory", "The forward
search misses its stop", the parallel follow-ups' missed stop and
portfolio. *Why Fable at `xhigh`.* The hot loops of the engine and its
parallel layer. *Verified by.* R6 to R11 and R13; D1 and D5 refused or
decided rightly; the stop tests; the target set against
`after-bias.csv`.

**20. The defaults a user meets.** *Goal.* `linlog prove SEQUENT` with
no flags answers within a default budget of time, deepening the copy
bound while the budget lasts, and its "unknown" says what was tried and
what to try; a refuted sequent says why where the counts know.
*Boundary.* The explicit flags keep their meaning; the harness passes
explicit bounds, so the baselines stay comparable. *Folds in.* The copy
bound's question, 5.3, the default thread count (question 3), "the
default's contract at the limit" as far as a different share fixes it.
*Why Fable at `high`.* Little code, but it restates what `Unprovable`
and `CopyBound` mean. *Verified by.* The run 5.2 names.

**21. Configurable output.** As 3.2: first session the font removal, one
options value per export with serde, `--style` and `--style-file`,
`--lemma` and `--prelude`, certifying a finished `interact` session,
per-formula ids in the SVG; second session the Typst tree from linlog's
own layout and the compact view of runs of structural rules.
*Boundary.* No new export target. *Folds in.* The export follow-ups.
*Verified by.* The `export` check without a font for LaTeX and Typst;
every option set from the command and from JSON in a test.

**22. The surface in order.** *Goal.* The lists of sections 1.1 and 1.2
outside the engines: a serde form for `search::Options`, what is `pub` and need not
be, the duplicated walks and tables, the lint allowances removed, the
command's repeated dispatch, the harness's one-offs, and every stale
claim of section 1.1 corrected. *Boundary.* No behaviour of the command
changes; the JSON forms and the snapshots do not change; the engines'
files are not touched beyond their public signatures. How far the Rust
API may break is question 5. *Verified by.* Every check after every
commit; the target set's counters.

**23. Batch mode.** As 3.12.

**24. The web front end.** As 3.14; its prompt is finished after the
reviews of 21 and 22, since it is written against their API.

**25. The Rocq library.** As 3.10, in the sketch's stages. It depends on
nothing but 18 and can run whenever the author wants it (question 6).

**26. The focused engine in order.** *Goal.* Section 1.2's list inside
`search/`: the file split along its seams, one engine interface, the cut
and dependency flags as values, one worker constructor, each rule once,
the three hot spots. With it, from the focused follow-ups: the Mix prune
(`n·2^n`), a loop for chains of free splits, the Horn test on the goal's
members. *Boundary.* Identical counters on the target set at every commit
that claims no change; a commit that changes the search says so and is
measured. No explicit-stack rewrite unless 24 has shown the need.
*Verified by.* `bench/targets.sh` per commit; a fresh-context
differential review as step 15 had.

**27. Horn programs.** As 5.4 and 3.15. Its prompt is finished after 26.

**28. The third baseline and a release.** The baseline under the new
defaults and beside the old flags, the comparison, and 5.6 if the author
wants it (question 9).

**Deferred, with the reason.** Ordinary logic's layer and MELL nets with
boxes: worth a session and two, placed by question 7. Cut and cut
elimination (5.5), the Lambek calculus (3.8), first-order logic (3.9):
only on the author's word. Net-engine pruning (3.3), essential nets
(3.5), the inverse method (3.6): the baselines give no reason.
**Dropped.** MALL nets (3.11); the external reachability tool (3.7);
Granule's and llprover's files as benchmarks (3.15); the restart of a
copy-bound level from the frontier and the unit-of-work tuning, if 27 is
built.

## 7. The author's decisions

Each can be answered in a line. The recommendation is first.

1. **What comes after the surface is in order (step 22)?** (a) As
   proposed: the batch mode, then the web front end, then the Rocq
   library, then the engine work for nets. (b) Research first: batch,
   the Rocq library, the Horn engine and the coverability suite, the web
   front end last. (c) Teaching first: the web front end straight after
   22, with the teaching objects of question 7 next. I recommend (a): the
   batch mode is one session and everything measured later runs through
   it; the web front end is what students see and it fixes the API while
   that is cheap to change; the Rocq library depends on nothing and can
   move freely.
2. **What does `linlog prove` do with no flags?** (a) Deepen the copy
   bound while a default time limit lasts (10 s, `--timeout none` to lift
   it, `--copies N` as a cap), and say which bound was reached. (b) As
   today: a bound of 3 and no time limit. (c) A fixed bound of 10. I
   recommend (a): the bound of 3 is the whole difference to the Maude
   prover's results (196 theorems) and leaves 727 problems of the
   library; a fixed 10 turns 293 answers that take a fraction of a
   millisecond into waits without end; and without any time limit 1 594
   nets of the library run until interrupted. The price of (a) is that an
   answer near the limit depends on the machine, so scripts and tests
   name their bounds.
3. **The command's default thread count.** (a) One thread first, and
   every core if that has not answered within a tenth of a second.
   (b) Every core from the start, as today. (c) A smaller pool. I
   recommend (a). The second baseline's rows alone do not argue against
   every core: 1.1 ms against 4.6 ms in the median, 106 problems more
   than 100 ms faster on sixteen threads against 15 slower, 37 decided
   only with the pool. But trying the command found three things the
   rows did not show, all on the pool: a time limit missed by minutes on
   a small problem (R7), the net engine's own sequents three hundred
   times slower (R13), and `partition-yes/24` lost on every pool. Step
   19 repairs the first two either way. One thread is the reference
   every review tested; with (a) every small problem is answered by it,
   deterministically and in microseconds, and the pool is kept for the
   problems where its gain is seconds. In a batch the cores go across
   the sequents instead.
4. **Do your research or teaching need quantifiers?** I recommend no for
   now: six sessions through every layer, no problem library to measure
   by, and a risk to the propositional case. If yes, the data model is
   planned before steps 22 and 25.
5. **How much may the refactoring change?** (a) The Rust API may break
   (renames, items made private, `Options::portfolio` removed, one engine
   interface), since nothing is released; the command's behaviour, the
   JSON forms, the snapshots and the counters may not. (b) Nothing
   public changes. I recommend (a); a change of behaviour the work finds
   necessary is reported and committed on its own, never slipped in.
6. **The Rocq library: when, under what name, published where?** I
   recommend after the web front end (or beside it, by a second session,
   since it shares no file with it), in this repository under `rocq/`,
   built by the flake only until a release, and the name is yours to
   give (the opam convention would be `rocq-linlog`).
7. **Which of these do you want, and which not at all?** Ordinary logic
   through its embeddings (one session for the layer); MELL proof nets
   with boxes (two); cut and cut elimination, on terms and on nets
   (three); the Lambek calculus (two); essential nets as a drawing. I
   would take the first two after step 24 and leave the others until you
   ask.
8. **A derivation written to a file or a pipe** (you left it open on
   2026-10-03). I recommend: on a terminal the tree is shown only if it
   fits; into a file, a pipe or a drawing format it is written unless
   its estimated size passes a safety bound (I propose 64 MiB of output
   as the default, an option of the library), in which case the verdict
   is written, standard error says why, and one flag lifts the bound.
9. **Two things only you can send.** A release (a tagged 0.1.0 on
   crates.io with a `CITATION.cff`), which I would place at step 28; and
   the report of the 28 wrong headers to `meta-logic/lltp`, which a
   session can prepare with the proofs and countermodels attached. Do
   you want either?

## 8. How this was established, and what was not

- **Read by this session:** `plan/README.md` in full, `plan/later.md`,
  the reports of steps 14 (in part), 15 and 16, the spec's overview and
  its sections on MALL, MELL, the intuitionistic fragments and the other
  fragments, `.claude/rules/core.md` and `cli.md`, `cli/src/prove.rs`,
  the front door of `core/src/search/mod.rs`, the checker's `derive`.
  The reports of steps 1 to 13 were read through the Status log's
  account of each, not again in full.
- **Read by sub-agents, in full, each with a brief that named its
  files:** `core/src/search/`; the rest of `core/src` and `core/tests`;
  `cli/`, `bench/`, the flake, CI, `.claude/` and the documents against
  the running program; the LLTP library's result files against the
  baselines' rows; the outside world's state for every candidate. Their
  findings are cited where used, and the ones the assessment turns on
  were run again here: R1 to R13 (R14 is a sub-agent's run), D1, and
  the counts of 2.1.
- **Run here:** clippy and the workspace's tests at the head (both
  pass); the probes of 2.6, each in a scope capped at 4 to 8 GiB without
  swap, on one to four pinned cores, the longest 150 s. No baseline,
  nothing on every core by this session.
- **A rule broken.** A sub-agent probing the command's handling of bad
  flags ran `--jobs 100000` and `--jobs 10000`, which loaded every core
  for about five minutes and gave the machine a load average in the
  thousands. It was not asked for and is reported as D13.
- **Not verified:** the cause of the pool's missed stop (R7); the text
  tree and the SVG above the sizes in D3 and R14 (extrapolated); whether
  the 19 rule labels outside the snapshots compile; `qbf/48#0` to its
  end; anything that needs every core or a night. The model
  documentation, the packages and the papers are as the sources said on
  2026-10-03; three were reached through search summaries only and say
  so where cited.
- **Scratch:** the probe scripts and the sub-agents' analysis scripts
  are in the session's scratch directory, outside the repository; none
  is committed.

## 9. The author's answers (2026-10-03), and what they changed

The answers, as given. "The proposed order looks good." Then, by
question:

1. "research and teaching should be eqal foci; in this case, implement
   the cli part first, since that's easier/less work"
2. "I want sensible defaults and the ability to fine-tune if needed.
   This should hold throughout the project!"
3. "same as above. if you're unsure, you can look what reputable sources
   say or how reputable projects handle it. But it sounds like a good
   idea in principle, so that running lots of small samples will not
   inflate the timings unnecessarily"
4. "Yes, they should be implemented eventually, so any architecture
   should keep that in mind. Though, the performance of the
   propositional calculi should not be meaningfully decreased! If this
   requires code duplication and/or generics, so be it. Go with whatever
   is best and most idiomatic currently"
5. "Yes, please! Make it as good as possible and don't care about API
   changes at all. Make sure the program runs efficiently, the
   code/project structure is idiomatic and best-practice and the outward
   facing API is ergonomic to use. Later changes to the API (once a
   version is released) will require version jumps."
6. "The timing is okay as shown, maybe also "linlog"? and for now just
   under rocq. Make sure the nix flake setup reflects that change
   idiomatically."
7. "All of them. Go with whatever order you recommend. Also, in the
   beginning the idea was that we have one most efficient solver (if
   deviating from the more general one) for each fragment and automatic
   smallest-fragment determination, so that each sequent has a tailored
   search algorithm that is as fast as possible. If there are factors
   other than fragment that might determine the algorithm to use, then
   that should also be implemented. Also go with the order that you
   recommend. You can also slot them into different gaps if it you
   recommend it"
8. "Sensible defaults, again"
9. "Only the release, you can draft the report somewhere and remember me
   when the time comes"

And beyond the questions: "at some point the different parts should
probably be separated into different repos", with release tags on
GitHub, publication on crates.io, in Rocq's package archive and maybe in
nixpkgs, checked for how and when and against the respective policies on
AI. The planning session's supervisor added four points through the
author: the JSON defect first, since it is a wrong verdict; step 19 as
proposed was too much for one session; the debug-assertion gap needs a
step that owns it; and sub-agents' briefs must carry the shared-machine
rules, in `plan/conduct.md`.

Asked, after part two, under which owner the first release goes out,
the author answered: "create an organization, linlog-prover". It is in
D22; creating it and transferring the repository are the author's acts,
and `plan/notes/distribution.md` lists what a transfer touches.

**What the answers changed.**

- *Decisions.* D16 (sensible defaults, every default an option), D17
  (quantifiers are coming, the propositional case does not pay), D18
  (the API is free until the first release), D19 (the fastest engine per
  fragment and feature, by measurement), D20 (the Rocq library is
  `linlog` under `rocq/`), D21 (research and teaching equal, the command
  first) and D22 (how the project is distributed) in `plan/README.md`.
- *The steps*, renumbered from section 6's proposal:

  | section 6 | now | what changed |
  |---|---|---|
  | 18 | 18 | begins with the JSON defect; owns the flake's test run with debug assertions |
  | 19 | 19 and 20 | split: the time limit (stops, the net engine's cubes, `--jobs`, the portfolio) and then the memory bound with the input boundaries |
  | 20 | 21 | the deepening default, a default time limit, one thread first, as the author answered |
  | 21 | 22 | – |
  | 22 | 23 and 24 | split by kind: the API and the data model with quantifiers in view (Fable 5.1), and the command, the harness and the documents (Opus 5.5) |
  | 23 | 25 | with the draft of the header report |
  | – | 26 | ordinary logic's layer, slotted after the batch mode: it is on the command line, small, and teaching's |
  | 24 | 27 | – |
  | 25 | 28 | named, placed, and written with quantifiers in view |
  | 26 | 29 | gains the dispatch as a measured table (D19) |
  | 27 | 30 | – |
  | 28 | 31 | the release prepared for the author to make; the reminder about the header report |
  | deferred | 32 to 37 | MELL nets, cut elimination, the engines for MLL and IMLL with essential nets, the Lambek calculus, the inverse method, first-order logic: all wanted, after the release, in this order |

- *The order of 32 to 37.* The objects a course shows come first (MELL
  nets, then cut elimination, which is drawn on them), then the engines
  D19 asks for where step 17 found the least to gain (so that each is
  measured against the third baseline), and first-order logic last,
  since it goes through everything and every earlier step has left its
  place.
- *What "not worth doing" became.* Net-engine pruning, essential nets
  as an engine and the inverse method are planned after all, by D19.
  The assessment of each stands in its prompt as the honest starting
  point: each must earn its row of the dispatch by a measurement, and a
  step that wins none says so and leaves an option, not a default.
- *The plan's own files.* The prompts `plan/18-…md` to `plan/37-…md`
  (18 to 23 in full; the others with what is fixed, finished at the
  review the step table names); `plan/later.md` with where each
  candidate went and every follow-up list assigned; `plan/conduct.md`
  with the sub-agents' rule and D16 to D18; `plan/notes/distribution.md`
  on publishing, repositories, policies and what other tools do about
  threads and time limits; README's list of what is planned.

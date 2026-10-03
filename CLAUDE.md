# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

<!--
Maintainer note (stripped before it reaches Claude's context). This file is
loaded into every session: keep it well under 200 lines, to commands, rules,
conventions and pointers. Guidance for one area of the tree goes in
`.claude/rules/<area>.md` behind a `paths:` glob; procedures go in
`.claude/skills/<name>/`. `.claude/rules/claude-infra.md` explains the split.
-->

## Project

linlog is a linear logic suite: parse, print, serialize and eventually prove
sequents, build proof nets, and export to various formats (see the README
roadmap). It is written in Rust with WebAssembly in mind for a planned
`linlog-web` crate, which does not exist yet. Performance is a stated goal: the
data structures are designed to be compact and cache-friendly.

Workspace crates:
- `core/` is package **`linlog`**: all logic.
  It has seven optional default features, `parse` (chumsky), `serialize`
  (serde), `interactive` (step-by-step proving), and `latex`, `typst`,
  `svg` and `rocq` (the exports), and one off by default, `parallel`
  (rayon: the search on a thread pool, off for wasm); the CLI enables the
  last six.
- `cli/` is package **`linlog-cli`**, library **`linlog_cli`** and binary
  **`linlog`** (one call into the library; `doc = false` because it shares
  the core crate's name): a clap front end with `prove`, `check`,
  `interact` (a line-based session that reads commands from standard
  input, `interact.rs`) and `seq print|json|fragment`, and the output
  formats `text`, `json`, `net`, `latex`, `typst` and `rocq` (with
  `--standalone` for a document), `svg` and `net-svg`; `--tree
  auto|always|never` prints the text tree on a terminal only where it
  fits, `--derivation-limit SIZE|none` bounds the derivation any format
  builds (the verdict and its exit status stand without it), and
  `--no-check` skips the check every proof otherwise passes. The tree and its
  `--help` text are the doc comments in `argument_parsing.rs`; `prove.rs`
  runs the search on a thread sized from `--recursion-limit` and owns the
  output; `--copies` bounds the copies of `?` formulas per branch and
  `--bias` picks the focused engines' atom bias (`--forward-copies` the
  bound of the forward search that the default runs on Horn programs);
  `--jobs N` (default: every core) runs the search on a pool and
  `--deterministic` the sequential engines; exit status 0 proved/valid,
  1 unprovable/invalid, 2 error, 3 unknown. Its
  invariants and extension points live in `.claude/rules/cli.md`, which
  loads when a file under `cli/` is read.
- `bench/` is package **`linlog-bench`**, binary `linlog-bench` (not
  published, `doc = false`): the benchmark harness. `run` times the
  generated families (`linlog::families`), LLTP problems (`--lltp`, under
  an `ILL` directory intuitionistic) and problem files
  (`bench/problems/*.txt`, lines `name; mode; expected; copies; sequent`)
  in every mode, engine and thread count asked for, one child process per
  run (the hidden `one` command) with a time limit and a kill after it,
  one CSV row per run; `summary` prints Markdown tables of CSV files, or
  with `--before DIR` compares them problem by problem with the files of
  the same names in an earlier baseline (`--against FILE`: with one file
  of the same baseline).
  `bench/baseline.sh` takes a baseline into `bench/results/DAY/` (DAY
  the day it started: CSV files, `starts.txt` with the commit measured,
  `RESULTS.md`) and copies its tables to `bench/RESULTS.md`, the latest
  baseline's; it needs the machine to itself for a night, 20:00 to
  07:00. The first baseline is `bench/results/2026-09-30/`, taken before
  the performance pass in two nights (the second added the reruns); the
  second, `bench/results/2026-10-02/`, after it in one night, with its
  reruns on the first's problems and the library under each bias alone;
  `bench/COMPARISON.md` compares the two.
  `bench/targets.sh LABEL` runs the target set of that pass (the
  instances the focused engine lost on, 165 runs on two pinned cores in
  a memory-capped user unit, about twenty minutes) into
  `bench/targets/LABEL.csv`; `bench/TARGETS.md` compares `before`,
  `after-search` and `after`. Its invariants live in
  `.claude/rules/bench.md`, which loads when a file under `bench/` is
  read.

The core API the CLI builds on: `"…".parse::<Sequent>()`, `Display` for
pretty-printing, serde behind `serialize`, `Sequent::fragment()` for the
fragment a sequent lives in (`Fragment::name_in(mode)` for its
intuitionistic name), `Forest::new(&sequent)` for the occurrence forest
that proof search works on, `Reading::new(&forest)` for the intuitionistic
reading of a sequent (the `Position` of every occurrence, the goal, and
two-sided printing `Γ ⊢ A`, or a `ShapeError`), and `Proof` for a proof
term over the forest: `Proof::new(forest, nodes, root)`, `check(mode)` for
the independent checker (in intuitionistic mode also the one-succedent
condition; one pass, in memory proportional to the proof),
`derivation()` for the standard-calculus view and
`two_sided_derivation()` for the intuitionistic one with the ILL rule
names, whose `Display` draws the tree, `derivation_size(two_sided)` for
the `Size` of either without building it, and `derivation_with(&view,
stop)` under a `ViewOptions` (the bound on the estimated size that every
path which builds a derivation honours, `ViewError::TooLarge` beyond it);
and `prove(&sequent, mode,
&options)` (or `prove_until` with a stop closure) for proof search, which
dispatches on the fragment and the mode and returns an `Outcome` with a
three-valued `Verdict`, its proof checked before it is returned
(`Options::check`). The engines: `search::net` (axiom-linking search
over a proof structure, the default for unit-free MLL with or without Mix
when no literal occurs more than twice, in intuitionistic mode by the
embedding of IMLL into MLL, whose `Outcome` also carries the net found),
`search::focus` (the focused sequent engine for everything else: MLL with
units, MALL, MELL and full LL on dyadic sequents with a per-branch copy
bound that deepens iteratively, `Options::copies`, answering
`Reason::CopyBound` when it binds, and `Options::bias` for how each atom's
positive literal is picked, `Bias::Rarer` or `Factors`, which
changes speed and the copies a proof needs, never provability, or
`Bias::Auto`, which on a sequent with exponentials runs a search under
each and answers with the first that decides, the forward one within
`Options::forward_copies` where the sequent is a Horn program
(alternating in slices on one core, or in turns from their start without
the `parallel` feature, and side by side on a pool);
affine
mode, the same search with
weakening at the leaves; and, given the reading, the two-sided search of
intuitionistic mode, `Engine::TwoSided`, the same engine keeping the goal
on the consequent's side of every `⊸L` split) and `search::additive` (two
additive-only formulas, by a memoized recursion on subformula pairs, in
every mode); `Options::engine` forces one. `prove_goal(&forest, goal, mode,
&options, stop)` decides any multiset of occurrences of a forest, the
roots being the sequent itself. With the `parallel` feature and
`Options::jobs` above one, the focused and the net engine run on a rayon
pool of their own (`search::parallel`: cube-and-conquer over the choices
near the root, and-parallel `&` premises, a sharded memo and one arena
shared by the workers, cubes of the first links for the net engine; the
caller's stop closure is polled on the calling thread and raises the
workers' flag); `Options::portfolio` gives every worker an order of its
own; the additive path stays sequential. `Interactive` (`proofs::interactive`,
feature `interactive`) is a proof in progress: `new(&sequent, mode)`,
`goals()`, `rules(goal, position)`, `apply(goal, position, rule, left)`,
`undo()`, `close(goal, options, view, stop)`/`close_all`, `derivation()` with open goals as
`Rule::Open` leaves, `proof()` translating the finished derivation into a
checked `Proof`, serde behind `serialize`, and `Refusal` saying why a rule
does not apply.
`export::latex` and `export::typst` (features of the same names) write
`sequent(&sequent, form)`, `two_sided(&reading, form)` and
`derivation(&derivation, form)` (finished or with open goals) as LaTeX for
ebproof and Typst for curryst, each as a `Form::Fragment` or a
`Form::Standalone` document (set in Euler); `export::svg` (feature `svg`)
draws `sequent(&sequent, &style)`, `two_sided(&reading, &style)`,
`derivation(&derivation, &style)` and `net(&structure, &style)` as SVG
documents, laid out from a committed table of Euler Math's advances, with
`Style` for the sizes, gaps and colours; `export::rocq` (feature `rocq`)
writes `derivation(&derivation, form, &options)` as a Rocq lemma with its
proof script for NanoYalla (`NANOYALLA` is the version), the fragment or
a whole file starting with `Options::prelude`, the lemma named by
`Options::lemma`, refusing an open goal, Mix and affine weakening with
`Unsupported`; `core/tests/snapshots/` pins the derivations, nets and
certificates (`BLESS=1 cargo test -p linlog --test export` rewrites
them).
`ProofStructure` (`nets`) is a proof net of unit-free MLL over the forest:
`from_links`, `link`/`unlink`, `is_correct()` (the Danos–Regnier criterion
through Yeo's deletion test, independent of the search and the checker),
`sequentialize()` to a `Proof`, `from_proof(&proof, mix)` back, and a
`Display` that the CLI's `--format net` prints. Sequents are
one-sided arena DAGs in negation normal form; fragments and modes are
runtime values, and indices are `u32` newtypes. The invariants live in
`.claude/rules/core.md`, which loads when a file under `core/` is read.
`lltp::read` (feature `parse`) reads a problem of the LLTP library
(`fof(name, role, formula).` clauses, the formulas in this crate's own
syntax) into a `Sequent` and the status its header claims; `families`
(feature `parse`) generates problem families with known verdicts at any
size, seeded (`FAMILIES`, `find`, `Family::instance`, and the encodings
themselves: `three_partition`, `three_partition_mll`, `partition`, `qbf`,
`counter`, `wide`, `mix`), which the harness and the engines' tests use.
`plan/README.md` is the proof-search plan the code follows, `plan/reports/`
what each step of it did.

## Commands

The dev environment is the flake's devshell (direnv, or `nix develop`): the
toolchain `rust-toolchain.toml` names (edition 2024; rustfmt, clippy and
rust-analyzer included), cargo-hack, cargo-deny, bacon and treefmt. `menu`
lists its commands: `check`, `tests`, `launch` (the CLI), `live` (bacon),
`dev` (zellij) and `up`.

```sh
cargo build
cargo test --workspace
cargo test -p linlog <test_name>                           # single test in core
cargo clippy --workspace --all-targets -- --deny warnings
cargo hack check --each-feature -p linlog                  # each feature alone, all, none
cargo hack check --feature-powerset --depth 2 -p linlog    # and every pair
cargo deny check                                           # licenses, bans, sources + advisories (online)
cargo run -p linlog-cli -- <args>
cargo run --release -p linlog-bench -- run --family partition-no=3,4 --engines focus,net
nix build .#lltp -o bench/lltp   # the LLTP library (1.1 GB, GPL-3.0, fetched at a pinned commit)
bench/targets.sh LABEL            # the focused engine's target set into bench/targets/LABEL.csv (two cores, about 20 min, detached)
bench/baseline.sh --arm --fresh   # the whole baseline, unattended from 20:00 to 07:00 (about 10.5 h): bench/results/DAY/, bench/RESULTS.md

nix flake check   # build, clippy, test, test-debug-assertions (the tests with debug_assert! on, which the release profile drops), doc, deny, features (cargo-hack), export (the LaTeX and Typst output compiles, the SVG renders), rocq (NanoYalla checks the certificates), bench (the harness on the smallest problems), deadnix, actionlint, treefmt, claude-hooks
nix build .#checks.x86_64-linux.rocq   # the certificates alone: Rocq is a 1.2 GB closure from the binary cache
nix fmt           # nixfmt, rustfmt, taplo, shfmt, shellcheck (a hook runs it on each file changed with Write or Edit; a file changed from the shell needs it run by hand before the commit, or the treefmt check fails)
nix build         # linlog-cli, whose binary is result/bin/linlog
nix build .#doc   # the rustdoc site, as the Docs workflow publishes it
```

Verify as much as the change needs:

| the change | the proof |
|---|---|
| any `.rs` edit | `cargo clippy …` and `cargo test --workspace` |
| touches `#[cfg(feature = …)]` or `[features]` | add `cargo hack check --each-feature -p linlog` and `cargo hack check --feature-powerset --depth 2 -p linlog` (both cover `parallel`, which is off by default: `--each-feature`'s `--all-features` run is the one that differs from the defaults) |
| touches `bench/` or `core/src/families.rs` | add `cargo run --release -p linlog-bench -- run --all-families --timeout 5` for the verdicts (a `MISMATCH` in `summary` is a bug); timings only from `bench/baseline.sh` on an idle machine |
| changes how the focused engine searches, or must not (a refactoring) | add `bench/targets.sh LABEL` and compare the columns `verdict`, `nodes`, `splits`, `memo_hits` and `memo_entries` of `bench/targets/LABEL.csv` with those of `bench/targets/after-bias.csv` (or of `after.csv` when both runs name a `--bias`): on one thread they are a function of the input, so a decided row has them equal exactly when the search is (`linlog-bench summary` of the two files sets the times side by side) |
| touches `search/parallel.rs`, `focus/parallel.rs` or `net::parallel` | `cargo test --workspace` covers them (the CLI depends on `parallel`, and cargo unifies features across a workspace run); `cargo test -p linlog` alone needs `--features parallel` |
| adds or changes a dependency | add `cargo deny check`. New deps must use a license `deny.toml` allows: EUPL-1.2, MIT, Apache-2.0 (± LLVM-exception), Unicode-3.0 or Zlib |
| `flake.nix`, `modules/`, `.github/`, the toolchain, a lock bump, or before a push | `nix flake check`, which runs all of the above |

## The flake

Dendritic flake-parts: `flake.nix` only declares inputs, and import-tree loads
every `.nix` file under `modules/` as a flake-parts module. No file is ever
added to an imports list; a path segment starting with `_` is skipped. One
aspect per file, contributing to every output it needs (`treefmt.nix` also puts
treefmt in the shell). Modules share values through `_module.args`:
`rustToolchain` and `craneLib` (`toolchain.nix`), `workspace` (the crane
arguments, `workspace.nix`, whose source is what `cleanCargoSource` keeps
plus `core/tests/snapshots`). `checks.nix`, `devshell.nix`, `treefmt.nix`
and `systems.nix` are what their names say; `export.nix` is the `export`
check, which compiles the snapshots and a CLI proof with pdfLaTeX and with
Typst and the curryst of nixpkgs (the version `export::typst::CURRYST`
names), and renders the SVG snapshots and CLI drawings with resvg, both
with only the Euler Math font of nixpkgs' TeX Live, offline; `rocq.nix`
is the `rocq` check, which builds NanoYalla from the non-flake input
`nanoyalla` (Click & coLLecT pinned to a commit; `export::rocq::NANOYALLA`
names the version) with nixpkgs' Rocq and standard library and compiles
the `.v` snapshots and two CLI certificates against it, requiring Rocq
to print nothing; `bench.nix` is the `linlog-bench` package, the `bench`
check (the harness on the smallest instance of every family and on the
problem file, failing on a verdict against a known one) and the `lltp`
package, the LLTP library fetched at a pinned commit with its Petri-net
archives unpacked and its one malformed file repaired, which no check
uses.

## CI

GitHub Actions runs `.github/workflows/ci.yml` on every push to `main`, on
every pull request and weekly: `nix flake check`, and the online
`cargo deny check advisories` in the devshell. A workflow installs nothing
but Nix, so CI checks with exactly the tools flake.lock pins.
`.github/workflows/docs.yml` publishes the flake's `doc` package, the rustdoc
of `main`, to GitHub Pages on every push to `main`. The rules for editing
workflows are in `.claude/rules/ci.md`, which loads under `.github/`.

## Version control: jj only

A jj repository, colocated with git only so that nix and the GitHub remote
`origin` (`github.com/linlog-prover/linlog`) keep working. **Never run git, not
even to read.** Every operation goes through `jj`, including lock updates:
`nix … --commit-lock-file` commits through git. The `Bash(git *)` deny rule and
`.claude/hooks/block-git.py` enforce this.

- Reading: `jj st`, `jj log`, `jj diff`, `jj show`, `jj file annotate`, `jj root`.
- **Commit thematically, without being asked**: one logical unit per change,
  committed as soon as it is done with `jj commit -m`, which describes `@` and
  opens a fresh change on top. `jj describe -m` only names `@`; `jj squash`
  folds `@` into its parent (the amend) and, when the parent is described,
  needs `--use-destination-message` or `-m`: without one it opens an editor,
  which hangs a session. Never `jj new -m`: it describes a new,
  empty change and leaves the work behind (a hook blocks it).
- Edits land in `@` retroactively. If `@` already holds unrelated work, run
  `jj new` before editing. Another session or the author may commit in the
  same working copy meanwhile, so run `jj st` right before every commit
  rather than assuming `@` and `@-` from the session's start.
- nix sees only files in the git tree, which any jj command updates: run
  `jj st` after creating a file, before a `nix` command that must see it.
- Subjects are short, imperative and capitalised, with no type prefix
  ("Fix bug in reachability analysis for exponentials"). Add a body only when
  the why is not obvious. jj signs every commit, and a snapshot of a
  changed working copy is one: when the signing key's passphrase is no
  longer cached and nobody answers the prompt, every jj command that
  writes fails with a signing timeout, `jj st` included. The edits stay
  on disk, `jj --ignore-working-copy …` still reads, and `nix flake
  check` runs on the tree as it is; do not retry in a loop, finish what
  needs no commit and ask the author to enter the passphrase. A long
  session commits as it goes rather than at the end for that reason.
- Lock bumps are changes of their own, "Cargo update" and "flake.lock: Update".
  `/update-deps` does both, verifying before it commits; the shell's `up` is
  the unverified shortcut.
- Work lands on `main` directly or through a GitHub pull request. Pushing is
  outward-facing, so only when asked: `jj bookmark set main -r @-`, then
  `jj git push --bookmark main`.
- The GitHub CLI (`gh`) is logged in with the author's own rights, over the
  organization `linlog-prover` and everything else the account reaches.
  Ask before every administrative task through it (a setting of the
  organization or a repository, Pages, Actions, branch rules, secrets,
  releases, a new repository, a workflow run): say what the call does and
  wait for a yes, which covers that task only. Deleting and transferring
  are the author's own acts. A permission rule asks before any `gh`
  command.

## Conventions

- Every source file starts with the license header, in the file's comment
  syntax: `//` in Rust, `#` in Nix, TOML, shell, Python and YAML.
  ```
  // linlog © Fabian Lukas Grubmüller 2026
  // Licensed under the EUPL
  ```
  Prose and configuration carry none: Markdown (CLAUDE.md, `.claude/`,
  `plan/`, README), JSON, `.gitignore`, `LICENSE` and the lock files. The
  project is EUPL-1.2.
- Comments are short and targeted: they say why, or what the code cannot.
  Types and names carry the rest.
- Comments are self-contained: they make sense from inside this repository,
  with no references to other repositories, machines or conversations, and
  none to the Claude Code sessions, prompts, plan steps or decision numbers
  (`plan/`) that produced the code. A comment says what the code does or
  why; how the work was organised belongs in `plan/` and the jj history.
- Every Rust item gets a concise doc comment that a human understands at
  first read: a function says what it does and returns, not how; a type,
  field, variant or module says what it is. The workspace lints in
  `Cargo.toml` enforce it (rustc's `missing_docs` for public items, clippy's
  `missing_docs_in_private_items` for the rest). clap shows the docs on
  `cli/`'s argument types as `--help` text.
- Dependencies are welcome where they earn their place: prefer well-made
  library code over an ad-hoc implementation, and among candidates the more
  popular, better maintained and faster one. Each serves a particular reason
  and is scoped to it: only the crate that uses it, behind the feature that
  needs it (or in `[dev-dependencies]`), with only the crate features used.
  Never add one for its own sake.
- `core/src/lib.rs` allows `dead_code` and `unused_variables` crate-wide while
  things are scaffolded.
- `scratchpad*.md` are the author's gitignored notes. `scratchpad1.md` is about
  250 KB, so don't read it in full.

## Claude Code setup

`.claude/` is checked in. It holds the hooks (the git and `jj new -m` guards,
the formatter, a SessionStart note on the jj working copy), permission rules,
the `crate-source-explorer` agent (dependency APIs against the locked sources;
use it before guessing at chumsky), the `update-deps` skill, the
`new-tool` skill (use it whenever a crate or tool is added or adopted), and the
path-scoped rules. Claude Code's built-in git instructions and git status
snapshot are switched off (`env` in `settings.json`).
`.claude/rules/claude-infra.md` documents it and loads when anything under
`.claude/` is opened. When the repo changes shape, amend `.claude/` and this
file in the same change.

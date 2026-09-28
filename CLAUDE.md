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
- `core/` is package **`linlog`** (the README calls it `linlog-core`): all logic.
  It has two optional default features, `parse` (chumsky) and `serialize` (serde).
- `cli/` is package **`linlog-cli`**: a clap front end. The subcommand tree
  (`seq pretty|serialize prose|json`) is declared in `argument_parsing.rs`, but
  `main.rs` does not dispatch to it yet.

The core API the CLI builds on: `"…".parse::<Sequent<usize, LL>>()`, `Display`
for pretty-printing, and serde behind `serialize`. Sequents are one-sided
arena DAGs in negation normal form. The invariants live in
`.claude/rules/core.md`, which loads when a file under `core/` is read.

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
cargo hack check --feature-powerset -p linlog              # every parse/serialize combination
cargo deny check                                           # licenses, bans, sources + advisories (online)
cargo run -p linlog-cli -- <args>

nix flake check   # build, clippy, test, doc, deny, features (cargo-hack), deadnix, treefmt
nix fmt           # nixfmt, rustfmt, taplo, shfmt, shellcheck (a hook runs it on each edited file)
nix build         # linlog-cli
```

Verify as much as the change needs:

| the change | the proof |
|---|---|
| any `.rs` edit | `cargo clippy …` and `cargo test --workspace` |
| touches `#[cfg(feature = …)]` or `[features]` | add `cargo hack check --feature-powerset -p linlog` |
| adds or changes a dependency | add `cargo deny check`. New deps must use a license `deny.toml` allows: EUPL-1.2, MIT, Apache-2.0 (± LLVM-exception), Unicode-3.0 or Zlib |
| `flake.nix`, `modules/`, the toolchain, a lock bump, or before a push | `nix flake check`, which runs all of the above |

## The flake

Dendritic flake-parts: `flake.nix` only declares inputs, and import-tree loads
every `.nix` file under `modules/` as a flake-parts module. No file is ever
added to an imports list; a path segment starting with `_` is skipped. One
aspect per file, contributing to every output it needs (`treefmt.nix` also puts
treefmt in the shell). Modules share values through `_module.args`:
`rustToolchain` and `craneLib` (`toolchain.nix`), `workspace` (the crane
arguments, `workspace.nix`). `checks.nix`, `devshell.nix`, `treefmt.nix` and
`systems.nix` are what their names say.

## Version control: jj only

A jj repository, colocated with git only so that nix and the Codeberg remote
`origin` (`codeberg.org/flgrubm/linlog`) keep working. **Never run git, not
even to read.** Every operation goes through `jj`, including lock updates:
`nix … --commit-lock-file` commits through git. The `Bash(git *)` deny rule and
`.claude/hooks/block-git.py` enforce this.

- Reading: `jj st`, `jj log`, `jj diff`, `jj show`, `jj file annotate`, `jj root`.
- **Commit thematically, without being asked**: one logical unit per change,
  committed as soon as it is done with `jj commit -m`, which describes `@` and
  opens a fresh change on top. `jj describe -m` only names `@`; `jj squash`
  folds `@` into its parent (the amend). Never `jj new -m`: it describes a new,
  empty change and leaves the work behind (a hook blocks it).
- Edits land in `@` retroactively. If `@` already holds unrelated work, run
  `jj new` before editing.
- nix sees only files in the git tree, which any jj command updates: run
  `jj st` after creating a file, before a `nix` command that must see it.
- Subjects are short, imperative and capitalised, with no type prefix
  ("Fix bug in reachability analysis for exponentials"). Add a body only when
  the why is not obvious. jj signs every commit.
- Lock bumps are changes of their own, "Cargo update" and "flake.lock: Update".
  `/update-deps` does both, verifying before it commits; the shell's `up` is
  the unverified shortcut.
- Work lands on `main` directly or through a Codeberg pull request. Pushing is
  outward-facing, so only when asked: `jj bookmark set main -r @-`, then
  `jj git push`.

## Conventions

- Every source file starts with the license header, in the file's comment syntax:
  ```
  // linlog © Fabian Lukas Grubmüller 2026
  // Licensed under the EUPL
  ```
  The project is EUPL-1.2.
- Comments are short and targeted: they say why, or what the code cannot.
  Types and names carry the rest.
- Comments are self-contained: they make sense from inside this repository,
  with no references to other repositories, machines or conversations.
- Every Rust function gets a concise `///` doc comment that a human
  understands at first read: what it does and returns, not how.
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

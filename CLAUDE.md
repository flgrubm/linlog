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

## Version control

- Plain git, with remote `origin` on Codeberg (`codeberg.org/flgrubm/linlog`).
  Work lands on `main` directly or through a Codeberg pull request.
- Commits are GPG-signed automatically (`commit.gpgsign = true`). Never pass
  `--no-gpg-sign`.
- Subjects are short, imperative and capitalised, with no type prefix
  ("Fix bug in reachability analysis for exponentials"). Add a body only when
  the why is not obvious.
- Lock bumps are commits of their own, "Cargo update" and "flake.lock: Update".
  `/update-deps` does both, verifying before it commits.
- Pushing is outward-facing: only when asked.

## Conventions

- Every source file starts with the license header, in the file's comment syntax:
  ```
  // linlog © Fabian Lukas Grubmüller 2026
  // Licensed under the EUPL
  ```
  The project is EUPL-1.2.
- `core/src/lib.rs` allows `dead_code` and `unused_variables` crate-wide while
  things are scaffolded. The `add` function and its test there are template leftovers.
- `scratchpad*.md` are the author's gitignored notes. `scratchpad1.md` is about
  250 KB, so don't read it in full.

## Claude Code setup

`.claude/` is checked in. It holds a formatter hook, permission rules, the
`crate-source-explorer` agent (dependency APIs against the locked sources; use it
before guessing at chumsky 0.12), the `update-deps` skill, and the path-scoped
rules. `.claude/rules/claude-infra.md` documents it and loads when anything
under `.claude/` is opened. When the repo changes shape, amend `.claude/` and
this file in the same change.

---
name: update-deps
description: >-
  Update linlog's lock files (Cargo.lock with `cargo update`, flake.lock with
  `nix flake update`, which also moves the Rust toolchain), verify each with
  `nix flake check` before committing, and commit each with jj as a change of
  its own in the repo's established form, "Cargo update" and "flake.lock:
  Update" with nix's summary. Reports semver-incompatible releases it did not
  take and any failure a bump introduces. Use when asked to update or bump
  dependencies, crates, the lock files, nixpkgs, the flake inputs or the Rust
  toolchain.
argument-hint: "[cargo|flake]"
allowed-tools: Bash(cargo update *) Bash(nix flake update *) Bash(nix develop -c rustc --version) Bash(jj new) Bash(jj restore *) Bash(jj commit *)
---

# Updating the lock files

Scope: `$ARGUMENTS` is `cargo`, `flake`, or empty for both (cargo first, then flake).

A lock file moving is not the interesting part. What matters is whether the
workspace still builds, tests and lints clean with it, and what moved.

## Invariants

- **Nothing unverified enters history.** Update, verify, then commit. Never
  `nix flake update --commit-lock-file`: it commits through git, before
  anything is checked, and a hook blocks it.
- **Each lock change holds its lock file and nothing else**:
  `jj commit <lock file> -m …` takes only that path. A code change a bump
  forces is a separate change on top. Propose it and ask before making it.
- **jj only.** Commits are signed by jj itself.
- **Pushing is not part of this.**

`$S` below is the session's scratchpad directory: outside the tree, and it
survives a context compaction.

## 0. Baseline

```sh
jj st
jj op log --limit 1 --no-graph -T 'id.short(16)'
nix flake check
```

`@` must be empty and undescribed. If it holds work, ask before going on, and
if the user agrees, `jj new` so the bumps start from a clean change. Keep the
operation id: `jj op restore <id>` puts the lock files *and* history back as
they were, whatever happens below.

Note every check that already fails. A failure present before a bump is not
the bump's, and one absent before it is.

## 1. Cargo.lock

```sh
cargo update --dry-run --verbose    # "Unchanged x (available: y)": semver-incompatible, not taken
cargo update
```

Verify:

```sh
nix flake check                     # build, clippy, tests, rustdoc, cargo-deny, cargo-hack, formatting
cargo deny check advisories         # online; the flake check cannot fetch the database
```

If anything newly fails, back out with `jj restore Cargo.lock` and report the
failure and the crate that caused it. Otherwise:

```sh
jj commit Cargo.lock -m "Cargo update"
```

## 2. flake.lock

A rust-overlay bump moves rustc, clippy and rustfmt along with nixpkgs; new
clippy lints are the usual fallout. The running shell still holds the old
toolchain, so ask the flake's shell for it:

```sh
nix develop -c rustc --version      # before
nix flake update 2> "$S/flake-update.txt"; cat "$S/flake-update.txt"
nix develop -c rustc --version      # after
```

Nothing moved (`jj diff --name-only` is empty)? Say so and stop. Verify:

```sh
nix flake check
```

If the only new failures are lints from the toolchain bump, stop and ask:
commit the lock with a lint fix as a separate change on top, or back out. If
anything else newly fails, back out with `jj restore flake.lock` and report
it. Otherwise commit with nix's own summary as the body, the form every
earlier `flake.lock: Update` in history has:

```sh
jj commit flake.lock -m "$(printf 'flake.lock: Update\n\nFlake lock file updates:\n\n'; awk '/^•/{p=1} p' "$S/flake-update.txt")"
jj log -r @- --no-graph -T 'signature.status()'    # good
```

## 3. Report

- per lock file: committed or backed out, and the change id
- crates that moved, as `name old -> new`. Call out the ones linlog depends on
  directly (chumsky, serde, foldhash, thiserror, clap, anyhow, serde_json,
  ctrlc)
- semver-incompatible releases available but not taken: each needs a
  `Cargo.toml` change and probably code changes
- the rustc version before → after
- every failure the bump introduced, and what fixing it would take

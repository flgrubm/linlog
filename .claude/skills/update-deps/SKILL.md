---
name: update-deps
description: >-
  Update linlog's lock files (Cargo.lock with `cargo update`, flake.lock with
  `nix flake update`, which also moves the Rust toolchain), verify each against
  the checks before committing, and commit each on its own in the repo's
  established form, "Cargo update" and "flake.lock: Update" with nix's summary.
  Reports semver-incompatible releases it did not take and any failure a bump
  introduces. Use when asked to update or bump dependencies, crates, the lock
  files, nixpkgs, the flake inputs or the Rust toolchain.
argument-hint: "[cargo|flake]"
allowed-tools: Bash(cargo update *) Bash(nix flake update *) Bash(nix develop -c cargo test *) Bash(nix develop -c cargo clippy *) Bash(nix develop -c rustc --version) Bash(git commit *)
---

# Updating the lock files

Scope: `$ARGUMENTS` is `cargo`, `flake`, or empty for both (cargo first, then flake).

A lock file moving is not the interesting part. What matters is whether the
workspace still builds, tests and lints clean with it, and what moved.

## Invariants

- **Nothing unverified enters history.** Update, verify, then commit, in that
  order. In particular, never use `nix flake update --commit-lock-file`, which
  commits before anything is checked.
- **Each lock commit touches its lock file and nothing else** (`git commit -- <file>`).
  A code change a bump forces is a separate commit on top. Propose it and ask
  before making it: it is a change the user did not request.
- **Commits are signed automatically** (`commit.gpgsign = true`). Never pass
  `--no-gpg-sign`. If signing hangs on a stale keyring lock, follow the note in
  `~/.claude/CLAUDE.md`.
- **Pushing is not part of this.** Only push when asked.

## 0. Baseline

```sh
git status --porcelain -- Cargo.lock flake.lock   # must print nothing; stop and ask otherwise
cargo clippy --workspace --all-targets -- --deny warnings
cargo test --workspace
```

Note every failure that already exists. A failure present before the bump is
not the bump's, and a failure absent before it is. `nix flake check` runs the
same clippy with the same toolchain, so a clippy failure here shows up there too.

## 1. Cargo.lock

```sh
cargo update --dry-run --verbose    # preview; "Unchanged x (available: y)" = semver-incompatible, not taken
cargo update
```

Verify:

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- --deny warnings
cargo hack check --feature-powerset -p linlog
cargo deny check                    # licenses and advisories of anything new
nix flake check                     # crane builds from the lock too
```

If anything new fails, restore the lock (`git checkout -- Cargo.lock`), and
report the failure and the crate that caused it. Otherwise:

```sh
git commit -m "Cargo update" -- Cargo.lock
```

## 2. flake.lock

`rust-overlay`'s `stable.latest` makes this bump also move rustc, clippy and
rustfmt. New clippy lints are the usual fallout. The current shell still holds
the old devshell, so verify through `nix develop -c`, not bare `cargo`.

Keep nix's summary: it becomes the commit body. Write it inside `.git/`,
where it survives a context compaction and stays out of the tree:

```sh
nix develop -c rustc --version      # before
log=$(git rev-parse --git-path flake-update.log)
nix flake update 2> "$log"
nix develop -c rustc --version      # after
```

Verify:

```sh
nix flake check                     # build, clippy --deny warnings, cargo fmt, taplo, treefmt
nix develop -c cargo test --workspace
```

If the only new failures are lints from the toolchain bump, stop and ask:
commit the lock with a separate lint-fix commit on top, or back out. If
anything else newly fails, restore (`git checkout -- flake.lock`) and report
it. Otherwise:

```sh
log=$(git rev-parse --git-path flake-update.log)
{ printf 'flake.lock: Update\n\nFlake lock file updates:\n\n'; sed -n '/^•/,$p' "$log"; } \
  | git commit -F - -- flake.lock
rm -f "$log"
```

This reproduces the message `nix flake update --commit-lock-file` writes,
which is the form every earlier `flake.lock: Update` commit in history has.

## 3. Report

- per lock file: committed or backed out, and the commit hash
- crates that moved, as `name old -> new`. Call out any that linlog depends on
  directly (chumsky, clap, serde, subenum, thiserror, anyhow, serde_json)
- semver-incompatible releases available but not taken, since each needs a
  `Cargo.toml` change and probably code changes
- the rustc version before → after
- every failure the bump introduced, and what fixing it would take

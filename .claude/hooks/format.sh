#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
#
# PostToolUse(Write|Edit): run the flake's formatter (`nix fmt`, i.e. treefmt
# as modules/treefmt.nix configures it) on the one file that was just written,
# so an edit never leaves the tree failing the `treefmt` check.
#
# `nix fmt` rather than calling rustfmt directly: treefmt passes rustfmt
# `--edition 2024 --config skip_children=true`, and going through it keeps one
# source of truth for those flags. It costs ~0.4 s on a clean tree and ~2 s when
# a tracked file is dirty (no eval cache).
#
# Fails open and always exits 0. A PostToolUse hook that exits 2 pushes its
# stderr at the model, and a formatter has nothing to say about an edit that
# already succeeded; formatting drift is caught by `nix flake check` anyway.

set -uo pipefail

command -v python3 >/dev/null 2>&1 || exit 0
command -v nix >/dev/null 2>&1 || exit 0

# Two lines out: the file, then the directory Claude is working in. The payload's
# `cwd`, not $CLAUDE_PROJECT_DIR: the latter stays at the original checkout after
# Claude enters a worktree, and treefmt refuses a path outside its tree root.
out=$(python3 -c '
import json, os, sys

try:
    payload = json.load(sys.stdin)
    path = payload.get("tool_input", {}).get("file_path", "")
    cwd = payload.get("cwd", "")
except Exception:
    sys.exit(0)

if not (isinstance(path, str) and isinstance(cwd, str) and path and cwd):
    sys.exit(0)

# Only what treefmt has a formatter for (modules/treefmt.nix), and only
# inside the checkout: a memory or scratchpad file is none of its business.
root = os.path.realpath(cwd)
real = os.path.realpath(path)
if real.endswith((".rs", ".nix", ".toml", ".sh", ".envrc")) and real.startswith(root + os.sep):
    print(real)
    print(root)
' 2>/dev/null) || exit 0

path=$(printf '%s\n' "$out" | sed -n 1p)
root=$(printf '%s\n' "$out" | sed -n 2p)

[ -n "$path" ] && [ -f "$path" ] || exit 0
cd "$root" 2>/dev/null || exit 0
[ -f flake.nix ] || exit 0

nix fmt "$path" >/dev/null 2>&1
exit 0

#!/usr/bin/env python3
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
"""PreToolUse(Bash) guard: this repository is managed with jj only.

Blocks every git invocation, read-only ones included, and nix's
`--commit-lock-file`, which commits through git. The `Bash(git *)` deny rule
already covers the plain form; this also sees git behind a wrapper (`env`,
`timeout`, `run0`, ...), a shell (`bash -c`), `nix shell ... -c`, `nix run
nixpkgs#git`, `jj util exec` and an absolute path.

Exit 2 blocks and shows stderr to Claude. Anything else allows, including
every error: a guard must never wedge the session. The `claude-hooks` flake
check (modules/claude-hooks.nix) pins the cases.
"""

import json
import os
import re
import shlex
import sys

# Programs that run their arguments: (options taking a value, positionals
# before the program).
WRAPPERS = {
    "builtin": (set(), 0),
    "command": (set(), 0),
    "doas": ({"-u", "-C"}, 0),
    "env": ({"-u", "--unset", "-C", "--chdir", "-S", "--split-string"}, 0),
    "exec": ({"-a"}, 0),
    "nice": ({"-n", "--adjustment"}, 0),
    "nohup": (set(), 0),
    "run0": (
        {"-u", "--user", "-g", "--group", "-D", "--chdir", "--setenv", "--unit"},
        0,
    ),
    "setsid": (set(), 0),
    "stdbuf": ({"-i", "-o", "-e"}, 0),
    "sudo": (
        {"-u", "--user", "-g", "--group", "-D", "--chdir", "-C", "-p", "--prompt"},
        0,
    ),
    "time": ({"-f", "--format", "-o", "--output"}, 0),
    "timeout": ({"-s", "--signal", "-k", "--kill-after"}, 1),
    "xargs": ({"-a", "-d", "-E", "-I", "-L", "-n", "-P", "-s"}, 0),
}
SHELLS = {"bash", "dash", "ksh", "sh", "zsh"}
ASSIGNMENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=\S*\Z")
GIT_PACKAGE = re.compile(r"#git(Full|Minimal)?\Z")

# `(` separates only where it opens a subshell or `$(`, not inside a word such
# as the permission pattern `Bash(git *)`.
SEPARATOR = re.compile(r"[;&|\n)]|(?<!\w)\(")

GIT_MESSAGE = """\
Blocked: this repository is managed with jj only; git is not used, not even to read.

  git status / log / diff / show  ->  jj st / jj log / jj diff / jj show
  git blame / ls-files            ->  jj file annotate / jj file list
  git rev-parse --show-toplevel   ->  jj root
  git add                         ->  nothing: every jj command snapshots the tree
  git commit -m                   ->  jj commit -m
  git commit --amend              ->  jj squash
  git stash / switch / checkout   ->  jj new / jj new <rev> / jj edit <rev>
  git reset / revert              ->  jj undo, jj op restore <id>, jj revert
  git branch                      ->  jj bookmark
  git fetch / pull / push         ->  jj git fetch / jj git push (ask before pushing)
"""

LOCK_MESSAGE = """\
Blocked: --commit-lock-file makes nix commit through git.

Update the lock, verify, then commit with jj: /update-deps does exactly that.
"""


def mask_quoted(line):
    """Replace quoted text with `x`, keeping every offset, so that a separator
    inside an argument (`grep "a|git push"`) does not split the line.
    An unterminated quote leaves the rest of the line visible."""
    out, i, n = list(line), 0, len(line)
    while i < n:
        c = line[i]
        if c == "\\":
            i += 2
        elif c in "\"'":
            j = i + 1
            while j < n and line[j] != c:
                j += 2 if c == '"' and line[j] == "\\" else 1
            if j >= n:
                break
            out[i + 1 : j] = "x" * (j - i - 1)
            i = j + 1
        else:
            i += 1
    return "".join(out)


def resolve(tokens):
    """The program a command runs and its arguments, past assignments and
    wrappers; `(None, [])` if it runs nothing (`command -v git`)."""
    i = 0
    while i < len(tokens):
        token = tokens[i]
        if ASSIGNMENT.match(token):
            i += 1
            continue
        program = os.path.basename(token)
        if program in WRAPPERS:
            if program == "command" and tokens[i + 1 : i + 2] in (["-v"], ["-V"]):
                return None, []
            takes_value, positionals = WRAPPERS[program]
            i += 1
            while i < len(tokens) and tokens[i].startswith("-"):
                if tokens[i] == "--":
                    i += 1
                    break
                i += 2 if tokens[i] in takes_value else 1
            i += positionals
            continue
        rest = tokens[i + 1 :]
        words = [w for w in rest if not w.startswith("-")]
        if program == "nix" and words[:1] in (["shell"], ["develop"]):
            for flag in ("-c", "--command"):
                if flag in rest:
                    return resolve(rest[rest.index(flag) + 1 :])
        if program == "jj" and words[:2] == ["util", "exec"]:
            command = rest[rest.index("exec") + 1 :]
            return resolve(command[1:] if command[:1] == ["--"] else command)
        return program, rest
    return None, []


def shell_script(tokens):
    """The SCRIPT of `bash -c SCRIPT`, under any wrapper, or None."""
    program, rest = resolve(tokens)
    if program not in SHELLS:
        return None
    has_c, j = False, 0
    while j < len(rest):
        token = rest[j]
        if token in ("--", "-"):
            j += 1
            break
        if token.startswith("--"):
            j += 2 if token in ("--rcfile", "--init-file") else 1
        elif token[0] in "-+" and len(token) > 1:
            has_c = has_c or (token[0] == "-" and "c" in token[1:])
            j += 2 if token[-1] in "oO" else 1
        else:
            break
    return rest[j] if has_c and j < len(rest) else None


def verdict(line, depth=0):
    """0 to allow the command line, 2 for git, 3 for `--commit-lock-file`."""
    start = 0
    for separator in [*SEPARATOR.finditer(mask_quoted(line)), None]:
        end = separator.start() if separator else len(line)
        segment, start = line[start:end], separator.end() if separator else end
        try:
            tokens = shlex.split(segment)
        except ValueError:
            tokens = segment.split()

        script = shell_script(tokens)
        if script is not None and depth < 4 and (code := verdict(script, depth + 1)):
            return code

        program, rest = resolve(tokens)
        if program == "git":
            return 2
        if program == "nix":
            if "--commit-lock-file" in rest:
                return 3
            if rest[:1] == ["run"] and any(GIT_PACKAGE.search(w) for w in rest):
                return 2
    return 0


def main():
    try:
        command = json.load(sys.stdin)["tool_input"]["command"]
        code = verdict(command) if isinstance(command, str) else 0
    except Exception:
        return 0
    if code:
        print(GIT_MESSAGE if code == 2 else LOCK_MESSAGE, file=sys.stderr, end="")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
"""PreToolUse(Bash) guard against `jj new -m`.

`jj new` starts an empty change and `-m` describes that empty change, so the
work meant to be committed stays behind, undescribed, in the previous one.
`jj commit -m` is what was meant.

Exit 2 blocks and shows stderr to Claude. Anything else allows, including
every error. The `claude-hooks` flake check (modules/claude-hooks.nix) pins
the cases.
"""

import json
import re
import sys

MESSAGE = """\
Blocked: `jj new -m` describes a new, empty change; the work stays behind in the previous one.

  commit the work in @ and start a fresh change:  jj commit -m "msg"
  only describe @:                                jj describe -m "msg"
  a described empty change, on purpose:           jj new && jj describe -m "msg"
"""


def is_jj_new_with_message(command):
    """Whether some command in the line is `jj new` carrying -m/--message.

    Splitting on separators keeps a `-m` of another command (`jj new && jj
    describe -m x`) from counting. A segment has to start with `jj new`, so
    text that merely mentions it (`grep "jj new -m"`) passes."""
    return any(
        re.match(r"\s*jj\s+new(\s|$)", segment)
        and re.search(r"(^|\s)(--message|-m)(?!-)", segment)
        for segment in re.split(r"[;&|\n(]", command)
    )


def main():
    try:
        command = json.load(sys.stdin)["tool_input"]["command"]
        blocked = isinstance(command, str) and is_jj_new_with_message(command)
    except Exception:
        return 0
    if blocked:
        print(MESSAGE, file=sys.stderr, end="")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())

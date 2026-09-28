#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
#
# SessionStart: print the working copy in jj's terms; the plain stdout becomes
# context. jj snapshots edits into @ retroactively, so whether @ already holds
# described work decides whether the next edit needs a `jj new` first.
#
# Silent on any failure. --ignore-working-copy keeps it from adding an
# operation to `jj op log`.

set -uo pipefail

command -v jj >/dev/null 2>&1 || exit 0
cd "${CLAUDE_PROJECT_DIR:-.}" 2>/dev/null || exit 0

state=$(
  jj log --ignore-working-copy --no-graph -r '@ | @-' \
    -T 'if(current_working_copy, "  @  ", "  @- ") ++ change_id.shortest(8) ++ if(empty, " (empty)", "") ++ " " ++ if(description, description.first_line(), "(no description set)") ++ "\n"' \
    2>/dev/null
) || exit 0
[ -n "$state" ] || exit 0

# Bookmarks under @ that main does not contain yet.
stacked=$(
  jj log --ignore-working-copy --no-graph -r 'bookmarks() & (main..@)' \
    -T 'bookmarks ++ " "' 2>/dev/null
) || stacked=""

cat <<MSG
jj working copy:

$state

Edits land in @ retroactively: if @ already holds described work, run \`jj new\`
before editing so the next unit of work gets its own change.
MSG

if [ -n "$stacked" ]; then
  cat <<MSG

@ builds on bookmarks not yet in main: ${stacked% }
Start unrelated work with \`jj new main\`.
MSG
fi

exit 0

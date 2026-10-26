#!/bin/sh
# codelaw-hook: post-commit
# Installed by `codelaw init --git-hooks` (SR-CLI-MCP decision 7). It prints
# reminders and ALWAYS exits 0: a commit is already made. Delete this file to
# uninstall; a hook that was here before is kept beside it as
# `post-commit.pre-codelaw` and runs first.

here=$(dirname "$0")
if [ -f "$here/post-commit.pre-codelaw" ]; then
  "$here/post-commit.pre-codelaw" "$@" || true
fi

command -v codelaw >/dev/null 2>&1 || exit 0
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0

# What you owe: your exceptions that expire within 7 days (SR-BASELINE
# decision 11). With nothing to say the text report is the as-of line and
# then exactly "no exceptions", so the hook prints unless its line 2 is that. No
# `user.email`, no baseline or any other failure prints nothing.
owed=$(cd "$root" && codelaw exceptions --mine --expiring 7d 2>/dev/null) || owed=
if [ -n "$owed" ] && [ "$(printf '%s\n' "$owed" | sed -n 2p)" != "no exceptions" ]; then
  echo "codelaw: exceptions of yours that expire within 7 days"
  printf '%s\n' "$owed"
fi

# The burn-down in one line (SR-BASELINE decision 10): the `total` row of the
# last 30 days (start, now, change). Nothing when there is no baseline history.
burn=$(cd "$root" && codelaw baseline history --since 30d 2>/dev/null | grep '^total') || burn=
[ -n "$burn" ] && echo "codelaw: baseline burn-down, last 30 days (start now change trend): $(printf '%s' "$burn" | tr -s ' ' | cut -d' ' -f2-)"

# The extension point: every executable in .codelaw/hooks/post-commit.d/, in
# name order, with the repo root as its working directory. What it prints is
# shown; how it exits is ignored.
for step in "$root"/.codelaw/hooks/post-commit.d/*; do
  [ -f "$step" ] && [ -x "$step" ] || continue
  (cd "$root" && "$step") 2>&1 || true
done

exit 0

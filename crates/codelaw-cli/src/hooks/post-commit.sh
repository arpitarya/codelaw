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

# The extension point: every executable in .codelaw/hooks/post-commit.d/, in
# name order, with the repo root as its working directory. What it prints is
# shown; how it exits is ignored.
for step in "$root"/.codelaw/hooks/post-commit.d/*; do
  [ -f "$step" ] && [ -x "$step" ] || continue
  (cd "$root" && "$step") 2>&1 || true
done

exit 0

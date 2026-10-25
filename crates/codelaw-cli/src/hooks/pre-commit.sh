#!/bin/sh
# codelaw-hook: pre-commit
# Installed by `codelaw init --git-hooks` (SR-CLI-MCP decision 7). Delete this
# file to uninstall; a hook that was here before is kept beside it as
# `pre-commit.pre-codelaw` and runs first. Skip once with `git commit --no-verify`.
#
# Extra flags for both commands, e.g. CODELAW_HOOK_ARGS="--profile ci".

here=$(dirname "$0")
if [ -f "$here/pre-commit.pre-codelaw" ]; then
  "$here/pre-commit.pre-codelaw" "$@" || exit $?
fi

if ! command -v codelaw >/dev/null 2>&1; then
  echo "codelaw: not on PATH, so the pre-commit check was skipped" >&2
  exit 0
fi

staged=$(mktemp) || exit 0
partial=$(mktemp) || exit 0
fixable=$(mktemp) || exit 0
trap 'rm -f "$staged" "$partial" "$fixable"' EXIT

# Added, copied, modified or renamed files, as the index holds them.
git -c core.quotepath=off diff --cached --name-only --diff-filter=ACMR >"$staged"
[ -s "$staged" ] || exit 0

# A file with changes that are not staged is partly staged: fixing it would
# commit the hunks the author left out, so it is checked and never fixed.
git -c core.quotepath=off diff --name-only >"$partial"
grep -vxFf "$partial" "$staged" >"$fixable"

if [ -s "$fixable" ]; then
  set --
  while IFS= read -r f; do set -- "$@" "$f"; done <"$fixable"
  # shellcheck disable=SC2086
  codelaw fix --staged $CODELAW_HOOK_ARGS -- "$@" || true
  git add -- "$@"
fi

set --
while IFS= read -r f; do set -- "$@" "$f"; done <"$staged"
# shellcheck disable=SC2086
codelaw check --staged $CODELAW_HOOK_ARGS -- "$@"
exit $?

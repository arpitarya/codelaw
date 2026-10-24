#!/usr/bin/env bash
# Stop / SessionEnd — SR-WORK-CLEANUP decisions 8-9: every lock a session takes
# is cleaned. Releases this session's per-asset write locks (session-lock.sh's)
# and removes git lock files older than 10 minutes while no git process runs.
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 0

INPUT=$(cat 2>/dev/null || true)
ME=$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)
ME="${ME:-${CLAUDE_SESSION_ID:-}}"

# Decision 8: this session's own write locks.
if [ -n "$ME" ] && [ -d .claude/.locks ]; then
  for d in .claude/.locks/*/; do
    [ -f "$d/owner" ] || continue
    [ "$(cut -d' ' -f1 "$d/owner")" = "$ME" ] && rm -rf "$d"
  done
fi

# Decision 9: stale git locks — only while no git process runs.
GITDIR=$(git rev-parse --git-common-dir 2>/dev/null) || exit 0
pgrep -x git >/dev/null 2>&1 && exit 0
find "$GITDIR" -name '*.lock' -type f -mmin +10 -not -path '*/_stale_locks/*' 2>/dev/null |
while IFS= read -r f; do
  rm -f "$f" 2>/dev/null && continue
  # A surface that cannot unlink renames it aside instead.
  mkdir -p "$GITDIR/_stale_locks" &&
    mv "$f" "$GITDIR/_stale_locks/$(printf '%s' "${f#"$GITDIR"/}" | tr / _).$(date +%s)" 2>/dev/null
done
exit 0

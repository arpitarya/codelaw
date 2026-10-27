#!/usr/bin/env bash
# Makes every codelaw npm package trust publish.yml (environment `npm`) for
# trusted publishing, so CI publishes with no token. Run by the decider, after
# each package exists on npm: `npm trust` needs npm 12 and an interactive
# login (2FA, one security-key tap per name).
#
# Usage: scripts/npm-trust.sh [--fux-index] [--dry-run]
#   (no flag)    the release packages, `scripts/package.py names` (npm rows) —
#                the names publish.yml publishes and trust-checks.
#   --fux-index  also the fifteen Fux index packages, `scripts/fux-index.py
#                names` (npm rows). A flag, not always: publish.yml does not
#                publish them yet (W-34), and each name costs a tap. Use it to
#                re-add a configuration npm let lapse (RELEASING.md §Every
#                release, step 0).
#   --dry-run    passed to `npm trust`; nothing is written.
#
# A name npm already trusts (E409, "a trusted publisher configuration ...
# already exists") counts as trusted. Any other failure is reported and the
# loop carries on; the script exits 1 at the end, listing the names that failed.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

fux=0 npm_args=()
for a in "$@"; do
  case "$a" in
    --fux-index) fux=1 ;;
    --dry-run) npm_args+=(--dry-run) ;;
    *) echo "npm-trust.sh: unknown argument: $a" >&2; exit 2 ;;
  esac
done

list=$(mktemp) log=$(mktemp)
trap 'rm -f "$list" "$log"' EXIT
python3 scripts/package.py names > "$list" || exit 1
[ "$fux" = 1 ] && { python3 scripts/fux-index.py names >> "$list" || exit 1; }

trusted=0 already=0 failed=()
while read -r reg name <&3; do  # fd 3: npm keeps stdin for its login prompt
  [ "$reg" = npm ] || continue
  echo "→ $name"
  # stderr is shown and copied to $log; stdout stays on the terminal.
  { npx -y npm@12 trust github "$name" --file publish.yml --repo arpitarya/codelaw \
      --env npm --allow-publish -y ${npm_args[@]+"${npm_args[@]}"} 2>&1 1>&4 | tee "$log" >&2
    rc=${PIPESTATUS[0]}; } 4>&1
  if [ "$rc" = 0 ]; then
    trusted=$((trusted + 1))
  elif grep -qE 'E409|already exists' "$log"; then
    echo "  = already trusted (E409)"; already=$((already + 1))
  else
    echo "  ✗ $name (exit $rc)" >&2; failed+=("$name")
  fi
done 3< "$list"

echo "trusted: $trusted · already trusted: $already · failed: ${#failed[@]}"
if [ "${#failed[@]}" -gt 0 ]; then
  printf '  ✗ %s\n' "${failed[@]}" >&2
  exit 1
fi

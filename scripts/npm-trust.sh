#!/usr/bin/env bash
# Makes every codelaw npm package trust publish.yml (environment `npm`) for
# trusted publishing, so CI publishes with no token. Run once by the decider,
# after each package exists on npm: `npm trust` needs npm 12 and an
# interactive login (2FA). Usage: scripts/npm-trust.sh [--dry-run]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
while read -r reg name <&3; do  # fd 3: npm keeps stdin for its login prompt
  [ "$reg" = npm ] || continue
  echo "→ $name"
  npx -y npm@12 trust github "$name" --file publish.yml --repo arpitarya/codelaw --env npm --allow-publish -y "$@"
done 3< <(python3 scripts/package.py names)

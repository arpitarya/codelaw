#!/usr/bin/env bash
# Withdraw a half-published release (L6 decision 9): deprecate every npm
# package and yank every PyPI release of VERSION that reached a registry.
#
#   scripts/withdraw.sh 0.2.0 --report   list what is published (CI runs this)
#   scripts/withdraw.sh 0.2.0            deprecate on npm (needs your npm login + 2FA),
#                                        and print the PyPI yank pages
#
# Run by the decider. A CI OIDC token may publish but may not deprecate, and
# PyPI offers no yank API at all — so the withdrawal is one hand, on purpose.
set -euo pipefail
v="${1:?usage: withdraw.sh VERSION [--report]}"
report="${2:-}"
cd "$(git rev-parse --show-toplevel)"
pv="$(python3 -c 'import sys; sys.path.insert(0, "scripts"); import package; print(package.pep440(sys.argv[1]))' "$v")"
found=0
echo "## codelaw $v — what reached a registry"
echo
while read -r reg name <&3; do  # fd 3: npm keeps stdin for its OTP prompt
  case "$reg" in
    npm)
      if npm view "$name@$v" version >/dev/null 2>&1; then
        found=1
        if [ "$report" = --report ]; then echo "- npm \`$name@$v\` — \`npm deprecate $name@$v \"withdrawn: release $v was incomplete\"\`"
        else npm deprecate "$name@$v" "withdrawn: release $v was incomplete" && echo "- npm $name@$v deprecated"; fi
      fi ;;
    pypi)
      if curl -fs -o /dev/null "https://pypi.org/pypi/$name/$pv/json"; then
        found=1
        echo "- PyPI \`$name $pv\` — yank it at https://pypi.org/manage/project/$name/release/$pv/ (PyPI has no API for this)"
      fi ;;
  esac
done 3< <(python3 scripts/package.py names)
[ "$found" = 1 ] || echo "- nothing: no registry has $v"

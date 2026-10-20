#!/usr/bin/env bash
# The install smoke (L6 "How L6 is enforced": install matrix, runtime fence,
# pack parity) on linux-x64, from the assembled dist/ and NO registry:
#   - npm: codelaw + @codelaw/cli-linux-x64-gnu + @codelaw/rules-data, run with no Python on PATH
#   - PyPI: codelaw + codelaw-rules-data in a fresh venv, run with no Node on PATH
#   - the same project checked by both: the JSON reports must be byte-equal,
#     and must contain diagnostics, so two empty runs can't pass
# Usage: scripts/smoke.sh dist
set -euo pipefail
dist="$(cd "${1:?usage: smoke.sh DIST}" && pwd)"
root="$(git rev-parse --show-toplevel)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

project() { # the L3 combined fixture plus a pack preset
  local d="$1"
  mkdir -p "$d/.codelaw/rules"
  cp -R "$root/fixtures/l3/rules/." "$d/.codelaw/rules/"
  for l in "$root"/fixtures/l3/*/; do
    [ "$(basename "$l")" = rules ] || cp -R "$l." "$d/"
  done
  printf 'preview = true\nextends = ["codelaw:md/recommended", "codelaw:data/recommended"]\n' >"$d/codelaw.toml"
  printf '# Smoke\n\n# A second title\n\n```json\n{"a": 1, "a": 2}\n```\n' >"$d/SMOKE.md"
  printf '{"name":"smoke","private":true}\n' >"$d/package.json" # both projects hold the same files
  git -C "$d" init -q
}

bare_path() { # a PATH with neither python nor node: only coreutils
  local p="$work/bin-$1"; mkdir -p "$p"
  for t in env sh bash cat mkdir rm ls; do ln -sf "$(command -v "$t")" "$p/$t"; done
  [ "$1" = npm ] && ln -sf "$(command -v node)" "$p/node"
  echo "$p"
}

# ── npm ──
project "$work/npm"
(cd "$work/npm" && cp package.json package.json.orig &&
  npm i --omit=optional --no-audit --no-fund --no-package-lock --no-save --loglevel=error \
    "$dist"/npm/codelaw-[0-9]*.tgz "$dist"/npm/codelaw-cli-linux-x64-gnu-*.tgz "$dist"/npm/codelaw-rules-data-*.tgz &&
  mv package.json.orig package.json)
npm_bin="$work/npm/node_modules/.bin/codelaw"
(cd "$work/npm" && env -i PATH="$(bare_path npm)" "$npm_bin" check --format json --no-cache >"$work/npm.json") || [ $? = 1 ]

(cd "$work/npm" && node --input-type=module -e 'import { check } from "codelaw"; process.stdout.write(JSON.stringify(check([], { noCache: true }).summary))' >"$work/api-npm.json")

# ── PyPI ──
project "$work/pypi"
python3 -m venv "$work/venv"
"$work/venv/bin/pip" install -q --no-index --disable-pip-version-check \
  "$dist"/pypi/codelaw-*manylinux*x86_64.whl "$dist"/pypi/codelaw_rules_data-*.whl
(cd "$work/pypi" && env -i PATH="$(bare_path pypi)" VIRTUAL_ENV="$work/venv" "$work/venv/bin/codelaw" check --format json --no-cache >"$work/pypi.json") || [ $? = 1 ]
# the Python API, through the same binary
(cd "$work/pypi" && "$work/venv/bin/python" -c 'import codelaw, json, sys; sys.stdout.write(json.dumps(codelaw.check(no_cache=True)["summary"], separators=(",", ":")))' >"$work/api-pypi.json")

# ── compare ──
n="$(node -e 'const r = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")); console.log(r.diagnostics.filter(d => d.rule !== "codelaw/no-rules").length)' "$work/npm.json")"
[ "$n" -gt 0 ] || { echo "✗ the smoke project produced no diagnostics — the packs were not found" >&2; cat "$work/npm.json" >&2; exit 1; }
cmp -s "$work/npm.json" "$work/pypi.json" || { echo "✗ npm and PyPI disagree:" >&2; diff "$work/npm.json" "$work/pypi.json" >&2 || true; exit 1; }
node -e 'const [a, b, c] = process.argv.slice(1).map(f => JSON.parse(require("fs").readFileSync(f, "utf8"))); const s = JSON.stringify(a.summary); if (s !== JSON.stringify(b) || s !== JSON.stringify(c)) { console.error("✗ an API disagrees with the CLI", s, JSON.stringify(b), JSON.stringify(c)); process.exit(1) }' "$work/npm.json" "$work/api-npm.json" "$work/api-pypi.json"
echo "✔ npm (no Python on PATH) and PyPI (no Node on PATH) agree byte for byte: $n diagnostics; the JS and Python APIs agree"

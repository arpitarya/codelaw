#!/usr/bin/env bash
# The install smoke (L6 "How L6 is enforced": install matrix, runtime fence,
# pack parity) on ONE platform, from the assembled dist/ and NO registry:
#   - npm: codelaw + @codelaw/cli-<KEY> + @codelaw/rules-data, run with no Python on PATH
#   - PyPI: codelaw + codelaw-rules-data in a fresh venv, pip reading dist/pypi as its
#     only index and choosing the wheel for this machine, run with no Node on PATH
#   - the same project checked by both: the JSON reports must be byte-equal,
#     and must contain diagnostics, so two empty runs can't pass
# ci.yml's `smoke` job runs it on each of the eight platforms (W-18).
# Usage: scripts/smoke.sh DIST [KEY]    KEY defaults to this machine's, e.g. linux-x64-gnu
set -euo pipefail
dist="$(cd "${1:?usage: smoke.sh DIST [KEY]}" && pwd)"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" # not git: a container's mount is not "safe" to git
here="$(node -p 'const h = process.report.getReport().header; process.platform + "-" + process.arch + (process.platform === "linux" ? (h.glibcVersionRuntime ? "-gnu" : "-musl") : "")')"
key="${2:-$here}"
[ "$key" = "$here" ] || { echo "✗ smoke.sh $key cannot run on a $here machine: npm and pip install for the host" >&2; exit 1; }
case "$key" in win32-*) win=1 ;; *) win=0 ;; esac
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

native() { # a path as the native program it is handed to reads it
  if [ "$win" = 1 ]; then cygpath -w "$1"; else echo "$1"; fi
}

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

bare_path() { # a PATH with neither python nor node (npm: node only)
  local p="$work/bin-$1"; mkdir -p "$p"
  if [ "$win" = 1 ]; then # the programs are run by absolute path; Windows finds its DLLs without PATH
    [ "$1" = npm ] && cp "$(command -v node)" "$p/"
  else
    for t in env sh bash cat mkdir rm ls; do ln -sf "$(command -v "$t")" "$p/$t"; done
    [ "$1" = npm ] && ln -sf "$(command -v node)" "$p/node"
  fi
  echo "$p"
}

fenced() { # run "$@" with only PATH (and, on Windows, the SYSTEMROOT every process needs)
  local path="$1"; shift
  if [ "$win" = 1 ]; then env -i PATH="$path" SYSTEMROOT="${SYSTEMROOT:-C:\\Windows}" "$@"
  else env -i PATH="$path" "$@"; fi
}

# ── npm ──
project "$work/npm"
(cd "$work/npm" && cp package.json package.json.orig &&
  npm i --omit=optional --no-audit --no-fund --no-package-lock --no-save --loglevel=error \
    "$dist"/npm/codelaw-[0-9]*.tgz "$dist"/npm/codelaw-cli-"$key"-[0-9]*.tgz "$dist"/npm/codelaw-rules-data-*.tgz &&
  mv package.json.orig package.json)
np="$(bare_path npm)"
if [ "$win" = 1 ]; then # npm's .bin shims need a POSIX shell; run the launcher they call
  (cd "$work/npm" && fenced "$np" "$np/node.exe" "$(native "$work/npm/node_modules/codelaw/bin/codelaw.js")" check --format json --no-cache >"$work/npm.json") || [ $? = 1 ]
else
  (cd "$work/npm" && fenced "$np" "$work/npm/node_modules/.bin/codelaw" check --format json --no-cache >"$work/npm.json") || [ $? = 1 ]
fi

(cd "$work/npm" && node --input-type=module -e 'import { check } from "codelaw"; process.stdout.write(JSON.stringify(check([], { noCache: true }).summary))' >"$work/api-npm.json")

# ── PyPI ──
project "$work/pypi"
if [ "$win" = 1 ]; then py=python; vbin="$work/venv/Scripts"; else py=python3; vbin="$work/venv/bin"; fi
"$py" -m venv "$work/venv"
"$vbin/python" -m pip install -q --no-index --disable-pip-version-check --find-links "$(native "$dist/pypi")" codelaw codelaw-rules-data
(cd "$work/pypi" && fenced "$(bare_path pypi)" VIRTUAL_ENV="$(native "$work/venv")" "$vbin/codelaw" check --format json --no-cache >"$work/pypi.json") || [ $? = 1 ]
# the Python API, through the same binary
(cd "$work/pypi" && "$vbin/python" -c 'import codelaw, json, sys; sys.stdout.write(json.dumps(codelaw.check(no_cache=True)["summary"], separators=(",", ":")))' >"$work/api-pypi.json")

# ── compare ──
n="$(node -e 'const r = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")); console.log(r.diagnostics.filter(d => d.rule !== "codelaw/no-rules").length)' "$(native "$work/npm.json")")"
[ "$n" -gt 0 ] || { echo "✗ the smoke project produced no diagnostics — the packs were not found" >&2; cat "$work/npm.json" >&2; exit 1; }
cmp -s "$work/npm.json" "$work/pypi.json" || { echo "✗ npm and PyPI disagree:" >&2; diff "$work/npm.json" "$work/pypi.json" >&2 || true; exit 1; }
node -e 'const [a, b, c] = process.argv.slice(1).map(f => JSON.parse(require("fs").readFileSync(f, "utf8"))); const s = JSON.stringify(a.summary); if (s !== JSON.stringify(b) || s !== JSON.stringify(c)) { console.error("✗ an API disagrees with the CLI", s, JSON.stringify(b), JSON.stringify(c)); process.exit(1) }' \
  "$(native "$work/npm.json")" "$(native "$work/api-npm.json")" "$(native "$work/api-pypi.json")"
echo "✔ $key: npm (no Python on PATH) and PyPI (no Node on PATH) agree byte for byte: $n diagnostics; the JS and Python APIs agree"

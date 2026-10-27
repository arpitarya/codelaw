#!/usr/bin/env bash
# The install smoke (L6 "How L6 is enforced": install matrix, runtime fence,
# pack parity) on ONE platform, from a STAGING INDEX (W-18): the assembled dist/
# is published to a throwaway npm registry and PyPI simple index on 127.0.0.1
# (scripts/staging-index.py, the standard library only), then installed from it
# exactly as a user installs codelaw. Nothing is published anywhere else.
#   - publish: `npm publish` each tarball; `uv publish` each wheel (the legacy upload API)
#   - npm: codelaw and @codelaw/rules-data, by name (npm picks @codelaw/cli-<KEY> itself),
#     run with no Python on PATH; codelaw --version through npx, from an empty folder
#   - PyPI: codelaw and codelaw-rules-data, by name, into a fresh venv with the staging index
#     as its only index, run with no Node on PATH; codelaw --version through uvx
#   - the same project checked by both: the JSON reports must be byte-equal,
#     and must contain diagnostics, so two empty runs can't pass
#   - the skills package (W-77), installed by name on both: `sync-agents` in
#     fixtures/skills-consumer writes the same agent files from either, byte for byte
# ci.yml's `smoke` job runs it on each of the eight platforms (W-18).
# Usage: scripts/smoke.sh DIST [KEY]    KEY defaults to this machine's, e.g. linux-x64-gnu
# Needs node, npm and python3 (>= 3.11, the wheel's floor); installs uv (UV_VERSION) into a
# throwaway venv. Every cache and config it uses is under its temp folder.
set -euo pipefail
dist="$(cd "${1:?usage: smoke.sh DIST [KEY]}" && pwd)"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" # not git: a container's mount is not "safe" to git
here="$(node -p 'const h = process.report.getReport().header; process.platform + "-" + process.arch + (process.platform === "linux" ? (h.glibcVersionRuntime ? "-gnu" : "-musl") : "")')"
key="${2:-$here}"
[ "$key" = "$here" ] || { echo "✗ smoke.sh $key cannot run on a $here machine: npm and pip pick the package for the host" >&2; exit 1; }
case "$key" in win32-*) win=1 ;; *) win=0 ;; esac
UV_VERSION=0.11.3 # MIT OR Apache-2.0; declared in crates/codelaw-cli/tests/licences.rs (L2)
work="$(mktemp -d)"
index_pid=""
watch_pid=""
stop_watch() { # a running `check --watch`, and on Windows the native process tree under it
  [ -n "$watch_pid" ] || return 0
  if [ "$win" = 1 ] && [ -r "/proc/$watch_pid/winpid" ]; then taskkill //F //T //PID "$(cat "/proc/$watch_pid/winpid")" >/dev/null 2>&1 || true; fi
  kill "$watch_pid" 2>/dev/null || true; wait "$watch_pid" 2>/dev/null || true
  watch_pid=""
}
stop() { # the script's own status, whatever stopping the index returns
  local st=$?
  stop_watch
  if [ -n "$index_pid" ]; then kill "$index_pid" 2>/dev/null || true; wait "$index_pid" 2>/dev/null || true; fi
  rm -rf "$work"
  exit "$st"
}
trap stop EXIT

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

# ── the staging index: publish every file of dist/ to it ──
if [ "$win" = 1 ]; then py=python; vbin="$work/venv/Scripts"; tbin="$work/tools/Scripts"; else py=python3; vbin="$work/venv/bin"; tbin="$work/tools/bin"; fi
"$py" -m venv "$work/tools" # uv, kept out of the venv under test
"$tbin/python" -m pip install -q --disable-pip-version-check "uv==$UV_VERSION"
"$py" "$(native "$root/scripts/staging-index.py")" "$(native "$work/index")" --port-file "$(native "$work/index.url")" 2>"$work/index.log" &
index_pid=$!
for _ in $(seq 1 100); do [ -s "$work/index.url" ] && break; kill -0 "$index_pid" 2>/dev/null || break; sleep 0.1; done
[ -s "$work/index.url" ] || { echo "✗ the staging index did not start" >&2; cat "$work/index.log" >&2; exit 1; }
url="$(cat "$work/index.url")"
tgz="$(basename "$(ls "$dist"/npm/codelaw-[0-9]*.tgz)")"; version="${tgz#codelaw-}"; version="${version%.tgz}"
pre=""; case "$version" in *-*) pre=1 ;; esac # a pre-release: npm's `next`, pip --pre, uv --prerelease
# npm reads only this config: the staging registry for every scope, a cache of its own
cat >"$work/npmrc" <<EOF
registry=$url/
${url#http:}/:_authToken=staging
cache=$(native "$work/npm-cache")
tag=$([ -n "$pre" ] && echo next || echo latest)
audit=false
fund=false
update-notifier=false
EOF
npm_config_userconfig="$(native "$work/npmrc")"
export npm_config_userconfig
# publishing caches each tarball by its digest: a cache of its own, so every install downloads from the index
for t in "$dist"/npm/*.tgz; do npm publish "$t" --access public --loglevel=error --cache "$(native "$work/npm-publish-cache")" >/dev/null; done
UV_NO_CONFIG=1 "$tbin/uv" publish -q --publish-url "$url/legacy/" --username staging --password staging --trusted-publishing never "$dist"/pypi/*.whl

# ── npm ──
project "$work/npm"
(cd "$work/npm" && cp package.json package.json.orig &&
  npm i --no-audit --no-fund --no-package-lock --no-save --loglevel=error codelaw @codelaw/rules-data @codelaw/skills &&
  mv package.json.orig package.json)
[ -d "$work/npm/node_modules/@codelaw/cli-$key" ] || { echo "✗ npm did not pick @codelaw/cli-$key from the staging index" >&2; ls "$work/npm/node_modules/@codelaw" >&2; exit 1; }
np="$(bare_path npm)"
if [ "$win" = 1 ]; then # npm's .bin shims need a POSIX shell; run the launcher they call
  (cd "$work/npm" && fenced "$np" "$np/node.exe" "$(native "$work/npm/node_modules/codelaw/bin/codelaw.js")" check --format json --no-cache >"$work/npm.json") || [ $? = 1 ]
else
  (cd "$work/npm" && fenced "$np" "$work/npm/node_modules/.bin/codelaw" check --format json --no-cache >"$work/npm.json") || [ $? = 1 ]
fi

(cd "$work/npm" && node --input-type=module -e 'import { check } from "codelaw"; process.stdout.write(JSON.stringify(check([], { noCache: true }).summary))' >"$work/api-npm.json")

# ── PyPI ──
project "$work/pypi"
"$py" -m venv "$work/venv"
"$vbin/python" -m pip install -q --isolated --no-cache-dir --disable-pip-version-check --index-url "$url/simple/" ${pre:+--pre} codelaw codelaw-rules-data codelaw-skills
(cd "$work/pypi" && fenced "$(bare_path pypi)" VIRTUAL_ENV="$(native "$work/venv")" "$vbin/codelaw" check --format json --no-cache >"$work/pypi.json") || [ $? = 1 ]
# the Python API, through the same binary
(cd "$work/pypi" && "$vbin/python" -c 'import codelaw, json, sys; sys.stdout.write(json.dumps(codelaw.check(no_cache=True)["summary"], separators=(",", ":")))' >"$work/api-pypi.json")

# ── one-shot runs: npx, then uvx, fetch codelaw from the staging index into caches of their own ──
mkdir -p "$work/once"
v_npm="$(cd "$work/npm" && node "$(native "$work/npm/node_modules/codelaw/bin/codelaw.js")" --version)"
v_npx="$(cd "$work/once" && npm_config_cache="$(native "$work/npx-cache")" npx --yes --loglevel=error codelaw --version)"
v_uvx="$(cd "$work/once" && UV_NO_CONFIG=1 UV_PYTHON_DOWNLOADS=never "$tbin/uvx" -q --isolated --cache-dir "$(native "$work/uv-cache")" \
  --python "$(native "$tbin/python")" --default-index "$url/simple/" ${pre:+--prerelease=allow} codelaw --version)"
[ "$v_npm" = "codelaw $version" ] && [ "$v_npx" = "$v_npm" ] && [ "$v_uvx" = "$v_npm" ] ||
  { echo "✗ --version: npm '$v_npm', npx '$v_npx', uvx '$v_uvx'; expected 'codelaw $version'" >&2; exit 1; }
# every install downloaded from the index: 4 tarballs for npm, 2 for npx; 3 wheels for pip, 1 for uvx
served_npm="$(grep -c '"GET /[^ ]*/-/[^ ]*\.tgz HTTP/1.1" 200' "$work/index.log" || true)"
served_pypi="$(grep -c '"GET /files/[^ ]*\.whl HTTP/1.1" 200' "$work/index.log" || true)"
[ "$served_npm" -ge 6 ] && [ "$served_pypi" -ge 4 ] ||
  { echo "✗ the staging index served $served_npm npm tarballs (6 expected) and $served_pypi wheels (4): something installed from elsewhere" >&2; cat "$work/index.log" >&2; exit 1; }

# ── the skills package (W-77): the agent files, from the npm install and from the PyPI install ──
for reg in npm pypi; do
  a="$work/$reg/agents"; mkdir -p "$a"; cp -R "$root/fixtures/skills-consumer/." "$a/"; git -C "$a" init -q
  # inside the smoke project, so its node_modules / venv are found upwards; `root = true`
  # makes it its own project, as it is in a consumer's repo (CI run 40390477529: a nested
  # config may not set `extends`, SR-CONFIG decision 2)
  { printf 'root = true\n'; cat "$a/codelaw.toml"; } >"$a/codelaw.toml.new" && mv "$a/codelaw.toml.new" "$a/codelaw.toml"
done
(cd "$work/npm/agents" && node "$(native "$work/npm/node_modules/codelaw/bin/codelaw.js")" sync-agents >/dev/null &&
  node "$(native "$work/npm/node_modules/codelaw/bin/codelaw.js")" sync-agents --check >/dev/null) ||
  { echo "✗ npm: sync-agents did not write the agent files from @codelaw/skills" >&2; exit 1; }
(cd "$work/pypi/agents" && VIRTUAL_ENV="$(native "$work/venv")" "$vbin/codelaw" sync-agents >/dev/null &&
  VIRTUAL_ENV="$(native "$work/venv")" "$vbin/codelaw" sync-agents --check >/dev/null) ||
  { echo "✗ PyPI: sync-agents did not write the agent files from codelaw-skills" >&2; exit 1; }
[ -f "$work/npm/agents/.claude/skills/codelaw-fix/SKILL.md" ] || { echo "✗ the skills package wrote no skill" >&2; exit 1; }
diff -r -x .git "$work/npm/agents" "$work/pypi/agents" >&2 || { echo "✗ @codelaw/skills and codelaw-skills wrote different agent files" >&2; exit 1; }
agent_files="$(cd "$work/npm/agents" && find . -type f -not -path './.git/*' | wc -l | tr -d ' ')"

# ── compare ──
n="$(node -e 'const r = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")); console.log(r.diagnostics.filter(d => d.rule !== "codelaw/no-rules").length)' "$(native "$work/npm.json")")"
[ "$n" -gt 0 ] || { echo "✗ the smoke project produced no diagnostics — the packs were not found" >&2; cat "$work/npm.json" >&2; exit 1; }
cmp -s "$work/npm.json" "$work/pypi.json" || { echo "✗ npm and PyPI disagree:" >&2; diff "$work/npm.json" "$work/pypi.json" >&2 || true; exit 1; }
node -e 'const [a, b, c] = process.argv.slice(1).map(f => JSON.parse(require("fs").readFileSync(f, "utf8"))); const s = JSON.stringify(a.summary); if (s !== JSON.stringify(b) || s !== JSON.stringify(c)) { console.error("✗ an API disagrees with the CLI", s, JSON.stringify(b), JSON.stringify(c)); process.exit(1) }' \
  "$(native "$work/npm.json")" "$(native "$work/api-npm.json")" "$(native "$work/api-pypi.json")"
# ── watch mode (SR-ARCHITECTURE decision 22a, W-44): in-process on every
# platform, Windows included — a save is re-checked, one JSON object per run.
# The save is undone afterwards: the win32 one-shot check below compares with
# pypi.json, taken before it (W-44: run 40388001095 failed that compare) ──
cp "$work/pypi/SMOKE.md" "$work/SMOKE.md.before-watch"
(cd "$work/pypi" && VIRTUAL_ENV="$(native "$work/venv")" "$vbin/codelaw" check --watch --format json --no-cache >"$work/watch.ndjson" 2>"$work/watch.err") &
watch_pid=$!
lines() { if [ -f "$work/watch.ndjson" ]; then wc -l <"$work/watch.ndjson" | tr -d ' '; else echo 0; fi; }
for _ in $(seq 1 300); do [ "$(lines)" -ge 1 ] && break; sleep 0.1; done
printf '\n# A third title\n' >>"$work/pypi/SMOKE.md"
for _ in $(seq 1 300); do [ "$(lines)" -ge 2 ] && break; sleep 0.1; done
stop_watch
node -e 'const lines = require("fs").readFileSync(process.argv[1], "utf8").split("\n").filter(Boolean); const runs = lines.map(l => JSON.parse(l));
  const [a, b] = runs; const md = r => r.diagnostics.filter(d => d.file === "SMOKE.md").length;
  if (runs.length < 2 || b.watch.run !== 2 || !b.watch.changed.includes("SMOKE.md") || md(b) <= md(a)) { console.error("✗ check --watch did not re-check a save:", lines.length, "lines", lines.map(l => l.slice(0, 200))); process.exit(1) }' \
  "$(native "$work/watch.ndjson")" || { cat "$work/watch.err" >&2; exit 1; }
cp "$work/SMOKE.md.before-watch" "$work/pypi/SMOKE.md"
# ── Windows is one-shot (SR-ARCHITECTURE decision 21): no warm socket, so
# `codelaw daemon` refuses with its stated message and exit 2, and
# `--prefer-warm` runs in-process with byte-identical output ──
oneshot=""
if [ "$win" = 1 ]; then
  st=0; (cd "$work/pypi" && "$vbin/codelaw" daemon status >"$work/daemon.out" 2>"$work/daemon.err") || st=$?
  want="error: \`codelaw daemon\` needs Unix domain sockets; on this system every command runs in its own process"
  [ "$st" = 2 ] && [ "$(tr -d '\r' <"$work/daemon.err")" = "$want" ] ||
    { echo "✗ win32 \`codelaw daemon status\` exited $st with '$(cat "$work/daemon.err")'; expected exit 2 and '$want'" >&2; exit 1; }
  (cd "$work/pypi" && fenced "$(bare_path pypi)" VIRTUAL_ENV="$(native "$work/venv")" "$vbin/codelaw" check --prefer-warm --format json --no-cache >"$work/warm.json") || [ $? = 1 ]
  cmp -s "$work/warm.json" "$work/pypi.json" || { echo "✗ win32: \`check --prefer-warm\` differs from \`check\`" >&2; exit 1; }
  oneshot="; one-shot: \`codelaw daemon\` refuses (exit 2) and \`--prefer-warm\` runs in-process, byte-identical"
fi
echo "✔ $key, installed from the staging index: npm (no Python on PATH) and PyPI (no Node on PATH) agree byte for byte: $n diagnostics; the JS and Python APIs agree; codelaw --version is codelaw $version through uvx and through npx; the index served $served_npm tarballs and $served_pypi wheels; check --watch re-checked a save; @codelaw/skills and codelaw-skills wrote the same $agent_files files$oneshot"

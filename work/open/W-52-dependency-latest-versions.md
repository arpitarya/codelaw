---
type: Handoff
name: W-52
description: "Arpit, 2026-10-25: an explicit, network-only `codelaw deps refresh` writes a committed snapshot of registry facts; an offline rule `data/outdated-dependency` flags each manifest line against it, with a suggest fix that bumps the range."
item: W-52
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), W-52 worktree"
ball: agent
---

# W-52 — Latest-version highlighting for npm and Python dependencies

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L5 decision 3: one named command, `deps refresh`, may use the network; the engine and every rule stay offline. Write `[deps]` into SR-CONFIG and the rule into SR-RULE-PACKS | the decider's ruling, recorded | ✅ L5 decision 3; SR-RULE-PACKS decision 15; SR-CONFIG §1, 2, 3; SR-CLI-MCP 1; SR-ARCHITECTURE 15b |
| 2 | `codelaw deps refresh`: read manifests, query npm and PyPI, write `.codelaw/deps-snapshot.json` | registry facts as a committed file | ✅ `crates/codelaw-cli/src/deps_cmd.rs`, through `curl` |
| 3 | `data/outdated-dependency` in the data pack: compare each manifest line with the snapshot | the highlight, in CLI, CI, LSP, hook | ✅ `crates/codelaw-native/src/deps/`; in `data/package-json` and `data/pyproject` |
| 4 | `suggest` fix: bump the range, keep its operator | one-click update of the manifest | ✅ |
| 5 | `data/stale-deps-snapshot` (info): the snapshot is older than `[deps] max_snapshot_age` at the as-of date | nobody trusts a stale list | ✅ |
| 6 | Docs and the auto-fix PR's refresh step (W-51) | stays fresh by itself | 🟡 docs done (USAGE, CHANGELOG, GLOSSARY, pack README); the refresh step waits for W-51 |

**Arpit, 2026-10-25:** *"I want latest versions of any dependencies that are present in Node or Python to be highlighted as well."* Then, on the snapshot design: *"Let's go with it. I like the suggestion."* — the ruling that a separate, explicit network command sits outside the engine (L5 decision 3 stays true of the engine).

**Model:** Opus — it amends a law and draws the network boundary; a leak of network into `check` breaks L5 silently.

## Goal

**Manifests read:** `package.json` (`dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`; every workspace), `pyproject.toml` (`[project] dependencies`, `optional-dependencies`, `[dependency-groups]`), `requirements*.txt`. Lockfiles (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `uv.lock`, `poetry.lock`) are read for the **installed** version when present.

**Registry facts fetched:** npm `dist-tags.latest`, every version with its publish time, `deprecated`; PyPI `info.version`, releases with upload time, `yanked`. `.npmrc` and `PIP_INDEX_URL`/`[[tool.uv.index]]` are honoured; tokens come from env, never written.

**The snapshot** (sorted, so diffs are small and reviewable):
```json
{ "fetched": "2026-10-25", "registries": {"npm": "https://registry.npmjs.org"},
  "packages": { "npm:react": { "latest": "19.2.0", "deprecated": null,
                               "versions": { "18.3.1": "2024-04-26", "19.2.0": "2025-10-01" } } } }
```

**What a finding says** (on the manifest line):
- `react ^18.3.1 — installed 18.3.1, latest 19.2.0 (major)` → **warn**
- minor or patch behind → **info**
- the version in use is deprecated (npm) or yanked (PyPI) → **error**
- the range already allows latest but the lockfile is behind → **info**, "run your install to pick up 18.3.2"

**`[deps]` config:**
```toml
[deps]
severity        = { major = "warn", minor = "info", patch = "off" }
min_release_age = "7d"      # ignore releases younger than this (supply-chain safety)
prerelease      = false
ignore          = ["npm:typescript"]   # or a dated exception (W-40) with expires/added_by
max_snapshot_age = "14d"
```

**Agents:** `rules_for_path` on a source file names the major-version gaps of the packages it imports ("this repo uses React 18; latest is 19 — write 18 APIs").

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `deps refresh` on a fixture with npm + Python manifests writes a sorted snapshot; a second run with no registry change writes identical bytes | ✅ `cargo test -p codelaw-cli --test deps` (a registry served on 127.0.0.1) |
| 2 · | `check` with the network off gives the same findings as with it on | ✅ `--test network_fence`, `l5_the_dependency_rules_need_no_network_and_refresh_is_fenced`: `check` (json, text, agent) and `fix --unsafe` identical in the OS fence; `deps refresh` fails in it |
| 3 · | major/minor/patch/deprecated/yanked/lockfile-behind each produce the stated severity | ✅ `cargo test -p codelaw-native --lib deps::rules` and `--test deps` |
| 4 · | `min_release_age` hides a release published 3 days before the snapshot's `fetched` date | ✅ `min_release_age_hides_a_young_release` |
| 5 · | the `suggest` fix keeps `^`, `~`, `>=`, `==` operators | ✅ `the_fix_keeps_the_operator`, and `fix --unsafe` in `--test deps` |
| 6 · | private registry via `.npmrc` works with a token in env | ✅ `--test deps`: the scope's registry and `${ACME_NPM_TOKEN}` from `.npmrc`; the token is sent, never written; without it, exit 2 and nothing written |

## Blockers

None.

## As built (2026-10-25, W-52 worktree)

- **Defaults:** `patch` is `info`, as "What a finding says" states; the `[deps]` example's `patch = "off"` is a choice a project can make.
- **Latest** is the highest stable version that is not deprecated or yanked, by semver / PEP 440 (`crates/codelaw-native/src/deps/pep440.rs`, written here: no new crate). The snapshot keeps versions from the lowest in use up, and prereleases only under `[deps] prerelease`, so `react`'s thousands of canaries never reach the committed file. Its `deprecated` / `yanked` are per-version maps, not the sketch's single field.
- **The network goes through `curl`** on the PATH, as the as-of date goes through `git`: no HTTP or TLS crate enters the binary (L2). Tokens go to curl on stdin.
- **Python indexes** are read through PEP 691's JSON simple API (PyPI serves it too), so a private index and pypi.org take one path.
- **Engine hooks** (SR-ARCHITECTURE 15b): `FileContext::as_of` (recorded, so the cache keys by date), `NativeRule::config_section` (`[deps]` arrives as options) and `NativeRule::severity_of` (a graded finding's severity).
- **Not done:** `requirements*.txt` — not an L3 language, so no rule can report on its lines (the decider would have to amend L3, or rule that `deps refresh` should still read it); the `rules_for_path` agent hint ("this repo uses React 18") — not in the DoD, not built; W-51's refresh step — waits for W-51.

## Record affected

L5 decision 3 (the `deps refresh` carve-out, the decider's ruling of 2026-10-25) · SR-CONFIG (`[deps]`, owned by SR-RULE-PACKS) · SR-RULE-PACKS (the two `data/*` rules) · SR-CLI-MCP (`deps refresh`).

## Hazards

- **Never** let `check`, `fix`, the hook, the LSP or MCP call the network — a test runs them with networking blocked.
- `latest` isn't always the right target (a package's `latest` tag can be a back-port): compare by semver, report `latest` only when it is the highest stable.
- Monorepos: one snapshot at the root, every workspace's manifest covered.
- Python version specifiers (PEP 440) are not semver: use a PEP 440 parser.

## Reference

[npm registry API](https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md) · [PyPI JSON API](https://docs.pypi.org/api/json/) · [PEP 440](https://peps.python.org/pep-0440/) · [Renovate `minimumReleaseAge`](https://docs.renovatebot.com/configuration-options/#minimumreleaseage)

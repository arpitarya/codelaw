---
type: Handoff
name: W-52
description: "Arpit, 2026-10-25: an explicit, network-only `codelaw deps refresh` writes a committed snapshot of registry facts; an offline rule `data/outdated-dependency` flags each manifest line against it, with a suggest fix that bumps the range."
item: W-52
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-52 — Latest-version highlighting for npm and Python dependencies

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L5 decision 3: one named command, `deps refresh`, may use the network; the engine and every rule stay offline. Write `[deps]` into SR-CONFIG and the rule into SR-RULE-PACKS | the decider's ruling, recorded | ⏳ |
| 2 | `codelaw deps refresh`: read manifests, query npm and PyPI, write `.codelaw/deps-snapshot.json` | registry facts as a committed file | ⏳ |
| 3 | `data/outdated-dependency` in the data pack: compare each manifest line with the snapshot | the highlight, in CLI, CI, LSP, hook | ⏳ |
| 4 | `suggest` fix: bump the range, keep its operator | one-click update of the manifest | ⏳ |
| 5 | `data/stale-deps-snapshot` (info): the snapshot is older than `[deps] max_snapshot_age` at the as-of date | nobody trusts a stale list | ⏳ |
| 6 | Docs and the auto-fix PR's refresh step (W-51) | stays fresh by itself | ⏳ |

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
| 1 · | `deps refresh` on a fixture with npm + Python manifests writes a sorted snapshot; a second run with no registry change writes identical bytes | ⏳ |
| 2 · | `check` with the network off gives the same findings as with it on | ⏳ |
| 3 · | major/minor/patch/deprecated/yanked/lockfile-behind each produce the stated severity | ⏳ |
| 4 · | `min_release_age` hides a release published 3 days before the snapshot's `fetched` date | ⏳ |
| 5 · | the `suggest` fix keeps `^`, `~`, `>=`, `==` operators | ⏳ |
| 6 · | private registry via `.npmrc` works with a token in env | ⏳ |

## Blockers

None.

## Record affected

L5 decision 3 (the `deps refresh` carve-out, the decider's ruling of 2026-10-25) · SR-CONFIG (`[deps]`, owned by SR-RULE-PACKS) · SR-RULE-PACKS (the two `data/*` rules) · SR-CLI-MCP (`deps refresh`).

## Hazards

- **Never** let `check`, `fix`, the hook, the LSP or MCP call the network — a test runs them with networking blocked.
- `latest` isn't always the right target (a package's `latest` tag can be a back-port): compare by semver, report `latest` only when it is the highest stable.
- Monorepos: one snapshot at the root, every workspace's manifest covered.
- Python version specifiers (PEP 440) are not semver: use a PEP 440 parser.

## Reference

[npm registry API](https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md) · [PyPI JSON API](https://docs.pypi.org/api/json/) · [PEP 440](https://peps.python.org/pep-0440/) · [Renovate `minimumReleaseAge`](https://docs.renovatebot.com/configuration-options/#minimumreleaseage)

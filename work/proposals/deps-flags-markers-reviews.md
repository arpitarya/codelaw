---
type: Proposal
title: The deps pack, feature-flag expiry, expiring markers, size and ownership rules, rules from review comments
description: "Ruled by the decider 2026-10-29/30 (Cowork, by voice): a nineteenth pack `deps` (vulnerable, outdated, licence, lockfile sync, version drift, runtime agreement) fed by configurable, merged advisory sources; generic feature-flag expiry; TODO and skipped-test markers with tickets and dates; complexity, component-size and CODEOWNERS rules; a skill that turns repeated review comments into rules."
status: graduated
timestamp: 2026-10-30T00:15:00+05:30
---

# The deps pack, flags, markers, size, ownership and review-born rules

**Ruled 2026-10-29 → 30** (the decider, by voice and chat, Cowork) → built by
[W-128](../open/W-128-deps-pack-and-vulnerabilities.md) ·
[W-129](../open/W-129-deps-hygiene-rules.md) ·
[W-130](../open/W-130-feature-flag-expiry.md) ·
[W-131](../open/W-131-todo-and-skipped-test-markers.md) ·
[W-132](../open/W-132-complexity-and-codeowners.md) ·
[W-133](../../archive/open/W-133-rules-from-review-comments.md).
This file is the design until each item writes its record; the record then wins.

*"Everything that we talked about looks good."* · *"We build everything in single shot."* · *"file it."*

## 1 · In one line

**`codelaw deps refresh` also fetches advisories (and licences) from a configurable,
merged list of sources into the committed snapshot; a new `deps` pack reads it
offline and says, per dependency — direct or transitive — whether it is
vulnerable, outdated, wrongly licensed, out of sync or drifting, and which exact
version to move to. Beside it: flag expiry, TODO/skip markers with tickets and
dates, complexity and CODEOWNERS rules, and a skill that mines review comments
for new rules.**

## 2 · The `deps` pack (W-128, W-129)

### 2.1 The pack

- **Name:** `deps` — `@codelaw/rules-deps` (npm), `codelaw-rules-deps` (PyPI), both
  registries; Fux-index companions `@codelaw/rules-deps-fux-index`,
  `codelaw-rules-deps-fux-index`. *"I like the first one, that is deps."* ·
  *"The cost part, I agree to the cost part. Let's have it as a separate package."*
- **L6 amended:** nineteen packs, seventeen on npm, twelve on PyPI.
- **Rules:**

| id | from | sev. default | fix |
|---|---|---|---|
| `deps/vulnerable` | new | by advisory severity | suggest (range edit / override); manual when no clean version |
| `deps/outdated` | `data/outdated-dependency` (renamed) | by grade (unchanged) | suggest (unchanged) |
| `deps/stale-snapshot` | `data/stale-deps-snapshot` (renamed) | info | manual |
| `deps/incomplete-scan` | new | warn | manual |
| `deps/licence` | new | error (denied) / warn (unknown) | manual |
| `deps/lockfile-sync` | new | error | manual |
| `deps/version-drift` | new | warn | suggest |
| `deps/runtime-agreement` | new | warn | suggest where the pin is in an L3 file, else manual |

- The renames go through the rule-deprecation path, so a config naming the old id
  is reported with a fix to the new one. Codelaw is not widely released yet, so the
  rename is cheap now.
- `data/package-json` and `data/pyproject` presets stop listing the dependency
  rules; `deps/recommended` lists them; `codelaw init` adds the `deps` pack where a
  `package.json` or `pyproject.toml` exists. **One pack only** — the security packs
  do not re-export it (the decider, 2026-10-29: one place to install and configure).

### 2.2 Advisory sources — configurable, on/off, merged

*"I want the URLs configurable … multiple sources … I can mark them as true or false
… find a URL that will definitely not be blocked and keep it as the first one."* ·
*"Merge all the enabled ones."*

```toml
[deps]
advisory_sources = [
  { name = "registry", kind = "registry", enabled = true },              # first: the registry you already install from
  { name = "osv",      kind = "osv", url = "https://api.osv.dev", enabled = true },
  { name = "mirror",   kind = "osv", url = "https://osv.company.example", enabled = false },
]
```

- **`registry`** uses the npm registry and Python index the project already resolves
  (`.npmrc`, `PIP_INDEX_URL`, uv's default index — SR-RULE-PACKS decision 15's
  resolution, tokens included). No host is ever "certainly unblocked", but this one is
  the closest: an org that can install packages can reach it (or its mirror).
  Measured from the decider's Mac on 2026-10-29: npm and PyPI answered, OSV and its
  bucket were refused (403 by the proxy). A mirror that does not pass the audit
  endpoints through makes this source `unavailable`, not silent.
- **Endpoints (verified 2026-10-29):**

| kind | request | gives |
|---|---|---|
| registry · npm | `POST <registry>/-/npm/v1/security/advisories/bulk` body `{"<name>": ["<ver>", …]}` | GHSA url/id, `vulnerable_versions` range, severity, CVSS, CWE — **no fixed version** (derived from the range) |
| registry · PyPI | `GET <index-host>/pypi/<name>/<ver>/json` → `vulnerabilities[]` | id, aliases, `fixed_in[]`, `withdrawn` — **per queried version** |
| osv | `POST <url>/v1/query` body `{"package":{"name":…,"ecosystem":"npm"\|"PyPI"}}` (by package, no version) | every advisory with `affected[].ranges[].events` (`introduced`/`fixed`), aliases, severity |

- **Custom sources** name a `kind` (the wire shape), so an internal OSV mirror or an
  Artifactory proxy of the registry is one entry. More kinds are added by a later item.
- **Merge, safer answer wins:** advisories are one record when their ids or aliases
  meet (GHSA ∪ PYSEC ∪ CVE kept); a version is affected if any source says so; the
  fixed bound is the highest named; severity the highest; each advisory lists the
  sources that reported it; withdrawn records dropped.
- **Known-clean:** a version counts as clean only where a source covers it — OSV and
  the npm bulk range cover every version; PyPI's registry source covers the versions
  queried, so `refresh` queries, per PyPI package, the version(s) in use, each
  `fixed_in`, and `latest`. A target not covered by any source is never offered as
  clean; the message says "fixed in X, not verified clean".
- **A blocked source does not stop the refresh:** a `warning:` line, and the snapshot
  records `sources.<name> = { state = "unavailable", on = <day>, error = "<HTTP 403 …>" }`.
  Refresh exits 2 and writes nothing only if **every** enabled source fails.
- **`deps/incomplete-scan`** — *"Yes, it should show … the scan wasn't complete."*
  One warning per project while any enabled source is `unavailable` in the snapshot,
  naming it and its day; fix (manual): refresh once it is reachable, or set
  `enabled = false` if the org will never allow it.
- **Licence metadata** is fetched by the same refresh for the versions in use: npm
  packument `license`; PyPI `info.license_expression` (PEP 639), else `license`,
  else the `License ::` classifiers.
- **L5:** still one networked command; decision 3 amended to name the advisory hosts
  and licence metadata. The engine and the MCP server stay offline.

### 2.3 `deps/vulnerable`

- **Version in use:** the lockfile's (`package-lock.json`, `pnpm-lock.yaml`,
  `yarn.lock`, `uv.lock`, `poetry.lock`), else the range's start — the existing code.
- **Direct and transitive in one build** (*"We build everything in single shot."*).
- **The upgrade target:** the lowest version ≥ the one in use that is known-clean
  against **every** merged advisory, not deprecated or yanked, and at least
  `[deps] min_release_age` old — **inside the current major first**, so a security fix
  never forces a major; if the major has none, the lowest clean version above it, and
  the message says it is a major.
- **Fix:** direct → the manifest range moves to the target, operator kept (decision
  15's edit). Transitive → an override in the manifest: npm `overrides`, yarn
  `resolutions`, pnpm `pnpm.overrides`, uv `[tool.uv] override-dependencies`; Poetry has
  none → manual (add a direct constraint). The message ends "then run `npm install` /
  `uv lock`" — codelaw never runs installers. No clean version → manual; the skill
  says replace, remove, or a dated ignore.
- **Where a transitive finding lands:** on the lockfile's line where the lockfile is
  an L3 language (package-lock JSON, pnpm YAML, uv/poetry TOML); yarn classic's
  `yarn.lock` is not → on `package.json` **and** on the `yarn.lock` line (§8 Q-A, ruled B; W-136).
- **Severity:** `[deps] severity` gains `critical`/`high` → `error`, `moderate` →
  `warn`, `low` → `info` (each settable, `off` drops it).
- **Ignore:** `[deps] ignore` entries may name an advisory —
  `{ package, advisory, expires, added_by, reason }`, a dated exception (SR-BASELINE
  decision 9).
- **Message:** `lodash 4.17.15 — GHSA-35jh-r3h4-6jhm (high, CVE-2021-23337) and 2 more; upgrade to 4.17.21` + snapshot day + `incomplete` when any source was unavailable.
- **Outdated stays as built;** a package both outdated and vulnerable gets both
  findings on its line. The editor shows them with no extension change.

### 2.4 Agents — *"this is a vulnerable package. This is the fixed version. Go ahead and use this version."*

- **MCP `check_dependency { ecosystem, name, version? }`**, offline from the snapshot:
  advisories, target, latest, licence, snapshot day, complete or not.
- **Post-edit hook** on a manifest or lockfile edit: the finding in plain words —
  "lodash 4.17.15 is vulnerable (GHSA-35jh, high). Change it to ^4.17.21, then run npm install."
- **Inform or block, per severity** — *"let agent decide … Or whether it needs to be
  blocked or not. … I want it to be configurable."*
  `[deps] agent = { critical = "inform", high = "inform", moderate = "inform", low = "inform" }`
  (default all `inform`). `block` stops the agent's turn until the version is fixed or
  a dated ignore exists. CI and `codelaw check` keep the severity table — the hook
  setting never changes a CI verdict.
- **`rules_for_path`** on a manifest, or a source file importing a vulnerable or
  major-behind package, names it (builds B-052).
- **The skills package:** codelaw-write and codelaw-fix (and every agent-facing file
  that tells an agent how to add a dependency) gain one instruction — check through
  codelaw before installing, use the version it gives.

### 2.5 The four hygiene rules (W-129)

- **`deps/licence`** — `[deps.licence] allow`, `deny` (globs over SPDX ids), `unknown =
  "warn"`. Default: permissive allowed (MIT, Apache-2.0, BSD-2/3-Clause, ISC, 0BSD,
  Unlicense, CC0-1.0, PSF-2.0, Python-2.0, BlueOak-1.0.0), copyleft and source-available
  denied (`GPL-*`, `AGPL-*`, `LGPL-*`, `SSPL-*`, `BUSL-*`, `EUPL-*`), anything else
  `unknown`. SPDX expressions parsed (`OR`: one allowed is enough; `AND`: all must be;
  `WITH` kept with its id). Direct and transitive. Fix manual: replace, or a dated ignore.
- **`deps/lockfile-sync`** — the manifest and its lockfile disagree: a dependency in
  one and not the other, or a range the locked root entry does not satisfy
  (package-lock `packages[""]`, pnpm `importers`, yarn's root, uv's root
  `requires-dist`; Poetry's `content-hash` recomputed). Fix manual: run the
  package manager's lock command.
- **`deps/version-drift`** — across a monorepo's workspaces (npm/pnpm/yarn workspaces,
  uv workspace members), one dependency declared with ranges that do not agree. Fix
  suggest: align to `[deps.drift] policy = "highest"` (or `"root"`); `ignore` list.
- **`deps/runtime-agreement`** — every place that names the Node or Python version
  agrees: `engines.node`, `volta.node`, `.nvmrc`, `.node-version`, `.tool-versions`,
  `mise.toml`, `requires-python`, `.python-version`, GitHub Actions `setup-node`
  `node-version` / `setup-python` `python-version`, Dockerfile `FROM node:` /
  `python:` (read only). Pins must satisfy the declared range and equal each other.
  `[deps.runtime] source` names the truth (default `engines.node` /
  `requires-python`). Fix suggest on L3 files; manual on the text files. Findings show on both the L3 file and the text file's line (§8 Q-A, ruled B; W-136).

## 3 · Feature-flag expiry (W-130)

*"We need to build it out for all of them. And … in a bit more generic way, because
… people might be using wrapper over Unleash, wrapper over LaunchDarkly or Split.io."*

- **Accessors are config:** a call pattern with a `$NAME` slot.
  ```toml
  [flags]
  presets   = ["launchdarkly", "unleash", "split", "openfeature", "growthbook", "flagsmith"]
  accessors = ["isFeatureOn($NAME)", "flags.get($NAME, $$$)", "useFeature({ key: $NAME })"]
  inline    = "allow"   # or "registry-only"
  warn_days = 14
  ```
  Presets are the same patterns, shipped. JS, TS (React hooks included) and Python.
- **Wrappers found automatically:** a function whose own parameter flows unchanged
  into a known accessor's `$NAME` is an accessor too, transitively — deterministic,
  through the project graph. `codelaw flags accessors` prints the list.
- **Names:** string literals; constants and enums resolved through the graph; a
  dynamic name → `info`, marked uncertain, never an error.
- **Where expiry lives — both** (*"Both."*): the registry `.codelaw/flags.toml`
  (`[[flag]] name, owner, ticket, expires, keep, reason`) and/or an inline marker
  `// codelaw-flag new-checkout expires 2027-02-28 owner @team`. Inline-only counts as
  registered; both disagree → the registry wins and the marker is reported with an
  auto fix to match; one flag inline in several places with different values → error,
  fix moves it into the registry. `inline = "registry-only"` forbids markers.
- **Findings at each use:** unregistered → error; within `warn_days` → warn; past
  `expires` (as-of date) → error; `expires = "never"` needs a `reason`. On the
  registry: an entry with no uses left → info, auto fix removes it.
- **Fix:** `keep = "on"|"off"` → suggest: `if (isOn("x")) A else B` becomes `A` (or
  `B`); without `keep` → manual, carried by a skill. Never asks LaunchDarkly/Unleash
  whether a flag is live — the registry is the truth (L5).

## 4 · TODO and skipped-test markers (W-131)

*"I want the ticket and the date configurable, whether I want it as mandatory thing or not."*

```toml
[todo]
tags           = ["TODO", "FIXME", "HACK"]
require_ticket = true
require_date   = false
ticket_pattern = "[A-Z][A-Z0-9]+-\\d+"
warn_days      = 14
```

- Syntax: `TODO(ABC-123, 2027-01-31): text` — ticket and date in parentheses, either
  order, either optional unless required. A written date is enforced even when not
  required. Every L3 language that has comments.
- Expired = the as-of date ≥ the date (SR-BASELINE decision 9's semantics). Fix manual.
- **Skipped tests:** `.only` / `fit` / `fdescribe` → error, fix suggest (drop the
  focus). `.skip`, `xit`, `xdescribe`, `test.skip`, `@pytest.mark.skip`/`skipif`/`xfail`,
  `unittest.skip` need a reason and an expiry: the framework's own reason argument
  where it has one, else a `// SKIP(ABC-123, 2027-01-31): reason` comment on the line
  above; `[tests.skip] require_reason = true, require_expiry = true, require_ticket = false`.
  Expired → error. Fix manual.

## 5 · Complexity and component size (W-132)

- Built already (W-58): `js/max-lines`, `js/max-lines-per-function`, `js/max-params`
  and the `py/` twins.
- New: `js/complexity`, `py/complexity` (cyclomatic, default 20); `js/max-depth`,
  `py/max-depth` (default 4); `react/component-max-lines` (default 200),
  `react/max-props` (default 8), `react/max-jsx-depth` (default 6). Opt-in presets
  (`js/size`, `py/size`, `react/size`). Fix manual + a skill (extract, split).

## 6 · CODEOWNERS coverage (W-132)

- Reads `.github/CODEOWNERS`, `CODEOWNERS`, `docs/CODEOWNERS`, `.gitlab/CODEOWNERS`,
  or `[codeowners] path`. gitignore-style patterns, last match wins, GitLab sections.
- Reports: a discovered file no pattern owns; a pattern that matches no file
  (stale); an owner line with no owners unless `allow_unowned = true`.
- Findings on the CODEOWNERS line itself (a stale pattern) and on the root manifest
  once per project listing unowned paths (§8 Q-A, ruled B; W-136). Fix manual (stale pattern: suggest delete).

## 7 · Rules from review comments (W-133)

- **A skill, not engine code** (L5: a model may run outside the engine). Ships in
  the skills package. No `codelaw` command fetches anything (L5 decision 3 stands).
- **Platforms — all, by adapter** (*"All of them."*): GitHub (`gh`), GitLab (`glab`),
  Bitbucket Cloud and Data Center (REST through `curl`), Azure DevOps (`az repos`).
  Detected from the git remote; self-hosted via `[reviews] platform`, `url`. Logins are
  the platforms' own CLIs or an environment token; codelaw stores no credential.
  Every adapter emits one shape: comment, file, line, diff hunk, reviewer, PR link, date.
- **Thresholds** (*"Repeats three times or more. Reviewers, it could be one person as
  well. … configurable. Window … configurable."*):
  `[reviews] min_repeats = 3, min_reviewers = 1, window = "6m"` (6 months chosen by
  the agent; no value was named).
- **Steps:** collect merged-PR comments in the window → cluster repeated feedback →
  propose candidates with evidence (count, reviewers, links, sample diffs), each
  graded *can be a rule* or *guidance only* (→ steering files) → for the approved
  ones, the existing rule-authoring flow builds it from the real diffs and runs
  `codelaw test` → lands as `preview` with existing violations frozen in the baseline.
  The decider approves every step; nothing turns on by itself. A reviewer's
  `codelaw: rule` comment fast-tracks a point into the next run.

## 8 · Ruled after filing (2026-10-30, voice)

- **Q-A — findings for files outside L3 → B.** *"I want the error to be displayed in
  both the places, in package.json as well as in nvmrc."* L3 is amended so `.nvmrc`,
  `.node-version`, `.python-version`, `.tool-versions`, Dockerfile, CODEOWNERS and yarn
  classic's `yarn.lock` may carry findings — never linted or formatted as a language. A
  finding about one shows on the L3 file it concerns **and** on the text file's line:
  [W-136](../open/W-136-l3-findings-on-plain-text-config.md).
- **Registry names for `deps`** — two items, *"just before the deps release"*:
  [W-134](../open/W-134-deps-pypi-pending-publishers.md) (PyPI pending publishers,
  Claude fills the form in Chrome, Arpit submits) and
  [W-135](../open/W-135-deps-npm-first-publish.md) (npm first hand publish).

## 9 · Laws touched

L5 decision 3 (advisory and licence hosts in `deps refresh`) · L6 (nineteen packs) ·
L4 (every new rule has its fix class above) · L8 (each new rule measured before its
budget) · L10 (any file `init` writes, e.g. a starter `flags.toml`, from a template) ·
L11 (no new dependency: semver, PEP 440 and SPDX parsing are codelaw's own) · L3
(plain-text config files carry findings — W-136).

## 10 · Sources (read 2026-10-29)

- OSV data and API — <https://google.github.io/osv.dev/data/>, <https://google.github.io/osv.dev/post-v1-querybatch/>
- PyPI JSON API (`vulnerabilities`, `fixed_in`) — <https://docs.pypi.org/api/json/>
- GitHub Advisory Database (CC-BY 4.0, OSV format) — <https://github.com/github/advisory-database>
- npm bulk advisory endpoint — probed live from the decider's Mac, 2026-10-29

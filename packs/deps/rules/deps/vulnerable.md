---
id: deps/vulnerable
title: "No dependency in use has a known vulnerability"
languages: [json, toml, yaml]
files: ["**/package.json", "**/pyproject.toml", "**/package-lock.json", "**/pnpm-lock.yaml"]
fix: suggest
fix_reason: "the fix moves a range or adds an override to the lowest known-clean version, which changes what installs; a new major can change behaviour, and an override changes the install of the whole tree"
stability: preview
since: 0.5.0
---

Every version a project installs — each dependency of a `package.json` or `pyproject.toml`, and every package its lockfile (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `uv.lock`, `poetry.lock`) installs through them — is compared with the advisories in `.codelaw/deps-snapshot.json`, which `codelaw deps refresh` wrote from the enabled `[deps] advisory_sources`. The rule reads only that file and the lockfiles; it never reads the network, so `check` gives the same answer offline. With no snapshot, or one that holds no advisories, nothing is reported (`deps/incomplete-scan` says why).

The finding names the worst advisory, its severity and CVE, how many more there are, and the version to move to: the **lowest known-clean** version above the one in use — a version no merged advisory affects and an advisory source covered — that is not deprecated or yanked and is at least `[deps] min_release_age` old, **inside the major in use first**. When the major has none, the lowest clean version above it is offered and the message says it is a new major. A fixed version no source covered is named but never offered ("not verified clean"); with no fixed version at all, the fix is manual.

Each finding is graded by its worst advisory, and `[deps] severity` sets each grade's severity (or `off`):

| advisory severity | default |
|---|---|
| `critical`, `high` | `error` |
| `moderate` | `warn` |
| `low` | `info` |

An advisory no source grades counts as `high`.

Where it is reported, and the fix:

- **A direct dependency:** on its line in the manifest; the fix moves the range's version to the target and keeps its operator (`^`, `~`, `>=`, `=` or none; `==`, `>=` or `~=`).
- **A transitive one, in `package-lock.json` or `pnpm-lock.yaml`:** on its lockfile entry; the message names the override to add — npm `overrides` or `pnpm.overrides` in `package.json`. A lockfile is never edited.
- **A transitive one, in `yarn.lock` or `uv.lock`:** on the first line of the manifest beside the lockfile (a `.lock` file is not one of the nine languages); the fix adds yarn `resolutions` or uv's `[tool.uv] override-dependencies`.
- **In `poetry.lock`:** Poetry has no override, so the message says to add a direct constraint.

Every message ends with the install to run (`npm install`, `uv lock`, …) and the snapshot's day, and says `incomplete` while an enabled source was unavailable. `[deps] ignore` takes a package, or `{ package, advisory, expires, added_by, reason }` for one advisory until a date.

**Why:** A vulnerable version stays exploitable until someone moves it, and the version that fixes one advisory can carry another; the lowest version every source leaves clean is the smallest safe change.

```codelaw
native: deps/vulnerable
```

## How to fix

1. Run `codelaw fix --unsafe` to move the range (or add the override) to the version named, or make the same edit by hand.
2. Run the install the message names (`npm install`, `pnpm install`, `yarn install`, `uv lock`, `poetry lock`) so the lockfile follows; codelaw never runs installers.
3. For a new major, read the package's changelog and update the code that uses it.
4. With no clean version: replace the package, remove it, or add a dated `[deps] ignore` entry naming the advisory, with a reason.

## Examples

### JSON

#### Don't

```json
// .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {
    "npm:lodash": {
      "versions": { "4.17.15": "2019-07-19", "4.17.21": "2021-02-20" },
      "advisories": [ { "id": "GHSA-35jh-r3h4-6jhm", "aliases": ["CVE-2021-23337"], "severity": "high", "affected": ["<4.17.21"], "fixed": ["4.17.21"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    }
  },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
// package.json
{
  "dependencies": {
    "lodash": "^4.17.15"
  }
}
```

#### Do

```json
// .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {
    "npm:lodash": {
      "versions": { "4.17.15": "2019-07-19", "4.17.21": "2021-02-20" },
      "advisories": [ { "id": "GHSA-35jh-r3h4-6jhm", "aliases": ["CVE-2021-23337"], "severity": "high", "affected": ["<4.17.21"], "fixed": ["4.17.21"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    }
  },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
// package.json
{
  "dependencies": {
    "lodash": "^4.17.21"
  }
}
```

### TOML

#### Don't

```toml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {
    "pypi:jinja2": {
      "versions": { "3.1.2": "2022-04-28", "3.1.3": "2024-01-10" },
      "advisories": [ { "id": "GHSA-h5c8-rqwp-cp95", "severity": "moderate", "affected": ["<3.1.3"], "fixed": ["3.1.3"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    }
  },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
# pyproject.toml
[project]
name = "app"
dependencies = ["jinja2>=3.1.2"]
```

#### Do

```toml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {
    "pypi:jinja2": {
      "versions": { "3.1.2": "2022-04-28", "3.1.3": "2024-01-10" },
      "advisories": [ { "id": "GHSA-h5c8-rqwp-cp95", "severity": "moderate", "affected": ["<3.1.3"], "fixed": ["3.1.3"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    }
  },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
# pyproject.toml
[project]
name = "app"
dependencies = ["jinja2>=3.1.3"]
```

### YAML

#### Don't

```yaml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {
    "npm:minimist": {
      "versions": { "1.2.5": "2020-03-12", "1.2.6": "2022-03-21" },
      "advisories": [ { "id": "GHSA-xvch-5gv4-984h", "severity": "critical", "affected": ["<1.2.6"], "fixed": ["1.2.6"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    }
  },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
# pnpm-lock.yaml
lockfileVersion: '9.0'
packages:
  minimist@1.2.5:
    resolution: {integrity: sha512-x}
```

#### Do

```yaml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {
    "npm:minimist": {
      "versions": { "1.2.5": "2020-03-12", "1.2.6": "2022-03-21" },
      "advisories": [ { "id": "GHSA-xvch-5gv4-984h", "severity": "critical", "affected": ["<1.2.6"], "fixed": ["1.2.6"], "sources": ["osv"] } ],
      "scanned": { "osv": ["*"] }
    }
  },
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
# pnpm-lock.yaml
lockfileVersion: '9.0'
packages:
  minimist@1.2.6:
    resolution: {integrity: sha512-y}
```

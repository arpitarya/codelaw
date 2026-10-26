---
id: data/outdated-dependency
title: "Dependencies are on their latest version"
languages: [json, toml]
files: ["**/package.json", "**/pyproject.toml"]
fix: suggest
fix_reason: "a new major or minor version can change behaviour; the fix moves the range to the latest version, and the code that uses the package may need changes"
stability: preview
since: 0.3.0
---

Each dependency of a `package.json` (`dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`) and of a `pyproject.toml` (`[project] dependencies`, `[project.optional-dependencies]`, `[dependency-groups]`) is compared with `.codelaw/deps-snapshot.json`, the registry facts `codelaw deps refresh` wrote. The rule reads only that file and the lockfiles; it never reads a registry, so `check` gives the same answer offline. With no snapshot, nothing is reported.

The version in use is the lockfile's (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `uv.lock` or `poetry.lock`, beside the manifest or above it), else the one the range starts from. The latest is the highest stable version in the snapshot — by semver for npm, PEP 440 for Python — that is not deprecated or yanked and is at least `[deps] min_release_age` old on the snapshot's `fetched` date.

Each finding is graded, and `[deps] severity` sets each grade's severity (or `off`):

| finding | default |
|---|---|
| a major version behind | `warn` |
| a minor or patch version behind | `info` |
| the version in use is deprecated (npm) or yanked (PyPI) | `error` |
| the range already allows the latest, but the lockfile is behind | `info` |

The fix moves the range's version to the latest and keeps its operator: `^`, `~`, `>=`, `=` or none for npm; `==`, `>=` or `~=` for Python, at the precision written (`~=4.2` becomes `~=5.1`). A range the fix can't move without changing its meaning — `||`, a hyphen, `x`, an upper bound the latest is outside — is reported without a fix. `[deps] ignore` names packages never reported.

**Why:** A dependency left behind stops getting fixes and security patches, and the jump grows with every release; a deprecated or yanked version was withdrawn by its own maintainers.

```codelaw
native: data/outdated-dependency
```

## How to fix

1. Run `codelaw fix --unsafe` to move the range to the latest version, or edit the version by hand.
2. Run your package manager's install (`npm install`, `uv lock`, …) so the lockfile follows.
3. For a major version, read the package's changelog and update the code that uses it.
4. If the project must stay on this version, add the package to `[deps] ignore`, with an `expires` date when the stay is temporary.

## Examples

### JSON

#### Don't

```json
// .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-25",
  "packages": {
    "npm:react": { "latest": "19.2.0", "versions": { "18.3.1": "2024-04-26", "19.2.0": "2025-10-01" } }
  }
}
// package.json
{
  "dependencies": {
    "react": "^18.3.1"
  }
}
```

#### Do

```json
// .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-25",
  "packages": {
    "npm:react": { "latest": "19.2.0", "versions": { "18.3.1": "2024-04-26", "19.2.0": "2025-10-01" } }
  }
}
// package.json
{
  "dependencies": {
    "react": "^19.2.0"
  }
}
```

### TOML

#### Don't

```toml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-25",
  "packages": {
    "pypi:django": { "latest": "5.1.2", "versions": { "4.2.0": "2023-04-03", "5.1.2": "2024-10-08" } }
  }
}
# pyproject.toml
[project]
name = "app"
dependencies = ["django~=4.2"]
```

#### Do

```toml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-25",
  "packages": {
    "pypi:django": { "latest": "5.1.2", "versions": { "4.2.0": "2023-04-03", "5.1.2": "2024-10-08" } }
  }
}
# pyproject.toml
[project]
name = "app"
dependencies = ["django~=5.1"]
```

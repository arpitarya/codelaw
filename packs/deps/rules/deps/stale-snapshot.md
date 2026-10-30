---
id: deps/stale-snapshot
title: "The dependency snapshot is recent"
languages: [json, toml]
files: ["package.json", "pyproject.toml"]
severity: info
fix: manual
fix_reason: "refreshing the snapshot reads the registries, which needs the network; the engine never uses it (L5)"
stability: preview
since: 0.3.0
---

`.codelaw/deps-snapshot.json` — the registry facts `deps/outdated` reads — was fetched no more than `[deps] max_snapshot_age` (14 days by default) before the as-of date, and can be read. The finding is on the root `package.json`, or the root `pyproject.toml` when there is no root `package.json`, so a project gets one. The age is judged on the as-of date (`--as-of`, `CODELAW_AS_OF`, else `HEAD`'s commit date), never the clock; with no as-of date, only an unreadable snapshot is reported.

**Why:** An old snapshot hides every release since it was written, so `deps/outdated` says a dependency is current when it isn't.

```codelaw
native: deps/stale-snapshot
```

## How to fix

1. Run `codelaw deps refresh`: it reads npm and the Python index and writes the snapshot again.
2. Commit `.codelaw/deps-snapshot.json`, so CI and every teammate judge by the same facts.

## Examples

### JSON

#### Don't

```json
// .codelaw/deps-snapshot.json
{ "fetched": "2026-10-25", "packages": [ }
// package.json
{
  "name": "app"
}
```

#### Do

```json
// .codelaw/deps-snapshot.json
{ "fetched": "2026-10-25", "packages": {} }
// package.json
{
  "name": "app"
}
```

### TOML

#### Don't

```toml
# .codelaw/deps-snapshot.json
{ "fetched": "2026-10-25", "packages": [ }
# pyproject.toml
[project]
name = "app"
```

#### Do

```toml
# .codelaw/deps-snapshot.json
{ "fetched": "2026-10-25", "packages": {} }
# pyproject.toml
[project]
name = "app"
```

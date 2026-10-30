---
id: deps/incomplete-scan
title: "Every advisory source was read"
languages: [json, toml]
files: ["package.json", "pyproject.toml"]
severity: warn
fix: manual
fix_reason: "reading an advisory source again needs the network, which the engine never uses (L5); turning a source off is the team's decision"
stability: preview
since: 0.5.0
---

`.codelaw/deps-snapshot.json` records how each advisory source of `[deps] advisory_sources` answered the last `codelaw deps refresh`. While a source that is still enabled was `unavailable` — refused by a proxy, down, or a mirror that does not pass the audit endpoint through — the vulnerabilities only it knows are not reported, and this rule says so: one finding per project, naming each such source, its day and its error, on the root `package.json` (else the root `pyproject.toml`). A snapshot that holds packages but no source at all — written before the advisory sources, or with every source turned off — is reported too.

**Why:** A clean `deps/vulnerable` result means "no source we read knows of one"; when a source was not read, a clean result says less than it seems to.

```codelaw
native: deps/incomplete-scan
```

## How to fix

1. Run `codelaw deps refresh` once the source is reachable, and commit the snapshot.
2. If the organisation will never allow that source, set `enabled = false` on it in `[deps] advisory_sources`, so the scan is complete as configured.

## Examples

### JSON

#### Don't

```json
// .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {},
  "sources": {
    "osv": { "kind": "osv", "state": "unavailable", "on": "2026-10-29", "error": "https://api.osv.dev/v1/query answered HTTP 403" },
    "registry": { "kind": "registry", "state": "ok", "on": "2026-10-29" }
  }
}
// package.json
{
  "name": "app"
}
```

#### Do

```json
// .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {},
  "sources": {
    "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" },
    "registry": { "kind": "registry", "state": "ok", "on": "2026-10-29" }
  }
}
// package.json
{
  "name": "app"
}
```

### TOML

#### Don't

```toml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {},
  "sources": { "osv": { "kind": "osv", "state": "unavailable", "on": "2026-10-29", "error": "HTTP 403" } }
}
# pyproject.toml
[project]
name = "app"
```

#### Do

```toml
# .codelaw/deps-snapshot.json
{
  "fetched": "2026-10-29",
  "packages": {},
  "sources": { "osv": { "kind": "osv", "state": "ok", "on": "2026-10-29" } }
}
# pyproject.toml
[project]
name = "app"
```

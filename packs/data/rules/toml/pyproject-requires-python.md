---
id: toml/pyproject-requires-python
title: "pyproject.toml declares requires-python"
languages: [toml]
files: ["**/pyproject.toml"]
fix: suggest
fix_reason: "the supported Python range is a project decision; the fix inserts the `python` option, which may not be the right range"
stability: preview
options:
  python: { type: "string", default: ">=3.11", description: "the range the fix writes into `requires-python`" }
since: 0.1.0
---

A `pyproject.toml` with a `[project]` table says which Python versions the package supports, in `requires-python`, unless it lists the field in `dynamic`.

**Why:** pip uses `requires-python` to pick a release that works on the running interpreter; without it, users on an old Python install a version that then fails at import.

```codelaw
native: toml/pyproject-requires-python
```

## Examples

### TOML

#### Don't

```toml path=pyproject.toml
[project]
name = "app"
version = "1.0.0"
```

#### Do

```toml path=pyproject.toml
[project]
name = "app"
version = "1.0.0"
requires-python = ">=3.11"
```

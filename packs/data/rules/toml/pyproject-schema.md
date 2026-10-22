---
id: toml/pyproject-schema
title: "pyproject.toml matches its schema"
languages: [toml]
files: ["**/pyproject.toml"]
fix: manual
fix_reason: "a value the schema rejects is wrong in a way only its author knows how to correct"
stability: preview
ported_from: [schemastore/pyproject.json]
since: 0.1.0
---

A `pyproject.toml` matches SchemaStore's schema for it: `[build-system]` and `[project]` hold the keys PEP 517, PEP 518 and PEP 621 define, each in the form they define.

**Why:** Build backends reject or ignore a `[project]` key of the wrong shape — `authors` as a string, `dependencies` as a table — and the error appears only when someone builds the package.

```codelaw
target: $
assert: { schema: schemas/pyproject }
message: "$ERROR"
```

## How to fix

1. Read the message: it names the key and the form the schema expects.
2. Rewrite the value in that form, or move a tool's setting under `[tool.<name>]`.
3. Run `codelaw check` on the file again until it is clean.

## Examples

### TOML

#### Don't

```toml path=pyproject.toml
[project]
name = "app"
version = "1.0"
authors = "Ada Lovelace"
dependencies = { requests = ">=2" }
```

#### Do

```toml path=pyproject.toml
[project]
name = "app"
version = "1.0"
authors = [{ name = "Ada Lovelace" }]
dependencies = ["requests>=2"]
```

## Notes

The schema is SchemaStore's `pyproject.json` schema, shipped with the pack in `data/schemas/pyproject.json` with its descriptions removed; nothing is fetched. The `[tool.*]` tables whose schemas live in other SchemaStore files (ruff, black, mypy, poetry, setuptools, uv, …) accept any value. TOML dates and times are read as strings.

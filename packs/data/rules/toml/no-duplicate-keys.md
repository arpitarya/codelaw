---
id: toml/no-duplicate-keys
title: "No key or table defined twice in TOML"
languages: [toml]
fix: manual
fix_reason: "which of the two definitions was meant can't be known from the file"
stability: preview
since: 0.1.0
---

A key is defined once in its table, and a `[table]` header appears once. That covers dotted keys (`a.b = 1` and then `a = 2`), a table defined both by a header and by dotted keys, and keys inside inline tables.

**Why:** TOML forbids defining a key or table twice and conforming parsers refuse the file, so the mistake breaks every tool that reads it.

```codelaw
native: toml/no-duplicate-keys
```

## How to fix

1. Decide which definition is right.
2. Delete the other one, or merge the two tables into one.

## Examples

### TOML

#### Don't

```toml
[tool.pytest]
addopts = "-q"
testpaths = ["tests"]
addopts = "-ra"
```

#### Do

```toml
[tool.pytest]
addopts = "-ra"
testpaths = ["tests"]
```

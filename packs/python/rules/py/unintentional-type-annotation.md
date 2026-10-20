---
id: py/unintentional-type-annotation
title: "No annotation where an assignment was meant"
languages: [python]
fix: manual
fix_reason: "whether an assignment was meant needs a person"
stability: preview
ported_from: [ruff/B032]
since: 0.1.0
---

An attribute or item isn't annotated without a value (`obj.x: 1`, `d["k"]: v`).

**Why:** This is almost always a typo for `=`: nothing is assigned.

```codelaw
native: py/unintentional-type-annotation
```

## How to fix

1. Replace `:` with `=` if an assignment was meant.

## Examples

### Python

#### Don't

```py
config["debug"]: True
```

#### Do

```py
config["debug"] = True
```

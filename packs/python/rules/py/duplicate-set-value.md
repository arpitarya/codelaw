---
id: py/duplicate-set-value
title: "No duplicate items in a set literal"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/B033]
since: 0.1.0
---

A set literal lists each literal value once.

**Why:** A repeated item is dropped silently and is often a typo for another value.

```codelaw
native: py/duplicate-set-value
```

## Examples

### Python

#### Don't

```py
colors = {"red", "green", "red"}
```

#### Do

```py
colors = {"red", "green"}
```

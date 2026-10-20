---
id: py/multiple-starred-expressions
title: "One starred target per assignment"
languages: [python]
fix: manual
fix_reason: "which target should collect the rest has to be chosen"
stability: preview
ported_from: [ruff/F622]
since: 0.1.0
---

An unpacking assignment has at most one starred target.

**Why:** Two starred targets are ambiguous and a `SyntaxError`.

```codelaw
native: py/multiple-starred-expressions
```

## How to fix

1. Keep the star on one target only.

## Examples

### Python

#### Don't

```py
*head, *tail = values
```

#### Do

```py
head, *tail = values
```

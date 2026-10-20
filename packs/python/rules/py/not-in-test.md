---
id: py/not-in-test
title: "Write `x not in y`"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/E713]
since: 0.1.0
---

A membership test is negated as `x not in y`, not `not x in y`.

**Why:** `not x in y` reads as `(not x) in y`; `not in` says what is meant.

```codelaw
native: py/not-in-test
```

## Examples

### Python

#### Don't

```py
if not item in seen:
    keep(item)
```

#### Do

```py
if item not in seen:
    keep(item)
```

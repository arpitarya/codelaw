---
id: py/not-is-test
title: "Write `x is not y`"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/E714]
since: 0.1.0
---

An identity test is negated as `x is not y`, not `not x is y`.

**Why:** `not x is y` reads as `(not x) is y`; `is not` says what is meant.

```codelaw
native: py/not-is-test
```

## Examples

### Python

#### Don't

```py
if not value is None:
    use(value)
```

#### Do

```py
if value is not None:
    use(value)
```

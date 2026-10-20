---
id: py/assert-false
title: "Don't `assert False`"
languages: [python]
fix: suggest
fix_reason: "the check now also runs under `python -O`, where the assert was skipped"
stability: preview
ported_from: [ruff/B011]
since: 0.1.0
---

Code that must never be reached raises `AssertionError` instead of using `assert False`.

**Why:** `python -O` removes asserts, so the check silently disappears.

```codelaw
native: py/assert-false
```

## Examples

### Python

#### Don't

```py
assert False, "unreachable"
```

#### Do

```py
raise AssertionError("unreachable")
```

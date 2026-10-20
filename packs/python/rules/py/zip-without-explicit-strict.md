---
id: py/zip-without-explicit-strict
title: "`zip()` says `strict=`"
languages: [python]
fix: suggest
fix_reason: "`strict=` exists only from Python 3.10, and codelaw cannot tell which Python the code runs on: on an older one the added keyword raises `TypeError`"
stability: preview
options:
  target_version: { type: string, description: "the oldest Python the code supports, e.g. \"3.9\"; below 3.10 the rule is off" }
ported_from: [ruff/B905]
since: 0.1.0
---

`zip()` is called with `strict=True` or `strict=False`.

**Why:** Without it, `zip` silently stops at the shortest input, hiding inputs of different lengths. The keyword exists from Python 3.10.

```codelaw
native: py/zip-without-explicit-strict
```

## Examples

### Python

#### Don't

```py
pairs = zip(names, ages)
```

#### Do

```py
pairs = zip(names, ages, strict=True)
```

#### Fixed

```py
pairs = zip(names, ages, strict=False)
```

## Differences from the source

The fix adds `strict=False`, which keeps today's behaviour; set `strict=True` where the inputs must match. Ruff reads the target version from its own settings; here it is the `target_version` option, and the rule runs when it is unset. Because it may be unset, the fix is `suggest` (`codelaw fix --unsafe`), not `auto`: on Python before 3.10 the keyword raises `TypeError`.

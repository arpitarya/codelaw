---
id: py/map-without-explicit-strict
title: "`map()` over several iterables says `strict=`"
languages: [python]
fix: suggest
fix_reason: "`strict=` exists only from Python 3.14, and codelaw cannot tell which Python the code runs on: on an older one the added keyword raises `TypeError`"
stability: preview
options:
  target_version: { type: string, description: "the oldest Python the code supports; below 3.14 the rule is off" }
ported_from: [ruff/B912]
since: 0.1.0
---

`map()` with two or more iterables is called with `strict=True` or `strict=False`.

**Why:** Without it, `map` silently stops at the shortest input. The keyword exists from Python 3.14.

```codelaw
native: py/map-without-explicit-strict
```

## Examples

### Python

#### Don't

```py
sums = map(add, xs, ys)
```

#### Do

```py
sums = map(add, xs, ys, strict=True)
```

#### Fixed

```py
sums = map(add, xs, ys, strict=False)
```

## Differences from the source

The fix is `suggest` (`codelaw fix --unsafe`), not `auto`: the `target_version` option may be unset, and on Python before 3.14 the keyword raises `TypeError`.

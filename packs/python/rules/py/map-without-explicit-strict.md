---
id: py/map-without-explicit-strict
title: "`map()` over several iterables says `strict=`"
languages: [python]
fix: auto
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

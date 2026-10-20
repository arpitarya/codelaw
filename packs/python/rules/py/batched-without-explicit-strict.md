---
id: py/batched-without-explicit-strict
title: "`itertools.batched()` says `strict=`"
languages: [python]
fix: suggest
fix_reason: "`strict=` exists only from Python 3.13, and codelaw cannot tell which Python the code runs on: on an older one the added keyword raises `TypeError`"
stability: preview
options:
  target_version: { type: string, description: "the oldest Python the code supports; below 3.13 the rule is off" }
ported_from: [ruff/B911]
since: 0.1.0
---

`itertools.batched()` is called with `strict=True` or `strict=False`.

**Why:** Without it, a short last batch passes silently. The keyword exists from Python 3.13.

```codelaw
native: py/batched-without-explicit-strict
```

## Examples

### Python

#### Don't

```py
import itertools

batches = itertools.batched(items, 2)
```

#### Do

```py
import itertools

batches = itertools.batched(items, 2, strict=True)
```

#### Fixed

```py
import itertools

batches = itertools.batched(items, 2, strict=False)
```

## Differences from the source

Ruff reports this without a fix; codelaw suggests `strict=False`, which keeps today's behaviour. The fix is `suggest` (`codelaw fix --unsafe`), not `auto`: the `target_version` option may be unset, and on Python before 3.13 the keyword raises `TypeError`.

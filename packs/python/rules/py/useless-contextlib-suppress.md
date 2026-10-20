---
id: py/useless-contextlib-suppress
title: "`contextlib.suppress()` names an exception"
languages: [python]
fix: manual
fix_reason: "the exceptions to suppress are the author's choice"
stability: preview
ported_from: [ruff/B022]
since: 0.1.0
---

`contextlib.suppress()` is called with at least one exception class.

**Why:** With no arguments it suppresses nothing, so the `with` block is misleading.

```codelaw
native: py/useless-contextlib-suppress
```

## How to fix

1. Pass the exceptions to suppress, or remove the `with` statement.

## Examples

### Python

#### Don't

```py
import contextlib

with contextlib.suppress():
    remove("cache")
```

#### Do

```py
import contextlib

with contextlib.suppress(FileNotFoundError):
    remove("cache")
```

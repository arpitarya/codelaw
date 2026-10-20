---
id: py/mutable-contextvar-default
title: "No mutable `ContextVar` defaults"
languages: [python]
fix: manual
fix_reason: "whether to use an immutable value or set a fresh one per context is a design choice"
stability: preview
ported_from: [ruff/B039]
since: 0.1.0
---

A `ContextVar`'s `default=` isn't a list, dict, set or the result of a call.

**Why:** The default is one object shared by every context, so mutating it leaks state between them.

```codelaw
native: py/mutable-contextvar-default
```

## How to fix

1. Use an immutable default, or no default and set a fresh value in each context.

## Examples

### Python

#### Don't

```py
from contextvars import ContextVar

items = ContextVar("items", default=[])
```

#### Do

```py
from contextvars import ContextVar

items = ContextVar("items", default=())
```

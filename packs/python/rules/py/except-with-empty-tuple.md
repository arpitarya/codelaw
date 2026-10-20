---
id: py/except-with-empty-tuple
title: "No `except ():`"
languages: [python]
fix: manual
fix_reason: "the exceptions to catch are the author's choice"
stability: preview
ported_from: [ruff/B029]
since: 0.1.0
---

An `except` clause doesn't name an empty tuple.

**Why:** `except ():` catches nothing, so the handler never runs.

```codelaw
native: py/except-with-empty-tuple
```

## How to fix

1. Name the exceptions to catch, or remove the handler.

## Examples

### Python

#### Don't

```py
try:
    run()
except ():
    log()
```

#### Do

```py
try:
    run()
except ValueError:
    log()
```

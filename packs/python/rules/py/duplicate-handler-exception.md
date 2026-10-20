---
id: py/duplicate-handler-exception
title: "No exception listed twice in one handler"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/B014]
since: 0.1.0
---

An `except` tuple names each exception once.

**Why:** A repeated exception is redundant and often a typo for a different one.

```codelaw
native: py/duplicate-handler-exception
```

## Examples

### Python

#### Don't

```py
try:
    run()
except (ValueError, ValueError, TypeError):
    retry()
```

#### Do

```py
try:
    run()
except (ValueError, TypeError):
    retry()
```

## Differences from the source

Exceptions are compared by name as written; aliases such as `IOError` and `OSError` are not merged.

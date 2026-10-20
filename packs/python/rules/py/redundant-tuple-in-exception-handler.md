---
id: py/redundant-tuple-in-exception-handler
title: "No one-element tuple in `except`"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/B013]
since: 0.1.0
---

An `except` clause catching one exception names it directly, not as a one-element tuple.

**Why:** `except (ValueError,):` means the same as `except ValueError:`, with extra noise.

```codelaw
native: py/redundant-tuple-in-exception-handler
```

## Examples

### Python

#### Don't

```py
try:
    run()
except (ValueError,):
    retry()
```

#### Do

```py
try:
    run()
except ValueError:
    retry()
```

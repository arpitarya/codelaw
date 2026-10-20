---
id: py/if-tuple
title: "`if` doesn't test a tuple"
languages: [python]
fix: manual
fix_reason: "the intended condition can't be inferred from the tuple"
stability: preview
ported_from: [ruff/F634]
since: 0.1.0
---

An `if` or `elif` condition isn't a non-empty tuple.

**Why:** A non-empty tuple is always true, so the branch always runs.

```codelaw
native: py/if-tuple
```

## How to fix

1. Write the condition you meant, usually the tuple's first element.

## Examples

### Python

#### Don't

```py
if (ready, "waiting"):
    start()
```

#### Do

```py
if ready:
    start()
```

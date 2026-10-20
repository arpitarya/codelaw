---
id: py/duplicate-try-block-exception
title: "An exception is caught by one handler only"
languages: [python]
fix: manual
fix_reason: "which handler's code is meant to run needs a person"
stability: preview
ported_from: [ruff/B025]
since: 0.1.0
---

Within one `try` statement, each exception class appears in only one `except` clause.

**Why:** Only the first matching handler runs, so the later one is dead code.

```codelaw
native: py/duplicate-try-block-exception
```

## How to fix

1. Merge the handlers' code into the first one, or remove the dead one.

## Examples

### Python

#### Don't

```py
try:
    run()
except ValueError:
    retry()
except ValueError:
    log()
```

#### Do

```py
try:
    run()
except ValueError:
    retry()
    log()
```

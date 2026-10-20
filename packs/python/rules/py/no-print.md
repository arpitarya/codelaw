---
id: py/no-print
title: "No `print()` in library code"
languages: [python]
files: ["src/**"]
fix: suggest
fix_reason: "the log level, and whether the output was meant for the user, need a person"
stability: preview
since: 0.1.0
---

Code under `src/` reports through `logging`, not `print()`.

**Why:** `print` writes to standard output whether or not the caller wants it; a logger can be filtered, redirected and silenced.

```codelaw
native: py/no-print
```

## Examples

### Python

#### Don't

```py path=src/app.py
import logging


def run():
    print("started")
```

#### Do

```py path=src/app.py
import logging


def run():
    logging.getLogger(__name__).info("%s", "started")
```

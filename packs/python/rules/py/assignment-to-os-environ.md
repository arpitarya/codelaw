---
id: py/assignment-to-os-environ
title: "Don't assign to `os.environ`"
languages: [python]
fix: manual
fix_reason: "whether to clear the environment first or only update it needs a person"
stability: preview
ported_from: [ruff/B003]
since: 0.1.0
---

Code changes the environment through `os.environ`'s items, never by assigning a new object to `os.environ`.

**Why:** Assigning replaces Python's mapping but not the process environment, so child processes don't see the change.

```codelaw
native: py/assignment-to-os-environ
```

## How to fix

1. Update `os.environ` item by item (`os.environ[key] = value`), clearing it first if that was meant.

## Examples

### Python

#### Don't

```py
import os

os.environ = {"PATH": "/bin"}
```

#### Do

```py
import os

os.environ.clear()
os.environ["PATH"] = "/bin"
```

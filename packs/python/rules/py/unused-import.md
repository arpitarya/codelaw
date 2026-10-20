---
id: py/unused-import
title: "Every imported name is used"
languages: [python]
fix: suggest
fix_reason: "an import can run code when it loads, and removing it drops that side effect"
stability: preview
ported_from: [ruff/F401]
since: 0.1.0
---

Each name an `import` binds is read somewhere in the module, listed in `__all__`, or re-exported with `import x as x`.

**Why:** An unused import is dead weight: it slows start-up, can hide a missing dependency, and makes readers look for a use that isn't there.

```codelaw
native: py/unused-import
```

## Examples

### Python

#### Don't

```py
import os
import sys

print(sys.argv)
```

#### Do

```py
import sys

print(sys.argv)
```

## Differences from the source

- An import counts as used when any read of its name exists, wherever it is; Ruff tracks each binding separately, so `import os` read and then imported again unused is reported by Ruff only.
- Ruff's special handling of `__init__.py` (suggesting `__all__` or a redundant alias instead of removal) is not reproduced; the fix is `suggest` everywhere.
- Imports inside a `try` body (the "is this installed?" pattern) are reported without an edit.

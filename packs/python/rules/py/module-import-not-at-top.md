---
id: py/module-import-not-at-top
title: "Module-level imports come first"
languages: [python]
fix: manual
fix_reason: "moving an import above other code changes the order the module runs in"
stability: preview
ported_from: [ruff/E402]
since: 0.1.0
---

Module-level imports come before other code. A docstring, `__future__` imports, dunder assignments (`__all__`, `__version__`), `if`/`try`/`with` blocks and changes to `sys.path` or `os.environ` may come before them.

**Why:** Imports scattered through a module hide its dependencies, and code that runs before them can change what they load.

```codelaw
native: py/module-import-not-at-top
```

## How to fix

1. Move the import up to the module's import section.
2. If it must run after the code above it, keep it and suppress the finding with a comment saying why.

## Examples

### Python

#### Don't

```py
import os

print(os.name)

import sys
```

#### Do

```py
import os
import sys

print(os.name)
```

---
id: py/no-star-import
title: "No `from module import *`"
languages: [python]
fix: manual
fix_reason: "which names the module needs can only be found by reading its uses"
stability: preview
ported_from: [ruff/F403]
since: 0.1.0
---

Modules import the names they use explicitly instead of `from module import *`.

**Why:** A star import hides where names come from, can silently replace local names, and stops a linter from finding undefined names.

```codelaw
native: py/no-star-import
```

## How to fix

1. List the names the module uses from the star-imported module.
2. Replace the star with those names.

## Examples

### Python

#### Don't

```py
from os.path import *

print(join("a", "b"))
```

#### Do

```py
from os.path import join

print(join("a", "b"))
```

## Differences from the source

Relative star imports (`from .sub import *`) are not reported: that is how a package re-exports its submodules' API, and Ruff leaves them alone in `__init__.py`.

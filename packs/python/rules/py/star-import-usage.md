---
id: py/star-import-usage
title: "No names that may only come from a star import"
languages: [python]
fix: manual
fix_reason: "the module the name comes from has to be confirmed by a person"
stability: preview
ported_from: [ruff/F405]
since: 0.1.0
---

In a module with `from module import *`, every name the module reads is defined locally or imported by name.

**Why:** With a star import in play, a misspelled name and a name the star import provides look the same; neither can be checked.

```codelaw
native: py/star-import-usage
```

## How to fix

1. Find which star-imported module provides the name.
2. Import it by name from that module.

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

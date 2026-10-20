---
id: py/late-future-import
title: "`from __future__` imports come first"
languages: [python]
fix: manual
fix_reason: "moving statements above it can change the order the module runs in"
stability: preview
ported_from: [ruff/F404]
since: 0.1.0
---

`from __future__ import …` statements come before any other statement except the module docstring.

**Why:** Python rejects a late `__future__` import with a `SyntaxError`.

```codelaw
native: py/late-future-import
```

## How to fix

1. Move the `from __future__` import to the top, after the docstring.

## Examples

### Python

#### Don't

```py
import os
from __future__ import annotations
```

#### Do

```py
from __future__ import annotations
import os
```

---
id: py/multiple-imports-on-one-line
title: "One module per `import` statement"
languages: [python]
fix: auto
stability: preview
ported_from: [ruff/E401]
since: 0.1.0
---

Each `import` statement imports one module; `import os, sys` is split into one line per module.

**Why:** One import per line keeps diffs and merges clean and lets import sorting work line by line.

```codelaw
native: py/multiple-imports-on-one-line
```

## Examples

### Python

#### Don't

```py
import os, sys
```

#### Do

```py
import os
import sys
```

## Differences from the source

`from module import a, b` is not affected; it imports from one module.

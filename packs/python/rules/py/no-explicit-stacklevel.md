---
id: py/no-explicit-stacklevel
title: "`warnings.warn` sets `stacklevel`"
languages: [python]
fix: suggest
fix_reason: "`stacklevel=2` is right for a warning about the caller; deeper helpers need a higher value"
stability: preview
ported_from: [ruff/B028]
since: 0.1.0
---

`warnings.warn()` is called with `stacklevel=` (or `skip_file_prefixes=`).

**Why:** The default `stacklevel=1` points the warning at the line inside your library, not at the caller who needs to change.

```codelaw
native: py/no-explicit-stacklevel
```

## Examples

### Python

#### Don't

```py
import warnings

warnings.warn("old API")
```

#### Do

```py
import warnings

warnings.warn("old API", stacklevel=2)
```

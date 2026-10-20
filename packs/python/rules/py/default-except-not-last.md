---
id: py/default-except-not-last
title: "A bare `except:` is the last handler"
languages: [python]
fix: manual
fix_reason: "reordering handlers changes which one runs"
stability: preview
ported_from: [ruff/F707]
since: 0.1.0
---

In a `try` statement, a bare `except:` comes after every other handler.

**Why:** Handlers after a bare `except:` can never run, and Python rejects it as a `SyntaxError`.

```codelaw
native: py/default-except-not-last
```

## How to fix

1. Move the bare `except:` to the end.

## Examples

### Python

#### Don't

```py
try:
    run()
except:
    log()
except ValueError:
    retry()
```

#### Do

```py
try:
    run()
except ValueError:
    retry()
except:
    log()
```

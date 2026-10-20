---
id: py/no-bare-except
title: "No bare `except:`"
languages: [python]
fix: suggest
fix_reason: "`except Exception:` no longer catches `KeyboardInterrupt` and `SystemExit`"
stability: preview
ported_from: [ruff/E722]
since: 0.1.0
---

Every `except` clause names the exceptions it handles.

**Why:** A bare `except:` also catches `KeyboardInterrupt` and `SystemExit`, so Ctrl-C and `sys.exit()` stop working inside it.

```codelaw
native: py/no-bare-except
```

## Examples

### Python

#### Don't

```py
try:
    run()
except:
    log()
```

#### Do

```py
try:
    run()
except Exception:
    log()
```

## Differences from the source

As in Ruff, a handler whose body re-raises with a bare `raise` is allowed: it doesn't swallow anything.

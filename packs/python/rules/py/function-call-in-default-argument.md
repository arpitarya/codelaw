---
id: py/function-call-in-default-argument
title: "No function calls in default arguments"
languages: [python]
fix: manual
fix_reason: "whether the value should be computed per call or once at import needs a person"
stability: preview
options:
  extend_immutable_calls: { type: "string[]", default: [], description: "more calls whose results are immutable, by qualified name" }
ported_from: [ruff/B008]
since: 0.1.0
---

A parameter's default isn't computed by a function call, except calls known to return immutable values (`tuple()`, `frozenset()`, `int()`, `re.compile()`, `datetime.timedelta()`, …).

**Why:** The call runs once, when the function is defined, so every call shares the same result — a stale timestamp, or one shared object.

```codelaw
native: py/function-call-in-default-argument
```

## How to fix

1. Default to `None` and compute the value in the body, or
2. compute it once into a module-level constant and use that as the default.

## Examples

### Python

#### Don't

```py
import time


def log(message, when=time.time()):
    print(when, message)
```

#### Do

```py
import time


def log(message, when=None):
    if when is None:
        when = time.time()
    print(when, message)
```

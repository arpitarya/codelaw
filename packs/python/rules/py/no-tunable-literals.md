---
id: py/no-tunable-literals
title: "No hard-coded timeouts, limits or thresholds"
languages: [python]
fix: manual
fix_reason: "which configuration key holds the value is the author's choice"
stability: preview
since: 0.1.0
---

A number passed as, defaulted to or assigned to a setting whose name says it is tunable (`timeout`, `retries`, `max_*`, `limit`, `threshold`, `interval`, `delay`, `ttl`, `backoff`, `deadline`, …) comes from configuration, not a literal in the code. `0` and `1` are allowed, and so are module-level `UPPER_CASE` constants, the one place such a value may be written down.

**Why:** Tunable values buried in code can't be changed per environment without a release, and the same number drifts apart in several places.

```codelaw
match:
  any:
    - all:
        - kind: keyword_argument
        - has: { field: name, regex: '(?i)(^|_)(timeouts?|retries|retry|attempts|limit|threshold|interval|delay|ttl|backoff|deadline|max|min|maximum|minimum)(_|$)' }
        - has: { field: value, any: [{ kind: integer }, { kind: float }], not: { regex: '^(0|1|0\.0|1\.0)$' } }
    - all:
        - any: [{ kind: default_parameter }, { kind: typed_default_parameter }]
        - has: { field: name, regex: '(?i)(^|_)(timeouts?|retries|retry|attempts|limit|threshold|interval|delay|ttl|backoff|deadline|max|min|maximum|minimum)(_|$)' }
        - has: { field: value, any: [{ kind: integer }, { kind: float }], not: { regex: '^(0|1|0\.0|1\.0)$' } }
    - all:
        - kind: assignment
        - has: { field: left, kind: identifier, regex: '(?i)(^|_)(timeouts?|retries|retry|attempts|limit|threshold|interval|delay|ttl|backoff|deadline|max|min|maximum|minimum)(_|$)', not: { regex: '^[A-Z0-9_]+$' } }
        - has: { field: right, any: [{ kind: integer }, { kind: float }], not: { regex: '^(0|1|0\.0|1\.0)$' } }
message: "This tunable value is hard-coded; read it from configuration"
```

## How to fix

1. Pick or add a configuration key for the value.
2. Read the value from configuration where it is used.
3. For a fixed protocol constant, name it as a module-level `UPPER_CASE` constant instead.

## Examples

### Python

#### Don't

```py
def fetch(url):
    return get(url, timeout=30)
```

#### Do

```py
from config import settings


def fetch(url):
    return get(url, timeout=settings.http_timeout)
```

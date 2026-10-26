---
id: sec-py/requests-timeout
title: 'HTTP requests have a timeout'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: suggest
fix_reason: "the right timeout depends on the service; the suggested value is a starting point"
stability: preview
ported_from: [bandit/B113, ruff/S113]
security: { cwe: [400], owasp: ["A06:2025"], severity: medium }
since: 0.2.0
---

`requests` and `httpx` calls pass `timeout=`.

**Why:** `requests` waits forever by default, so one stuck server holds a worker for good and a flood of slow responses exhausts them (CWE-400).

```codelaw
match:
  all:
    - { pattern: '$F($$$A)' }
    - { kind: call }
    - not:
        has:
          field: arguments
          has: { kind: keyword_argument, has: { field: name, regex: '^timeout$' } }
constraints: { F: { regex: '^requests\.(get|post|put|patch|delete|head|options|request)$' } }
message: 'This request has no timeout; pass `timeout=`'
fix: '$F($$$A, timeout=10)'
```

## How to fix

1. Pass `timeout=` in seconds, or a `(connect, read)` tuple.
2. Keep the value in one setting if several calls share it.

## Examples

### Python

#### Don't

```py
import requests

resp = requests.get(url)
```

#### Do

```py
import requests

resp = requests.get(url, timeout=10)
```

#### Fixed

```py
import requests

resp = requests.get(url, timeout=10)
```

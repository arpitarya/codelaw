---
id: sec-py/no-bind-all-interfaces
title: 'A server does not bind to every interface in code'
languages: [python]
severity: warn
fix: manual
fix_reason: "the address belongs in configuration, with a loopback default"
stability: preview
ported_from: [bandit/B104, ruff/S104]
security: { cwe: [605], owasp: ["A02:2025"], severity: low }
since: 0.2.0
---

A socket or server is not bound to `0.0.0.0` in the source.

**Why:** The service is then reachable from every network the machine is on, including ones that were not meant to reach it (CWE-605).

```codelaw
match:
  any:
    - all:
        - { kind: call }
        - { has: { field: function, regex: '\.(bind|connect_ex)$' } }
        - has:
            field: arguments
            has:
              nthChild: 1
              kind: tuple
              has: { nthChild: 1, kind: string, regex: '^[''"](0\.0\.0\.0|::)[''"]$' }
    - all:
        - { kind: call }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^host$' } }
                - { has: { field: value, kind: string, regex: '^[''"](0\.0\.0\.0|::)[''"]$' } }
    - all:
        - { kind: call }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^bind$' } }
                - { has: { field: value, kind: string, regex: '^[''"](0\.0\.0\.0|\[::\]):[0-9]+[''"]$' } }
message: 'This binds to every network interface; read the address from configuration'
```

## How to fix

1. Bind to `127.0.0.1` by default and take the address from an environment variable.
2. A container that must listen on every interface sets it in its deployment settings, not in the code.

## Examples

### Python

#### Don't

```py
import uvicorn

uvicorn.run(app, host="0.0.0.0")
```

#### Do

```py
import os
import uvicorn

uvicorn.run(app, host=os.environ.get("HOST", "127.0.0.1"))
```

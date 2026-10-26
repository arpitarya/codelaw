---
id: sec-py/no-insecure-random
title: "`secrets` for tokens, not `random`"
languages: [python]
fix: suggest
fix_reason: "`secrets` has different functions (`token_hex`, `choice`, `randbelow`), so the call has to be chosen, not renamed"
stability: preview
ported_from: [bandit/B311, ruff/S311]
security: { cwe: [338], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

A password, token, key, nonce or code is not made with the `random` module.

**Why:** `random` is a predictable generator, so a few outputs reveal the rest (CWE-338).

```codelaw
match:
  all:
    - { kind: assignment }
    - has:
        field: left
        regex: '(?i)(pass(word|wd)?|secret|token|api_?key|auth|otp|nonce|salt|session_?id|csrf|reset_?code|verification_?code)'
    - has:
        field: right
        kind: call
        pattern: $R
        has:
          field: function
          regex: '^(random\.)?(random|randint|randrange|choice|choices|getrandbits|sample|randbytes|uniform)$'
message: 'This secret comes from `random`; use `secrets`'
fix:
  - { replace: { target: $R, with: 'secrets.token_hex(16)' } }
```

## How to fix

1. Use `secrets.token_urlsafe(32)` or `secrets.token_hex(16)` for a token.
2. Use `secrets.choice` for a character picked from an alphabet, `secrets.randbelow(n)` for a number.

## Examples

### Python

#### Don't

```py
import random

reset_token = random.getrandbits(64)
```

#### Do

```py
import secrets

reset_token = secrets.token_hex(16)
```

#### Fixed

```py
import random

reset_token = secrets.token_hex(16)
```

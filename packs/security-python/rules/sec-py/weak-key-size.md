---
id: sec-py/weak-key-size
title: 'RSA and DSA keys are at least 2048 bits'
languages: [python]
fix: suggest
fix_reason: "a longer key makes existing signatures and certificates incompatible and slows each operation"
stability: preview
ported_from: [bandit/B505, ruff/S505]
security: { cwe: [326], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

An RSA or DSA key is not generated shorter than 2048 bits, and an elliptic curve is at least 224 bits.

**Why:** Short keys can be factored or searched with hardware that is within reach (CWE-326).

```codelaw
match:
  any:
    - kind: call
      all:
        - { has: { field: function, regex: '(^|\.)generate_private_key$|^(RSA|DSA)\.generate$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^key_size$' } }
                - has:
                    field: value
                    kind: integer
                    regex: '^([0-9]{1,3}|1[0-9]{3}|20[0-3][0-9]|204[0-7])$'
                    pattern: $KS
    - kind: call
      all:
        - { has: { field: function, regex: '^(RSA|DSA)\.generate$' } }
        - has:
            field: arguments
            has:
              nthChild: 1
              kind: integer
              regex: '^([0-9]{1,3}|1[0-9]{3}|20[0-3][0-9]|204[0-7])$'
              pattern: $KS
    - { kind: attribute, regex: '\.SECP(1[0-9]{2}|2[01][0-9]|22[0-3])[RK]1$' }
message: 'This key is too short; use 2048 bits or more'
fix:
  - { replace: { target: $KS, with: '2048' } }
```

## How to fix

1. Use at least 2048 bits for RSA and DSA, 3072 for keys that must last past 2030.
2. Use `ec.SECP256R1()` or Ed25519 for new keys.
3. Re-issue certificates signed with the old key.

## Examples

### Python

#### Don't

```py
from cryptography.hazmat.primitives.asymmetric import rsa

key = rsa.generate_private_key(public_exponent=65537, key_size=1024)
```

#### Do

```py
from cryptography.hazmat.primitives.asymmetric import rsa

key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
```

#### Fixed

```py
from cryptography.hazmat.primitives.asymmetric import rsa

key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
```

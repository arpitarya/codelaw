---
id: sec-py/password-hash-effort
title: 'Passwords are hashed with a slow function'
languages: [python]
fix: manual
fix_reason: "moving to Argon2 or bcrypt means rehashing at the next login and changing the stored format"
stability: preview
security: { cwe: [916], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

A password is not hashed with a plain `hashlib` digest, `pbkdf2_hmac` runs at least 600 000 iterations, and `bcrypt` uses at least 10 rounds.

**Why:** A fast hash lets an attacker who has the table test billions of guesses per second (CWE-916).

```codelaw
match:
  any:
    - kind: call
      all:
        - { has: { field: function, regex: '^hashlib\.(md5|sha1|sha224|sha256|sha384|sha512)$' } }
        - { has: { field: arguments, has: { nthChild: 1, regex: '(?i)pass(word|wd)?' } } }
    - kind: call
      all:
        - { has: { field: function, regex: '^(hashlib\.)?pbkdf2_hmac$' } }
        - any:
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all:
                    - { has: { field: name, regex: '^iterations$' } }
                    - { has: { field: value, kind: integer, regex: '^([0-9]{1,5}|[1-5][0-9]{5})$' } }
            - has:
                field: arguments
                has: { nthChild: 4, kind: integer, regex: '^([0-9]{1,5}|[1-5][0-9]{5})$' }
    - kind: call
      all:
        - { has: { field: function, regex: '^(bcrypt\.)?gensalt$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^rounds$' } }
                - { has: { field: value, kind: integer, regex: '^[0-9]$' } }
message: 'This password hash is too fast; use Argon2 or bcrypt, or more iterations'
```

## How to fix

1. Use `argon2-cffi` (`PasswordHasher().hash(password)`) or `bcrypt.hashpw(password, bcrypt.gensalt(rounds=12))`.
2. If you must use PBKDF2, use SHA-256 with at least 600 000 iterations (OWASP).
3. Rehash old entries when a user logs in.

## Examples

### Python

#### Don't

```py
import hashlib

stored = hashlib.sha256(password.encode()).hexdigest()
```

#### Do

```py
from argon2 import PasswordHasher

stored = PasswordHasher().hash(password)
```

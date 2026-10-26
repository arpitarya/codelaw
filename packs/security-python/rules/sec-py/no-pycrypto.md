---
id: sec-py/no-pycrypto
title: 'No `pycrypto`'
languages: [python]
fix: manual
fix_reason: "the replacement package changes the import path and some call signatures"
stability: preview
ported_from: [bandit/B413, ruff/S413]
security: { cwe: [1104], owasp: ["A03:2025"], severity: medium }
since: 0.2.0
---

`Crypto` is not imported from the unmaintained `pycrypto` package.

**Why:** `pycrypto` has had no release since 2013 and carries known flaws, including a weak random generator and a buffer overflow (CWE-1104).

```codelaw
match:
  any:
    - all: [{ kind: import_statement }, { has: { field: name, regex: '^Crypto(\..*)?$' } }]
    - all:
        - { kind: import_from_statement }
        - { has: { field: module_name, regex: '^Crypto(\..*)?$' } }
message: '`Crypto` is the unmaintained `pycrypto`; use `cryptography` or `pycryptodomex`'
```

## How to fix

1. Prefer `cryptography` (`pip install cryptography`).
2. `pycryptodomex` is a maintained drop-in with the package name `Cryptodome`: change `Crypto` to `Cryptodome`.
3. `pycryptodome` (not `x`) also installs a package named `Crypto`; if that is what you use, switch to `Cryptodome` so this rule can tell them apart.

## Examples

### Python

#### Don't

```py
from Crypto.Cipher import AES
```

#### Do

```py
from Cryptodome.Cipher import AES
```

---
id: sec-py/no-hardcoded-crypto-key
title: 'No keys or IVs in the source'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "where the key comes from (a key store, an environment variable) is a deployment decision"
stability: preview
security: { cwe: [321, 329], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

A cipher key, HMAC key, IV or nonce is not a literal.

**Why:** A key in the source is known to everyone who can read the source, and a fixed IV or nonce breaks the cipher's guarantees (CWE-321, CWE-329).

```codelaw
match:
  any:
    - kind: call
      all:
        - has:
            field: function
            regex: '^((Crypto|Cryptodome)\.Cipher\.)?(AES|DES|DES3|ChaCha20|Blowfish|ARC4|Salsa20)\.new$|^(Fernet|hmac\.new|ChaCha20Poly1305|AESGCM|AESCCM)$'
        - has:
            field: arguments
            has: { nthChild: 1, kind: string, not: { has: { kind: interpolation } } }
    - kind: call
      all:
        - { has: { field: function, regex: '^(modes\.)?(CBC|CTR|GCM|CFB|OFB|CCM)$' } }
        - has:
            field: arguments
            has: { nthChild: 1, kind: string, not: { has: { kind: interpolation } } }
    - all:
        - { kind: call }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^iv$' } }
                - { has: { field: value, kind: string, not: { has: { kind: interpolation } } } }
    - all:
        - { kind: call }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^nonce$' } }
                - { has: { field: value, kind: string, not: { has: { kind: interpolation } } } }
    - all:
        - { kind: call }
        - has:
            field: function
            regex: '(?i)(cipher|crypt|aes|fernet|hmac|jwt|sign|cmac|pbkdf|scrypt)'
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^key$' } }
                - { has: { field: value, kind: string, not: { has: { kind: interpolation } } } }
message: 'A key, IV or nonce is a literal; load the key from a key store and make the IV random'
```

## How to fix

1. Generate keys with `os.urandom` or the library (`Fernet.generate_key()`) once, and store them in a secret manager or an environment variable.
2. Make a fresh random IV or nonce for every message (`os.urandom(12)`) and store it beside the ciphertext.

## Examples

### Python

#### Don't

```py
from cryptography.fernet import Fernet

f = Fernet(b"ZmFrZWtleWZha2VrZXlmYWtla2V5ZmFrZWtleTA9")
```

#### Do

```py
import os
from cryptography.fernet import Fernet

f = Fernet(os.environ["FERNET_KEY"])
```

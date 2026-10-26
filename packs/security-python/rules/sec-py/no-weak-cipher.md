---
id: sec-py/no-weak-cipher
title: 'No DES, RC4, Blowfish or ECB'
languages: [python]
fix: manual
fix_reason: "the new cipher changes key size, nonce handling and the stored format"
stability: preview
ported_from: [bandit/B304, bandit/B305, ruff/S304, ruff/S305]
security: { cwe: [327], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

A weak cipher (DES, 3DES, ARC2, ARC4, Blowfish, CAST5, IDEA) or ECB mode is not used.

**Why:** The ciphers have small blocks or known attacks, and ECB encrypts equal blocks to equal output, so the data's structure shows through (CWE-327).

```codelaw
match:
  any:
    - kind: call
      all:
        - has:
            field: function
            regex: '^((Crypto|Cryptodome)\.Cipher\.)?(DES|DES3|ARC2|ARC4|Blowfish|CAST|XOR)\.new$'
    - kind: call
      all:
        - has:
            field: function
            regex: '^(algorithms\.)(TripleDES|ARC4|ARC2|Blowfish|CAST5|IDEA|SEED)$'
    - kind: call
      all: [{ has: { field: function, regex: '^(modes\.)?ECB$' } }]
    - { kind: attribute, regex: '\.MODE_ECB$' }
message: 'This cipher or mode is weak; use AES-GCM or ChaCha20-Poly1305'
```

## How to fix

1. Use `cryptography`'s `AESGCM` or `ChaCha20Poly1305`: they encrypt and authenticate.
2. A new random nonce for every message; never reuse one with the same key.
3. Re-encrypt stored data under the new scheme.

## Examples

### Python

#### Don't

```py
from Crypto.Cipher import AES

cipher = AES.new(key, AES.MODE_ECB)
```

#### Do

```py
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

cipher = AESGCM(key)
```

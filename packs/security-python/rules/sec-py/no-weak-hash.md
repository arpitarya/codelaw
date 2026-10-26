---
id: sec-py/no-weak-hash
title: 'No MD5 or SHA-1 for security'
languages: [python]
fix: suggest
fix_reason: "whether the digest is used for security (use SHA-256) or only as a checksum (say so) is the author's call"
stability: preview
ported_from: [bandit/B303, bandit/B324, ruff/S324]
security: { cwe: [327], owasp: ["A04:2025"], severity: medium }
since: 0.2.0
---

`hashlib.md5` and `hashlib.sha1` pass `usedforsecurity=False`, or are replaced by SHA-256.

**Why:** Both can be forged by collision, so they prove nothing about who wrote the data (CWE-327).

```codelaw
match:
  all:
    - { pattern: 'hashlib.$H($$$A)' }
    - not:
        has:
          field: arguments
          has: { kind: keyword_argument, has: { field: name, regex: '^usedforsecurity$' } }
constraints: { H: { regex: '^(md5|sha1)$' } }
message: '`hashlib.$H` is not collision resistant; use SHA-256, or pass `usedforsecurity=False` for a checksum'
fix: 'hashlib.sha256($$$A)'
```

## How to fix

1. If the digest guards integrity or identity, use `hashlib.sha256` or `hashlib.blake2b`.
2. If it is only a cache key or checksum, pass `usedforsecurity=False` and say so in a comment.
3. For passwords see `sec-py/password-hash-effort`.

## Examples

### Python

#### Don't

```py
import hashlib

digest = hashlib.md5(data).hexdigest()
```

#### Do

```py
import hashlib

digest = hashlib.sha256(data).hexdigest()
```

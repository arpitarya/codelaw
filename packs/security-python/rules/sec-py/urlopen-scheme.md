---
id: sec-py/urlopen-scheme
title: "`urlopen` of a value that may be `file://`"
languages: [python]
fix: manual
fix_reason: "the allowed schemes and hosts are policy"
stability: preview
ported_from: [bandit/B310, ruff/S310]
security: { cwe: [22, 918], owasp: ["A01:2025"], severity: medium }
since: 0.2.0
---

`urllib.request.urlopen` and `urlretrieve` are not given a value that is not an `http(s)` literal.

**Why:** `urlopen` also opens `file://` and `ftp://` URLs, so a value that is not checked can read local files (CWE-22, CWE-918).

```codelaw
match:
  any:
    - { pattern: 'urlopen($X)' }
    - { pattern: 'urlopen($X, $$$R)' }
    - { pattern: 'urllib.request.urlopen($X)' }
    - { pattern: 'urllib.request.urlopen($X, $$$R)' }
    - { pattern: 'urllib.request.urlretrieve($X)' }
    - { pattern: 'urllib.request.urlretrieve($X, $$$R)' }
    - { pattern: 'urlretrieve($X, $$$R)' }
constraints: { X: { not: { kind: string, regex: '^[rbf]?[''\"]https?://' } } }
message: '`$X` may be a `file://` URL; check the scheme first'
```

## How to fix

1. Parse the URL and check `scheme in ("http", "https")` before opening it.
2. Or use `requests`, which supports only HTTP(S), with a `timeout`.

## Examples

### Python

#### Don't

```py
from urllib.request import urlopen

data = urlopen(url).read()
```

#### Do

```py
from urllib.request import urlopen

data = urlopen("https://example.com/data").read()
```

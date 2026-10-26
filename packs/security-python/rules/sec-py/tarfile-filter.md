---
id: sec-py/tarfile-filter
title: 'Archive extraction uses `filter="data"`'
languages: [python]
fix: suggest
fix_reason: "`filter=\"data\"` refuses members that carry device files or special permissions some archives need"
stability: preview
ported_from: [bandit/B202, ruff/S202]
security: { cwe: [22], owasp: ["A01:2025"], severity: high }
since: 0.2.0
---

`TarFile.extractall` and `shutil.unpack_archive` pass `filter="data"`.

**Why:** A member named `../../etc/cron.d/x`, an absolute path or a symlink writes outside the target folder (CWE-22).

```codelaw
match:
  any:
    - all:
        - { pattern: '$T.extractall($$$A)' }
        - not:
            has:
              field: arguments
              has: { kind: keyword_argument, has: { field: name, regex: '^filter$' } }
    - all:
        - { pattern: 'shutil.unpack_archive($$$B)' }
        - not:
            has:
              field: arguments
              has: { kind: keyword_argument, has: { field: name, regex: '^filter$' } }
message: '`extractall` follows member paths out of the folder; pass `filter="data"`'
fix: '$T.extractall($$$A, filter="data")'
```

## How to fix

1. Pass `filter="data"` (Python 3.12, and the 3.8–3.11 security releases).
2. Pass it to `shutil.unpack_archive(…, filter="data")` too.
3. Extract into a fresh folder, and never run as root.

## Examples

### Python

#### Don't

```py
import tarfile

with tarfile.open(path) as tar:
    tar.extractall(dest)
```

#### Do

```py
import tarfile

with tarfile.open(path) as tar:
    tar.extractall(dest, filter="data")
```

#### Fixed

```py
import tarfile

with tarfile.open(path) as tar:
    tar.extractall(dest, filter="data")
```

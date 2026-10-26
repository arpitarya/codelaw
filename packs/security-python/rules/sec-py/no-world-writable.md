---
id: sec-py/no-world-writable
title: 'No world-writable files'
languages: [python]
fix: suggest
fix_reason: "the right mode depends on who has to write the file"
stability: preview
ported_from: [bandit/B103, ruff/S103]
security: { cwe: [732], owasp: ["A01:2025"], severity: medium }
since: 0.2.0
---

`os.chmod` and `os.umask` do not give write access to everyone.

**Why:** Any local user can then change the file, which for a script, config or socket means running code as its owner (CWE-732).

```codelaw
match: { all: [{ pattern: 'os.chmod($P, $M)' }] }
constraints:
  M:
    regex: '^(0o?[0-7]*[2367]|0o?[0-7]*7[0-7]?[2367]|stat\.S_IWOTH|stat\.S_IRWXO|0o?[0-7]{0,2}77[0-7]|0o?[0-7]*[2367])$'
message: '`$M` makes the file writable by everyone'
fix: 'os.chmod($P, 0o600)'
```

## How to fix

1. Use `0o600` for a private file, `0o644` for one others read, `0o755` for a program.
2. Never set the others' write bit.

## Examples

### Python

#### Don't

```py
import os

os.chmod(path, 0o777)
```

#### Do

```py
import os

os.chmod(path, 0o600)
```

#### Fixed

```py
import os

os.chmod(path, 0o600)
```

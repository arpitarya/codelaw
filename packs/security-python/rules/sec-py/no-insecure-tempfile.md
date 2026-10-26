---
id: sec-py/no-insecure-tempfile
title: "`mkstemp` instead of `mktemp`"
languages: [python]
fix: suggest
fix_reason: "`mkstemp` returns an open file descriptor and a path, not just a name, so the surrounding code changes"
stability: preview
ported_from: [bandit/B306, bandit/B108, bandit/B325, ruff/S306, ruff/S108]
security: { cwe: [377], owasp: ["A01:2025"], severity: medium }
since: 0.2.0
---

Temporary files are made with `tempfile.mkstemp`, `NamedTemporaryFile` or `TemporaryDirectory`, not `mktemp`, and not at a fixed path under `/tmp`.

**Why:** `mktemp` returns a name without creating the file, so another user can create it first and the program then writes into a file they control (CWE-377).

```codelaw
match:
  any:
    - { pattern: 'tempfile.mktemp($$$A)' }
    - { pattern: 'os.tempnam($$$A)' }
    - { pattern: 'os.tmpnam()' }
    - kind: string
      regex: '^[''"]/(var/)?tmp/[^''"{}]*[''"]$'
      inside: { kind: call, stopBy: end }
message: 'A predictable temporary file name; use `tempfile.mkstemp` or `NamedTemporaryFile`'
fix: 'tempfile.mkstemp($$$A)[1]'
```

## How to fix

1. Use `with tempfile.NamedTemporaryFile(delete=False) as f:` and write through `f`.
2. Use `tempfile.mkdtemp()` or `TemporaryDirectory()` for a directory.
3. Never build a path under `/tmp` by hand.

## Examples

### Python

#### Don't

```py
import tempfile

path = tempfile.mktemp()
```

#### Do

```py
import tempfile

path = tempfile.mkstemp()[1]
```

#### Fixed

```py
import tempfile

path = tempfile.mkstemp()[1]
```

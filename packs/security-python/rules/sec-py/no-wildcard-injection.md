---
id: sec-py/no-wildcard-injection
title: 'A shell wildcard after `tar`, `chown`, `chmod` or `rsync`'
languages: [python]
fix: manual
fix_reason: "the files have to be listed (`glob`) and passed after `--`, which changes the call"
stability: preview
ported_from: [bandit/B609, ruff/S609]
security: { cwe: [155], owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

A command that runs `tar`, `chown`, `chmod` or `rsync` on `*` is not started through a shell.

**Why:** A file named `--checkpoint-action=exec=sh x.sh` in the folder is expanded by the shell into an option, so whoever can create files there runs commands (CWE-155).

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: '^(os\.system|os\.popen|subprocess\.(run|call|check_call|check_output|Popen))$'
    - has:
        field: arguments
        has: { nthChild: 1, kind: string, regex: '\b(tar|chown|chmod|rsync)\b[^''\"]*\*' }
message: '`*` is expanded by the shell, so a file named like an option changes the command'
```

## How to fix

1. List the files in Python: `files = sorted(glob.glob("*.txt"))`.
2. Pass them as a list after `--`: `subprocess.run(["tar", "cf", out, "--", *files])`.

## Examples

### Python

#### Don't

```py
import os

os.system("tar cf backup.tar *")
```

#### Do

```py
import subprocess

subprocess.run(["tar", "cf", "backup.tar", "--", *files], check=True)
```

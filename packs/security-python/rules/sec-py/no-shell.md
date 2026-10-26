---
id: sec-py/no-shell
title: 'A shell command built from data'
languages: [python]
fix: suggest
fix_reason: "dropping `shell=True` only works when the command is already a list of arguments; a string command must be split"
stability: preview
ported_from: [bandit/B602, bandit/B603, bandit/B604, bandit/B605, bandit/B609]
security: { cwe: [78], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A subprocess is not started with `shell=True`, and `os.system` / `os.popen` are not given a value that is not a literal.

**Why:** The command line goes through the shell, so a quote, `;` or `$(…)` in any part of it runs another command (CWE-78).

```codelaw
match:
  any:
    - kind: call
      all:
        - has:
            field: function
            regex: '^subprocess\.(run|call|check_call|check_output|Popen|getoutput|getstatusoutput)$'
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all: [{ has: { field: name, regex: '^shell$' } }, { has: { field: value, kind: 'true' } }]
              pattern: $KW
    - { pattern: 'os.system($X)' }
    - { pattern: 'os.popen($X)' }
constraints: { X: { not: { kind: string, not: { has: { kind: interpolation } } } } }
message: 'This command runs through a shell; pass a list of arguments and leave `shell` off'
fix:
  - { remove: $KW }
```

## How to fix

1. Pass the command as a list: `subprocess.run(["ls", "-l", path])`.
2. Remove `shell=True`.
3. Replace `os.system(cmd)` with `subprocess.run([...], check=True)`; use `shlex.split` only for a trusted fixed string.

## Examples

### Python

#### Don't

```py
import subprocess

subprocess.run(["ls", path], shell=True)
```

#### Do

```py
import subprocess

subprocess.run(["ls", path])
```

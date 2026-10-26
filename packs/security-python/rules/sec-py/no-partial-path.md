---
id: sec-py/no-partial-path
title: 'A program is started by its full path'
languages: [python]
severity: warn
fix: manual
fix_reason: "the absolute path differs by system, so it has to come from configuration or `shutil.which` at start-up"
stability: preview
ported_from: [bandit/B607, ruff/S607]
security: { cwe: [426], owasp: ["A08:2025"], severity: low }
since: 0.2.0
---

A subprocess is not started with a bare program name such as `"git"`.

**Why:** The shell looks the name up in `PATH`, so a program of the same name in an earlier folder runs instead (CWE-426).

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: '^(os\.system|os\.popen|subprocess\.(run|call|check_call|check_output|Popen))$'
    - any:
        - has:
            field: arguments
            has: { nthChild: 1, kind: string, regex: '^[rb]?[''\"][^/''\"\s]+(\s[^''\"]*)?[''\"]$' }
        - has:
            field: arguments
            has:
              nthChild: 1
              kind: list
              has: { nthChild: 1, kind: string, regex: '^[rb]?[''\"][^/''\"\s]+(\s[^''\"]*)?[''\"]$' }
message: 'This starts a program found through `PATH`; use its full path'
```

## How to fix

1. Resolve it once at start-up: `GIT = shutil.which("git")`, and fail if it is `None`.
2. Or put the absolute path in configuration.

## Examples

### Python

#### Don't

```py
import subprocess

subprocess.run(["git", "status"])
```

#### Do

```py
import subprocess

subprocess.run(["/usr/bin/git", "status"])
```

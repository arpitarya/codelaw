---
id: sec-py/no-hardcoded-secret
title: 'No secrets in the source'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "where the secret lives instead (environment, secret manager) is a decision, and the leaked one must be rotated"
stability: preview
ported_from: [bandit/B105, bandit/B106, bandit/B107, ruff/S105, ruff/S106, ruff/S107]
security: { cwe: [798], owasp: ["A07:2025"], severity: high }
since: 0.2.0
---

A password, key or token is not a string literal in the code, and a cloud access key is not pasted into it.

**Why:** The source goes to every clone, build log and backup, so the secret does too, and revoking it means a release (CWE-798).

```codelaw
match:
  any:
    - all:
        - { kind: assignment }
        - has:
            field: left
            regex: '(?i)^_*[a-z0-9_]*(pass(word|wd)?|secret|api_?key|private_?key|token|credentials?|auth_?key|access_?key)[a-z0-9_]*$'
            not:
              regex: '(?i)(_type|_url|_uri|_path|_file|_header|_name|_field|_class|_backend|_func|_label|_prefix)$'
        - has:
            field: right
            kind: string
            regex: '^[rb]?("[^"]{4,}"|''[^'']{4,}'')$'
            not:
              any:
                - { has: { kind: interpolation } }
                - { regex: '^[rb]?[''"][*x#.\-]+[''"]$' }
                - { regex: '^[rb]?[''"][A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+[''"]$' }
    - all:
        - { kind: keyword_argument }
        - has:
            field: name
            regex: '(?i)^_*[a-z0-9_]*(pass(word|wd)?|secret|api_?key|private_?key|token|credentials?|auth_?key|access_?key)[a-z0-9_]*$'
            not:
              regex: '(?i)(_type|_url|_uri|_path|_file|_header|_name|_field|_class|_backend|_func|_label|_prefix)$'
        - has:
            field: value
            kind: string
            regex: '^[rb]?("[^"]{4,}"|''[^'']{4,}'')$'
            not:
              any:
                - { has: { kind: interpolation } }
                - { regex: '^[rb]?[''"][*x#.\-]+[''"]$' }
                - { regex: '^[rb]?[''"][A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+[''"]$' }
    - { kind: string, regex: '^[''"](AKIA|ASIA)[0-9A-Z]{16}[''"]$' }
    - { kind: string, regex: '^[''"](ghp|gho|ghs|ghu)_[A-Za-z0-9]{36,}[''"]$' }
    - { kind: string, regex: '^[''"]xox[abprs]-[A-Za-z0-9-]{10,}[''"]$' }
    - { kind: string, regex: '^[''"]sk_live_[A-Za-z0-9]{16,}[''"]$' }
message: 'A secret is written in the source; read it from the environment or a secret manager'
```

## How to fix

1. Rotate the secret: it is in version control now.
2. Read it at start-up: `os.environ["DB_PASSWORD"]`, or from your secret manager.
3. Keep local values in an untracked `.env` file.

## Examples

### Python

#### Don't

```py
DB_PASSWORD = "hunter2-prod"
```

#### Do

```py
import os

DB_PASSWORD = os.environ["DB_PASSWORD"]
```

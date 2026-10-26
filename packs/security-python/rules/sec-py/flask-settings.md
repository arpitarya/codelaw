---
id: sec-py/flask-settings
title: 'Flask configuration is safe'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "the production values come from the deployment (environment variables, a secret manager)"
stability: preview
security: { cwe: [798, 352, 614], owasp: ["A02:2025"], severity: high }
since: 0.2.0
---

`SECRET_KEY` is not a literal, and `WTF_CSRF_ENABLED` and `SESSION_COOKIE_SECURE` are not switched off in the app configuration.

**Why:** A literal key is in the repository (CWE-798), turning off CSRF protection lets other sites submit forms as the user (CWE-352), and a cookie without `Secure` travels over plain HTTP (CWE-614).

```codelaw
match:
  any:
    - all:
        - { kind: assignment }
        - { has: { field: left, regex: 'config\[\s*[''"]SECRET_KEY[''"]\s*\]$' } }
        - has:
            field: right
            kind: string
            regex: '^[rb]?("[^"]{4,}"|''[^'']{4,}'')$'
            not: { has: { kind: interpolation } }
    - all:
        - { kind: assignment }
        - has:
            field: left
            regex: 'config\[\s*[''"](WTF_CSRF_ENABLED|WTF_CSRF_CHECK_DEFAULT|SESSION_COOKIE_SECURE|SESSION_COOKIE_HTTPONLY)[''"]\s*\]$'
        - { has: { field: right, kind: 'false' } }
    - kind: call
      all:
        - { has: { field: function, regex: '\.config\.(update|from_mapping)$' } }
        - any:
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all:
                    - { has: { field: name, regex: '^SECRET_KEY$' } }
                    - { has: { field: value, kind: string, not: { has: { kind: interpolation } } } }
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all:
                    - { has: { field: name, regex: '^WTF_CSRF_ENABLED$' } }
                    - { has: { field: value, kind: 'false' } }
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all:
                    - { has: { field: name, regex: '^SESSION_COOKIE_SECURE$' } }
                    - { has: { field: value, kind: 'false' } }
message: 'This Flask setting is unsafe outside development'
```

## How to fix

1. `app.config["SECRET_KEY"] = os.environ["FLASK_SECRET_KEY"]`, and rotate the key that was committed.
2. Leave `WTF_CSRF_ENABLED` on; send the token with `{{ csrf_token() }}`.
3. Set `SESSION_COOKIE_SECURE`, `SESSION_COOKIE_HTTPONLY` and `SESSION_COOKIE_SAMESITE` to safe values.

## Examples

### Python

#### Don't

```py
app.config["SECRET_KEY"] = "dev-secret-key"
```

#### Do

```py
import os

app.config["SECRET_KEY"] = os.environ["FLASK_SECRET_KEY"]
```

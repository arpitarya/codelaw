---
id: sec-py/django-settings
title: 'Django settings are safe'
languages: [python]
files: ["**/settings.py", "**/settings/**/*.py", "**/*_settings.py", "**/settings_*.py"]
exclude: ["**/*dev*.py", "**/*local*.py", "**/*test*.py"]
fix: manual
fix_reason: "the production value comes from the deployment (environment variables, a secret manager)"
stability: preview
ported_from: [ruff/S105]
security: { cwe: [489, 798, 614], owasp: ["A02:2025"], severity: high }
since: 0.2.0
---

In a settings module, `DEBUG`, `ALLOWED_HOSTS`, `SECRET_KEY` and the cookie flags are not left in their development values.

**Why:** Debug pages show settings and source (CWE-489); `ALLOWED_HOSTS = ["*"]` accepts forged Host headers; a literal `SECRET_KEY` is in the repository (CWE-798); cookies without `Secure` travel over plain HTTP (CWE-614).

```codelaw
match:
  all:
    - { kind: assignment }
    - any:
        - all: [{ has: { field: left, regex: '^DEBUG$' } }, { has: { field: right, kind: 'true' } }]
        - all:
            - { has: { field: left, regex: '^ALLOWED_HOSTS$' } }
            - { has: { field: right, regex: '^\[\s*[''"]\*[''"]\s*\]$' } }
        - all:
            - { has: { field: left, regex: '^SECRET_KEY$' } }
            - has:
                field: right
                kind: string
                regex: '^[rb]?("[^"]{4,}"|''[^'']{4,}'')$'
                not: { has: { kind: interpolation } }
        - all:
            - has:
                field: left
                regex: '^(SESSION_COOKIE_SECURE|CSRF_COOKIE_SECURE|SECURE_SSL_REDIRECT|SESSION_COOKIE_HTTPONLY)$'
            - { has: { field: right, kind: 'false' } }
message: 'This Django setting is unsafe outside development'
```

## How to fix

1. `DEBUG = os.environ.get("DJANGO_DEBUG") == "1"`.
2. `ALLOWED_HOSTS = ["app.example.com"]`: list the host names you serve.
3. `SECRET_KEY = os.environ["DJANGO_SECRET_KEY"]`, and rotate the one that was committed.
4. Set `SESSION_COOKIE_SECURE`, `CSRF_COOKIE_SECURE` and `SECURE_SSL_REDIRECT` to `True`; run `python manage.py check --deploy`.

## Examples

### Python

#### Don't

```py path=settings.py
DEBUG = True
ALLOWED_HOSTS = ["*"]
```

#### Do

```py path=settings.py
import os

DEBUG = os.environ.get("DJANGO_DEBUG") == "1"
ALLOWED_HOSTS = ["app.example.com"]
```

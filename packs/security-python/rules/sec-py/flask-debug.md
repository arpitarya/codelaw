---
id: sec-py/flask-debug
title: 'Flask''s debugger is off'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: suggest
fix_reason: "the setting belongs to the environment, so the debug flag has to come from configuration"
stability: preview
ported_from: [bandit/B201, ruff/S201]
security: { cwe: [489], owasp: ["A02:2025"], severity: high }
since: 0.2.0
---

`app.run(debug=True)` is not written in code.

**Why:** The debugger serves an interactive console that runs code on the server, and a stack trace shows source and settings (CWE-489).

```codelaw
match:
  kind: call
  all:
    - { has: { field: function, regex: '^(app|application|flask_app|self\.app)\.run$' } }
    - has:
        field: arguments
        has:
          kind: keyword_argument
          all: [{ has: { field: name, regex: '^debug$' } }, { has: { field: value, kind: 'true' } }]
          pattern: $KW
message: 'Flask runs with the debugger on; take `debug` from the environment'
fix:
  - { remove: $KW }
```

## How to fix

1. Remove `debug=True`; turn the debugger on with `flask --debug run` while developing.
2. In production run a WSGI server (`gunicorn app:app`), not `app.run`.

## Examples

### Python

#### Don't

```py
from flask import Flask

app = Flask(__name__)
app.run(debug=True)
```

#### Do

```py
from flask import Flask

app = Flask(__name__)
app.run()
```

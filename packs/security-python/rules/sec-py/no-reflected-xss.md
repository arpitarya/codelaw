---
id: sec-py/no-reflected-xss
title: 'A request value is not written into the response as it is'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "escaping the value or rendering a template are both fixes; which fits depends on the response"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A value from the request is not passed to `HttpResponse`, `make_response` or `HTMLResponse`, or built into a returned string, without escaping.

**Why:** The page then contains whatever the client sent, including a script (CWE-79). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  any:
    - { pattern: 'HttpResponse($V)' }
    - { pattern: 'make_response($V)' }
    - { pattern: 'HTMLResponse($V)' }
    - all:
        - { pattern: 'return $V' }
        - has:
            any:
              - { kind: string, has: { kind: interpolation } }
              - { kind: binary_operator, has: { field: operator, regex: '^(%|\+)$' } }
              - { kind: call, has: { field: function, regex: '\.format$' } }
where: { $V: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$V` comes from the request and is written into the page without escaping'
```

## How to fix

1. Escape it: `markupsafe.escape(value)` or Django's `django.utils.html.escape`.
2. Or render a template with autoescape on and pass the value in.
3. If the response is not HTML, set `Content-Type: text/plain` or `application/json`.

## Examples

### Python

#### Don't

```py
def hello(request):
    return HttpResponse(request.GET["name"])
```

#### Do

```py
def hello(request):
    return HttpResponse(escape(request.GET["name"]))
```

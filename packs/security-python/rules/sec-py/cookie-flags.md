---
id: sec-py/cookie-flags
title: 'Cookies are `Secure`, `HttpOnly` and `SameSite`'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: suggest
fix_reason: "`secure=True` stops the cookie being set over plain HTTP in development, and `HttpOnly` hides it from scripts that read it on purpose"
stability: preview
security: { cwe: [614, 1004], owasp: ["A02:2025"], severity: medium }
since: 0.2.0
---

`set_cookie` passes `secure`, `httponly` and `samesite`, and none of them is `False` or `None`.

**Why:** Without `Secure` the cookie travels over plain HTTP, without `HttpOnly` a script can read it, and without `SameSite` other sites can send it along (CWE-614, CWE-1004).

```codelaw
match:
  any:
    - all:
        - { pattern: '$R.set_cookie($$$A)' }
        - not:
            has:
              field: arguments
              has: { kind: keyword_argument, has: { field: name, regex: '^secure$' } }
        - not:
            has:
              field: arguments
              has: { kind: keyword_argument, has: { field: name, regex: '^httponly$' } }
        - not:
            has:
              field: arguments
              has: { kind: keyword_argument, has: { field: name, regex: '^samesite$' } }
    - all:
        - { kind: call }
        - { has: { field: function, regex: '\.set_cookie$' } }
        - any:
            - not:
                has:
                  field: arguments
                  has: { kind: keyword_argument, has: { field: name, regex: '^secure$' } }
            - not:
                has:
                  field: arguments
                  has: { kind: keyword_argument, has: { field: name, regex: '^httponly$' } }
            - not:
                has:
                  field: arguments
                  has: { kind: keyword_argument, has: { field: name, regex: '^samesite$' } }
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all: [{ has: { field: name, regex: '^samesite$' } }, { has: { field: value, kind: none } }]
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all: [{ has: { field: name, regex: '^secure$' } }, { has: { field: value, kind: 'false' } }]
            - has:
                field: arguments
                has:
                  kind: keyword_argument
                  all: [{ has: { field: name, regex: '^httponly$' } }, { has: { field: value, kind: 'false' } }]
message: 'This cookie is missing `Secure`, `HttpOnly` or `SameSite`'
fix: '$R.set_cookie($$$A, secure=True, httponly=True, samesite="Lax")'
```

## How to fix

1. Add `secure=True, httponly=True, samesite="Lax"`.
2. Use `samesite="Strict"` for a session cookie; `None` needs `secure=True` and a reason.
3. A cookie a script must read can omit `httponly`; say so in a comment.

## Examples

### Python

#### Don't

```py
resp.set_cookie("sid", sid)
```

#### Do

```py
resp.set_cookie("sid", sid, secure=True, httponly=True, samesite="Lax")
```

#### Fixed

```py
resp.set_cookie("sid", sid, secure=True, httponly=True, samesite="Lax")
```

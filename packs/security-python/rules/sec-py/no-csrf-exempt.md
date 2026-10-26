---
id: sec-py/no-csrf-exempt
title: 'No `@csrf_exempt`'
languages: [python]
fix: manual
fix_reason: "an endpoint that needs to be exempt (a webhook) needs its own authentication, which only the author can choose"
stability: preview
security: { cwe: [352], owasp: ["A01:2025"], severity: high }
since: 0.2.0
---

A view is not decorated with `@csrf_exempt`.

**Why:** Without the token check any page the user visits can submit the form as the user (CWE-352).

```codelaw
match:
  kind: decorator
  regex: '^@\s*((django\.views\.decorators\.csrf\.)|(csrf\.))?csrf_exempt\b'
message: 'This view skips the CSRF check'
```

## How to fix

1. Remove `@csrf_exempt` and send the token (`{% csrf_token %}`, or the `X-CSRFToken` header).
2. For a webhook, authenticate the sender (a signed header) and keep the exemption on that one view with a comment saying why.

## Examples

### Python

#### Don't

```py
from django.views.decorators.csrf import csrf_exempt

@csrf_exempt
def update(request):
    ...
```

#### Do

```py
def update(request):
    ...
```

---
id: sec-py/fastapi-cors
title: 'CORS does not allow every origin with credentials'
languages: [python]
fix: manual
fix_reason: "the list of trusted origins is the application's own"
stability: preview
security: { cwe: [942], owasp: ["A02:2025"], severity: high }
since: 0.2.0
---

`CORSMiddleware` is not given `allow_origins=["*"]` together with `allow_credentials=True`.

**Why:** Any site can then make authenticated requests as the user and read the answers (CWE-942).

```codelaw
match:
  any:
    - all:
        - { kind: call }
        - { has: { field: function, regex: '^(CORSMiddleware|.*\.add_middleware)$' } }
        - { has: { field: arguments, regex: 'allow_origins\s*=\s*\[\s*[''"]\*[''"]\s*\]' } }
        - { has: { field: arguments, regex: 'allow_credentials\s*=\s*True' } }
message: 'Every origin is allowed with credentials; list the trusted origins'
```

## How to fix

1. List the origins: `allow_origins=["https://app.example.com"]`.
2. If the API needs no cookies, set `allow_credentials=False` and keep `*`.

## Examples

### Python

#### Don't

```py
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_credentials=True)
```

#### Do

```py
app.add_middleware(CORSMiddleware, allow_origins=["https://app.example.com"], allow_credentials=True)
```

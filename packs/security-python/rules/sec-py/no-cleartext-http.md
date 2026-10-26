---
id: sec-py/no-cleartext-http
title: 'Requests go over HTTPS'
languages: [python]
fix: manual
fix_reason: "the server must serve HTTPS first, and the URL change has to follow it"
stability: preview
security: { cwe: [319], owasp: ["A04:2025"], severity: medium }
since: 0.2.0
---

`requests` and `httpx` are not called with an `http://` URL, except for localhost.

**Why:** The data, and any token in it, crosses the network readable and changeable by anyone on the path (CWE-319).

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: '^(requests|httpx)\.(get|post|put|patch|delete|head|options|request|stream)$'
    - has:
        field: arguments
        has:
          nthChild: 1
          kind: string
          regex: '^[rbf]?[''\"]http://'
          not: { regex: '^[rbf]?[''\"]http://(localhost|127\.0\.0\.1|\[::1\]|0\.0\.0\.0)' }
message: 'This URL is plain HTTP; use `https://`'
```

## How to fix

1. Change the URL to `https://`.
2. If the server cannot do HTTPS, put it behind a proxy that can, or reach it over a private tunnel.

## Examples

### Python

#### Don't

```py
import requests

resp = requests.get("http://api.example.com/data", timeout=5)
```

#### Do

```py
import requests

resp = requests.get("https://api.example.com/data", timeout=5)
```

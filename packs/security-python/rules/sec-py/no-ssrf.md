---
id: sec-py/no-ssrf
title: 'A request value is not a URL to fetch'
languages: [python]
fix: manual
fix_reason: "which hosts are allowed is a policy only the author knows"
stability: preview
security: { cwe: [918], owasp: ["A01:2025"], severity: high }
since: 0.2.0
---

A value from the request is not the URL given to `requests`, `httpx` or `urllib`.

**Why:** The server fetches whatever address the client names, including internal services and cloud metadata (CWE-918). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  all:
    - { kind: call }
    - has:
        field: function
        regex: '^(requests|httpx)\.(get|post|put|patch|delete|head|options|request|stream)$|^(urllib\.request\.)?urlopen$|^urllib\.request\.urlretrieve$|^(session|client)\.(get|post|put|delete|request)$'
    - { has: { field: arguments, has: { nthChild: 1, pattern: $U } } }
where: { $U: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$U` comes from the request and is fetched by the server'
```

## How to fix

1. Take an id or a key from the client and map it to a URL you wrote.
2. Or parse the URL and check the scheme is `https` and the host is on an allow-list, then connect to the address you resolved (not the name again).
3. Refuse private, loopback and link-local addresses.

## Examples

### Python

#### Don't

```py
def fetch(request):
    url = request.args["url"]
    return requests.get(url, timeout=5)
```

#### Do

```py
def fetch(request):
    host = urlparse(request.args["url"]).hostname
    if host not in ALLOWED_HOSTS:
        abort(400)
    return requests.get(ALLOWED_HOSTS[host], timeout=5)
```

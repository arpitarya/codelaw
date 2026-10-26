---
id: sec-py/no-header-injection
title: 'A request value is not a response header'
languages: [python]
fix: manual
fix_reason: "whether to reject, strip or encode the value depends on what the header is for"
stability: preview
security: { cwe: [113], owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

A value from the request is not written into a response header or a cookie value without its line breaks removed.

**Why:** A CR or LF in the value ends the header and starts a new one, so an attacker can add headers or a second response (CWE-113). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  any:
    - { pattern: '$R.headers[$K] = $V' }
    - { pattern: '$R.headers.add($K, $V)' }
    - { pattern: '$R.headers.set($K, $V)' }
    - { pattern: '$R[$K] = $V' }
    - { pattern: '$R.set_cookie($K, $V)' }
constraints: { R: { regex: '^(resp|response|res|r|rv|rsp|self\.response|make_response\(.*\))$' } }
where: { $V: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$V` comes from the request and is written into a header'
```

## How to fix

1. Reject a value that contains `\r` or `\n`: `if "\n" in value or "\r" in value: abort(400)`.
2. Or percent-encode it: `urllib.parse.quote(value)`.
3. Modern servers and frameworks refuse newlines in headers; do not rely on that.

## Examples

### Python

#### Don't

```py
def view(request):
    resp = make_response("ok")
    resp.headers["X-Name"] = request.args["name"]
    return resp
```

#### Do

```py
def view(request):
    resp = make_response("ok")
    resp.headers["X-Name"] = quote(request.args["name"])
    return resp
```

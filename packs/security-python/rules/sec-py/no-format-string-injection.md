---
id: sec-py/no-format-string-injection
title: 'A request value is not a format string'
languages: [python]
fix: manual
fix_reason: "the template has to be a literal, which needs the format string moved out of the request"
stability: preview
security: { cwe: [134], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A value from the request is not the receiver of `.format`.

**Why:** A field such as `{obj.__class__.__init__.__globals__[SECRET]}` reads attributes of the arguments, so the client reads secrets (CWE-134). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  any: [{ pattern: '$V.format($$$A)' }, { pattern: '$V.format_map($$$A)' }]
where: { $V: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$V` comes from the request and is used as a format string'
```

## How to fix

1. Make the template a literal and pass the request value as an argument: `"Hello {}".format(name)`.
2. For client-written templates use `string.Template(...).safe_substitute(...)`, which has no attribute access.

## Examples

### Python

#### Don't

```py
def greet(request):
    return request.GET["tpl"].format(user)
```

#### Do

```py
def greet(request):
    return "Hello {}".format(request.GET["name"])
```

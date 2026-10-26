---
id: sec-py/no-dynamic-import
title: 'A request value does not choose a module or attribute'
languages: [python]
fix: manual
fix_reason: "the allow-list of names is the author's"
stability: preview
security: { cwe: [470], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A value from the request is not passed to `__import__`, `importlib.import_module` or `getattr`.

**Why:** The client then picks which module is loaded or which attribute is read or called, including ones that run commands (CWE-470). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  any:
    - { pattern: '__import__($X)' }
    - { pattern: 'importlib.import_module($X)' }
    - { pattern: 'import_module($X)' }
    - { pattern: 'getattr($O, $X)' }
    - { pattern: 'getattr($O, $X, $D)' }
    - { pattern: 'importlib.import_module($X, $P)' }
where: { $X: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$X` comes from the request and chooses a module or attribute'
```

## How to fix

1. Map the client's value to a fixed dict of allowed handlers: `HANDLERS = {"csv": export_csv}`; `HANDLERS[name]`.
2. Reject a name that is not a key.

## Examples

### Python

#### Don't

```py
def run(request):
    action = request.GET["action"]
    return getattr(handlers, action)()
```

#### Do

```py
def run(request):
    return HANDLERS[request.GET["action"]]()
```

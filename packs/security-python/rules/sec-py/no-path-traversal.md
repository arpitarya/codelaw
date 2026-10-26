---
id: sec-py/no-path-traversal
title: 'A request value is not a file path'
languages: [python]
fix: manual
fix_reason: "the safe form (a lookup by id, a resolved path checked against a base folder) depends on the storage layout"
stability: preview
security: { cwe: [22], owasp: ["A01:2025"], severity: high }
since: 0.2.0
---

A value from the request is not passed to `open`, `send_file`, `send_from_directory` or `FileResponse` as a path.

**Why:** `../../etc/passwd` in the value reads any file the process can (CWE-22). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  all:
    - { kind: call }
    - has:
        field: function
        regex: '^(open|(.*\.)?(send_file|send_from_directory|FileResponse|FileSystemStorage|read_text|read_bytes)|Path|pathlib\.Path|os\.path\.join)$'
    - { has: { field: arguments, has: { nthChild: 1, pattern: $P } } }
where: { $P: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$P` comes from the request and is used as a path'
```

## How to fix

1. Look the file up by an id the program issued, not by a name the client chose.
2. Or resolve the path and check it stays inside the base folder: `p = (BASE / name).resolve(); p.relative_to(BASE.resolve())`.
3. Flask: use `send_from_directory(BASE, secure_filename(name))`.

## Examples

### Python

#### Don't

```py
def download(request):
    name = request.GET["file"]
    return open(name, "rb")
```

#### Do

```py
def download(request):
    name = request.GET["file"]
    path = (BASE / name).resolve()
    path.relative_to(BASE.resolve())
    return open(path, "rb")
```

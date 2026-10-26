---
id: sec-py/no-xmlrpc
title: 'No `xmlrpc` without `defusedxml`'
languages: [python]
fix: manual
fix_reason: "the patch has to run once before any xmlrpc object exists, and where that is depends on the program's start-up"
stability: preview
ported_from: [bandit/B411, ruff/S411]
security: { cwe: [776], owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

An `xmlrpc` client or server is not created without `defusedxml.xmlrpc.monkey_patch()` in the program.

**Why:** `xmlrpc` parses XML with the standard parser, so a crafted request or response carries an entity expansion bomb or reads files (CWE-776).

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: '^(xmlrpc\.(client|server)\.)?(ServerProxy|Server|SimpleXMLRPCServer|SimpleXMLRPCRequestHandler|MultiCall|DocXMLRPCServer|CGIXMLRPCRequestHandler)$'
message: '`xmlrpc` parses XML unsafely; call `defusedxml.xmlrpc.monkey_patch()` at start-up, or use JSON'
```

## How to fix

1. Add `import defusedxml.xmlrpc` and call `defusedxml.xmlrpc.monkey_patch()` once, before any `xmlrpc` object is made.
2. Or move the API to JSON over HTTP.

## Examples

### Python

#### Don't

```py
from xmlrpc.client import ServerProxy

proxy = ServerProxy(url)
```

#### Do

```py
import requests

resp = requests.post(url, json=payload, timeout=10)
```

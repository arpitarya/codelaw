---
id: sec-py/no-regex-injection
title: 'A request value is not a regular expression'
languages: [python]
fix: suggest
fix_reason: "`re.escape` turns the value into a literal search; a client that is meant to send a pattern needs a length limit and a timeout instead"
stability: preview
security: { cwe: [730, 1333], owasp: ["A06:2025"], severity: medium }
since: 0.2.0
---

A value from the request is not used as the pattern of `re.compile`, `re.search`, `re.match` or the other `re` functions.

**Why:** The client can write a pattern that takes exponential time (a ReDoS) or changes what the search matches (CWE-730, CWE-1333). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match: { any: [{ pattern: '$F($P, $$$R)' }, { pattern: '$F($P)' }] }
constraints: { F: { regex: '^re\.(compile|search|match|fullmatch|findall|finditer|sub|subn|split)$' } }
where: { $P: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$P` comes from the request and is used as a regular expression; escape it'
fix: '$F(re.escape($P), $$$R)'
```

## How to fix

1. If the value is text to look for, wrap it in `re.escape(value)`.
2. If the client must send a pattern, limit its length and run it with a timeout (the `regex` package's `timeout=`).

## Examples

### Python

#### Don't

```py
import re

def find(request, text):
    return re.search(request.GET["q"], text)
```

#### Do

```py
import re

def find(request, text):
    return re.search(re.escape(request.GET["q"]), text)
```

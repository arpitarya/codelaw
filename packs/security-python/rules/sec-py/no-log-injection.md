---
id: sec-py/no-log-injection
title: 'A request value is not logged raw'
languages: [python]
severity: warn
fix: manual
fix_reason: "whether to strip, escape or reject the value depends on how the log is read"
stability: preview
security: { cwe: [117], owasp: ["A09:2025"], severity: low }
since: 0.2.0
---

A value from the request is not passed to a logger without its line breaks removed.

**Why:** A CR or LF in the value writes a line that looks like another log entry, so an attacker can forge audit records (CWE-117). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match: { any: [{ pattern: '$L($V)' }, { pattern: '$L($A, $V)' }] }
constraints:
  L:
    regex: '^(logger|logging|log|self\.logger|self\.log|app\.logger|_logger|_log)\.(debug|info|warning|warn|error|exception|critical|log)$'
  A: { regex: '^[rbf]?[''\"]' }
where: { $V: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$V` comes from the request and is logged as it is'
```

## How to fix

1. Pass the value through a helper that strips the line breaks, such as `clean(value)` with `text.replace("\r", "").replace("\n", " ")`; a method called on the request value itself is still seen as the request value.
2. Or log it as structured data (`extra={"name": value}`) and use a JSON formatter that escapes it.

## Examples

### Python

#### Don't

```py
def view(request):
    logger.info("login %s", request.POST["user"])
```

#### Do

```py
def clean(text):
    return text.replace("\r", "").replace("\n", " ")


def view(request):
    logger.info("login %s", clean(request.POST["user"]))
```

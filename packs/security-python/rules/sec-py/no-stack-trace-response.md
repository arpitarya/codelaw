---
id: sec-py/no-stack-trace-response
title: 'A stack trace is not sent to the client'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "what the client sees instead (a generic message and an error id) is a product decision"
stability: preview
security: { cwe: [209], owasp: ["A10:2025"], severity: medium }
since: 0.2.0
---

A response does not contain `traceback.format_exc()` or the text of the exception.

**Why:** The text names files, queries, library versions and sometimes values, which tells an attacker where to push (CWE-209).

```codelaw
match:
  any:
    - all:
        - { kind: return_statement }
        - has:
            stopBy: end
            kind: call
            regex: '^(str|repr)\((e|exc|err|ex|error|exception)\)$|^traceback\.format_exc\(\)$|^(e|exc|err|ex|error|exception)\.(args|message)$'
    - all:
        - { kind: raise_statement }
        - has:
            stopBy: end
            kind: call
            regex: '^(str|repr)\((e|exc|err|ex|error|exception)\)$|^traceback\.format_exc\(\)$|^(e|exc|err|ex|error|exception)\.(args|message)$'
        - has:
            stopBy: end
            kind: identifier
            regex: '^(HTTPException|abort|HttpResponseServerError)$'
message: 'The text of an exception is sent to the client; log it and send a generic message'
```

## How to fix

1. Log it: `logger.exception("upload failed")`.
2. Return a generic message and an id the client can quote: `{"error": "internal error", "id": error_id}`.

## Examples

### Python

#### Don't

```py
def view():
    try:
        run()
    except Exception as e:
        return jsonify(error=str(e))
```

#### Do

```py
def view():
    try:
        run()
    except Exception:
        logger.exception("run failed")
        return jsonify(error="internal error")
```

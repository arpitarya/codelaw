---
id: sec-py/no-silent-except
title: 'Errors are not swallowed'
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "what to do with the error (log, retry, stop) is a decision"
stability: preview
ported_from: [bandit/B110, bandit/B112, ruff/S110, ruff/S112]
security: { cwe: [703], owasp: ["A10:2025"], severity: low }
since: 0.2.0
---

An `except` that catches everything (bare, `Exception` or `BaseException`) does not just `pass` or `continue`.

**Why:** A failure that is dropped hides a broken check or a failed login step, and the code goes on as though it had worked (CWE-703).

```codelaw
match:
  all:
    - { kind: except_clause }
    - any:
        - { regex: '^except\s*:' }
        - { has: { kind: identifier, regex: '^(Exception|BaseException)$' } }
    - { has: { kind: block, regex: '^(pass|continue)$' } }
message: 'This `except` drops the error; handle it or log it'
```

## How to fix

1. Catch the specific exceptions you expect.
2. Log the others with `logger.exception("…")` and let them go on or re-raise.

## Examples

### Python

#### Don't

```py
try:
    check(token)
except Exception:
    pass
```

#### Do

```py
try:
    check(token)
except InvalidToken:
    logger.warning("bad token")
```

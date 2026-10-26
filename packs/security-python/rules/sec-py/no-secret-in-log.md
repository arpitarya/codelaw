---
id: sec-py/no-secret-in-log
title: 'Secrets are not logged'
languages: [python]
fix: manual
fix_reason: "whether to drop the value or mask it depends on what the log is for"
stability: preview
security: { cwe: [532], owasp: ["A09:2025"], severity: medium }
since: 0.2.0
---

A value named `password`, `token` or `secret` is not written to a log.

**Why:** Logs are kept longer and read by more people than the secret was meant for (CWE-532).

```codelaw
match:
  all:
    - { kind: call }
    - has:
        field: function
        regex: '^((logger|logging|log|self\.logger|self\.log|app\.logger)\.(debug|info|warning|warn|error|exception|critical|log)|print)$'
    - has:
        field: arguments
        has:
          regex: '(?i)(pass(word|wd)?|secret|token|api_?key|auth|otp|nonce|salt|session_?id|csrf|reset_?code|verification_?code)'
          any:
            - { kind: identifier }
            - { kind: attribute }
            - { kind: subscript }
            - { kind: string, has: { kind: interpolation } }
            - { kind: keyword_argument }
message: 'A secret may be written to the log; log that it was set, not its value'
```

## How to fix

1. Remove the value from the message.
2. Log an id or `"set"/"missing"`, or a masked form such as the last four characters.

## Examples

### Python

#### Don't

```py
logger.info("login with password %s", password)
```

#### Do

```py
logger.info("login for user %s", username)
```

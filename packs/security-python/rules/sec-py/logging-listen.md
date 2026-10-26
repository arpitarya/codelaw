---
id: sec-py/logging-listen
title: "`logging.config.listen` is not used"
languages: [python]
fix: manual
fix_reason: "reconfiguring a running process needs another route (a signal handler, a config reload) the author has to choose"
stability: preview
ported_from: [bandit/B612, ruff/S612]
security: { cwe: [94], owasp: ["A08:2025"], severity: high }
since: 0.2.0
---

`logging.config.listen()` is not called.

**Why:** It opens a socket that accepts a logging configuration, which can name any callable to run, so anyone who can connect runs code (CWE-94).

```codelaw
match:
  kind: call
  all: [{ has: { field: function, regex: '^(logging\.)?config\.listen$' } }]
message: 'This opens a socket that accepts logging configuration, which can run code'
```

## How to fix

1. Remove the call; load the configuration from a file at start-up with `logging.config.dictConfig`.
2. To change levels at run time use a signal handler that re-reads the file.

## Examples

### Python

#### Don't

```py
import logging.config

t = logging.config.listen(9999)
```

#### Do

```py
import logging.config

logging.config.dictConfig(CONFIG)
```

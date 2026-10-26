---
id: sec-py/no-mark-safe
title: 'No `mark_safe` of dynamic text'
languages: [python]
fix: manual
fix_reason: "the right fix is escaping the parts (`format_html`, `escape`) or rendering through a template, not removing the call"
stability: preview
ported_from: [bandit/B308, bandit/B703, bandit/B704, ruff/S308, ruff/S703, ruff/S704]
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

`mark_safe`, `SafeString`, `Markup` and `format_html` are not given a value that is not a literal.

**Why:** They switch HTML escaping off for that value, so any part of it that came from a user is a script (CWE-79).

```codelaw
match:
  all:
    - { kind: call }
    - has:
        field: function
        regex: '^((.*\.)?(mark_safe|SafeString|SafeText|SafeData)|(markupsafe\.)?Markup)$'
    - has:
        field: arguments
        has: { nthChild: 1, not: { kind: string, not: { has: { kind: interpolation } } }, pattern: $X }
message: '`$X` is marked safe, so it is not escaped; escape the parts with `format_html` or `escape`'
```

## How to fix

1. Build HTML with `format_html("<b>{}</b>", value)`: it escapes `value` and trusts only the literal.
2. Or render a template with autoescape on.
3. Use `mark_safe` only on a literal, or on text you produced by escaping.

## Examples

### Python

#### Don't

```py
from django.utils.safestring import mark_safe

html = mark_safe("<b>" + name + "</b>")
```

#### Do

```py
from django.utils.html import format_html

html = format_html("<b>{}</b>", name)
```

---
id: sec-py/jinja-autoescape
title: 'Jinja2 keeps autoescape on'
languages: [python]
fix: suggest
fix_reason: "turning autoescape on changes the output of templates that write HTML on purpose; those need `|safe` or `Markup`"
stability: preview
ported_from: [bandit/B701, ruff/S701]
security: { cwe: [79, 1336], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

`Environment` is not created with `autoescape=False` or without `autoescape`, and `render_template_string` / `Template` / `from_string` are not given a template that is not a literal.

**Why:** Without autoescape a value written into a page can carry a script (CWE-79); a template built from a non-literal runs template code on the server (CWE-1336).

```codelaw
match:
  any:
    - kind: call
      all:
        - { has: { field: function, regex: '^(jinja2\.)?Environment$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^autoescape$' } }
                - { has: { field: value, kind: 'false' } }
              pattern: $KW
    - all:
        - { kind: call }
        - { has: { field: function, regex: '^(jinja2\.)?Environment$' } }
        - not:
            has:
              field: arguments
              has: { kind: keyword_argument, has: { field: name, regex: '^(autoescape)$' } }
    - { pattern: 'render_template_string($T)' }
    - { pattern: 'jinja2.Template($T)' }
    - { pattern: '$E.from_string($T)' }
constraints:
  T: { not: { kind: string, not: { has: { kind: interpolation } } } }
  E: { regex: '^(env|environment|jinja_env|jinja2\.Environment\(.*\))$' }
message: 'Jinja2 is not escaping, or the template comes from the request'
fix:
  - { replace: { target: $KW, with: 'autoescape=True' } }
```

## How to fix

1. Create the environment with `autoescape=True`, or `select_autoescape()`.
2. Never use request data as the template: pass it as a value to a template file instead.

## Examples

### Python

#### Don't

```py
from jinja2 import Environment

env = Environment(autoescape=False)
```

#### Do

```py
from jinja2 import Environment

env = Environment(autoescape=True)
```

#### Fixed

```py
from jinja2 import Environment

env = Environment(autoescape=True)
```

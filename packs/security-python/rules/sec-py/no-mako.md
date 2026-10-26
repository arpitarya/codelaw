---
id: sec-py/no-mako
title: 'No Mako templates'
languages: [python]
fix: manual
fix_reason: "moving to another engine (or turning on `default_filters`) changes every template"
stability: preview
ported_from: [bandit/B702, ruff/S702]
security: { cwe: [79], owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

`mako.template.Template` is not used.

**Why:** Mako does not escape output unless told to, so every `${value}` is a script waiting for a hostile value (CWE-79).

```codelaw
match: { any: [{ pattern: '$T($$$A)' }] }
constraints: { T: { regex: '^(Template|[A-Za-z_]+\.Template|mako\.template\.Template)$' } }
where:
  $T:
    any:
      - { imported-from: { module: mako.template, name: Template } }
      - { imported-from: mako.template }
      - { imported-from: { module: mako, name: template } }
message: 'Mako does not escape by default; use Jinja2 with autoescape, or set `default_filters=[''h'']`'
```

## How to fix

1. Use Jinja2 with `autoescape=True`.
2. If you keep Mako, pass `default_filters=["h"]` to `Template` / `TemplateLookup`, and review each `|n`.

## Examples

### Python

#### Don't

```py
from mako.template import Template

t = Template(source)
```

#### Do

```py
from jinja2 import Environment

env = Environment(autoescape=True)
```

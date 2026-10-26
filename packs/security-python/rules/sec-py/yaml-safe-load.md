---
id: sec-py/yaml-safe-load
title: "`yaml.safe_load` instead of `yaml.load`"
languages: [python]
fix: suggest
fix_reason: "`safe_load` rejects the Python-specific tags (`!!python/object`) a document may rely on"
stability: preview
ported_from: [bandit/B506, ruff/S506]
security: { cwe: [502], owasp: ["A08:2025"], severity: high }
since: 0.2.0
---

YAML is read with `yaml.safe_load`, or with `yaml.load` and `SafeLoader`.

**Why:** The default and full loaders build arbitrary Python objects, so a crafted document runs code (CWE-502).

```codelaw
match:
  any:
    - { pattern: 'yaml.load($X)' }
    - { pattern: 'yaml.load($X, Loader=$L)' }
    - { pattern: 'yaml.load($X, $L)' }
constraints: { L: { not: { regex: '^(yaml\.)?(C?SafeLoader)$' } } }
message: '`yaml.load` can build arbitrary objects; use `yaml.safe_load`'
fix: 'yaml.safe_load($X)'
```

## How to fix

1. Replace `yaml.load(stream)` with `yaml.safe_load(stream)`.
2. If you pass a `Loader`, pass `yaml.SafeLoader`.

## Examples

### Python

#### Don't

```py
import yaml

data = yaml.load(stream)
```

#### Do

```py
import yaml

data = yaml.safe_load(stream)
```

---
id: yaml/no-ambiguous-booleans
title: "No yes/no/on/off as unquoted values"
languages: [yaml]
fix: suggest
fix_reason: "quoting keeps the YAML 1.2 meaning (a string); if a boolean was meant, write `true` or `false` instead"
stability: preview
since: 0.1.0
---

A value is not a bare `yes`, `no`, `on` or `off` (in any of their capitalisations). Quote it if it is text, or write `true` or `false` if it is a boolean. Keys are not checked, so a GitHub Actions `on:` is fine.

**Why:** YAML 1.2 reads these words as strings, but YAML 1.1 loaders, still common (PyYAML, Ruby's Psych, many tools built on them), read them as booleans: `country: no` becomes `false`.

```codelaw
native: yaml/no-ambiguous-booleans
```

## Examples

### YAML

#### Don't

```yaml
country: no
```

#### Do

```yaml
country: "no"
```

#### Do

```yaml
enabled: true
```

## Differences from the source

The idea comes from the YAML 1.1 and 1.2 specifications. `y` and `n` are not reported: the YAML 1.1 type repository lists them, but the common 1.1 loaders don't treat them as booleans.

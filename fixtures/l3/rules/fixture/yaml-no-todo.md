---
id: fixture/yaml-no-todo
title: L3 fixture rule
languages: [yaml]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: block_mapping_pair
  has: { field: key, regex: ^todo$ }
message: "a `todo` key"
```

## How to fix

1. Change it by hand.

## Examples

### YAML

#### Don't

```yaml
todo: x
```

#### Do

```yaml
done: x
```

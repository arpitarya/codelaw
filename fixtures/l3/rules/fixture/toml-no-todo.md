---
id: fixture/toml-no-todo
title: L3 fixture rule
languages: [toml]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: bare_key
  regex: ^todo$
message: "a `todo` key"
```

## How to fix

1. Change it by hand.

## Examples

### TOML

#### Don't

```toml
todo = "x"
```

#### Do

```toml
done = "x"
```

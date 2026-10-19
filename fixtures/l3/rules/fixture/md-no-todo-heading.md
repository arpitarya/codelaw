---
id: fixture/md-no-todo-heading
title: L3 fixture rule
languages: [markdown]
fix: manual
fix_reason: "fixture rule: a human picks the change"
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  kind: atx_heading
  regex: TODO
message: "a TODO heading"
```

## How to fix

1. Change it by hand.

## Examples

### Markdown

#### Don't

```md
# TODO
```

#### Do

```md
# Done
```

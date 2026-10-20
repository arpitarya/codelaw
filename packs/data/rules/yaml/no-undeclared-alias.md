---
id: yaml/no-undeclared-alias
title: "Every alias refers to an anchor defined before it"
languages: [yaml]
fix: manual
fix_reason: "whether the anchor is missing, misspelt or defined too late needs the author"
stability: preview
since: 0.1.0
---

An alias `*name` refers to an anchor `&name` defined earlier in the same document.

**Why:** An alias with no anchor before it is an error: YAML loaders refuse the whole file, and anchors don't carry over from one document to the next.

```codelaw
native: yaml/no-undeclared-alias
```

## How to fix

1. Find the anchor the alias meant.
2. Move the anchored node above the alias, fix the spelling, or add the anchor.

## Examples

### YAML

#### Don't

```yaml
production:
  <<: *defaults
  host: example.com
defaults: &defaults
  port: 443
```

#### Do

```yaml
defaults: &defaults
  port: 443
production:
  <<: *defaults
  host: example.com
```

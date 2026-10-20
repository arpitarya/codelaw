---
id: html/map-id-name
title: "A map's id and name match"
languages: [html]
fix: manual
fix_reason: "which of the two names is right depends on what refers to the map"
stability: preview
ported_from: [html-validate/map-id-name]
since: 0.1.0
---

When a `<map>` has both `id` and `name`, they are the same.

**Why:** HTML requires them to match; tools that look maps up by `id` and browsers that use `name` would find different maps.

```codelaw
native: html/map-id-name
```

## How to fix

1. Make `id` and `name` equal, and update references to the one you changed.

## Examples

### HTML

#### Don't

```html
<map id="nav" name="menu"></map>
```

#### Do

```html
<map id="nav" name="nav"></map>
```

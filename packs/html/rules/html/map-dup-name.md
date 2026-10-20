---
id: html/map-dup-name
title: "Image maps have unique names"
languages: [html]
fix: manual
fix_reason: "renaming a map means updating the `usemap` that points at it"
stability: preview
ported_from: [html-validate/map-dup-name]
since: 0.1.0
---

No two `<map>` elements share a `name`.

**Why:** An image's `usemap` picks one by name, so the other map is unreachable.

```codelaw
native: html/map-dup-name
```

## How to fix

1. Rename one map and the `usemap` attributes that refer to it.

## Examples

### HTML

#### Don't

```html
<map name="nav"></map>
<map name="nav"></map>
```

#### Do

```html
<map name="nav"></map>
<map name="footer-nav"></map>
```

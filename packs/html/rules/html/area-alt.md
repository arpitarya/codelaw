---
id: html/area-alt
title: "Image-map areas with a link have alt text"
languages: [html]
fix: manual
fix_reason: "the text that describes the area needs a human"
stability: preview
options:
  accessible: { type: "boolean", default: true, description: "require `alt` on every linked area, not just one area per link target" }
ported_from: [html-validate/area-alt]
tags: [a11y]
since: 0.1.0
---

An `<area>` with `href` has non-empty `alt` text, and an `<area>` without `href` has no `alt`.

**Why:** A screen reader announces a linked area by its `alt`; without it the link has no name.

```codelaw
native: html/area-alt
```

## How to fix

1. Give each linked `<area>` an `alt` that says where it leads.
2. Remove `alt` from an `<area>` that has no `href`.

## Examples

### HTML

#### Don't

```html
<img src="floor.png" usemap="#floor" alt="Floor plan">
<map name="floor">
  <area shape="rect" coords="0,0,50,50" href="/kitchen">
</map>
```

#### Do

```html
<img src="floor.png" usemap="#floor" alt="Floor plan">
<map name="floor">
  <area shape="rect" coords="0,0,50,50" href="/kitchen" alt="Kitchen">
</map>
```

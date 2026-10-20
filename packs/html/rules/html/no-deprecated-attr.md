---
id: html/no-deprecated-attr
title: "No obsolete attributes"
languages: [html]
fix: manual
fix_reason: "the replacement is usually CSS, which needs a class and a rule"
stability: preview
ported_from: [html-validate/no-deprecated-attr]
since: 0.1.0
---

Elements don't use attributes HTML has made obsolete, such as `align`, `bgcolor`, `border` on `<img>`, or `name` on `<a>`.

**Why:** Browsers support them unevenly; CSS and current attributes do the same job reliably.

```codelaw
native: html/no-deprecated-attr
```

## How to fix

1. Replace presentational attributes with a class and CSS.
2. Replace other obsolete attributes with their current equivalent (for example `id` for `<a name>`).

## Examples

### HTML

#### Don't

```html
<img src="a.png" alt="Chart" align="left">
```

#### Do

```html
<img src="a.png" alt="Chart" class="float-left">
```

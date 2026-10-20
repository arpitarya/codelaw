---
id: html/aria-hidden-body
title: "<body> is not aria-hidden"
languages: [html]
fix: suggest
fix_reason: "removing it exposes the page to screen readers again, which changes what users hear"
stability: preview
ported_from: [html-validate/aria-hidden-body]
tags: [a11y]
since: 0.1.0
---

`<body>` does not have `aria-hidden="true"`.

**Why:** It hides the whole page from assistive technology, while sighted users still see it.

```codelaw
native: html/aria-hidden-body
```

## Examples

### HTML

#### Don't

```html
<body aria-hidden="true">
  <p>Welcome</p>
</body>
```

#### Do

```html
<body>
  <p>Welcome</p>
</body>
```

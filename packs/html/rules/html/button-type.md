---
id: html/button-type
title: "<button> says what type it is"
languages: [html]
fix: suggest
fix_reason: "`type=\"button\"` stops the button submitting its form; a button meant to submit needs `type=\"submit\"` instead"
stability: preview
ported_from: [html-validate/no-implicit-button-type, htmlhint/button-type-require]
tags: [a11y]
since: 0.1.0
---

A `<button>` has a `type` attribute (`button`, `submit` or `reset`).

**Why:** Without one a button submits its form, so a button meant to open a menu also sends the form, also when the user presses Enter.

```codelaw
native: html/button-type
```

## Examples

### HTML

#### Don't

```html
<button>Open menu</button>
```

#### Do

```html
<button type="button">Open menu</button>
```

---
id: html/prefer-native-element
title: "Native elements instead of ARIA roles"
languages: [html]
fix: manual
fix_reason: "swapping the element changes its styles and scripts"
stability: preview
ported_from: [html-validate/prefer-native-element]
tags: [a11y]
since: 0.1.0
---

An element doesn't take a role a native element provides, such as `role="button"` on a `<div>` or `role="navigation"` on a `<div>`.

**Why:** Native elements bring keyboard handling, focus and semantics for free; a role brings only the announcement.

```codelaw
native: html/prefer-native-element
```

## How to fix

1. Replace the element with the native one the message names.
2. Remove the `role` and any keyboard handling the native element now provides.

## Examples

### HTML

#### Don't

```html
<div role="button" tabindex="0">Save</div>
```

#### Do

```html
<button type="button">Save</button>
```

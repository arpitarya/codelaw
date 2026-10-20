---
id: html/element-required-ancestor
title: "Elements are inside the ancestor they need"
languages: [html]
fix: manual
fix_reason: "whether to add the ancestor or use other elements depends on the intent"
stability: preview
ported_from: [html-validate/element-required-ancestor]
since: 0.1.0
---

Elements that only work inside a given ancestor are there: `<area>` in a `<map>`, `<dt>` and `<dd>` in a `<dl>`.

**Why:** Outside it they have no meaning and are rendered as plain text, or not at all.

```codelaw
native: html/element-required-ancestor
```

## How to fix

1. Wrap the elements in the ancestor the message names.
2. Or replace them with elements that make sense where they are.

## Examples

### HTML

#### Don't

```html
<div>
  <dt>Term</dt>
  <dd>Definition</dd>
</div>
```

#### Do

```html
<dl>
  <dt>Term</dt>
  <dd>Definition</dd>
</dl>
```

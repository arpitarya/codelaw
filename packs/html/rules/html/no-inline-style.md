---
id: html/no-inline-style
title: "No inline styles"
languages: [html]
fix: manual
fix_reason: "the styles need a class and a stylesheet rule"
stability: preview
options:
  allowedProperties: { type: "string[]", default: ["display"], description: "properties an inline style may set" }
  allowVariables: { type: "boolean", default: true, description: "let inline styles set custom properties (`--name`)" }
ported_from: [html-validate/no-inline-style, htmlhint/inline-style-disabled]
since: 0.1.0
---

Elements don't have a `style` attribute, except one that only sets `display` or custom properties.

**Why:** Inline styles can't be reused, override stylesheets with high specificity, and are blocked by a strict Content Security Policy.

```codelaw
native: html/no-inline-style
```

## How to fix

1. Move the declarations into a stylesheet rule for a class.
2. Replace the `style` attribute with that class.

## Examples

### HTML

#### Don't

```html
<p style="color: red; font-weight: bold">Warning</p>
```

#### Do

```html
<p class="warning">Warning</p>
```

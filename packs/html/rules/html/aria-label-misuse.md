---
id: html/aria-label-misuse
title: "aria-label is used only where it names something"
languages: [html]
fix: manual
fix_reason: "whether to move the label, add a role or use another element is a design decision"
stability: preview
options:
  allowAnyNamable: { type: "boolean", default: false, description: "accept the label on any element ARIA lets be named, even where support is poor" }
ported_from: [html-validate/aria-label-misuse]
tags: [a11y]
since: 0.1.0
---

`aria-label` and `aria-labelledby` appear only on elements that can carry an accessible name: interactive and labelable elements, landmarks, elements with a `role` or `tabindex`, and a few others (`img`, `table`, `fieldset`, `dialog`, …).

**Why:** On a plain `<span>` or `<div>` screen readers ignore the label, so the text never reaches the user.

```codelaw
native: html/aria-label-misuse
```

## How to fix

1. If the element acts as a control, use the matching element (`<button>`, `<a href>`) or add a `role`.
2. Otherwise put the text in the content, where everyone can read it, and remove the attribute.

## Examples

### HTML

#### Don't

```html
<span aria-label="Close">×</span>
```

#### Do

```html
<button type="button" aria-label="Close">×</button>
```

---
id: html/element-name
title: "Element names are HTML elements or valid custom elements"
languages: [html]
fix: manual
fix_reason: "renaming an element changes its CSS and script hooks"
stability: preview
options:
  allow: { type: "string[]", default: [], description: "more element names to accept" }
ported_from: [html-validate/element-name]
since: 0.1.0
---

Every element is either an HTML element or a custom element whose name starts with a lowercase letter and contains a hyphen.

**Why:** An unknown element without a hyphen is a typo or an invalid custom element, and the browser treats it as an inline element with no meaning.

```codelaw
native: html/element-name
```

## How to fix

1. If the name is a typo of an HTML element, correct it.
2. If it is a custom element, give it a hyphenated name and rename its definition.
3. If a framework needs the name, add it to the `allow` option.

## Examples

### HTML

#### Don't

```html
<widget>Hi</widget>
```

#### Do

```html
<x-widget>Hi</x-widget>
```

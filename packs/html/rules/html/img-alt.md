---
id: html/img-alt
title: "<img> has alt text"
languages: [html]
fix: manual
fix_reason: "only a human can describe the image, or decide it is decorative"
stability: preview
options:
  allowEmpty: { type: "boolean", default: true, description: "accept `alt=\"\"` for decorative images" }
  alias: { type: "string[]", default: [], description: "other attributes that count as alt text, such as a framework binding" }
ported_from: [html-validate/wcag/h37, htmlhint/alt-require]
tags: [a11y]
since: 0.1.0
---

Every `<img>` has an `alt` attribute: a description, or `alt=""` for a purely decorative image.

**Why:** Screen readers read the file name of an image without `alt`; `alt` is also shown when the image fails to load.

```codelaw
native: html/img-alt
```

## How to fix

1. Describe what the image shows, or what it is for, in `alt`.
2. If the image is decorative, write `alt=""`.

## Examples

### HTML

#### Don't

```html
<img src="team.jpg">
```

#### Do

```html
<img src="team.jpg" alt="The team at the 2024 offsite">
<img src="divider.png" alt="">
```

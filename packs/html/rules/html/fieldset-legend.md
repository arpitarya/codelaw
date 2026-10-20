---
id: html/fieldset-legend
title: "<fieldset> has a legend"
languages: [html]
fix: manual
fix_reason: "the group's name needs a human"
stability: preview
ported_from: [html-validate/wcag/h71]
tags: [a11y]
since: 0.1.0
---

A `<fieldset>` has a `<legend>` child.

**Why:** The legend names the group of controls; screen readers announce it with each control in the group.

```codelaw
native: html/fieldset-legend
```

## How to fix

1. Add a `<legend>` as the fieldset's first child, naming the group.

## Examples

### HTML

#### Don't

```html
<fieldset>
  <label><input type="radio" name="size" value="s"> Small</label>
</fieldset>
```

#### Do

```html
<fieldset>
  <legend>Size</legend>
  <label><input type="radio" name="size" value="s"> Small</label>
</fieldset>
```

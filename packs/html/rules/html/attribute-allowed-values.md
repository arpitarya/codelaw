---
id: html/attribute-allowed-values
title: "Attributes have values HTML allows"
languages: [html]
fix: manual
fix_reason: "the intended value can only be guessed"
stability: preview
ported_from: [html-validate/attribute-allowed-values, htmlhint/src-not-empty]
since: 0.1.0
---

An attribute with a fixed set of values (`type`, `dir`, `loading`, …) uses one of them; a boolean attribute (`disabled`, `checked`) is written bare or with its own name; a numeric attribute holds a number.

**Why:** Browsers fall back to a default for a value they don't know, so a typo silently changes behaviour.

```codelaw
native: html/attribute-allowed-values
```

## How to fix

1. Check the attribute's allowed values (the message lists the one written).
2. Correct the value, or remove the attribute to get the default.

## Examples

### HTML

#### Don't

```html
<input type="emial" name="email">
<img src="a.png" alt="A" loading="later">
```

#### Do

```html
<input type="email" name="email">
<img src="a.png" alt="A" loading="lazy">
```

---
id: html/unrecognized-char-ref
title: "Character references exist and are terminated"
languages: [html]
fix: manual
fix_reason: "the intended character is not always clear"
stability: preview
options:
  ignoreCase: { type: "boolean", default: false, description: "compare names without case" }
  requireSemicolon: { type: "boolean", default: true, description: "report references without `;`" }
ported_from: [html-validate/unrecognized-char-ref]
since: 0.1.0
---

Every named character reference (`&name;`) is one HTML defines, and ends with `;`.

**Why:** An unknown reference is shown literally, and an unterminated one may be read with the text after it.

```codelaw
native: html/unrecognized-char-ref
```

## How to fix

1. Correct the name, or escape the `&` as `&amp;` if no reference was meant.
2. End the reference with `;`.

## Examples

### HTML

#### Don't

```html
<p>Price: 5&euros; &copy 2024</p>
```

#### Do

```html
<p>Price: 5&euro; &copy; 2024</p>
```

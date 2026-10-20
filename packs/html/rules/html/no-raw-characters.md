---
id: html/no-raw-characters
title: "Special characters are escaped"
languages: [html]
fix: auto
stability: preview
options:
  relaxed: { type: "boolean", default: false, description: "don't report `&`" }
ported_from: [html-validate/no-raw-characters, htmlhint/spec-char-escape]
since: 0.1.0
---

Text doesn't contain a raw `<` or an `&` that could be read as a character reference, and an unquoted attribute value doesn't contain `"`, `'`, `<`, `=`, `>`, `` ` `` or such an `&`.

**Why:** The parser can read them as markup: `&copy=1` becomes `©=1`, and `<` may start a tag.

```codelaw
native: html/no-raw-characters
```

## Examples

### HTML

#### Don't

```html
<p>Fish & chips</p>
```

#### Do

```html
<p>Fish &amp; chips</p>
```

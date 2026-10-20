---
id: html/void-content
title: "Void elements have no end tag"
languages: [html]
fix: suggest
fix_reason: "`</br>` is read as a second line break, so removing it changes the rendering"
stability: preview
ported_from: [html-validate/void-content]
since: 0.1.0
---

Void elements (`<br>`, `<img>`, `<input>`, `<meta>`, …) are written without an end tag.

**Why:** They can't have content; browsers ignore most such end tags, and read `</br>` and `</p>` as new elements.

```codelaw
native: html/void-content
```

## Examples

### HTML

#### Don't

```html
<input type="text" name="q"></input>
```

#### Do

```html
<input type="text" name="q">
```

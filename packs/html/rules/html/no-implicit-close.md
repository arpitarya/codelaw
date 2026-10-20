---
id: html/no-implicit-close
title: "Elements are closed explicitly"
languages: [html]
fix: suggest
fix_reason: "the end tag goes after the element's last content, so whitespace that followed it moves outside the element"
stability: preview
ported_from: [html-validate/no-implicit-close]
since: 0.1.0
---

Elements whose end tag HTML lets you leave out (`<li>`, `<p>`, `<td>`, `<option>`, `<head>`, …) are closed with it anyway.

**Why:** Implied end tags make nesting hard to read, and an element closed by the next tag is easy to break when markup moves.

```codelaw
native: html/no-implicit-close
```

## Examples

### HTML

#### Don't

```html
<ul>
  <li>One
  <li>Two
</ul>
```

#### Do

```html
<ul>
  <li>One</li>
  <li>Two</li>
</ul>
```

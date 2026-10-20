---
id: html/doctype-html
title: "The doctype is the HTML5 one"
languages: [html]
fix: suggest
fix_reason: "switching to standards mode can change the page's layout"
stability: preview
ported_from: [html-validate/doctype-html, htmlhint/doctype-html5]
since: 0.1.0
---

The doctype is `<!DOCTYPE html>`, not an HTML 4 or XHTML one.

**Why:** Legacy doctypes can put the browser in a quirks or "almost standards" mode, which lays pages out differently.

```codelaw
native: html/doctype-html
```

## Examples

### HTML

#### Don't

```html
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd">
<p>Hello</p>
```

#### Do

```html
<!DOCTYPE html>
<p>Hello</p>
```

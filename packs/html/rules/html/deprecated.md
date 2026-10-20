---
id: html/deprecated
title: "No obsolete elements"
languages: [html]
fix: manual
fix_reason: "the replacement — CSS, a semantic element, or nothing — depends on the intent"
stability: preview
ported_from: [html-validate/deprecated, htmlhint/tag-no-obsolete]
since: 0.1.0
---

The document uses no element HTML has made obsolete: `<center>`, `<font>`, `<marquee>`, `<frame>`, `<tt>` and the rest.

**Why:** Obsolete elements have no defined meaning, and some are rendered differently, or not at all, by current browsers.

```codelaw
native: html/deprecated
```

## How to fix

1. Read the message for the usual replacement.
2. Use that element, or a class and CSS, instead.

## Examples

### HTML

#### Don't

```html
<center>Welcome</center>
```

#### Do

```html
<div class="centered">Welcome</div>
```

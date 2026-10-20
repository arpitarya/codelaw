---
id: html/missing-doctype
title: "The document has a doctype"
languages: [html]
fix: suggest
fix_reason: "leaving quirks mode can change the page's layout"
stability: preview
ported_from: [html-validate/missing-doctype]
since: 0.1.0
---

A document starts with `<!DOCTYPE html>`.

**Why:** Without it browsers render in quirks mode, which lays the page out differently from the standard.

```codelaw
native: html/missing-doctype
```

## Examples

### HTML

#### Don't

```html
<html lang="en">
  <title>Home</title>
</html>
```

#### Do

```html
<!DOCTYPE html>
<html lang="en">
  <title>Home</title>
</html>
```

## Differences from the source

Only files with an `<html>` element are checked: a fragment or template partial is not a document, and gets no doctype of its own.

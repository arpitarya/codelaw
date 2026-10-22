---
id: html/no-utf8-bom
title: "No UTF-8 byte-order mark"
languages: [html]
fix: manual
fix_reason: "the mark is in the file's bytes, outside the text a fix can edit; the editor's encoding setting removes it"
stability: preview
ported_from: [html-validate/no-utf8-bom]
since: 0.1.0
---

An HTML file doesn't start with a UTF-8 byte-order mark (`EF BB BF`).

**Why:** UTF-8 needs no byte-order mark. Served or concatenated, the mark can show up as stray characters, or break a server-side include or a `<!DOCTYPE>` that must come first.

```codelaw
native: html/no-utf8-bom
```

## How to fix

1. Save the file as UTF-8 without a byte-order mark. In VS Code, click the encoding in the status bar, choose *Save with Encoding*, then *UTF-8*.
2. Or strip it from the command line: `sed -i '1s/^\xEF\xBB\xBF//' <file>` (GNU sed).

## Examples

### HTML

#### Don't

```html
﻿<!DOCTYPE html>
<html lang="en"><title>Home</title></html>
```

#### Do

```html
<!DOCTYPE html>
<html lang="en"><title>Home</title></html>
```

## Differences from the source

Only the file itself is checked: an HTML region inside another file (a Markdown fence, a template) holds no byte-order mark of its own.

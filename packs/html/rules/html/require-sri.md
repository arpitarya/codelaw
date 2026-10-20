---
id: html/require-sri
title: "Scripts and stylesheets carry an integrity hash"
languages: [html]
fix: manual
fix_reason: "the hash must be computed from the exact file served"
stability: preview
options:
  target: { type: "string", default: "all", description: "`all` checks every resource; `crossorigin` only those loaded from another origin" }
ported_from: [html-validate/require-sri]
since: 0.1.0
---

Every `<script src>` and `<link rel="stylesheet">` (and `modulepreload`, and `preload` of a script or style) has an `integrity` attribute.

**Why:** Subresource Integrity makes the browser refuse a file that changed on the CDN or in transit.

```codelaw
native: html/require-sri
```

## How to fix

1. Compute the file's hash, for example `openssl dgst -sha384 -binary lib.js | openssl base64 -A`.
2. Add `integrity="sha384-…"` and `crossorigin="anonymous"`.

## Examples

### HTML

#### Don't

```html
<script src="https://cdn.example.com/lib.js"></script>
```

#### Do

```html
<script src="https://cdn.example.com/lib.js" integrity="sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC" crossorigin="anonymous"></script>
```

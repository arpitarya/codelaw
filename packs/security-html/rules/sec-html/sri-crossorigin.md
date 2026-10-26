---
id: sec-html/sri-crossorigin
title: "A script or stylesheet with an integrity hash has crossorigin"
languages: [html]
fix: suggest
fix_reason: "`anonymous` is right unless the file needs credentials; the suggestion assumes it does not"
stability: preview
security: { cwe: [353], owasp: ["A08:2025"], severity: low }
since: 0.2.0
---

A `<script>` or `<link>` from another origin that has `integrity` also has `crossorigin`.

**Why:** Without `crossorigin` the browser fetches the file without CORS and cannot read it to check the hash, so the `integrity` protects nothing.

```codelaw
native: sec-html/sri-crossorigin
```

## Examples

### HTML

#### Don't

```html
<script src="https://cdn.example.com/lib.js" integrity="sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC"></script>
```

#### Do

```html
<script src="https://cdn.example.com/lib.js" integrity="sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC" crossorigin="anonymous"></script>
```

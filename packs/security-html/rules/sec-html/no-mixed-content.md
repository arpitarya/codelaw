---
id: sec-html/no-mixed-content
title: "Subresources and forms load over HTTPS"
languages: [html]
fix: suggest
fix_reason: "the host may not serve HTTPS; check that it does before switching"
stability: preview
security: { cwe: [319], owasp: ["A04:2025"], severity: medium }
since: 0.2.0
---

A `src`, a form `action`, a stylesheet or script `href` and a download link never use `http://`.

**Why:** A resource fetched over plain HTTP can be read and changed by anyone on the network, and a browser blocks or warns about it on an HTTPS page.

```codelaw
native: sec-html/no-mixed-content
```

## Examples

### HTML

#### Don't

```html
<script src="http://cdn.example.com/lib.js"></script>
```

#### Do

```html
<script src="https://cdn.example.com/lib.js"></script>
```

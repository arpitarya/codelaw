---
id: sec-html/no-external-base
title: "<base> stays on this origin"
languages: [html]
fix: manual
fix_reason: "which origin the page may use is a decision for its owner"
stability: preview
security: { owasp: ["A05:2025"], severity: medium }
since: 0.2.0
---

A `<base href>` is a path, not a URL on another origin.

**Why:** Every relative URL, form action and script on the page resolves against `<base>`, so one that points elsewhere hands them all to that site.

```codelaw
native: sec-html/no-external-base
```

## How to fix

1. Delete the `<base>` and write the URLs you need in full.
2. Or make its `href` a path on this origin, such as `/app/`.

## Examples

### HTML

#### Don't

```html
<base href="https://cdn.example.com/app/">
```

#### Do

```html
<base href="/app/">
```

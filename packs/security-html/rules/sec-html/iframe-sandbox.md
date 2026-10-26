---
id: sec-html/iframe-sandbox
title: "An iframe is sandboxed"
languages: [html]
fix: suggest
fix_reason: "the framed page may need `allow-*` tokens, which only its author knows"
stability: preview
security: { cwe: [1021], owasp: ["A02:2025"], severity: medium }
since: 0.2.0
---

An `<iframe>` has a `sandbox` attribute, and one that allows scripts does not also allow the same origin.

**Why:** A framed page without `sandbox` runs with every power a top-level page has; `allow-scripts` with `allow-same-origin` lets it remove its own sandbox.

```codelaw
native: sec-html/iframe-sandbox
```

## Examples

### HTML

#### Don't

```html
<iframe src="/widget"></iframe>
```

#### Do

```html
<iframe sandbox src="/widget"></iframe>
```

---
id: sec-html/csp-meta-unsafe
title: "A meta Content Security Policy does not allow unsafe script"
languages: [html]
fix: manual
fix_reason: "the page's scripts have to be given nonces or hashes, or moved to files"
stability: preview
security: { cwe: [693], owasp: ["A02:2025"], severity: medium }
since: 0.2.0
---

A `<meta http-equiv="Content-Security-Policy">` does not allow `'unsafe-inline'`, `'unsafe-eval'`, `*` or `data:` as a script source.

**Why:** Each of them lets injected script run, which is what the policy exists to stop.

```codelaw
native: sec-html/csp-meta-unsafe
```

## How to fix

1. Move inline script into files, or give it a nonce or a hash (`'nonce-…'`, `'sha256-…'`).
2. Remove `'unsafe-eval'` and replace `eval`, `new Function` and string timers.
3. List the origins scripts load from instead of `*` or `data:`.

## Examples

### HTML

#### Don't

```html
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'unsafe-inline'">
```

#### Do

```html
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'nonce-r4nd0m'">
```

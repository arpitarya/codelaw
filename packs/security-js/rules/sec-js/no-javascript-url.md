---
id: sec-js/no-javascript-url
title: "No javascript: URL, no navigation to a URL taken from the address"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which targets a page may navigate to is a decision for the author"
stability: preview
ported_from: [eslint/no-script-url]
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

A `javascript:` URL is not written into `location` or `href`, and the address, the referrer or `window.name` is not assigned to `location` or passed to `location.assign` and `location.replace`.

**Why:** A `javascript:` URL runs its script in the page when it is followed, and a navigation target taken from the address lets a link choose where the page goes or what it runs (CWE-79).

```codelaw
match:
  any:
    - kind: assignment_expression
      regex: '^[\w$.\[\]"'']*(location|href)\s*=\s*["''`]\s*(javascript|vbscript|data:text/html)'
    - kind: call_expression
      regex: '^[\w$.]*location\.(assign|replace)\(\s*["''`]\s*(javascript|vbscript|data:text/html)'
    - kind: call_expression
      regex: '^[\w$.]*setAttribute\(\s*["'']href["'']\s*,\s*["''`]\s*(javascript|vbscript)'
    - kind: assignment_expression
      regex: '^[\w$.]*(location|href)\s*=\s*(window\.|document\.)?(location\.(hash|search|href)|URL|documentURI|baseURI|referrer|name)\b'
    - kind: call_expression
      regex: '^[\w$.]*location\.(assign|replace)\(\s*(window\.|document\.)?(location\.(hash|search|href)|URL|documentURI|baseURI|referrer|name)\b'
message: "This navigation target can run script or is taken from the address"
```

## How to fix

1. Link to a path or an `https:` URL; put script in an event handler, not in the URL.
2. Parse the target with `new URL(value, location.origin)` and check `protocol` and `origin` against an allow-list before navigating.

## Examples

### JavaScript

#### Don't

```js
window.location.href = "javascript:" + code;
window.location = location.hash.slice(1);
```

#### Do

```js
window.location.href = "/next";
window.location = new URL(location.hash.slice(1), location.origin).pathname;
```

### JSX

#### Don't

```jsx
window.location.href = "javascript:" + code;
window.location = location.hash.slice(1);
```

#### Do

```jsx
window.location.href = "/next";
window.location = new URL(location.hash.slice(1), location.origin).pathname;
```

### TypeScript

#### Don't

```ts
window.location.href = "javascript:" + code;
window.location = location.hash.slice(1);
```

#### Do

```ts
window.location.href = "/next";
window.location = new URL(location.hash.slice(1), location.origin).pathname;
```

### TSX

#### Don't

```tsx
window.location.href = "javascript:" + code;
window.location = location.hash.slice(1);
```

#### Do

```tsx
window.location.href = "/next";
window.location = new URL(location.hash.slice(1), location.origin).pathname;
```

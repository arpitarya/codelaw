---
id: sec-js/no-bad-tag-filter
title: "No regular expression that filters HTML tags"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which tags and attributes may stay is a decision for the author"
stability: preview
security: { cwe: [116, 185], owasp: ["A05:2025"], severity: medium }
since: 0.3.0
---

A regular expression is not used to strip or test HTML tags (`/<script>/`, `/<[^>]*>/`); a sanitizer is.

**Why:** HTML is not regular: `<script >`, `<SCRIPT>`, `<scr<script>ipt>` and a tag split across lines slip past a pattern, and what is left still runs (CWE-116, CWE-185).

```codelaw
match:
  all:
    - pattern: $S.replace($RE, $R)
    - regex: '\.replace\(\s*/<\\?/?[A-Za-z\[(]'
message: "A regular expression cannot reliably remove HTML tags"
```

## How to fix

1. Sanitize with a library built for it: `DOMPurify.sanitize(html)` in the browser, `sanitize-html` on the server.
2. To show markup as text, escape it (`&lt;`) instead of removing it, or write it with `textContent`.

## Examples

### JavaScript

#### Don't

```js
const clean = input.replace(/<script>/gi, "");
```

#### Do

```js
const clean = DOMPurify.sanitize(input);
```

### JSX

#### Don't

```jsx
const clean = input.replace(/<script>/gi, "");
```

#### Do

```jsx
const clean = DOMPurify.sanitize(input);
```

### TypeScript

#### Don't

```ts
const clean = input.replace(/<script>/gi, "");
```

#### Do

```ts
const clean = DOMPurify.sanitize(input);
```

### TSX

#### Don't

```tsx
const clean = input.replace(/<script>/gi, "");
```

#### Do

```tsx
const clean = DOMPurify.sanitize(input);
```

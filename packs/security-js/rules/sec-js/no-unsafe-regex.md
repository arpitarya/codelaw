---
id: sec-js/no-unsafe-regex
title: "No regular expression that can run for ever, no regular expression built from a request"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the pattern should accept is a decision for the author"
stability: preview
ported_from: [eslint-plugin-security/detect-unsafe-regex, eslint-plugin-security/detect-non-literal-regexp]
security: { cwe: [1333], owasp: ["A06:2025"], severity: medium }
since: 0.3.0
---

A regular expression has no quantified group that itself holds a quantifier (`(a+)+`), and `new RegExp(…)` is not built from request data.

**Why:** A nested quantifier gives the engine exponentially many ways to match a near-miss, so a short string freezes the process (ReDoS, CWE-1333); a pattern the client writes does the same on purpose.

```codelaw
match:
  any:
    - kind: regex
      regex: '\((\?[:=!])?[^()]*[+*][^()]*\)[+*]|\((\?:)?[^()]*[+*][^()]*\)\{\d+,\}'
    - all:
        - pattern: new RegExp($$$A)
        - regex: '\b(req|request)\.(params|query|body|headers)\b|\bctx\.(query|params|request\.(query|body))\b'
    - all:
        - pattern: RegExp($$$B)
        - regex: '\b(req|request)\.(params|query|body|headers)\b|\bctx\.(query|params|request\.(query|body))\b'
message: "This regular expression can take exponential time, or is written by the client"
```

## How to fix

1. Rewrite the pattern so no two parts can match the same text: `(a+)+` is `a+`.
2. Escape request data before it goes into a pattern: `value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")`.
3. Cap the length of the input, or use an engine without backtracking (`re2`).

## Examples

### JavaScript

#### Don't

```js
const valid = /^(a+)+$/.test(input);
```

#### Do

```js
const valid = /^a+$/.test(input);
```

### JSX

#### Don't

```jsx
const valid = /^(a+)+$/.test(input);
```

#### Do

```jsx
const valid = /^a+$/.test(input);
```

### TypeScript

#### Don't

```ts
const valid = /^(a+)+$/.test(input);
```

#### Do

```ts
const valid = /^a+$/.test(input);
```

### TSX

#### Don't

```tsx
const valid = /^(a+)+$/.test(input);
```

#### Do

```tsx
const valid = /^a+$/.test(input);
```

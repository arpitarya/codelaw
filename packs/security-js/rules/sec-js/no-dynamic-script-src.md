---
id: sec-js/no-dynamic-script-src
title: "No script or frame source taken from a value that is not a literal"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which sources the page may load is a decision for the author"
stability: preview
security: { cwe: [829], owasp: ["A08:2025"], severity: medium }
since: 0.3.0
---

The `src` of a `<script>`, `<iframe>`, `<frame>` or `<embed>` element is not set from a value that is not a literal.

**Why:** A script source the page computes is code it loads and runs; a value an attacker influences loads code of their choosing (CWE-829).

```codelaw
match:
  any:
    - all:
        - pattern: $E.src = $V
        - regex: '(?i)^[\w$.]*(script|iframe|frame|embed)[\w$]*\.src\s*=[^=]'
    - all:
        - pattern: $E.setAttribute("src", $W)
        - regex: '(?i)^[\w$.]*(script|iframe|frame|embed)[\w$]*\.setAttribute'
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
  W: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "The source of this script or frame is not a literal; whatever sets it chooses the code that loads"
```

## How to fix

1. Use a literal URL, or pick one from a table of literal URLs by a key the code controls.
2. If the URL must come from configuration, check its origin against an allow-list and add `integrity` (Subresource Integrity) to the element.

## Examples

### JavaScript

#### Don't

```js
const script = document.createElement("script");
script.src = scriptUrl;
```

#### Do

```js
const script = document.createElement("script");
script.src = "https://cdn.example.com/lib.js";
```

### JSX

#### Don't

```jsx
const script = document.createElement("script");
script.src = scriptUrl;
```

#### Do

```jsx
const script = document.createElement("script");
script.src = "https://cdn.example.com/lib.js";
```

### TypeScript

#### Don't

```ts
const script = document.createElement("script");
script.src = scriptUrl;
```

#### Do

```ts
const script = document.createElement("script");
script.src = "https://cdn.example.com/lib.js";
```

### TSX

#### Don't

```tsx
const script = document.createElement("script");
script.src = scriptUrl;
```

#### Do

```tsx
const script = document.createElement("script");
script.src = "https://cdn.example.com/lib.js";
```

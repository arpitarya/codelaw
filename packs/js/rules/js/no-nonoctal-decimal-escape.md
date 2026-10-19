---
id: js/no-nonoctal-decimal-escape
title: "No \\8 or \\9 escapes in strings"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether the digit or a literal backslash was meant needs the author"
stability: preview
ported_from: [eslint/no-nonoctal-decimal-escape]
since: 0.1.0
---

A string contains no `\8` or `\9` escape.

**Why:** `\8` and `\9` are legacy escapes that just mean the digits, and are errors in template
literals and strict code.

```codelaw
match:
  kind: escape_sequence
  regex: ^\\[89]$
message: "`$$MATCH` is not an escape; it's the digit itself"
```

## How to fix

1. Write the digit without the backslash, or `\\8` for a backslash and a digit.

## Examples

### JavaScript

#### Don't

```js
const msg = "version \8";
```

#### Do

```js
const msg = "version 8";
```

### JSX

#### Don't

```jsx
const msg = "version \8";
```

#### Do

```jsx
const msg = "version 8";
```

### TypeScript

#### Don't

```ts
const msg = "version \8";
```

#### Do

```ts
const msg = "version 8";
```

### TSX

#### Don't

```tsx
const msg = "version \8";
```

#### Do

```tsx
const msg = "version 8";
```

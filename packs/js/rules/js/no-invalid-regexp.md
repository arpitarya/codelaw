---
id: js/no-invalid-regexp
title: "RegExp() is given a valid pattern and flags"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the intended pattern needs the author"
stability: preview
ported_from: [eslint/no-invalid-regexp]
since: 0.1.0
---

A `RegExp(...)` or `new RegExp(...)` whose pattern and flags are string literals is valid: brackets
and groups close, quantifiers follow something, and the flags are known and not repeated.

**Why:** An invalid pattern throws a SyntaxError when that line runs, not when the file loads.

```codelaw
native: js/no-invalid-regexp
```

## How to fix

1. Correct the pattern or the flags the message names.

## Examples

### JavaScript

#### Don't

```js
const re = new RegExp("[a-z");
```

#### Do

```js
const re = new RegExp("[a-z]");
```

### JSX

#### Don't

```jsx
const re = new RegExp("[a-z");
```

#### Do

```jsx
const re = new RegExp("[a-z]");
```

### TypeScript

#### Don't

```ts
const re = new RegExp("[a-z");
```

#### Do

```ts
const re = new RegExp("[a-z]");
```

### TSX

#### Don't

```tsx
const re = new RegExp("[a-z");
```

#### Do

```tsx
const re = new RegExp("[a-z]");
```

## Differences from the source

Only structural problems are found: unbalanced brackets and groups, misplaced quantifiers, out-of-
order ranges, invalid group syntax and flags.

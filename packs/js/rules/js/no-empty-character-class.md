---
id: js/no-empty-character-class
title: "No empty character classes in regular expressions"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which characters the class should hold needs the author"
stability: preview
ported_from: [eslint/no-empty-character-class]
since: 0.1.0
---

A regular expression contains no empty character class `[]`; `[^]`, which matches any character, is
fine.

**Why:** `[]` matches no character at all, so the whole pattern can never match.

```codelaw
native: js/no-empty-character-class
```

## How to fix

1. Put the characters the class should match inside it.
2. If `[]` was meant literally, escape it: `\[\]`.

## Examples

### JavaScript

#### Don't

```js
const re = /^abc[]/;
```

#### Do

```js
const re = /^abc[a-z]/;
```

```js
const brackets = /^\s*[{[]/;
```

### JSX

#### Don't

```jsx
const re = /^abc[]/;
```

#### Do

```jsx
const re = /^abc[a-z]/;
```

```jsx
const brackets = /^\s*[{[]/;
```

### TypeScript

#### Don't

```ts
const re = /^abc[]/;
```

#### Do

```ts
const re = /^abc[a-z]/;
```

```ts
const brackets = /^\s*[{[]/;
```

### TSX

#### Don't

```tsx
const re = /^abc[]/;
```

#### Do

```tsx
const re = /^abc[a-z]/;
```

```tsx
const brackets = /^\s*[{[]/;
```

---
id: js/no-useless-backreference
title: "No backreference that can only match the empty string"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the reference was meant to repeat needs the author"
stability: preview
ported_from: [eslint/no-useless-backreference]
since: 0.1.0
---

A backreference such as `\1` or `\k<name>` appears after its group closes, in the same alternative,
and not into a negative lookaround.

**Why:** A backreference that comes first, sits inside its own group or in another alternative
always matches the empty string.

```codelaw
native: js/no-useless-backreference
```

## How to fix

1. Move the backreference after the group it refers to.
2. Or remove it if it was meant to be literal text.

## Examples

### JavaScript

#### Don't

```js
const re = /\1(a)/;
```

#### Do

```js
const re = /(a)\1/;
```

### JSX

#### Don't

```jsx
const re = /\1(a)/;
```

#### Do

```jsx
const re = /(a)\1/;
```

### TypeScript

#### Don't

```ts
const re = /\1(a)/;
```

#### Do

```ts
const re = /(a)\1/;
```

### TSX

#### Don't

```tsx
const re = /\1(a)/;
```

#### Do

```tsx
const re = /(a)\1/;
```

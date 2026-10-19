---
id: js/no-regex-spaces
title: "No runs of spaces in a regular expression"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-regex-spaces]
since: 0.1.0
---

A regular expression writes several spaces as ` {n}`, not as a run of spaces.

**Why:** A run of spaces is impossible to count by eye.

```codelaw
native: js/no-regex-spaces
```

## Examples

### JavaScript

#### Don't

```js
const re = /a   b/;
```

#### Do

```js
const re = /a {3}b/;
```

### JSX

#### Don't

```jsx
const re = /a   b/;
```

#### Do

```jsx
const re = /a {3}b/;
```

### TypeScript

#### Don't

```ts
const re = /a   b/;
```

#### Do

```ts
const re = /a {3}b/;
```

### TSX

#### Don't

```tsx
const re = /a   b/;
```

#### Do

```tsx
const re = /a {3}b/;
```

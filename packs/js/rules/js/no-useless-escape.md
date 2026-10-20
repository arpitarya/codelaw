---
id: js/no-useless-escape
title: "No backslash that changes nothing"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-useless-escape]
since: 0.1.0
---

A string or template literal escapes only characters that need it: quotes that match the delimiter,
`\\`, line breaks and the escape letters.

**Why:** A useless escape suggests the author meant something else, and makes the string harder to
read.

```codelaw
native: js/no-useless-escape
```

## Examples

### JavaScript

#### Don't

```js
const path = "a\.b";
```

#### Do

```js
const path = "a.b";
```

### JSX

#### Don't

```jsx
const path = "a\.b";
```

#### Do

```jsx
const path = "a.b";
```

### TypeScript

#### Don't

```ts
const path = "a\.b";
```

#### Do

```ts
const path = "a.b";
```

### TSX

#### Don't

```tsx
const path = "a\.b";
```

#### Do

```tsx
const path = "a.b";
```

## Differences from the source

Escapes inside regular expressions are not checked yet. A tagged template (`String.raw`, `html`) is not checked: its tag sees the raw text, backslashes included, as in ESLint.

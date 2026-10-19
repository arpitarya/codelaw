---
id: react/jsx-no-duplicate-props
title: "No prop given twice on one element"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "which value should win needs the author"
stability: preview
ported_from: [eslint-plugin-react/jsx-no-duplicate-props]
since: 0.1.0
---

A JSX element gives each prop once.

**Why:** The last value silently wins, so the first is dead code or a mistake.

```codelaw
native: react/jsx-no-duplicate-props
```

## How to fix

1. Keep the value that is right and remove the other.

## Examples

### JavaScript

#### Don't

```js
const a = <input type="text" type="email" />;
```

#### Do

```js
const a = <input type="email" />;
```

### JSX

#### Don't

```jsx
const a = <input type="text" type="email" />;
```

#### Do

```jsx
const a = <input type="email" />;
```

### TSX

#### Don't

```tsx
const a = <input type="text" type="email" />;
```

#### Do

```tsx
const a = <input type="email" />;
```

---
id: js/no-irregular-whitespace
title: "No irregular whitespace outside strings"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-irregular-whitespace]
since: 0.1.0
---

Code contains only ordinary spaces, tabs and line breaks as whitespace; no-break spaces, zero-width
spaces and the like appear only inside strings.

**Why:** Irregular whitespace is invisible in most editors and breaks tokens in ways that are hard
to find.

```codelaw
native: js/no-irregular-whitespace
```

## Examples

### JavaScript

#### Don't

```js
const total = 1;
```

#### Do

```js
const total = 1;
```

### JSX

#### Don't

```jsx
const total = 1;
```

#### Do

```jsx
const total = 1;
```

### TypeScript

#### Don't

```ts
const total = 1;
```

#### Do

```ts
const total = 1;
```

### TSX

#### Don't

```tsx
const total = 1;
```

#### Do

```tsx
const total = 1;
```

## Differences from the source

Inside templates, regular expressions and JSX text the character is part of a value, so it is
reported but not rewritten.

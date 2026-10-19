---
id: js/no-control-regex
title: "No control characters in regular expressions"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which character was meant needs the author"
stability: preview
ported_from: [eslint/no-control-regex]
since: 0.1.0
---

A regular expression literal doesn't match ASCII control characters (`\x00`–`\x1F`).

**Why:** Control characters rarely appear in text, so matching them is usually an escaping mistake.

```codelaw
match:
  kind: regex_pattern
  regex: \\x[01][0-9a-fA-F]|\\u00[01][0-9a-fA-F]|[\x00-\x1f]
message: "Control characters in a regular expression are almost always a mistake"
```

## How to fix

1. Use the named escape (`\t`, `\n`, `\f`) if one was meant.
2. Otherwise remove the control character from the pattern.

## Examples

### JavaScript

#### Don't

```js
const re = /\x0C/;
```

#### Do

```js
const re = /\f/;
```

### JSX

#### Don't

```jsx
const re = /\x0C/;
```

#### Do

```jsx
const re = /\f/;
```

### TypeScript

#### Don't

```ts
const re = /\x0C/;
```

#### Do

```ts
const re = /\f/;
```

### TSX

#### Don't

```tsx
const re = /\x0C/;
```

#### Do

```tsx
const re = /\f/;
```

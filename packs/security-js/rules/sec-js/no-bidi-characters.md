---
id: sec-js/no-bidi-characters
title: "No hidden bidirectional control characters in code"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a comment is removed whole, since its hidden text cannot be told from the rest; a string or regular expression with such a character has no mechanical fix"
stability: preview
ported_from: [eslint-plugin-security/detect-bidi-characters]
security: { owasp: ["A08:2025"], severity: high }
since: 0.3.0
---

Source code, comments, strings and regular expressions hold no bidirectional control characters (U+202A–U+202E, U+2066–U+2069).

**Why:** These characters reorder how an editor and a review tool show the code, so what a reviewer reads is not what the compiler runs: the Trojan Source attack (CVE-2021-42574).

```codelaw
match:
  any:
    - pattern: $C
      kind: comment
      regex: '[\u202A-\u202E\u2066-\u2069]'
    - kind: string
      regex: '[\u202A-\u202E\u2066-\u2069]'
    - kind: template_string
      regex: '[\u202A-\u202E\u2066-\u2069]'
    - kind: regex
      regex: '[\u202A-\u202E\u2066-\u2069]'
message: "This code holds a hidden bidirectional control character, which can make it read differently from what it does"
fix:
  - remove: $C
```

## How to fix

1. Delete the character: show hidden characters in the editor (or run `git diff --word-diff-regex=.`) and remove each one.
2. If a string needs right-to-left text, write the characters as escapes (`\u202E`) so the source does not hide them.

## Examples

### JavaScript

#### Don't

```js
const level = readLevel();
// check the access level ‮ ⁦ admin
```

#### Do

```js
const level = readLevel();
```

#### Fixed

```js
const level = readLevel();
```

### JSX

#### Don't

```jsx
const level = readLevel();
// check the access level ‮ ⁦ admin
```

#### Do

```jsx
const level = readLevel();
```

#### Fixed

```jsx
const level = readLevel();
```

### TypeScript

#### Don't

```ts
const level = readLevel();
// check the access level ‮ ⁦ admin
```

#### Do

```ts
const level = readLevel();
```

#### Fixed

```ts
const level = readLevel();
```

### TSX

#### Don't

```tsx
const level = readLevel();
// check the access level ‮ ⁦ admin
```

#### Do

```tsx
const level = readLevel();
```

#### Fixed

```tsx
const level = readLevel();
```

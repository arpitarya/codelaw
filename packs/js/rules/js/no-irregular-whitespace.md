---
id: js/no-irregular-whitespace
title: "No irregular whitespace in templates, regular expressions, JSX text or comments"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "the escape keeps a template's cooked value but changes the raw string a tagged template such as `String.raw` sees"
stability: preview
ported_from: [eslint/no-irregular-whitespace]
since: 0.1.0
---

Templates, regular expressions and JSX text spell a no-break space, zero-width space or other
irregular whitespace as an escape (`\u00A0`, `&#x00A0;`), and comments use ordinary spaces.

**Why:** Irregular whitespace is invisible in most editors. In a template, regular expression or JSX
text it silently becomes part of the value, so the text looks right and compares wrong.

```codelaw
native: js/no-irregular-whitespace
```

## Examples

### JavaScript

#### Don't

```js
const label = `Total: 1`;
```

#### Do

```js
const label = `Total:\u00A01`;
```

### JSX

#### Don't

```jsx
const label = <p>Total: 1</p>;
```

#### Do

```jsx
const label = <p>Total:&#x00A0;1</p>;
```

### TypeScript

#### Don't

```ts
const label = `Total: 1`;
```

#### Do

```ts
const label = `Total:\u00A01`;
```

### TSX

#### Don't

```tsx
const label = <p>Total: 1</p>;
```

#### Do

```tsx
const label = <p>Total:&#x00A0;1</p>;
```

## Differences from the source

- **Between tokens the formatter owns it.** ESLint also reports irregular whitespace between tokens;
  here that is layout, and `codelaw format` replaces it with a space (SR-FORMATTER decision 3), so
  this rule does not report it.
- **Strings are skipped**, as ESLint's `skipStrings` default does.
- **The fix escapes, it doesn't delete:** `\uXXXX` in a template or regular expression, `&#xXXXX;`
  in JSX text, so the value is unchanged; a comment gets an ordinary space.

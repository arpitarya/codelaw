---
id: react/jsx-no-comment-textnodes
title: "No comment syntax as text inside JSX"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "whether the text is a comment or content needs the author"
stability: preview
ported_from: [eslint-plugin-react/jsx-no-comment-textnodes]
since: 0.1.0
---

Comments inside JSX are written as `{/* … */}`; `// …` and `/* … */` in JSX text are not.

**Why:** In JSX, `// note` is text that ends up on the page.

```codelaw
match:
  kind: jsx_text
  regex: ^\s*(//|/\*)
message: "This is rendered as text, not a comment; write `{/* … */}`"
```

## How to fix

1. Wrap the comment in braces: `{/* … */}`.

## Examples

### JavaScript

#### Don't

```js
const a = <div>
  // the title
  <h1>Hi</h1>
</div>;
```

#### Do

```js
const a = <div>
  {/* the title */}
  <h1>Hi</h1>
</div>;
```

### JSX

#### Don't

```jsx
const a = <div>
  // the title
  <h1>Hi</h1>
</div>;
```

#### Do

```jsx
const a = <div>
  {/* the title */}
  <h1>Hi</h1>
</div>;
```

### TSX

#### Don't

```tsx
const a = <div>
  // the title
  <h1>Hi</h1>
</div>;
```

#### Do

```tsx
const a = <div>
  {/* the title */}
  <h1>Hi</h1>
</div>;
```

---
id: react/jsx-no-target-blank
title: "Links that open a new tab set rel=\"noreferrer\""
languages: [javascript, jsx, tsx]
fix: auto
stability: preview
ported_from: [eslint-plugin-react/jsx-no-target-blank]
since: 0.1.0
---

An `<a target="_blank">` with an `href` also has `rel="noreferrer"`.

**Why:** Without it, older browsers give the opened page a handle on this one (`window.opener`), and
the referrer leaks.

```codelaw
match:
  kind: jsx_attribute
  regex: ^target=["']_blank["']$
  inside:
    stopBy: neighbor
    any: [{ kind: jsx_opening_element }, { kind: jsx_self_closing_element }]
    all:
      - has: { field: name, regex: ^a$ }
      - has: { stopBy: neighbor, kind: jsx_attribute, regex: ^href= }
      - not: { has: { stopBy: neighbor, kind: jsx_attribute, regex: ^rel= } }
message: "A link with `target=\"_blank\"` gives the new page access to this one; add `rel=\"noreferrer\"`"
fix:
  - insert-after: ' rel="noreferrer"'
```

## Examples

### JavaScript

#### Don't

```js
const a = <a href={url} target="_blank">Docs</a>;
```

#### Do

```js
const a = <a href={url} target="_blank" rel="noreferrer">Docs</a>;
```

### JSX

#### Don't

```jsx
const a = <a href={url} target="_blank">Docs</a>;
```

#### Do

```jsx
const a = <a href={url} target="_blank" rel="noreferrer">Docs</a>;
```

### TSX

#### Don't

```tsx
const a = <a href={url} target="_blank">Docs</a>;
```

#### Do

```tsx
const a = <a href={url} target="_blank" rel="noreferrer">Docs</a>;
```

## Differences from the source

A link that already has a `rel` is not checked.

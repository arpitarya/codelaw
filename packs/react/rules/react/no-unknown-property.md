---
id: react/no-unknown-property
title: "DOM elements use React's prop names, not HTML attribute names"
languages: [javascript, jsx, tsx]
fix: auto
stability: preview
ported_from: [eslint-plugin-react/no-unknown-property]
since: 0.1.0
---

A DOM element in JSX uses React's names for attributes: `className` not `class`, `htmlFor` not
`for`, `onClick` not `onclick`, `tabIndex` not `tabindex`.

**Why:** React ignores or warns about the HTML spellings, so the attribute silently does nothing.

```codelaw
native: react/no-unknown-property
```

## Examples

### JavaScript

#### Don't

```js
const a = <input tabindex="1" onclick={go} />;
```

#### Do

```js
const a = <input tabIndex="1" onClick={go} />;
```

### JSX

#### Don't

```jsx
const a = <input tabindex="1" onclick={go} />;
```

#### Do

```jsx
const a = <input tabIndex="1" onClick={go} />;
```

### TSX

#### Don't

```tsx
const a = <label class="x" for="name" onclick={go}>Name</label>;
```

#### Do

```tsx
const a = <label className="x" htmlFor="name" onClick={go}>Name</label>;
```

## Differences from the source

Only HTML spellings React renames are checked (the pack's `react/dom-property-aliases` data table);
unknown attributes in general are not. In `.js` and `.jsx` files the JavaScript grammar can't parse
`class=` and `for=` as JSX attributes (they are reserved words), so those two are found only in
`.tsx`.

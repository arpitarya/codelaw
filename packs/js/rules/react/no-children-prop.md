---
id: react/no-children-prop
title: "Children are passed as children, not as a prop"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "moving the children changes the element's shape by hand"
stability: preview
ported_from: [eslint-plugin-react/no-children-prop]
since: 0.1.0
---

Children go between an element's tags (or as the third argument of `createElement`), not in a
`children` prop.

**Why:** Passing `children` as a prop works, but hides the element's content where readers don't
look for it.

```codelaw
match:
  any:
    - kind: jsx_attribute
      regex: ^children\b
    - pattern: 'createElement($T, { children: $C, $$$REST })'
    - pattern: 'React.createElement($T, { children: $C, $$$REST })'
message: "Pass children between the tags, not as the `children` prop"
```

## How to fix

1. Move the value between the opening and closing tags.

## Examples

### JavaScript

#### Don't

```js
const a = <Panel children={<p>Hi</p>} />;
```

#### Do

```js
const a = <Panel>
  <p>Hi</p>
</Panel>;
```

### JSX

#### Don't

```jsx
const a = <Panel children={<p>Hi</p>} />;
```

#### Do

```jsx
const a = <Panel>
  <p>Hi</p>
</Panel>;
```

### TSX

#### Don't

```tsx
const a = <Panel children={<p>Hi</p>} />;
```

#### Do

```tsx
const a = <Panel>
  <p>Hi</p>
</Panel>;
```

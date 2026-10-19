---
id: react/no-danger-with-children
title: "No dangerouslySetInnerHTML together with children"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "which content the element should have needs the author"
stability: preview
ported_from: [eslint-plugin-react/no-danger-with-children]
since: 0.1.0
---

An element with `dangerouslySetInnerHTML` has no children.

**Why:** React throws when an element has both, since it can't know which content wins.

```codelaw
match:
  any:
    - kind: jsx_element
      all:
        - has: { field: open_tag, has: { stopBy: neighbor, kind: jsx_attribute, regex: ^dangerouslySetInnerHTML } }
        - has:
            stopBy: neighbor
            any:
              - kind: jsx_element
              - kind: jsx_self_closing_element
              - kind: jsx_expression
              - kind: jsx_text
                regex: \S
    - kind: jsx_self_closing_element
      all:
        - has: { stopBy: neighbor, kind: jsx_attribute, regex: ^dangerouslySetInnerHTML }
        - has: { stopBy: neighbor, kind: jsx_attribute, regex: ^children\b }
message: "An element can't have both `dangerouslySetInnerHTML` and children"
```

## How to fix

1. Keep either the HTML or the children, not both.

## Examples

### JavaScript

#### Don't

```js
const a = <div dangerouslySetInnerHTML={{ __html: html }}>Fallback</div>;
```

#### Do

```js
const a = <div dangerouslySetInnerHTML={{ __html: html }} />;
```

### JSX

#### Don't

```jsx
const a = <div dangerouslySetInnerHTML={{ __html: html }}>Fallback</div>;
```

#### Do

```jsx
const a = <div dangerouslySetInnerHTML={{ __html: html }} />;
```

### TSX

#### Don't

```tsx
const a = <div dangerouslySetInnerHTML={{ __html: html }}>Fallback</div>;
```

#### Do

```tsx
const a = <div dangerouslySetInnerHTML={{ __html: html }} />;
```

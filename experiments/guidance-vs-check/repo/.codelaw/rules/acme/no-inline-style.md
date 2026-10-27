---
id: acme/no-inline-style
title: "Style with a class, not an inline `style` prop"
languages: [jsx, tsx]
fix: manual
fix_reason: "the declarations need a class name and a place in the stylesheet, which only the author can choose"
stability: preview
since: 0.2.0
---

A component is styled with `className` and the stylesheet, never a `style={…}` prop.

**Why:** inline styles bypass the design tokens and the dark theme, and cannot be overridden by the
stylesheet.

```codelaw
match:
  kind: jsx_attribute
  has: { kind: property_identifier, regex: "^style$" }
message: "Use a `className` and the stylesheet, not an inline `style`"
```

## How to fix

1. Move the declarations into a class in the component's stylesheet.
2. Replace `style={…}` with `className="…"`.

## Examples

### TSX

#### Don't

```tsx
const A = () => <div style={{ color: "red" }}>hi</div>;
```

#### Do

```tsx
const A = () => <div className="alert">hi</div>;
```

### JSX

#### Don't

```jsx
const A = () => <div style={{ color: "red" }}>hi</div>;
```

#### Do

```jsx
const A = () => <div className="alert">hi</div>;
```

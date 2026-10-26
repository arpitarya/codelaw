---
id: fixture/js-eqeqeq
title: L3 fixture rule
languages: [javascript, jsx]
fix: auto
stability: stable
---

A fixture rule for the L3 language matrix (SR-LAW-3); it checks nothing real.

**Why:** every language must be checkable alone and together.

```codelaw
match:
  pattern: $A == $B
message: "Use === instead of =="
fix: "$A === $B"
```

## Examples

### JavaScript

#### Don't

```js
a == b;
```

#### Do

```js
a === b;
```

### JSX

#### Don't

```jsx
const x = <p>{a == b}</p>;
```

#### Do

```jsx
const x = <p>{a === b}</p>;
```

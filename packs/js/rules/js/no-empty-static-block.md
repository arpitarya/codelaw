---
id: js/no-empty-static-block
title: "No empty static blocks"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-empty-static-block]
since: 0.1.0
---

A class has no empty `static { }` block.

**Why:** An empty static block does nothing; it is leftover or unfinished code.

```codelaw
match:
  kind: class_static_block
  regex: ^static\s*\{\s*\}$
message: "Empty static block"
fix: [remove]
```

## Examples

### JavaScript

#### Don't

```js
class Config {
  static {}
  load() {}
}
```

#### Do

```js
class Config {
  load() {}
}
```

### JSX

#### Don't

```jsx
class Config {
  static {}
  load() {}
}
```

#### Do

```jsx
class Config {
  load() {}
}
```

### TypeScript

#### Don't

```ts
class Config {
  static {}
  load() {}
}
```

#### Do

```ts
class Config {
  load() {}
}
```

### TSX

#### Don't

```tsx
class Config {
  static {}
  load() {}
}
```

#### Do

```tsx
class Config {
  load() {}
}
```

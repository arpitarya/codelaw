---
id: ts/no-this-alias
title: "No aliasing this to a variable"
languages: [typescript, tsx]
fix: manual
fix_reason: "the callbacks that use the alias must become arrow functions by hand"
stability: preview
ported_from: [typescript-eslint/no-this-alias]
since: 0.1.0
---

`this` is not stored in a variable such as `self` or `that`; destructuring it is fine.

**Why:** Arrow functions keep the outer `this`, so an alias only adds a name to keep track of.

```codelaw
match:
  kind: variable_declarator
  all:
    - has: { field: name, kind: identifier }
    - has: { field: value, kind: this }
message: "Use an arrow function instead of aliasing `this`"
```

## How to fix

1. Turn the callbacks that use the alias into arrow functions.
2. Use `this` directly and remove the alias.

## Examples

### TypeScript

#### Don't

```ts
const self = this;
button.onclick = function () {
  self.save();
};
```

#### Do

```ts
button.onclick = () => {
  this.save();
};
```

### TSX

#### Don't

```tsx
const self = this;
button.onclick = function () {
  self.save();
};
```

#### Do

```tsx
button.onclick = () => {
  this.save();
};
```

---
id: js/constructor-super
title: "A derived class's constructor calls super(); a base class's does not"
languages: [javascript, jsx]
fix: manual
fix_reason: "what `super()` must receive depends on the base class"
stability: preview
ported_from: [eslint/constructor-super]
since: 0.1.0
---

A class that extends another must call `super()` in its constructor before it returns, and a class
that extends nothing must not call `super()` at all.

**Why:** Without `super()` the instance is never initialised and the constructor throws; in a base
class `super()` is a syntax error.

```codelaw
native: js/constructor-super
```

## How to fix

1. In a class with `extends`, call `super(...)` with the arguments the base class expects, before any use of `this`.
2. In a class without `extends`, remove the `super()` call.

## Examples

### JavaScript

#### Don't

```js
class Cat extends Animal {
  constructor() {
    this.lives = 9;
  }
}
```

#### Do

```js
class Cat extends Animal {
  constructor() {
    super();
    this.lives = 9;
  }
}
```

### JSX

#### Don't

```jsx
class Cat extends Animal {
  constructor() {
    this.lives = 9;
  }
}
```

#### Do

```jsx
class Cat extends Animal {
  constructor() {
    super();
    this.lives = 9;
  }
}
```

## Differences from the source

Paths are followed in straight-line order: a `super()` on any branch counts as called.

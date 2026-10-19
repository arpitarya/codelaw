---
id: js/no-this-before-super
title: "this is not used before super() in a derived constructor"
languages: [javascript, jsx]
fix: manual
fix_reason: "moving statements across `super()` can change what the base class sees"
stability: preview
ported_from: [eslint/no-this-before-super]
since: 0.1.0
---

In the constructor of a class that extends another, `this` and `super.x` are used only after
`super()` has been called.

**Why:** Before `super()` runs, `this` is not initialised and touching it throws a ReferenceError.

```codelaw
native: js/no-this-before-super
```

## How to fix

1. Move the `super(...)` call above the first use of `this`.
2. If the base class needs a value computed from `this`, compute it into a local first.

## Examples

### JavaScript

#### Don't

```js
class Cat extends Animal {
  constructor() {
    this.lives = 9;
    super();
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
    super();
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

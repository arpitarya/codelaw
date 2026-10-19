---
id: js/no-unused-private-class-members
title: "Every private class member is read"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "removing a member may also remove writes that matter, so it's a reader's call"
stability: preview
ported_from: [eslint/no-unused-private-class-members]
since: 0.1.0
---

A private field, method or accessor (`#name`) is read somewhere in its class; only being written
doesn't count.

**Why:** A private member nothing reads is dead code, since nothing outside the class can reach it.

```codelaw
native: js/no-unused-private-class-members
```

## How to fix

1. Check that nothing needs the member's value.
2. Remove the member and the writes to it, or read it where it was meant to be used.

## Examples

### JavaScript

#### Don't

```js
class Counter {
  #count = 0;
  #unused = 1;
  get value() {
    return this.#count;
  }
}
```

#### Do

```js
class Counter {
  #count = 0;
  get value() {
    return this.#count;
  }
}
```

### JSX

#### Don't

```jsx
class Counter {
  #count = 0;
  #unused = 1;
  get value() {
    return this.#count;
  }
}
```

#### Do

```jsx
class Counter {
  #count = 0;
  get value() {
    return this.#count;
  }
}
```

### TypeScript

#### Don't

```ts
class Counter {
  #count = 0;
  #unused = 1;
  get value() {
    return this.#count;
  }
}
```

#### Do

```ts
class Counter {
  #count = 0;
  get value() {
    return this.#count;
  }
}
```

### TSX

#### Don't

```tsx
class Counter {
  #count = 0;
  #unused = 1;
  get value() {
    return this.#count;
  }
}
```

#### Do

```tsx
class Counter {
  #count = 0;
  get value() {
    return this.#count;
  }
}
```

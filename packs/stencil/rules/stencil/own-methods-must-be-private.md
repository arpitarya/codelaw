---
id: stencil/own-methods-must-be-private
title: "A component's own methods are private"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [stencil/own-methods-must-be-private]
since: 0.1.0
---

A method of a component that isn't a `@Method()`, a lifecycle method or `render()` is `private`.

**Why:** Only `@Method()` members reach users of the element; a public own method suggests an API
that isn't there.

```codelaw
native: stencil/own-methods-must-be-private
```

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  helper() {
    return 1;
  }
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  private helper() {
    return 1;
  }
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  helper() {
    return 1;
  }
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  private helper() {
    return 1;
  }
}
```

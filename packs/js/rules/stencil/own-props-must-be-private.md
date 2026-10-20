---
id: stencil/own-props-must-be-private
title: "A component's own fields are private"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [stencil/own-props-must-be-private]
since: 0.1.0
---

A class field of a component without a Stencil decorator is `private`.

**Why:** Only `@Prop()` fields reach users of the element; a public plain field suggests an API that
isn't there.

```codelaw
native: stencil/own-props-must-be-private
```

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  cache = new Map();
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  private cache = new Map();
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  cache = new Map();
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  private cache = new Map();
}
```

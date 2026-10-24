---
id: stencil/prop-readonly
title: "@Prop() members are readonly unless mutable"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [stencil/props-must-be-readonly]
since: 0.1.0
---

A `@Prop()` is `readonly`, unless its options say `mutable: true`.

**Why:** Props are set from outside; a component that writes one by accident fights its parent.

```codelaw
native: stencil/prop-readonly
```

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** The heading. */
  @Prop() heading: string;
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** The heading. */
  @Prop() readonly heading: string;
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** The heading. */
  @Prop() heading: string;
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** The heading. */
  @Prop() readonly heading: string;
}
```

---
id: stencil/ban-default-true
title: "A boolean @Prop defaults to false"
languages: [typescript, tsx]
fix: manual
fix_reason: "inverting the prop renames it for every user"
stability: preview
ported_from: [stencil/ban-default-true]
since: 0.1.0
---

A `@Prop()` of a Stencil component doesn't default to `true`.

**Why:** An HTML boolean attribute can only switch something on; a prop that starts `true` can't be
turned off from markup.

```codelaw
native: stencil/ban-default-true
```

## How to fix

1. Invert the prop's meaning so it defaults to `false` (e.g. `closable` → `hideClose`).
2. Update the component's markup users.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** Shows the close button. */
  @Prop() readonly closable = true;
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** Hides the close button. */
  @Prop() readonly hideClose = false;
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** Shows the close button. */
  @Prop() readonly closable = true;
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** Hides the close button. */
  @Prop() readonly hideClose = false;
}
```

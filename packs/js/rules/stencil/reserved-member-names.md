---
id: stencil/reserved-member-names
title: "Public members don't reuse HTML element names"
languages: [typescript, tsx]
fix: manual
fix_reason: "renaming a public member changes every user"
stability: preview
ported_from: [stencil/reserved-member-names]
since: 0.1.0
---

A `@Prop()` or `@Method()` isn't named after an HTML global attribute or an `HTMLElement` member
(`title`, `hidden`, `id`, `click`, …).

**Why:** The element already has a member of that name, so the component's version clashes with it.

```codelaw
native: stencil/reserved-member-names
```

## How to fix

1. Rename the member.
2. Update its users.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** The card's title. */
  @Prop() readonly title: string;
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** The card's heading. */
  @Prop() readonly heading: string;
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** The card's title. */
  @Prop() readonly title: string;
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** The card's heading. */
  @Prop() readonly heading: string;
}
```

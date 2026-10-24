---
id: stencil/required-jsdoc
title: "Public members have JSDoc"
languages: [typescript, tsx]
fix: manual
fix_reason: "the documentation text needs the author"
stability: preview
ported_from: [stencil/required-jsdoc]
since: 0.1.0
---

Every `@Prop()`, `@Method()` and `@Event()` has a `/** … */` comment.

**Why:** Stencil turns these comments into the component's documentation and editor hints.

```codelaw
native: stencil/required-jsdoc
```

## How to fix

1. Write a `/** … */` comment above the member saying what it is for.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  @Prop() readonly heading: string;
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  /** The heading shown at the top. */
  @Prop() readonly heading: string;
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  @Prop() readonly heading: string;
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  /** The heading shown at the top. */
  @Prop() readonly heading: string;
}
```

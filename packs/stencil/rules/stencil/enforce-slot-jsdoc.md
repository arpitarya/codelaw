---
id: stencil/enforce-slot-jsdoc
title: "Every slot a component renders has a @slot tag, and every @slot tag a slot"
languages: [tsx]
fix: manual
fix_reason: "the slot's description, or whether it should exist, needs the author"
stability: preview
ported_from: [stencil/enforce-slot-jsdoc]
since: 0.2.0
---

A `@Component` class's JSDoc has a `@slot` tag for each `<slot>` it renders — `@slot name - what it
holds` for a named slot, `@slot - what it holds` for the default one — and no `@slot` tag for a slot
it doesn't render.

**Why:** Stencil builds the component's documentation and editor hints from these tags, so a missing
tag hides a slot from its users, and a stale one promises a slot that isn't there.

```codelaw
native: stencil/enforce-slot-jsdoc
```

## How to fix

1. For a slot reported as rendered without a tag, add `@slot <name> - <what it holds>` (or
   `@slot - <what it holds>` for the default slot) to the JSDoc above `@Component`.
2. For a tag reported as never rendered, remove the tag, or render the `<slot>` it describes.

## Examples

### TSX

#### Don't

```tsx
/**
 * A card.
 */
@Component({ tag: 'my-card' })
export class Card {
  render() {
    return <slot name="header" />;
  }
}
```

#### Do

```tsx
/**
 * A card.
 * @slot header - The card's title row.
 */
@Component({ tag: 'my-card' })
export class Card {
  render() {
    return <slot name="header" />;
  }
}
```

## Differences from the source

- A `<slot>` whose `name` is an expression (`name={this.slot}`) is left out; the source records it as
  a slot named `undefined`.
- A `@slot` tag's text stops at the end of its line; the source lets a bare `@slot` take its
  description from the next line.
- TSX only: a `.ts` file has no `<slot>` elements to check.

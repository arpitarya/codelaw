---
id: react/props-type-name
title: "A component's props type is named <Component>Props"
languages: [tsx]
fix: suggest
fix_reason: "renaming a type other files import would break them, so it is offered, not applied"
stability: preview
since: 0.1.0
---

The type of a component's props is named after the component: `CardProps` for `Card`.

**Why:** One naming pattern makes props types easy to find and import.

```codelaw
native: react/props-type-name
```

## Examples

### TSX

#### Don't

```tsx
type CardOptions = { title: string };
export function Card(props: CardOptions) {
  return <h2>{props.title}</h2>;
}
```

#### Do

```tsx
type CardProps = { title: string };
export function Card(props: CardProps) {
  return <h2>{props.title}</h2>;
}
```

## Differences from the source

The type is renamed with its uses in the file; an exported type is reported without a fix.

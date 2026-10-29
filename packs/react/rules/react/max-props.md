---
id: react/max-props
title: "A React component takes at most a maximum number of props"
languages: [jsx, tsx]
fix: manual
fix_reason: "grouping props changes the component's interface and every place that renders it; which props belong together is the author's call"
stability: preview
options:
  max: { type: "integer", default: 8, description: "the most props one component may take" }
  ignore_names: { type: "string", description: "a regular expression; components named like this are exempt" }
ported_from: ["@eslint-react/no-too-many-props"]
since: 0.5.0
---

A function component takes at most `max` props (8 by default). The props are counted from the first parameter: the keys it destructures (`({ a, b = 1, c: d })`; a `...rest` is not counted), or the members of its declared type — an inline `{ a: string; b: number }`, or an `interface` or `type` of the same file (`props: RowProps`) — whichever is larger. A class component and a type declared in another file are not counted. The rule reports on the component's name: `Component Row takes 11 props (max 8)`.

**Why:** Every prop is a decision made at each place the component is rendered, and a component with a dozen has no single job. Props that always travel together are one thing with a name.

```codelaw
native: react/max-props
```

## How to fix

1. Group props that always travel together into one object prop (`user={user}` instead of `id`, `name`, `email`).
2. Pass children instead of a prop per slot (`<Card><Header /></Card>`).
3. Split the component if its props fall into two sets used by different parts.
4. If the component is wide on purpose, name it in `ignore_names`, or freeze it with `codelaw baseline add --rule react/max-props`.

## Examples

### JSX

#### Don't

```jsx options={"max":3}
function Row({ id, name, email, phone }) {
  return <tr>{id}{name}{email}{phone}</tr>;
}
```

#### Do

```jsx options={"max":3}
function Row({ user }) {
  return <tr>{user.id}{user.name}{user.email}{user.phone}</tr>;
}
```

### TSX

#### Don't

```tsx options={"max":3}
interface RowProps {
  id: string;
  name: string;
  email: string;
  phone: string;
}
function Row(props: RowProps) {
  return <tr>{props.id}</tr>;
}
```

#### Do

```tsx options={"max":3}
interface RowProps {
  user: User;
}
function Row(props: RowProps) {
  return <tr>{props.user.id}</tr>;
}
```

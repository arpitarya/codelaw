---
id: js/no-console
title: "No console calls left in the code"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "deleting the call also drops its arguments, which may have side effects; and a call whose value is used needs the author"
stability: preview
options:
  allow: { type: "string[]", default: [], description: "console methods that may stay, e.g. [\"warn\", \"error\"]" }
ported_from: [eslint/no-console]
since: 0.3.0
---

Code makes no calls on the global `console` (`console.log(…)`, `console["debug"](…)`), except the
methods the `allow` option names. A local variable named `console` is not the global one.

**Why:** `console` output is debugging left behind: it clutters the browser console and the server
logs, and it can leak data. A team that wants it on a laptop and not in CI sets the rule to `warn`
in `[profiles.local]` and `error` in `[profiles.ci]`.

```codelaw
native: js/no-console
```

## How to fix

1. Delete the call (`codelaw fix --unsafe` does when the call is a whole statement).
2. If the output is wanted, report through the project's logger instead.
3. If the method should stay everywhere, add it to the rule's `allow` option.

## Examples

### JavaScript

#### Don't

```js
function load(items) {
  console.log(items);
  return items.length;
}
```

#### Do

```js
function load(items) {
  return items.length;
}
```

### JSX

#### Don't

```jsx
function List({ items }) {
  console.log(items);
  return <ul>{items}</ul>;
}
```

#### Do

```jsx
function List({ items }) {
  return <ul>{items}</ul>;
}
```

### TypeScript

#### Don't

```ts
function load(items: string[]): number {
  console.log(items);
  return items.length;
}
```

#### Do

```ts
function load(items: string[]): number {
  return items.length;
}
```

### TSX

#### Don't

```tsx
function List({ items }: { items: string[] }) {
  console.log(items);
  return <ul>{items}</ul>;
}
```

#### Do

```tsx
function List({ items }: { items: string[] }) {
  return <ul>{items}</ul>;
}
```

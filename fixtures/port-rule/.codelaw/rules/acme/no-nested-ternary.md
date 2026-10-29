---
id: acme/no-nested-ternary
title: "No conditional expression inside another one's branches"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "an if/else chain, a lookup table or a named helper each fit different code; choosing needs the author"
stability: preview
ported_from: [eslint/no-nested-ternary]
since: 0.4.0
---

A conditional expression (`a ? b : c`) has no other conditional expression as its `?` branch or its `:`
branch, with or without parentheses around it. One in the condition itself is not reported, as in the
source rule.

**Why:** Chained ternaries read right to left and hide which value goes with which condition; an
`if`/`else` or a lookup says it once. Ported from ESLint's `no-nested-ternary` by the
`codelaw-port-rule` skill: if a first-party pack ships a rule with the same `ported_from:`, turn that one
on and delete this file.

```codelaw
match:
  kind: ternary_expression
  any:
    - has: { field: consequence, kind: ternary_expression }
    - has: { field: alternative, kind: ternary_expression }
    - has: { field: consequence, kind: parenthesized_expression, has: { kind: ternary_expression } }
    - has: { field: alternative, kind: parenthesized_expression, has: { kind: ternary_expression } }
message: "a conditional expression nested in another one's branch: use if/else or a lookup"
```

## How to fix

1. Find the value each condition picks.
2. Write them as an `if`/`else if` chain that assigns or returns, or as an object or `Map` keyed by the
   condition's value when the conditions compare one variable.
3. If the expression must stay an expression (a JSX attribute, a default parameter), move the chain into
   a small named function and call it.

## Examples

### JavaScript

#### Don't

```js
const label = count === 0 ? "none" : count === 1 ? "one" : "many";
```

#### Do

```js
function labelFor(count) {
  if (count === 0) return "none";
  if (count === 1) return "one";
  return "many";
}
const label = labelFor(count);
```

### JSX

#### Don't

```jsx
const badge = <Badge tone={ok ? "green" : late ? "amber" : "red"} />;
```

#### Do

```jsx
const tones = { ok: "green", late: "amber", failed: "red" };
const badge = <Badge tone={tones[status]} />;
```

### TypeScript

#### Don't

```ts
const size: string = width > 900 ? "large" : width > 500 ? "medium" : "small";
```

#### Do

```ts
function sizeFor(width: number): string {
  if (width > 900) return "large";
  if (width > 500) return "medium";
  return "small";
}
const size: string = sizeFor(width);
```

### TSX

#### Don't

```tsx
const icon = <Icon name={open ? (pinned ? "pin" : "open") : "closed"} />;
```

#### Do

```tsx
function iconName(open: boolean, pinned: boolean): string {
  if (!open) return "closed";
  return pinned ? "pin" : "open";
}
const icon = <Icon name={iconName(open, pinned)} />;
```

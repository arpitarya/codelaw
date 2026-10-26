---
id: js/max-params
title: "A function takes at most a maximum number of parameters"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "grouping parameters changes the function's signature and every call to it; which belong together is the author's call"
stability: preview
options:
  max: { type: "integer", default: 3, description: "the most parameters a function may take" }
  count_this: { type: "boolean", default: false, description: "TypeScript: count a `this:` parameter" }
  count_varargs: { type: "boolean", default: false, description: "count a rest parameter, `...rest`" }
  ignore_names: { type: "string", default: "^_", description: "a regular expression; parameters named like this do not count" }
  ignore_overrides: { type: "boolean", default: true, description: "skip a TypeScript `override` method: its signature is not yours" }
  ignore_declarations: { type: "boolean", default: true, description: "skip an overload signature, a `declare` function, an abstract method and a `.d.ts` file" }
  ignore_constructor_injection: { type: "boolean", default: false, description: "TypeScript: skip a constructor whose parameters are all parameter properties (Angular, NestJS)" }
ported_from: [eslint/max-params, typescript-eslint/max-params]
since: 0.3.0
---

A function takes at most `max` parameters (3 by default). A destructured object or array parameter counts as one, which
is the usual way out. A rest parameter and a TypeScript `this:` parameter are not counted unless `count_varargs` or
`count_this` is `true`, and neither is a parameter whose name matches `ignore_names` (`"^_"` by default: a
parameter named `_unused` is already said to be unused). A TypeScript `override` method, an overload signature, a
`declare` function, an abstract method and anything in a `.d.ts` file are skipped, since you do not own that
signature; set `ignore_overrides` or `ignore_declarations` to `false` to count them. A constructor that only
declares parameter properties (`constructor(private a: A, private b: B)`) can be skipped with
`ignore_constructor_injection`. The rule reports on the function's name: `createOrder takes 6 parameters (max 3)`.

**Why:** A call with many positional arguments is easy to get in the wrong order, and every added parameter is a change to every
caller. Grouping related parameters in one object names them at the call site.

```codelaw
native: js/max-params
```

## How to fix

1. Group the parameters that belong together into one options object: `createOrder({ customer, items, coupon, address })`.
2. Update the callers to pass the object, and give the destructured parameter a type in TypeScript.
3. If the signature is not yours to change, exempt it: an `override`, an overload and a `declare` function already are.

## Examples

### JavaScript

#### Don't

```js
function createOrder(customer, items, coupon, address) {
  return { customer, items, coupon, address };
}
```

#### Do

```js
function createOrder({ customer, items, coupon, address }) {
  return { customer, items, coupon, address };
}
```

### JSX

#### Don't

```jsx
function Card(title, body, footer, onClose) {
  return <div>{title}{body}{footer}{onClose}</div>;
}
```

#### Do

```jsx
function Card({ title, body, footer, onClose }) {
  return <div>{title}{body}{footer}{onClose}</div>;
}
```

### TypeScript

#### Don't

```ts
function createOrder(customer: string, items: string[], coupon: string, address: string) {
  return { customer, items, coupon, address };
}
```

#### Do

```ts
function createOrder({ customer, items, coupon, address }: Order) {
  return { customer, items, coupon, address };
}
```

### TSX

#### Don't

```tsx
function Card(title: string, body: string, footer: string, onClose: () => void) {
  return <div>{title}{body}{footer}{onClose}</div>;
}
```

#### Do

```tsx
function Card({ title, body, footer, onClose }: CardProps) {
  return <div>{title}{body}{footer}{onClose}</div>;
}
```

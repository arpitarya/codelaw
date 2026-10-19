---
id: js/no-with
title: "No with statements"
languages: [javascript, jsx]
fix: manual
fix_reason: "each name inside needs to be qualified by hand"
stability: preview
ported_from: [eslint/no-with]
since: 0.1.0
---

Code doesn't use `with`.

**Why:** `with` makes it impossible to tell which names are properties and which are variables, and
is an error in strict code.

```codelaw
match:
  kind: with_statement
message: "`with` makes every name inside it ambiguous"
```

## How to fix

1. Replace each property used inside `with` by `object.property`.
2. Remove the `with`.

## Examples

### JavaScript

#### Don't

```js
with (Math) {
  area = PI * r * r;
}
```

#### Do

```js
area = Math.PI * r * r;
```

### JSX

#### Don't

```jsx
with (Math) {
  area = PI * r * r;
}
```

#### Do

```jsx
area = Math.PI * r * r;
```

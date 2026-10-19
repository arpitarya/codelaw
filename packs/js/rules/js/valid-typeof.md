---
id: js/valid-typeof
title: "typeof is compared with a type it can return"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which type was meant needs the author"
stability: preview
ported_from: [eslint/valid-typeof]
since: 0.1.0
---

`typeof x` is compared only with a string it can return: `"undefined"`, `"object"`, `"boolean"`,
`"number"`, `"string"`, `"function"`, `"symbol"` or `"bigint"`.

**Why:** A misspelled type such as `"strnig"` makes the comparison always false.

```codelaw
match:
  any:
    - pattern: typeof $X == $S
    - pattern: typeof $X === $S
    - pattern: typeof $X != $S
    - pattern: typeof $X !== $S
    - pattern: $S == typeof $X
    - pattern: $S === typeof $X
    - pattern: $S != typeof $X
    - pattern: $S !== typeof $X
constraints:
  S:
    kind: string
    not: { regex: "^[\"'](undefined|object|boolean|number|string|function|symbol|bigint)[\"']$" }
message: "`typeof` never returns $S"
```

## How to fix

1. Correct the string to the type that was meant.

## Examples

### JavaScript

#### Don't

```js
if (typeof name === "strnig") {
  greet(name);
}
```

#### Do

```js
if (typeof name === "string") {
  greet(name);
}
```

### JSX

#### Don't

```jsx
if (typeof name === "strnig") {
  greet(name);
}
```

#### Do

```jsx
if (typeof name === "string") {
  greet(name);
}
```

### TypeScript

#### Don't

```ts
if (typeof name === "strnig") {
  greet(name);
}
```

#### Do

```ts
if (typeof name === "string") {
  greet(name);
}
```

### TSX

#### Don't

```tsx
if (typeof name === "strnig") {
  greet(name);
}
```

#### Do

```tsx
if (typeof name === "string") {
  greet(name);
}
```

---
id: js/no-new-native-nonconstructor
title: "No new on Symbol or BigInt"
languages: [javascript, jsx]
fix: suggest
fix_reason: "the call throws today, so removing `new` changes what the code does"
stability: preview
ported_from: [eslint/no-new-native-nonconstructor]
since: 0.1.0
---

`Symbol` and `BigInt` are called as functions, never with `new`.

**Why:** `new Symbol()` and `new BigInt()` throw a TypeError.

```codelaw
match:
  pattern: new $C($$$ARGS)
constraints:
  C: { regex: ^(Symbol|BigInt)$ }
where:
  $C: builtin
message: "`$C` is not a constructor; call it without `new`"
fix: "$C($$$ARGS)"
```

## Examples

### JavaScript

#### Don't

```js
const id = new Symbol("id");
```

#### Do

```js
const id = Symbol("id");
```

### JSX

#### Don't

```jsx
const id = new Symbol("id");
```

#### Do

```jsx
const id = Symbol("id");
```

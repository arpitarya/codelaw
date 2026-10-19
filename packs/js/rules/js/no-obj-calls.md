---
id: js/no-obj-calls
title: "No calling Math, JSON, Reflect, Atomics or Intl as functions"
languages: [javascript, jsx]
fix: manual
fix_reason: "which method was meant needs the author"
stability: preview
ported_from: [eslint/no-obj-calls]
since: 0.1.0
---

The built-in namespace objects `Math`, `JSON`, `Reflect`, `Atomics` and `Intl` are not called or
constructed.

**Why:** They are plain objects; calling one throws a TypeError.

```codelaw
match:
  any:
    - pattern: $C($$$ARGS)
    - pattern: new $C($$$ARGS)
constraints:
  C: { regex: ^(Math|JSON|Reflect|Atomics|Intl)$ }
where:
  $C: builtin
message: "`$C` is a namespace object, not a function"
```

## How to fix

1. Call the method of the namespace that was meant, e.g. `Math.round(x)`.

## Examples

### JavaScript

#### Don't

```js
const n = Math(2.5);
```

#### Do

```js
const n = Math.round(2.5);
```

### JSX

#### Don't

```jsx
const n = Math(2.5);
```

#### Do

```jsx
const n = Math.round(2.5);
```

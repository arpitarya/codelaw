---
id: js/no-prototype-builtins
title: "No calling Object.prototype methods on the object itself"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "an object whose own method is meant would change behaviour"
stability: preview
ported_from: [eslint/no-prototype-builtins]
since: 0.1.0
---

`hasOwnProperty`, `isPrototypeOf` and `propertyIsEnumerable` are called through `Object.prototype`,
not on the object being tested.

**Why:** An object can shadow these methods, or have no prototype at all, and then the call breaks
or lies.

```codelaw
match:
  pattern: $O.$M($$$ARGS)
constraints:
  M: { regex: ^(hasOwnProperty|isPrototypeOf|propertyIsEnumerable)$ }
message: "Call `Object.prototype.$M` through `.call`, or use `Object.hasOwn`"
fix: "Object.prototype.$M.call($O, $$$ARGS)"
```

## Examples

### JavaScript

#### Don't

```js
if (options.hasOwnProperty("debug")) {
  enable();
}
```

#### Do

```js
if (Object.prototype.hasOwnProperty.call(options, "debug")) {
  enable();
}
```

### JSX

#### Don't

```jsx
if (options.hasOwnProperty("debug")) {
  enable();
}
```

#### Do

```jsx
if (Object.prototype.hasOwnProperty.call(options, "debug")) {
  enable();
}
```

### TypeScript

#### Don't

```ts
if (options.hasOwnProperty("debug")) {
  enable();
}
```

#### Do

```ts
if (Object.prototype.hasOwnProperty.call(options, "debug")) {
  enable();
}
```

### TSX

#### Don't

```tsx
if (options.hasOwnProperty("debug")) {
  enable();
}
```

#### Do

```tsx
if (Object.prototype.hasOwnProperty.call(options, "debug")) {
  enable();
}
```

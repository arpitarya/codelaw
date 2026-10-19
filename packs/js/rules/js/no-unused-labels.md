---
id: js/no-unused-labels
title: "Every label is used"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
ported_from: [eslint/no-unused-labels]
since: 0.1.0
---

Every statement label is the target of a `break` or `continue`.

**Why:** An unused label is leftover from a refactor and makes the reader look for a jump that isn't
there.

```codelaw
match:
  kind: labeled_statement
  all:
    - has: { field: label, pattern: $L }
    - has: { field: body, pattern: $BODY }
  not:
    has:
      stopBy: end
      any: [{ pattern: "break $L;" }, { pattern: "continue $L;" }]
message: "The label `$L` is never used"
fix: "$BODY"
```

## Examples

### JavaScript

#### Don't

```js
outer: for (const row of rows) {
  draw(row);
}
```

#### Do

```js
for (const row of rows) {
  draw(row);
}
```

### JSX

#### Don't

```jsx
outer: for (const row of rows) {
  draw(row);
}
```

#### Do

```jsx
for (const row of rows) {
  draw(row);
}
```

### TypeScript

#### Don't

```ts
outer: for (const row of rows) {
  draw(row);
}
```

#### Do

```ts
for (const row of rows) {
  draw(row);
}
```

### TSX

#### Don't

```tsx
outer: for (const row of rows) {
  draw(row);
}
```

#### Do

```tsx
for (const row of rows) {
  draw(row);
}
```

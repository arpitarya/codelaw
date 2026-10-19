---
id: js/no-const-assign
title: "A const is not reassigned"
languages: [javascript, jsx]
fix: manual
fix_reason: "whether the variable should be `let` or the assignment should go is the author's call"
stability: preview
ported_from: [eslint/no-const-assign]
since: 0.1.0
---

Code doesn't assign a new value to a `const`.

**Why:** Assigning to a `const` throws a TypeError at run time.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [write, { binding: const }]
message: "`$$MATCH` is a const; assigning to it throws"
```

## How to fix

1. If the value is meant to change, declare it with `let`.
2. Otherwise assign to a new variable.

## Examples

### JavaScript

#### Don't

```js
const total = 0;
total = 1;
```

#### Do

```js
let total = 0;
total = 1;
```

### JSX

#### Don't

```jsx
const total = 0;
total = 1;
```

#### Do

```jsx
let total = 0;
total = 1;
```

---
id: js/no-func-assign
title: "A function declaration is not reassigned"
languages: [javascript, jsx]
fix: manual
fix_reason: "what the assignment was for is the author's call"
stability: preview
ported_from: [eslint/no-func-assign]
since: 0.1.0
---

Code doesn't assign a new value to the name a function declaration binds.

**Why:** Overwriting a declared function is almost always a mistake, and every later call means
something else.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [write, { binding: function }]
message: "`$$MATCH` is a function declaration; don't assign to it"
```

## How to fix

1. Assign to a new variable instead.
2. If the function must be replaceable, declare it with `let` as a function expression.

## Examples

### JavaScript

#### Don't

```js
function save() {}
save = null;
```

#### Do

```js
function save() {}
const saveHandler = null;
```

### JSX

#### Don't

```jsx
function save() {}
save = null;
```

#### Do

```jsx
function save() {}
const saveHandler = null;
```

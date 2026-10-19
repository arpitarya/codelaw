---
id: js/no-class-assign
title: "A class name is not reassigned"
languages: [javascript, jsx]
fix: manual
fix_reason: "what the assignment was for is the author's call"
stability: preview
ported_from: [eslint/no-class-assign]
since: 0.1.0
---

Code doesn't assign a new value to the name a class declaration binds.

**Why:** Reassigning a class name is almost always a mistake, and makes every later use mean
something else.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [write, { binding: class }]
message: "`$$MATCH` is a class; don't assign to it"
```

## How to fix

1. Assign to a new variable instead of the class name.

## Examples

### JavaScript

#### Don't

```js
class Point {}
Point = null;
```

#### Do

```js
class Point {}
const origin = null;
```

### JSX

#### Don't

```jsx
class Point {}
Point = null;
```

#### Do

```jsx
class Point {}
const origin = null;
```

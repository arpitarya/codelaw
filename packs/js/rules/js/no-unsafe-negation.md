---
id: js/no-unsafe-negation
title: "No negating the left side of in or instanceof"
languages: [javascript, jsx]
fix: suggest
fix_reason: "code that really meant `(!a) in b` would change"
stability: preview
ported_from: [eslint/no-unsafe-negation]
since: 0.1.0
---

`!` before `a in b` or `a instanceof B` wraps the whole test in parentheses.

**Why:** `!a in b` means `(!a) in b`, which tests whether `true` or `false` is a key — never what
was meant.

```codelaw
match:
  kind: binary_expression
  all:
    - has: { field: operator, regex: ^(in|instanceof)$ }
    - has:
        field: left
        kind: unary_expression
        regex: ^!
        pattern: $NEG
        has: { field: argument, pattern: $A }
message: "`!` applies to `$A`, not to the whole test; wrap the test in parentheses"
fix:
  - insert-before: "!("
  - replace: { target: $NEG, with: "$A" }
  - insert-after: ")"
```

## Examples

### JavaScript

#### Don't

```js
if (!key in cache) {
  load(key);
}
```

#### Do

```js
if (!(key in cache)) {
  load(key);
}
```

### JSX

#### Don't

```jsx
if (!key in cache) {
  load(key);
}
```

#### Do

```jsx
if (!(key in cache)) {
  load(key);
}
```

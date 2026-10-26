---
id: sec-js/no-csrf-before-method-override
title: "No method-override before the CSRF check"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "moving a middleware changes the order every route in the file runs them in"
stability: preview
security: { cwe: [352], owasp: ["A01:2025"], severity: medium }
since: 0.3.0
---

`methodOverride` is registered after the CSRF middleware, not before it.

**Why:** The CSRF middleware decides by the request's method: a `POST` rewritten to `DELETE` after the check was never checked as one that changes state (CWE-352).

```codelaw
match:
  all:
    - kind: expression_statement
    - has:
        pattern: $A.use(methodOverride($$$M))
    - precedes:
        stopBy: end
        pattern: $CSRF
        kind: expression_statement
        regex: '^\w+\.use\((csrf|csurf|lusca\.csrf|csrfProtection)\b'
message: "`methodOverride` runs before the CSRF check, so the check sees the wrong method"
fix:
  - insert-after: { target: $CSRF, text: "\n$$MATCH" }
  - remove
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
app.use(methodOverride("_method"));
app.use(csurf());
```

#### Do

```js
app.use(csurf());
app.use(methodOverride("_method"));
```

#### Fixed

```js
app.use(csurf());
app.use(methodOverride("_method"));
```

### JSX

#### Don't

```jsx
app.use(methodOverride("_method"));
app.use(csurf());
```

#### Do

```jsx
app.use(csurf());
app.use(methodOverride("_method"));
```

#### Fixed

```jsx
app.use(csurf());
app.use(methodOverride("_method"));
```

### TypeScript

#### Don't

```ts
app.use(methodOverride("_method"));
app.use(csurf());
```

#### Do

```ts
app.use(csurf());
app.use(methodOverride("_method"));
```

#### Fixed

```ts
app.use(csurf());
app.use(methodOverride("_method"));
```

### TSX

#### Don't

```tsx
app.use(methodOverride("_method"));
app.use(csurf());
```

#### Do

```tsx
app.use(csurf());
app.use(methodOverride("_method"));
```

#### Fixed

```tsx
app.use(csurf());
app.use(methodOverride("_method"));
```

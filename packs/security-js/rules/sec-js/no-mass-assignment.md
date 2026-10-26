---
id: sec-js/no-mass-assignment
title: "No request body written straight into a model"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which fields a client may set is a decision for the author"
stability: preview
security: { cwe: [915], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

A model is not created or updated from `req.body` whole: `Model.create(req.body)`, `new User(req.body)`, `Object.assign(entity, req.body)`.

**Why:** The client sets every field it names, including `isAdmin`, `role` or `userId`, and any field the form never showed (CWE-915).

```codelaw
match:
  any:
    - pattern: $M.create($B)
    - pattern: $M.insert($B)
    - pattern: $M.update($B)
    - pattern: $M.insertMany($B)
    - pattern: $M.build($B)
    - pattern: $M.upsert($B)
    - pattern: $M.save($B)
    - pattern: $M.findByIdAndUpdate($ID, $B)
    - pattern: $M.findOneAndUpdate($ID, $B)
    - pattern: $M.updateOne($ID, $B)
    - pattern: $M.updateMany($ID, $B)
    - pattern: $M.findByIdAndUpdate($ID, $B)
    - pattern: new $C($B)
    - pattern: Object.assign($T, $B)
constraints:
  B: { regex: '^(req|request|ctx\.request)\.body$' }
message: "The whole request body is written into a model; the client chooses every field"
```

## How to fix

1. Pick the fields: `{ name: req.body.name, email: req.body.email }`, or `_.pick(req.body, ["name", "email"])`.
2. Validate the body with a schema that rejects unknown keys (`zod` `.strict()`, `joi` `.unknown(false)`) before it is used.
3. Mark fields a client may never set (`role`, `isAdmin`) as read-only in the model.

## Examples

### JavaScript

#### Don't

```js
const user = await User.create(req.body);
```

#### Do

```js
const user = await User.create({ name: req.body.name, email: req.body.email });
```

### JSX

#### Don't

```jsx
const user = await User.create(req.body);
```

#### Do

```jsx
const user = await User.create({ name: req.body.name, email: req.body.email });
```

### TypeScript

#### Don't

```ts
const user = await User.create(req.body);
```

#### Do

```ts
const user = await User.create({ name: req.body.name, email: req.body.email });
```

### TSX

#### Don't

```tsx
const user = await User.create(req.body);
```

#### Do

```tsx
const user = await User.create({ name: req.body.name, email: req.body.email });
```

---
id: sec-js/no-unsafe-deserialize
title: "No deserializer that builds objects or runs code"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a document that uses a custom tag stops loading; js-yaml 4's `load` is already safe, so the fix only makes that explicit"
stability: preview
ported_from: [bandit/B506, ruff/S506]
security: { cwe: [502], owasp: ["A08:2025"], severity: high }
since: 0.3.0
---

`node-serialize`'s `unserialize` is not used, and js-yaml's `load` is given a safe schema.

**Why:** `unserialize` runs a function stored in the data (CWE-502), and js-yaml before version 4 builds JavaScript objects and functions from tags in the document.

```codelaw
match:
  any:
    - pattern: unserialize($X)
    - pattern: serialize.unserialize($X)
    - pattern: nodeSerialize.unserialize($X)
    - pattern: $Y.load($Z)
    - pattern: $Y.loadAll($Z)
constraints:
  Y: { regex: '^(yaml|jsyaml|jsYaml)$' }
message: "This deserializer can build objects or run code from its input"
fix:
  - replace: { target: $Z, with: "$Z, { schema: $Y.CORE_SCHEMA }" }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const config = yaml.load(text);
```

#### Do

```js
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

#### Fixed

```js
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

### JSX

#### Don't

```jsx
const config = yaml.load(text);
```

#### Do

```jsx
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

#### Fixed

```jsx
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

### TypeScript

#### Don't

```ts
const config = yaml.load(text);
```

#### Do

```ts
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

#### Fixed

```ts
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

### TSX

#### Don't

```tsx
const config = yaml.load(text);
```

#### Do

```tsx
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

#### Fixed

```tsx
const config = yaml.load(text, { schema: yaml.CORE_SCHEMA });
```

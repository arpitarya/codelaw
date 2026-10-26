---
id: sec-js/no-incomplete-sanitization
title: "No escaping that stops at the first match, no one-pass stripping"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "it changes how many matches are replaced, and `replaceAll` needs an ES2021 runtime"
stability: preview
security: { cwe: [116], owasp: ["A05:2025"], severity: medium }
since: 0.3.0
---

`replace` with a string replaces only the first match, so a markup character is escaped with `replaceAll` or a global regular expression; and a path or script marker is not stripped in one pass.

**Why:** `"<<b>".replace("<", "&lt;")` leaves the second `<`; and stripping `../` once turns `....//` back into `../` (CWE-116).

```codelaw
match:
  any:
    - all:
        - pattern: $S.replace($P, $R)
        - regex: '\.replace\(\s*["''][<>&/]["'']\s*,'
    - all:
        - pattern: $S2.replace($RE, "")
        - regex: '\.replace\(\s*/(\\\.\\\.(\\/|\\\\)|javascript:)[^/]*/[gimsuy]*\s*,\s*[\x22\x27]{2}\s*\)$'
message: "This replacement misses later matches or can be rebuilt by the input"
fix:
  - replace: "$S.replaceAll($P, $R)"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const safe = input.replace("<", "&lt;");
```

#### Do

```js
const safe = input.replaceAll("<", "&lt;");
```

#### Fixed

```js
const safe = input.replaceAll("<", "&lt;");
```

### JSX

#### Don't

```jsx
const safe = input.replace("<", "&lt;");
```

#### Do

```jsx
const safe = input.replaceAll("<", "&lt;");
```

#### Fixed

```jsx
const safe = input.replaceAll("<", "&lt;");
```

### TypeScript

#### Don't

```ts
const safe = input.replace("<", "&lt;");
```

#### Do

```ts
const safe = input.replaceAll("<", "&lt;");
```

#### Fixed

```ts
const safe = input.replaceAll("<", "&lt;");
```

### TSX

#### Don't

```tsx
const safe = input.replace("<", "&lt;");
```

#### Do

```tsx
const safe = input.replaceAll("<", "&lt;");
```

#### Fixed

```tsx
const safe = input.replaceAll("<", "&lt;");
```

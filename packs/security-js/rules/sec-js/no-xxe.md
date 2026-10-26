---
id: sec-js/no-xxe
title: "No XML entity expansion"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a document that relies on its own entities stops parsing the same way"
stability: preview
security: { cwe: [611, 776], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

An XML parser is not asked to expand entities: no `noent: true`.

**Why:** Entity expansion lets a document read files from the server or fetch URLs (XXE, CWE-611), and expand into gigabytes (CWE-776).

```codelaw
match:
  any:
    - pattern: 'libxmljs.parseXml($X, { noent: $V })'
    - pattern: 'libxmljs.parseXml($X, { noent: $V, $$$R })'
    - pattern: 'libxmljs.parseXml($X, { $$$P, noent: $V })'
    - pattern: 'libxmljs.parseXmlString($X, { noent: $V })'
    - pattern: 'libxmljs.parseXmlString($X, { noent: $V, $$$R })'
    - pattern: 'libxmljs.parseXmlString($X, { $$$P, noent: $V })'
    - pattern: 'libxml.parseXml($X, { noent: $V })'
    - pattern: 'libxml.parseXml($X, { noent: $V, $$$R })'
    - pattern: 'libxml.parseXml($X, { $$$P, noent: $V })'
    - pattern: 'parseXml($X, { noent: $V })'
    - pattern: 'parseXml($X, { noent: $V, $$$R })'
    - pattern: 'parseXml($X, { $$$P, noent: $V })'
    - pattern: 'parseXmlString($X, { noent: $V })'
    - pattern: 'parseXmlString($X, { noent: $V, $$$R })'
    - pattern: 'parseXmlString($X, { $$$P, noent: $V })'
constraints:
  V: { regex: '^true$' }
message: "Entity expansion is switched on for this XML parser"
fix:
  - replace: { target: $V, with: "false" }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const doc = libxmljs.parseXml(xml, { noent: true });
```

#### Do

```js
const doc = libxmljs.parseXml(xml);
```

#### Fixed

```js
const doc = libxmljs.parseXml(xml, { noent: false });
```

### JSX

#### Don't

```jsx
const doc = libxmljs.parseXml(xml, { noent: true });
```

#### Do

```jsx
const doc = libxmljs.parseXml(xml);
```

#### Fixed

```jsx
const doc = libxmljs.parseXml(xml, { noent: false });
```

### TypeScript

#### Don't

```ts
const doc = libxmljs.parseXml(xml, { noent: true });
```

#### Do

```ts
const doc = libxmljs.parseXml(xml);
```

#### Fixed

```ts
const doc = libxmljs.parseXml(xml, { noent: false });
```

### TSX

#### Don't

```tsx
const doc = libxmljs.parseXml(xml, { noent: true });
```

#### Do

```tsx
const doc = libxmljs.parseXml(xml);
```

#### Fixed

```tsx
const doc = libxmljs.parseXml(xml, { noent: false });
```

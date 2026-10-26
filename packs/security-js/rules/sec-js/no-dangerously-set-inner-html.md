---
id: sec-js/no-dangerously-set-inner-html
title: "No dangerouslySetInnerHTML from a value that is not sanitized"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether the value needs markup at all is a decision for the author"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
options:
  sanitizers:
    type: "call[]"
    description: "functions whose result may reach the sink: a callee path (global) or { name, module }; `sanitizers_mode: replace` makes them the only ones that count"
    default:
      - { name: DOMPurify.sanitize, module: [dompurify, isomorphic-dompurify] }
      - DOMPurify.sanitize
      - { name: sanitizeHtml, module: sanitize-html }
      - { name: xss, module: xss }
      - { name: filterXSS, module: xss }
      - { name: escapeHtml, module: escape-html }
      - { name: escape, module: [lodash, lodash-es, lodash/escape, he] }
      - { name: _.escape, module: [lodash, lodash-es] }
      - { name: he.escape, module: he }
      - "*.createHTML"
---

React's `dangerouslySetInnerHTML` is given a literal, or a value that went through a sanitizer such as `DOMPurify.sanitize`.

**Your sanitizer:** a value passes when it is a call to one of the `sanitizers`: the built-in list (DOMPurify, sanitize-html, xss, escape-html, lodash and he `escape`, Trusted Types `createHTML`) plus your own, each matched by name and by the module it is imported from, so a local `function sanitize(x) { return x }` does not pass. `sanitizers_mode = "replace"` makes yours the only ones that count. It does not follow a value through a variable: `const c = clean(x)` then `c` at the sink is not a call. Add a project function with `sanitizers = [{ name = "sanitizeHtml", module = "@acme/security" }]`.

**Why:** The name is the warning: the value becomes the element's HTML, so markup an attacker supplies runs in the page (CWE-79).

```codelaw
match:
  pattern:
    context: '({ __html: $V })'
    selector: pair
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
where:
  $V: { not: { calls: sanitizers } }
message: "`$V` becomes HTML here without one of: $OPTION.sanitizers (add yours to `sanitizers`); markup in it runs in the page"
```

## How to fix

1. Render the value as text (`{comment.body}`) wherever it need not carry markup.
2. Where it must, pass it through a sanitizer first: `DOMPurify.sanitize(html)`, and keep the allowed tags narrow.
3. Sanitize once, at the edge, and keep the sanitized string in its own variable so the call is next to the use.

## Examples

### JavaScript

#### Don't

```js
const props = { dangerouslySetInnerHTML: { __html: comment.body } };
```

#### Do

```js
const props = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(comment.body) } };
```

### JSX

#### Don't

```jsx
const props = { dangerouslySetInnerHTML: { __html: comment.body } };
```

#### Do

```jsx
const props = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(comment.body) } };
```

### TypeScript

#### Don't

```ts
const props = { dangerouslySetInnerHTML: { __html: comment.body } };
```

#### Do

```ts
const props = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(comment.body) } };
```

### TSX

#### Don't

```tsx
const props = { dangerouslySetInnerHTML: { __html: comment.body } };
```

#### Do

```tsx
const props = { dangerouslySetInnerHTML: { __html: DOMPurify.sanitize(comment.body) } };
```

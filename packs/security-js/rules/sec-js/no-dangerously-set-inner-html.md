---
id: sec-js/no-dangerously-set-inner-html
title: "No dangerouslySetInnerHTML from a value that is not sanitized"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether the value needs markup at all is a decision for the author"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

React's `dangerouslySetInnerHTML` is given a literal, or a value that went through a sanitizer such as `DOMPurify.sanitize`.

**Why:** The name is the warning: the value becomes the element's HTML, so markup an attacker supplies runs in the page (CWE-79).

```codelaw
match:
  pattern:
    context: '({ __html: $V })'
    selector: pair
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' }, { regex: '^(DOMPurify\.sanitize|purify\.sanitize|sanitizeHtml|sanitize|xss|filterXSS|escapeHtml|escape|_\.escape|he\.escape|[A-Za-z_$][\w$]*\.createHTML)\s*\(' } ] } }
message: "`$V` becomes HTML here without a sanitizer; markup in it runs in the page"
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

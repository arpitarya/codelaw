---
id: sec-js/no-unsanitized-html
title: "No HTML written from a value that is not a literal"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a value that carries markup on purpose shows its tags as text once it is written with `textContent`"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

A value that is not a literal is not written as HTML: `innerHTML`, `outerHTML`, `srcdoc`, `insertAdjacentHTML`, `document.write`, `createContextualFragment`, `setHTMLUnsafe`, or `DOMParser.parseFromString` as `text/html`.

**Why:** The browser parses the value as markup, so a `<script>` or an `onerror=` attribute in it runs in the page: cross-site scripting (CWE-79).

```codelaw
match:
  any:
    - all:
        - pattern: $E.$P = $V
        - regex: '^[^=]*\.innerHTML\s*=[^=]'
    - all:
        - pattern: $E.$P += $V
        - regex: '^[^=]*\.innerHTML\s*\+='
    - all:
        - pattern: $E.$Q = $V
        - regex: '^[^=]*\.(outerHTML|srcdoc)\s*=[^=]'
    - pattern: $E.insertAdjacentHTML($POS, $V)
    - pattern: document.write($V)
    - pattern: document.writeln($V)
    - pattern: $R.createContextualFragment($V)
    - pattern: $E.setHTMLUnsafe($V)
    - pattern: $X.parseFromString($V, "text/html")
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' }, { regex: '^(DOMPurify\.sanitize|purify\.sanitize|sanitizeHtml|sanitize|xss|filterXSS|escapeHtml|escape|_\.escape|he\.escape|[A-Za-z_$][\w$]*\.createHTML)\s*\(' } ] } }
message: "`$V` is written as HTML here; markup in it runs in the page"
fix:
  - replace: { target: $P, with: textContent }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
element.innerHTML = comment.body;
```

#### Do

```js
element.textContent = comment.body;
```

#### Fixed

```js
element.textContent = comment.body;
```

### JSX

#### Don't

```jsx
element.innerHTML = comment.body;
```

#### Do

```jsx
element.textContent = comment.body;
```

#### Fixed

```jsx
element.textContent = comment.body;
```

### TypeScript

#### Don't

```ts
element.innerHTML = comment.body;
```

#### Do

```ts
element.textContent = comment.body;
```

#### Fixed

```ts
element.textContent = comment.body;
```

### TSX

#### Don't

```tsx
element.innerHTML = comment.body;
```

#### Do

```tsx
element.textContent = comment.body;
```

#### Fixed

```tsx
element.textContent = comment.body;
```

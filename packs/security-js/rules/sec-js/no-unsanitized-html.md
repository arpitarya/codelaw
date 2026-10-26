---
id: sec-js/no-unsanitized-html
title: "No HTML written from a value that is not a literal"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a value that carries markup on purpose shows its tags as text once it is written with `textContent`"
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

A value that is not a literal is not written as HTML: `innerHTML`, `outerHTML`, `srcdoc`, `insertAdjacentHTML`, `document.write`, `createContextualFragment`, `setHTMLUnsafe`, or `DOMParser.parseFromString` as `text/html`.

**Your sanitizer:** a value passes when it is a call to one of the `sanitizers`: the built-in list (DOMPurify, sanitize-html, xss, escape-html, lodash and he `escape`, Trusted Types `createHTML`) plus your own, each matched by name and by the module it is imported from, so a local `function sanitize(x) { return x }` does not pass. `sanitizers_mode = "replace"` makes yours the only ones that count. It does not follow a value through a variable: `const c = clean(x)` then `c` at the sink is not a call. Add a project function with `sanitizers = [{ name = "sanitizeHtml", module = "@acme/security" }]`.

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
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
where:
  $V: { not: { calls: sanitizers } }
message: "`$V` is written as HTML here without one of: $OPTION.sanitizers (add yours to `sanitizers`); markup in it runs in the page"
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

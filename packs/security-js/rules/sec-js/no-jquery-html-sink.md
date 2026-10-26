---
id: sec-js/no-jquery-html-sink
title: "No jQuery call that parses a built string as HTML"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether the value is text or markup is a decision for the author"
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

jQuery's `.html(x)` is not given a value that is not a literal, and `$(x)`, `.append(x)`, `.prepend(x)`, `.after(x)`, `.before(x)`, `.replaceWith(x)` and `.wrap(x)` are not given a template or concatenation that holds a tag.

**Your sanitizer:** a value passes when it is a call to one of the `sanitizers`: the built-in list (DOMPurify, sanitize-html, xss, escape-html, lodash and he `escape`, Trusted Types `createHTML`) plus your own, each matched by name and by the module it is imported from, so a local `function sanitize(x) { return x }` does not pass. `sanitizers_mode = "replace"` makes yours the only ones that count. It does not follow a value through a variable: `const c = clean(x)` then `c` at the sink is not a call. Add a project function with `sanitizers = [{ name = "sanitizeHtml", module = "@acme/security" }]`.

**Why:** These calls parse their argument as HTML, so markup in a value built into it runs in the page (CWE-79).

```codelaw
match:
  any:
    - pattern: $O.html($V)
    - pattern: $($H)
    - pattern: jQuery($H)
    - pattern: $O.append($H)
    - pattern: $O.prepend($H)
    - pattern: $O.after($H)
    - pattern: $O.before($H)
    - pattern: $O.replaceWith($H)
    - pattern: $O.wrap($H)
    - pattern: $O.appendTo($H)
    - pattern: $O.prependTo($H)
    - pattern: $O.wrapAll($H)
    - pattern: $O.wrapInner($H)
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
  H:
    any:
      - kind: template_string
      - kind: binary_expression
    regex: '<'
where:
  $V: { not: { calls: sanitizers } }
message: "A built string is parsed as HTML here; markup in a value inside it runs in the page"
```

## How to fix

1. Use `.text(value)` to write a value as text.
2. Build elements with `$("<li>").text(name)` and append them, rather than concatenating markup.
3. Where markup is needed, pass the value through a sanitizer (`DOMPurify.sanitize`) first.

## Examples

### JavaScript

#### Don't

```js
$("#out").html(comment.body);
$("#list").append("<li>" + name + "</li>");
```

#### Do

```js
$("#out").text(comment.body);
$("#list").append($("<li>").text(name));
```

### JSX

#### Don't

```jsx
$("#out").html(comment.body);
$("#list").append("<li>" + name + "</li>");
```

#### Do

```jsx
$("#out").text(comment.body);
$("#list").append($("<li>").text(name));
```

### TypeScript

#### Don't

```ts
$("#out").html(comment.body);
$("#list").append("<li>" + name + "</li>");
```

#### Do

```ts
$("#out").text(comment.body);
$("#list").append($("<li>").text(name));
```

### TSX

#### Don't

```tsx
$("#out").html(comment.body);
$("#list").append("<li>" + name + "</li>");
```

#### Do

```tsx
$("#out").text(comment.body);
$("#list").append($("<li>").text(name));
```

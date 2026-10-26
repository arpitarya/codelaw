---
id: sec-ts/angular-native-element-html
title: "No HTML written to a native element or through Renderer2"
languages: [typescript, tsx]
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

`ElementRef.nativeElement.innerHTML`, `outerHTML` and `insertAdjacentHTML`, and `Renderer2.setProperty(el, "innerHTML", value)`, are not given a value that is not a literal.

**Your sanitizer:** a value passes when it is a call to one of the `sanitizers`: the built-in list (DOMPurify, sanitize-html, xss, escape-html, lodash and he `escape`, Trusted Types `createHTML`) plus your own, each matched by name and by the module it is imported from, so a local `function sanitize(x) { return x }` does not pass. `sanitizers_mode = "replace"` makes yours the only ones that count. It does not follow a value through a variable: `const c = clean(x)` then `c` at the sink is not a call. Add a project function with `sanitizers = [{ name = "sanitizeHtml", module = "@acme/security" }]`.

**Why:** These write the value as markup without Angular's sanitizer, so a `<script>` or an `onerror=` in it runs in the page (CWE-79).

```codelaw
match:
  any:
    - all:
        - pattern: $E.$P = $V
        - regex: '^[^=]*\.nativeElement[\w$.]*\.(inner|outer)HTML\s*=[^=]'
    - pattern: $E2.nativeElement.insertAdjacentHTML($POS, $V)
    - all:
        - pattern: $R.setProperty($EL, $NAME, $V)
        - regex: '\.setProperty\(\s*[^,]+,\s*["''](inner|outer)HTML["'']'
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
where:
  $V: { not: { calls: sanitizers } }
message: "`$V` is written to the page as HTML without Angular's sanitizer or one of: $OPTION.sanitizers (add yours to `sanitizers`)"
```

## How to fix

1. Bind the value in the template: `[innerText]="text"` for text, `[innerHTML]="html"` for markup that Angular sanitizes.
2. With `Renderer2`, use `setProperty(el, "textContent", value)` or `createText` and `appendChild`.
3. If markup is needed and trusted, sanitize it first (`DomSanitizer.sanitize(SecurityContext.HTML, value)`).

## Examples

### TypeScript

#### Don't

```ts
class Box {
  show(html: string) {
    this.el.nativeElement.innerHTML = html;
  }
}
```

#### Do

```ts
class Box {
  show(html: string) {
    this.text = html;
  }
}
```

### TSX

#### Don't

```tsx
class Box {
  show(html: string) {
    this.el.nativeElement.innerHTML = html;
  }
}
```

#### Do

```tsx
class Box {
  show(html: string) {
    this.text = html;
  }
}
```

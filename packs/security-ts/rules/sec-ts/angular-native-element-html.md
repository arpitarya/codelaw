---
id: sec-ts/angular-native-element-html
title: "No HTML written to a native element or through Renderer2"
languages: [typescript, tsx]
fix: manual
fix_reason: "whether the value is text or markup is a decision for the author"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

`ElementRef.nativeElement.innerHTML`, `outerHTML` and `insertAdjacentHTML`, and `Renderer2.setProperty(el, "innerHTML", value)`, are not given a value that is not a literal.

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
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' }, { regex: '^(DOMPurify\.sanitize|purify\.sanitize|sanitizeHtml|sanitize|xss|filterXSS|escapeHtml|escape|_\.escape|he\.escape|[A-Za-z_$][\w$]*\.createHTML)\s*\(' } ] } }
message: "`$V` is written to the page as HTML without Angular's sanitizer"
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

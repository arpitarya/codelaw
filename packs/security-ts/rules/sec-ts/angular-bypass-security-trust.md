---
id: sec-ts/angular-bypass-security-trust
title: "No bypassSecurityTrust call on a value that is not a literal"
languages: [typescript, tsx]
fix: manual
fix_reason: "whether the value needs to be trusted at all is a decision for the author"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

`DomSanitizer.bypassSecurityTrustHtml`, `…Script`, `…Style`, `…Url` and `…ResourceUrl` are not given a value that is not a literal.

**Why:** Angular sanitizes what it binds; these calls tell it to stop, so markup, a script or a `javascript:` URL in the value runs in the page (CWE-79).

```codelaw
match:
  any:
    - pattern: $S.bypassSecurityTrustHtml($V)
    - pattern: $S.bypassSecurityTrustScript($V)
    - pattern: $S.bypassSecurityTrustStyle($V)
    - pattern: $S.bypassSecurityTrustUrl($V)
    - pattern: $S.bypassSecurityTrustResourceUrl($V)
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "`$V` is marked as trusted, so Angular no longer sanitizes it"
```

## How to fix

1. Bind the value (`[innerHTML]="body"`) and let Angular sanitize it: it keeps safe tags and drops scripts.
2. If an element must be trusted (a fixed SVG), pass a literal, or sanitize the value with a library first (`DOMPurify`) and add a comment that says why trust is needed.
3. Never trust a value that came from a request, a URL or a user.

## Examples

### TypeScript

#### Don't

```ts
class Post {
  safe = this.sanitizer.bypassSecurityTrustHtml(this.body);
}
```

#### Do

```ts
class Post {
  safe = this.body;
}
```

### TSX

#### Don't

```tsx
class Post {
  safe = this.sanitizer.bypassSecurityTrustHtml(this.body);
}
```

#### Do

```tsx
class Post {
  safe = this.body;
}
```

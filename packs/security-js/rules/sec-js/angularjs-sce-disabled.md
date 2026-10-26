---
id: sec-js/angularjs-sce-disabled
title: "No AngularJS strict contextual escaping switched off"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which sources the application loads from is a decision for the author"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

AngularJS's `$sce` stays on, `resourceUrlWhitelist` has no `"**"`, the href and image whitelists are not match-everything, and `$sce.trustAs*` is not given an unsanitized value.

**Why:** `$sce` is what stops a bound string from becoming HTML, a script or a frame source; switching it off or trusting every value turns each binding into an XSS sink (CWE-79).

```codelaw
match:
  any:
    - pattern: $sceProvider.enabled(false)
    - all:
        - pattern: $SD.resourceUrlWhitelist($A)
        - regex: '["'']\*\*["'']'
    - all:
        - pattern: $CP.$M($R)
        - regex: '(aHref|imgSrc)SanitizationWhitelist\(\s*/(\^?\.\*?\$?|\^)/[a-z]*\s*\)'
    - pattern: $SCE.trustAsHtml($V)
    - pattern: $SCE.trustAsJs($V)
    - pattern: $SCE.trustAsUrl($V)
    - pattern: $SCE.trustAsResourceUrl($V)
constraints:
  V: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' }, { regex: '^(DOMPurify\.sanitize|purify\.sanitize|sanitizeHtml|sanitize|xss|filterXSS|escapeHtml|escape|_\.escape|he\.escape|[A-Za-z_$][\w$]*\.createHTML)\s*\(' } ] } }
message: "AngularJS's strict contextual escaping is weakened here"
```

## How to fix

1. Leave `$sceProvider.enabled` at its default (`true`).
2. List the hosts a template may load from in `resourceUrlWhitelist`: `["self", "https://cdn.example.com/**"]`, not `"**"`.
3. Pass a value to `$sce.trustAsHtml` only after a sanitizer (`$sanitize`, DOMPurify) has cleaned it.

## Examples

### JavaScript

#### Don't

```js
app.config(function ($sceProvider) {
  $sceProvider.enabled(false);
});
```

#### Do

```js
app.config(function ($sceProvider) {
  $sceProvider.enabled(true);
});
```

### JSX

#### Don't

```jsx
app.config(function ($sceProvider) {
  $sceProvider.enabled(false);
});
```

#### Do

```jsx
app.config(function ($sceProvider) {
  $sceProvider.enabled(true);
});
```

### TypeScript

#### Don't

```ts
app.config(function ($sceProvider) {
  $sceProvider.enabled(false);
});
```

#### Do

```ts
app.config(function ($sceProvider) {
  $sceProvider.enabled(true);
});
```

### TSX

#### Don't

```tsx
app.config(function ($sceProvider) {
  $sceProvider.enabled(false);
});
```

#### Do

```tsx
app.config(function ($sceProvider) {
  $sceProvider.enabled(true);
});
```

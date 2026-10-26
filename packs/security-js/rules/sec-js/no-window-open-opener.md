---
id: sec-js/no-window-open-opener
title: "No window.open without noopener"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a page that does need `window.opener` (a popup that returns a result) stops getting it"
stability: preview
security: { cwe: [1022], owasp: ["A05:2025"], severity: low }
since: 0.3.0
---

`window.open` is given `noopener` (and `noreferrer`) in its features, so the new page cannot reach back through `window.opener`.

**Why:** A page opened without `noopener` gets `window.opener` and can navigate the opener to a phishing page (CWE-1022, reverse tabnabbing).

```codelaw
match:
  any:
    - pattern: window.open($U, $T)
    - pattern: window.open($U)
    - all:
        - pattern: window.open($A, $B, $FEATURES)
        - not:
            has:
              regex: 'noopener'
              stopBy: end
message: "`window.open` without `noopener` leaves `window.opener` set on the new page"
fix:
  - replace: "window.open($U, $T, \"noopener,noreferrer\")"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
window.open(url, "_blank");
```

#### Do

```js
window.open(url, "_blank", "noopener,noreferrer");
```

#### Fixed

```js
window.open(url, "_blank", "noopener,noreferrer");
```

### JSX

#### Don't

```jsx
window.open(url, "_blank");
```

#### Do

```jsx
window.open(url, "_blank", "noopener,noreferrer");
```

#### Fixed

```jsx
window.open(url, "_blank", "noopener,noreferrer");
```

### TypeScript

#### Don't

```ts
window.open(url, "_blank");
```

#### Do

```ts
window.open(url, "_blank", "noopener,noreferrer");
```

#### Fixed

```ts
window.open(url, "_blank", "noopener,noreferrer");
```

### TSX

#### Don't

```tsx
window.open(url, "_blank");
```

#### Do

```tsx
window.open(url, "_blank", "noopener,noreferrer");
```

#### Fixed

```tsx
window.open(url, "_blank", "noopener,noreferrer");
```

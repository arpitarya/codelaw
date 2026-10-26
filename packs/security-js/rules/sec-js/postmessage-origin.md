---
id: sec-js/postmessage-origin
title: "postMessage names its target, a message handler checks the sender"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the origin to name is known only to the application"
stability: preview
security: { cwe: [346], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

`postMessage` is given a target origin, not `"*"`, and a `message` listener that is a function reads `event.origin` before it trusts `event.data`.

**Why:** With `"*"` any page that holds the window receives the message, and a listener that skips the origin check accepts messages from any page that can open or frame this one (CWE-346).

```codelaw
match:
  any:
    - pattern: $W.postMessage($M, "*")
    - pattern: $W.postMessage($M, '*')
    - pattern: $W.postMessage($M, "*", $$$REST)
    - pattern: $T.addEventListener("message", $FN)
    - pattern: $T.addEventListener("message", $FN, $$$REST)
    - pattern: $T.onmessage = $FN
constraints:
  FN:
    any:
      - kind: arrow_function
      - kind: function_expression
    not:
      regex: 'origin|source'
message: "This message exchange does not name or check the other window's origin"
```

## How to fix

1. Give `postMessage` the exact origin of the receiver as its second argument.
2. In the listener, compare `event.origin` with the exact origin you expect (not `includes` or `endsWith`) and return before touching `event.data`.
3. Check `event.source` is the window you opened or framed when more than one window can send.

## Examples

### JavaScript

#### Don't

```js
parent.postMessage(payload, "*");
window.addEventListener("message", (event) => {
  use(event.data);
});
```

#### Do

```js
parent.postMessage(payload, "https://app.example.com");
window.addEventListener("message", (event) => {
  if (event.origin !== "https://app.example.com") return;
  use(event.data);
});
```

### JSX

#### Don't

```jsx
parent.postMessage(payload, "*");
window.addEventListener("message", (event) => {
  use(event.data);
});
```

#### Do

```jsx
parent.postMessage(payload, "https://app.example.com");
window.addEventListener("message", (event) => {
  if (event.origin !== "https://app.example.com") return;
  use(event.data);
});
```

### TypeScript

#### Don't

```ts
parent.postMessage(payload, "*");
window.addEventListener("message", (event) => {
  use(event.data);
});
```

#### Do

```ts
parent.postMessage(payload, "https://app.example.com");
window.addEventListener("message", (event) => {
  if (event.origin !== "https://app.example.com") return;
  use(event.data);
});
```

### TSX

#### Don't

```tsx
parent.postMessage(payload, "*");
window.addEventListener("message", (event) => {
  use(event.data);
});
```

#### Do

```tsx
parent.postMessage(payload, "https://app.example.com");
window.addEventListener("message", (event) => {
  if (event.origin !== "https://app.example.com") return;
  use(event.data);
});
```

---
id: sec-html/iframe-allow-powerful
title: "A cross-origin iframe is not given a powerful feature"
languages: [html]
fix: manual
fix_reason: "which framed sites deserve the permission is the owner's decision"
stability: preview
security: { cwe: [250], owasp: ["A02:2025"], severity: low }
since: 0.2.0
---

An `<iframe>` whose `src` is on another origin does not list `camera`, `microphone`, `geolocation`, `payment` or another powerful feature in `allow`.

**Why:** The framed site gets the permission prompt, with the page's name on it, and whatever that site serves later inherits the grant.

```codelaw
native: sec-html/iframe-allow-powerful
```

## How to fix

1. Drop the feature from `allow`.
2. Or limit it to the origin that needs it: `allow="camera https://meet.example.com"`, and keep `'none'` for the rest.

## Examples

### HTML

#### Don't

```html
<iframe src="https://meet.example.com/room" allow="camera; microphone"></iframe>
```

#### Do

```html
<iframe src="https://meet.example.com/room" allow="fullscreen"></iframe>
```

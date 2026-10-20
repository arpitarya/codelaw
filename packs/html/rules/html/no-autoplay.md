---
id: html/no-autoplay
title: "Media does not play by itself"
languages: [html]
fix: suggest
fix_reason: "the media no longer starts on load"
stability: preview
options:
  include: { type: "string[]", default: ["audio", "video"], description: "elements on which `autoplay` is reported" }
ported_from: [html-validate/no-autoplay]
tags: [a11y]
since: 0.1.0
---

`<audio>` and `<video>` don't have `autoplay`.

**Why:** Sound that starts by itself talks over screen readers and startles users; WCAG asks that users control playback.

```codelaw
native: html/no-autoplay
```

## Examples

### HTML

#### Don't

```html
<video src="intro.mp4" autoplay controls></video>
```

#### Do

```html
<video src="intro.mp4" controls></video>
```

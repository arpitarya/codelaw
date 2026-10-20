---
id: css/media-type-no-deprecated
title: "No deprecated media types"
languages: [css]
fix: manual
fix_reason: "the media features that describe the device are the author's call"
stability: preview
options:
  ignoreMediaTypes: { type: "string[]", default: [], description: "media types to accept" }
ported_from: [stylelint/media-type-no-deprecated]
since: 0.1.0
---

Media queries use no media type the specifications deprecated: `tv`, `handheld`, `projection`, `aural`, `braille`, `embossed`, `speech` and `tty`.

**Why:** Deprecated media types match nothing, so the block never applies.

```codelaw
native: css/media-type-no-deprecated
```

## How to fix

1. Replace the media type with `screen`, `print` or `all`, adding media features that describe the device.

## Examples

### CSS

#### Don't

```css
@media tv {
  a {
    color: red;
  }
}
```

#### Do

```css
@media screen {
  a {
    color: red;
  }
}
```

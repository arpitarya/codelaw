---
id: css/import-notation
title: "`@import` with `url()`"
languages: [css]
fix: auto
stability: stable
ported_from: [stylelint/import-notation]
since: 0.1.0
---

`@import` names its stylesheet with `url()`, not a bare string.

**Why:** One notation for imports is easier to search and read.

```codelaw
native: css/import-notation
```

## Examples

### CSS

#### Don't

```css
@import "theme.css";
```

#### Do

```css
@import url("theme.css");
```

## Differences from the source

Ported with stylelint-config-standard's setting, `url`.

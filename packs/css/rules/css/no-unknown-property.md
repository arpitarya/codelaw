---
id: css/no-unknown-property
title: "No unknown properties"
languages: [css, scss]
fix: suggest
fix_reason: "the closest known name is a guess at what was meant"
stability: preview
options:
  ignoreProperties: { type: "string[]", default: [], description: "properties to accept; `/regex/` for a pattern" }
  checkPrefixed: { type: "boolean", default: false, description: "also check vendor-prefixed properties" }
ported_from: [stylelint/property-no-unknown]
since: 0.1.0
---

Every property is a known CSS property. Custom properties, vendor-prefixed properties and descriptors inside at-rules such as `@font-face` are skipped.

**Why:** The browser drops a declaration whose property it does not know, so a typo silently loses the style.

```codelaw
native: css/no-unknown-property
```

## Examples

### CSS

#### Don't

```css
a {
  colr: red;
}
```

#### Do

```css
a {
  color: red;
}
```

### SCSS

#### Don't

```scss
a {
  colr: red;
}
```

#### Do

```scss
a {
  color: red;
}
```

## Differences from the source

The property table is `known-css-properties` plus the names stylelint keeps from before it moved to CSSTree, not CSSTree's MDN-derived data. A fix is offered when a known property is one or two edits away.

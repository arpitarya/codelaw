---
id: css/alpha-value-notation
title: "Alpha values are percentages"
languages: [css]
fix: suggest
fix_reason: "old browsers accept only numbers for alpha in `rgba()`"
stability: preview
options:
  exceptProperties: { type: "string[]", default: ["opacity", "fill-opacity", "flood-opacity", "stop-opacity", "stroke-opacity"], description: "properties whose alpha is a number" }
ported_from: [stylelint/alpha-value-notation]
since: 0.1.0
---

The alpha channel of a colour function is written as a percentage (`50%`, not `0.5`). The opacity properties (`opacity`, `fill-opacity`, `flood-opacity`, `stop-opacity`, `stroke-opacity`) take numbers instead.

**Why:** One notation for alpha makes values easier to compare at a glance.

```codelaw
native: css/alpha-value-notation
```

## Examples

### CSS

#### Don't

```css
a {
  color: rgb(0 0 0 / 0.5);
}
```

#### Do

```css
a {
  color: rgb(0 0 0 / 50%);
}
```

## Differences from the source

Ported with stylelint-config-standard's setting, `percentage`; the `number` setting is not offered.

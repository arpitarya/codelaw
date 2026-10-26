---
id: css/declaration-block-no-redundant-longhand-properties
title: "Shorthands instead of complete sets of longhands"
languages: [css, scss]
fix: manual
fix_reason: "combining longhands into a shorthand value needs the shorthand's grammar"
stability: preview
ported_from: [stylelint/declaration-block-no-redundant-longhand-properties]
since: 0.1.0
---

A block does not declare every longhand of a shorthand (`margin-top`, `margin-right`, `margin-bottom`, `margin-left`) where the shorthand would do, nor repeat a shorthand's value in one of its longhands.

**Why:** One shorthand is shorter and says plainly that every side is set.

```codelaw
native: css/declaration-block-no-redundant-longhand-properties
```

## How to fix

1. Replace the longhands with the shorthand, giving their values in the shorthand's order.

## Examples

### CSS

#### Don't

```css
a {
  margin-top: 1px;
  margin-right: 2px;
  margin-bottom: 3px;
  margin-left: 4px;
}
```

#### Do

```css
a {
  margin: 1px 2px 3px 4px;
}
```

### SCSS

#### Don't

```scss
a {
  margin-top: 1px;
  margin-right: 2px;
  margin-bottom: 3px;
  margin-left: 4px;
}
```

#### Do

```scss
a {
  margin: 1px 2px 3px 4px;
}
```

## Differences from the source

Reported without a fix; stylelint rewrites some sets into the shorthand.

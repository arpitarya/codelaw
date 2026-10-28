---
id: css/at-rule-no-deprecated
title: "No deprecated at-rules"
languages: [css, scss]
fix: auto
stability: stable
options:
  ignoreAtRules: { type: "string[]", default: [], description: "at-rules to accept; `/regex/` for a pattern" }
ported_from: [stylelint/at-rule-no-deprecated]
since: 0.1.0
---

The sheet uses no at-rule that was removed from or deprecated by the CSS specifications: `@viewport`, `@document` and `@nest`.

**Why:** Browsers dropped or never shipped these at-rules, so what they wrap does not apply.

```codelaw
native: css/at-rule-no-deprecated
```

## Examples

### CSS

#### Don't

```css
.card {
  @nest .theme-dark & {
    color: white;
  }
}
```

#### Do

```css
.card {
  .theme-dark & {
    color: white;
  }
}
```

### SCSS

#### Don't

```scss
.card {
  @nest .theme-dark & {
    color: white;
  }
}
```

#### Do

```scss
.card {
  .theme-dark & {
    color: white;
  }
}
```

## Differences from the source

Only `@nest` has a fix (drop the keyword); `@viewport` and `@document` are reported without one.

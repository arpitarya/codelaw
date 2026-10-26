---
id: scss/max-nesting-depth
title: "Limit nesting depth"
languages: [scss]
fix: manual
fix_reason: "flattening a selector is the author's call"
stability: preview
options:
  max: { type: "number", default: 3, description: "the deepest nesting allowed" }
ported_from: [stylelint/max-nesting-depth]
since: 0.3.0
---

A style rule nests at most `max` levels inside other style rules (default 3): `.a { .b { .c { .d {} } } }` is three levels of nesting.

**Why:** Deep nesting couples a rule to the markup's shape, raises specificity, and produces long selectors nobody wrote on purpose.

```codelaw
native: scss/max-nesting-depth
```

## How to fix

1. Lift the inner rule out and give it its own class.
2. Use `&-suffix` (BEM-style names) instead of a new level.

## Examples

### SCSS

#### Don't

```scss
.a {
  .b {
    .c {
      .d {
        .e {
          color: red;
        }
      }
    }
  }
}
```

#### Do

```scss
.a {
  .b {
    .c {
      .d {
        color: red;
      }
    }
  }
}
```

## Differences from the source

Nested properties (`font: { … }`) and at-rules such as `@media` and `@include` do not count as a level; only style rules do.

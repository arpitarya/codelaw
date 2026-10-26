---
id: css-modules/class-naming
title: "CSS Module class names can be read as styles.name"
languages: [css, scss]
fix: manual
fix_reason: "renaming a class must update every file that reads it"
options:
  style: { type: "string", default: "camelCase", description: "`camelCase` (a name `styles.x` reaches without brackets), `kebab` (kebab-case) or `any` (no check)" }
stability: preview
since: 0.3.0
---

A class of a CSS Module is named so that JavaScript reads it as `styles.name`: camelCase by default. With `style = "kebab"` it is kebab-case instead, and `any` turns the check off.

**Why:** `.btn-primary` is `styles["btn-primary"]` and cannot be `styles.btn-primary`; a name that needs brackets is easy to mistype and is not found by a search for `styles.btnPrimary`.

The bundler's `localsConvention` (`[css] modules_convention`) counts: under `camelCase` or `dashes` a kebab-case class is reached as `styles.btnPrimary`, so it passes. Classes inside `:global(…)` are not checked.

```codelaw
native: css-modules/class-naming
```

## How to fix

1. Rename the class (and every `styles.…` that reads it) to camelCase.
2. Or set `[css] modules_convention = "camelCase"` if the bundler already maps the names.

## Examples

### CSS

#### Don't

```css files
/* src/button.module.css */
.btn-primary { color: red; }
```

#### Do

```css files
/* src/button.module.css */
.btnPrimary { color: red; }
```

### SCSS

#### Don't

```scss files
/* src/button.module.scss */
.btn-primary { color: red; }
```

#### Do

```scss files
/* src/button.module.scss */
.btnPrimary { color: red; }
```

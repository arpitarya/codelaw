---
id: css-modules/no-global
title: "A CSS Module stays local"
languages: [css, scss]
fix: manual
fix_reason: "a global selector exists to reach markup outside the module, which is the author's call"
stability: preview
since: 0.3.0
---

A CSS Module has no `:global` selector.

**Why:** `:global` leaves the module's scope: the rule can match, and be matched by, any element on the page, which is what modules exist to prevent. Put global rules in a plain stylesheet.

```codelaw
native: css-modules/no-global
```

## How to fix

1. Move the rule into a plain `.css` file, or give the markup a local class.

## Examples

### CSS

#### Don't

```css files
/* src/page.module.css */
:global(.dark) .page { color: white; }
```

#### Do

```css files
/* src/page.module.css */
.dark .page { color: white; }
```

### SCSS

#### Don't

```scss files
/* src/page.module.scss */
:global(.dark) .page { color: white; }
```

#### Do

```scss files
/* src/page.module.scss */
.dark .page { color: white; }
```

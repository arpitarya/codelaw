---
id: css-modules/no-unused-class
title: "A CSS Module class is read somewhere"
languages: [css, scss]
scope: project
severity: warn
fix: suggest
fix_reason: "a class can be read by a name built at run time or by code outside the repo, so deleting its rule needs a person to accept the diff"
stability: preview
since: 0.3.0
ported_from: [eslint-plugin-css-modules/no-unused-class]
---

Every class a CSS Module defines is read by a JavaScript or TypeScript file that imports the module: `styles.name`, `styles["name"]`, `const { name } = styles`, `import { name } from "./x.module.css"`, or composed by another class with `composes`.

**Why:** a class nobody reads is dead CSS that every page downloads, and it hides the classes the component really uses.

A module is left alone when any importer reads it by a computed name (`styles[name]`, `styles[\`btn-${size}\`]`), spreads it, or passes it to a function: the rule cannot tell which classes that reaches, so it reports none rather than guess. A module no file imports is left alone too.

```codelaw
graph:
  module:
    classes: unused
message: "`.$CLASS` is defined in `$FILE` but no file reads `styles.$CLASS`"
fix:
  - else:
    do: [remove-rule]
```

## How to fix

1. Search for the class: `styles.name`, a bracket access, and any `composes` that names it.
2. If a component builds the name at run time, read it by a plain `styles.name` somewhere, or accept that this module is dynamic.
3. Otherwise delete the rule (`codelaw fix --unsafe` does it when the class is the rule's only selector).

## Examples

### CSS

#### Don't

```css files
/* src/Card.module.css */
.card { padding: 1rem; }
.legacy { color: red; }
// src/Card.jsx
import styles from "./Card.module.css";
export const Card = () => <div className={styles.card} />;
```

#### Do

```css files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.jsx
import styles from "./Card.module.css";
export const Card = () => <div className={styles.card} />;
```

### SCSS

#### Don't

```scss files
/* src/Card.module.scss */
$gap: 1rem;
.card { padding: $gap; }
.legacy { color: red; }
// src/Card.jsx
import styles from "./Card.module.scss";
export const Card = () => <div className={styles.card} />;
```

#### Do

```scss files
/* src/Card.module.scss */
$gap: 1rem;
.card { padding: $gap; }
// src/Card.jsx
import styles from "./Card.module.scss";
export const Card = () => <div className={styles.card} />;
```

## Differences from the source

eslint-plugin-css-modules's `no-unused-class` reads one JS file against its stylesheet; this rule reads the project, so a module imported from several files is judged by all of them, `composes` counts as a use, and a computed access silences the module (the plugin's `camelCase` option is `[css] modules_convention`).

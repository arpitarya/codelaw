---
id: css-modules/no-undefined-class
title: "A CSS Module class that is read exists"
languages: [javascript, jsx, typescript, tsx]
scope: project
severity: error
fix: manual
fix_reason: "whether to add the class or fix the name is the author's call"
stability: preview
since: 0.3.0
ported_from: [eslint-plugin-css-modules/no-undef-class]
---

`styles.name` reads a class the imported CSS Module defines. The finding is in the JavaScript or TypeScript file, at the name.

**Why:** reading a class the module does not define gives `undefined`, so the element silently loses its style; a typo or a renamed class shows up only on screen.

What the module defines counts `:export` keys and `@value` names, and the names `[css] modules_convention` makes reachable (`.btn-primary` is `styles.btnPrimary` under `camelCase`). A module whose class names the rule cannot work out (Sass interpolation, `&-suffix` on a selector list) is left alone, and so is a computed access such as `styles[name]`.

```codelaw
graph:
  module:
    classes: undefined
message: "`styles.$CLASS`: `$FILE` defines no `.$CLASS`"
```

## How to fix

1. Correct the name in the JavaScript or TypeScript file, or add the class to the module.
2. If the bundler maps names (`localsConvention`), set `[css] modules_convention` so the mapped names count.

## Examples

### JavaScript

#### Don't

```js files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.js
import styles from "./Card.module.css";
export const card = styles.cards;
```

#### Do

```js files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.js
import styles from "./Card.module.css";
export const card = styles.card;
```

### JSX

#### Don't

```jsx files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.jsx
import styles from "./Card.module.css";
export const Card = () => <div className={styles.cards} />;
```

#### Do

```jsx files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.jsx
import styles from "./Card.module.css";
export const Card = () => <div className={styles.card} />;
```

### TypeScript

#### Don't

```ts files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.ts
import styles from "./Card.module.css";
export const card: string = styles["cards"];
```

#### Do

```ts files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.ts
import styles from "./Card.module.css";
export const card: string = styles["card"];
```

### TSX

#### Don't

```tsx files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.tsx
import styles from "./Card.module.css";
export const Card = () => <div className={styles.cards} />;
```

#### Do

```tsx files
/* src/Card.module.css */
.card { padding: 1rem; }
// src/Card.tsx
import styles from "./Card.module.css";
export const Card = () => <div className={styles.card} />;
```

## Differences from the source

eslint-plugin-css-modules's `no-undef-class` needs the stylesheet's path in each import; this rule follows the import through the project graph, and understands `[css] modules`, `composes`, `:export` and `@value`.

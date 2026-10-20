---
id: js/import-groups
title: "Imports are grouped builtin, external, internal, relative"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
options:
  internal: { type: "string[]", default: ["@/", "~/"], description: "prefixes of internal imports" }
since: 0.1.0
---

The file's leading imports come in four groups, in order — Node.js built-ins, packages, internal
aliases (the `internal` option, `@/` and `~/` by default), relative paths — with a blank line
between groups.

**Why:** Grouped imports show at a glance what a module depends on, and keep diffs of the import
block small.

```codelaw
native: js/import-groups
```

## Examples

### JavaScript

#### Don't

```js
import { helper } from "./helper";
import fs from "node:fs";
import React from "react";
helper(fs, React);
```

#### Do

```js
import fs from "node:fs";

import React from "react";

import { helper } from "./helper";
helper(fs, React);
```

### JSX

#### Don't

```jsx
import { helper } from "./helper";
import fs from "node:fs";
import React from "react";
helper(fs, React);
```

#### Do

```jsx
import fs from "node:fs";

import React from "react";

import { helper } from "./helper";
helper(fs, React);
```

### TypeScript

#### Don't

```ts
import { helper } from "./helper";
import fs from "node:fs";
import React from "react";
helper(fs, React);
```

#### Do

```ts
import fs from "node:fs";

import React from "react";

import { helper } from "./helper";
helper(fs, React);
```

### TSX

#### Don't

```tsx
import { helper } from "./helper";
import fs from "node:fs";
import React from "react";
helper(fs, React);
```

#### Do

```tsx
import fs from "node:fs";

import React from "react";

import { helper } from "./helper";
helper(fs, React);
```

## Differences from the source

A block of imports with a comment inside is left alone, so no comment is moved away from the import
it describes. A side-effect import (`import './polyfills'`, a stylesheet) stays where it is, since moving it
changes what runs first; only the imports between two of them are ordered.

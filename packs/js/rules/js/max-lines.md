---
id: js/max-lines
title: "A file stays under a maximum number of lines"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "splitting a file changes its structure; which exports belong together is the author's call"
stability: preview
options:
  max: { type: "integer", default: 300, description: "the most lines a file may have" }
  skip_blank_lines: { type: "boolean", default: true, description: "blank lines do not count" }
  skip_comments: { type: "boolean", default: true, description: "lines that hold only a comment do not count" }
  skip_imports: { type: "boolean", default: false, description: "the import block does not count" }
ported_from: [eslint/max-lines]
since: 0.3.0
---

A file has at most `max` lines (300 by default). Blank lines and lines that hold only a comment (a `{/* … */}`
in JSX is one) are not counted unless `skip_blank_lines` or `skip_comments` is `false`; a line that holds code and a
comment counts as code. `skip_imports` leaves the import block out too. The rule reports once, on the first line over
the limit, with the measured count: `412 lines (max 300; blank lines and comments not counted)`.

**Why:** A long file is hard to hold in your head, slow to review and the usual place a second responsibility hides. The
limit is a prompt to split, not a goal in itself: raise `max` for a file that is long on purpose, or put
a generated file where `[files] lint_generated = false` skips it. Only the host file is measured, never a `<script>` in
HTML or a code block in Markdown.

```codelaw
native: js/max-lines
```

## How to fix

1. Find a cohesive group of exports (a component and its helpers, one feature's types) and move it to a new module next to the file.
2. Import it back where it is used, or re-export it from the old file so callers do not change.
3. If the file is long on purpose, raise `max` for it in an `[[overrides]]` entry, or freeze it with `codelaw baseline add --rule js/max-lines`.

## Examples

### JavaScript

#### Don't

```js options={"max":3}
const a = 1;
const b = 2;
const c = 3;
const d = 4;
```

#### Do

```js options={"max":3}
import { d } from "./more.js";
const a = 1;
const b = 2;
```

### JSX

#### Don't

```jsx options={"max":3}
const a = <i />;
const b = <b />;
const c = <u />;
const d = <s />;
```

#### Do

```jsx options={"max":3}
import { d } from "./more.jsx";
const a = <i />;
const b = <b />;
```

### TypeScript

#### Don't

```ts options={"max":3}
const a: number = 1;
const b: number = 2;
const c: number = 3;
const d: number = 4;
```

#### Do

```ts options={"max":3}
import { d } from "./more";
const a: number = 1;
const b: number = 2;
```

### TSX

#### Don't

```tsx options={"max":3}
const a: JSX.Element = <i />;
const b: JSX.Element = <b />;
const c: JSX.Element = <u />;
const d: JSX.Element = <s />;
```

#### Do

```tsx options={"max":3}
import { d } from "./more";
const a: JSX.Element = <i />;
const b: JSX.Element = <b />;
```

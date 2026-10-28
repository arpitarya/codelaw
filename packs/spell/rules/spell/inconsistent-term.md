---
id: spell/inconsistent-term
title: "A word is spelled one way across the project"
languages: [javascript, jsx, typescript, tsx, python, json, yaml, toml, markdown, css, scss, html]
scope: project
fix: suggest
fix_reason: "the spelling a project uses most is usually its house style, but a quotation, a product name or a file in the other locale may be right as it is"
stability: preview
since: 0.3.0
---

The project spells one word two ways: `canceled` in one place, `cancelled` in another; `color` and `colour`; `organize` and `organise`. Each use of the spelling the project writes less often is reported, wherever it is, and the fix writes the spelling it writes more often, in the word's case — on a tie, the American one. The two ways are VarCon's one-to-one US and British pairs, as `spell/locale-variant` reads them (a pair right in both locales under some meaning, such as `check` and `cheque`, is never reported), and the British `-ise` and the Oxford `-ize` count as two ways. Prose is read as `spell/known-typo` reads it — comments, docstrings, prose strings, Markdown and HTML text, and string values — in every file of the project, the files `[files]` leaves out included, but not names: a declared name is an API. Each form is its own word, so `canceled` and `cancelling` are not compared. A word in `[spell] words` is not counted. With `[spell] locale` set the rule reports nothing: the locale says which spelling is right, and `spell/locale-variant` reports the other.

**Why:** one word written one way reads as one voice, and a search for it finds every place it is used.

It needs every file's words at once, so it is a project rule: the engine reads each file's prose on the graph stage and caches its words by the file's content.

```codelaw
graph:
  terms: inconsistent
message: "a word the project spells another way more often"
fix:
  do: [respell]
```

The message names the word, the other spelling and how often the project writes each.

## How to fix

1. Run `codelaw fix --unsafe` to write the spelling the project uses most, or change the word by hand.
2. If the project means one locale, set `[spell] locale` and turn on `spell/locale-variant`, which fixes the other locale's spelling unattended.
3. For a quotation or a proper name (`Colour Science Ltd`), excuse the line: `// codelaw-ignore spell/inconsistent-term: <reason>`.

## Examples

### JavaScript

#### Don't

```js
// The order is canceled when the buyer leaves.
// A canceled order is kept for a week.
// The cancelled orders are listed below.
export const orders = [];
```

#### Do

```js
// The order is canceled when the buyer leaves.
// A canceled order is kept for a week.
// The canceled orders are listed below.
export const orders = [];
```

### JSX

#### Don't

```jsx
export const Note = () => <p>The order is canceled. A canceled order is kept. The cancelled orders are listed.</p>;
```

#### Do

```jsx
export const Note = () => <p>The order is canceled. A canceled order is kept. The canceled orders are listed.</p>;
```

### TypeScript

#### Don't

```ts
// The order is canceled when the buyer leaves.
// A canceled order is kept for a week.
// The cancelled orders are listed below.
export const orders: string[] = [];
```

#### Do

```ts
// The order is canceled when the buyer leaves.
// A canceled order is kept for a week.
// The canceled orders are listed below.
export const orders: string[] = [];
```

### TSX

#### Don't

```tsx
export const Note = (): JSX.Element => <p>The order is canceled. A canceled order is kept. The cancelled orders are listed.</p>;
```

#### Do

```tsx
export const Note = (): JSX.Element => <p>The order is canceled. A canceled order is kept. The canceled orders are listed.</p>;
```

### Python

#### Don't

```python
def orders():
    """The order is canceled when the buyer leaves.

    A canceled order is kept for a week; the cancelled orders are listed.
    """
```

#### Do

```python
def orders():
    """The order is canceled when the buyer leaves.

    A canceled order is kept for a week; the canceled orders are listed.
    """
```

### JSON

#### Don't

```json
{
  "left": "The order is canceled when the buyer leaves",
  "kept": "A canceled order is kept for a week",
  "list": "The cancelled orders are listed below"
}
```

#### Do

```json
{
  "left": "The order is canceled when the buyer leaves",
  "kept": "A canceled order is kept for a week",
  "list": "The canceled orders are listed below"
}
```

### YAML

#### Don't

```yaml
left: The order is canceled when the buyer leaves
kept: A canceled order is kept for a week
list: The cancelled orders are listed below
```

#### Do

```yaml
left: The order is canceled when the buyer leaves
kept: A canceled order is kept for a week
list: The canceled orders are listed below
```

### TOML

#### Don't

```toml
# The order is canceled when the buyer leaves.
# A canceled order is kept for a week.
# The cancelled orders are listed below.
kept = 7
```

#### Do

```toml
# The order is canceled when the buyer leaves.
# A canceled order is kept for a week.
# The canceled orders are listed below.
kept = 7
```

### Markdown

#### Don't

```md
The order is canceled when the buyer leaves. A canceled order is kept for a week.

The cancelled orders are listed below.
```

#### Do

```md
The order is canceled when the buyer leaves. A canceled order is kept for a week.

The canceled orders are listed below.
```

### CSS

#### Don't

```css
/* The order is canceled when the buyer leaves. */
/* A canceled order is kept for a week. */
/* The cancelled orders are listed below. */
.order { color: gray; }
```

#### Do

```css
/* The order is canceled when the buyer leaves. */
/* A canceled order is kept for a week. */
/* The canceled orders are listed below. */
.order { color: gray; }
```

### SCSS

#### Don't

```scss
// The order is canceled when the buyer leaves.
// A canceled order is kept for a week.
// The cancelled orders are listed below.
.order { color: gray; }
```

#### Do

```scss
// The order is canceled when the buyer leaves.
// A canceled order is kept for a week.
// The canceled orders are listed below.
.order { color: gray; }
```

### HTML

#### Don't

```html
<p>The order is canceled when the buyer leaves.</p>
<p>A canceled order is kept for a week.</p>
<p>The cancelled orders are listed below.</p>
```

#### Do

```html
<p>The order is canceled when the buyer leaves.</p>
<p>A canceled order is kept for a week.</p>
<p>The canceled orders are listed below.</p>
```

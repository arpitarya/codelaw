---
id: spell/unknown-word
title: "Every word of prose is in a word list"
languages: [javascript, jsx, typescript, tsx, python, json, yaml, toml, markdown, css, scss, html]
fix: suggest
fix_reason: "a word no list holds is often a real name, a product or a term of the trade; the nearest dictionary word is a guess the author confirms"
stability: preview
since: 0.3.0
---

A word in a comment, a docstring, a prose string, Markdown text, HTML text or an HTML `alt`, `title`, `aria-label` or `placeholder`, or a string value of JSON, YAML or TOML is in no word list: not in the pack's dictionary (SCOWL's US and British English up to size 60, with every form written out, plus the software terms), not on the known-typo list (`spell/known-typo` reports those) and not in `[spell] words`. A word in capitals is an acronym and is not checked; so are words shorter than four letters and every token `spell/known-typo` skips (URLs, paths, identifiers, hashes).

**Why:** the known-typo list catches the misspellings codelaw can predict; the dictionary catches the rest (`seperately`, `responce`). It is in `spell/strict`, not `spell/recommended`, because code is full of real words no dictionary holds.

The fix replaces the word with the nearest dictionary word: fewest edits (Damerau-Levenshtein, at most two, the first letter kept), then the commoner word, then alphabetical order. The message names up to three.

```codelaw
native: spell/unknown-word
```

## How to fix

1. If the word is misspelled, run `codelaw fix --unsafe` to take the first suggestion, or write the word the message names that you meant.
2. If the word is right (a name, a product, a term of your trade), add it to the root `codelaw.toml`: `[spell]` `words = ["kubectl"]`. `spell/suspicious-word` checks the list.
3. For a one-off, excuse the line: `// codelaw-ignore spell/unknown-word: <reason>`.

## Examples

### JavaScript

#### Don't

```js
// Retry seperately until it works.
```

#### Do

```js
// Retry separately until it works.
```

### JSX

#### Don't

```jsx
const Hint = () => <p>Retry seperately until it works</p>;
```

#### Do

```jsx
const Hint = () => <p>Retry separately until it works</p>;
```

### TypeScript

#### Don't

```ts
// Retry seperately until it works.
```

#### Do

```ts
// Retry separately until it works.
```

### TSX

#### Don't

```tsx
const Hint = (): JSX.Element => <p>Retry seperately until it works</p>;
```

#### Do

```tsx
const Hint = (): JSX.Element => <p>Retry separately until it works</p>;
```

### Python

#### Don't

```python
def run():
    """Retry seperately until it works."""
```

#### Do

```python
def run():
    """Retry separately until it works."""
```

### JSON

#### Don't

```json
{ "description": "Retry seperately until it works" }
```

#### Do

```json
{ "description": "Retry separately until it works" }
```

### YAML

#### Don't

```yaml
# Retry seperately until it works.
summary: Retry seperately until it works
```

#### Do

```yaml
# Retry separately until it works.
summary: Retry separately until it works
```

### TOML

#### Don't

```toml
# Retry seperately until it works.
```

#### Do

```toml
# Retry separately until it works.
```

### Markdown

#### Don't

```md
Retry seperately until it works.
```

#### Do

```md
Retry separately until it works.
```

### CSS

#### Don't

```css
/* Retry seperately until it works. */
.box { color: red; }
```

#### Do

```css
/* Retry separately until it works. */
.box { color: red; }
```

### SCSS

#### Don't

```scss
// Retry seperately until it works.
.box { color: red; }
```

#### Do

```scss
// Retry separately until it works.
.box { color: red; }
```

### HTML

#### Don't

```html
<p title="Retry seperately">Retry until it works</p>
```

#### Do

```html
<p title="Retry separately">Retry until it works</p>
```

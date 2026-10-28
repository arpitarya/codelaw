---
id: spell/locale-variant
title: "Prose is spelled in the project's locale"
languages: [javascript, jsx, typescript, tsx, python, json, yaml, toml, markdown, css, scss, html]
fix: auto
stability: preview
options:
  locale: { type: "string", default: "", description: "`en-US` or `en-GB`: the locale whose spellings are right; read from `[spell] locale`, and the rule reports nothing while it is unset" }
since: 0.3.0
---

With `[spell] locale` set, a word of prose (as `spell/known-typo` reads it) is the other locale's spelling: `colour`, `behaviour` or `analyse` under `en-US`; `color` or `center` under `en-GB`. The pairs are VarCon's one-to-one US and British spellings. A pair is left out when either spelling is right in the other locale under some meaning (`check` and `cheque`, `program` and `programme`, `license` and `licence`), and under `en-GB` the Oxford `-ize` spellings are never reported. The rule is in neither preset: turn it on with `"spell/locale-variant" = "warn"` and set the locale.

**Why:** one locale reads as one voice, and a search for `color` finds every place the word is used.

The fix writes the locale's spelling, in the word's case.

```codelaw
native: spell/locale-variant
```

## How to fix

1. Run `codelaw fix`: each word becomes the locale's spelling.
2. For a quotation or a proper name (`Colour Science Ltd`), excuse the line: `// codelaw-ignore spell/locale-variant: <reason>`, or add the word to `[spell] words`.

## Examples

### JavaScript

#### Don't

```js options={"locale":"en-US"}
// Retry colour until it works.
```

#### Do

```js options={"locale":"en-US"}
// Retry color until it works.
```

### JSX

#### Don't

```jsx options={"locale":"en-US"}
const Hint = () => <p>Retry colour until it works</p>;
```

#### Do

```jsx options={"locale":"en-US"}
const Hint = () => <p>Retry color until it works</p>;
```

### TypeScript

#### Don't

```ts options={"locale":"en-US"}
// Retry colour until it works.
```

#### Do

```ts options={"locale":"en-US"}
// Retry color until it works.
```

### TSX

#### Don't

```tsx options={"locale":"en-US"}
const Hint = (): JSX.Element => <p>Retry colour until it works</p>;
```

#### Do

```tsx options={"locale":"en-US"}
const Hint = (): JSX.Element => <p>Retry color until it works</p>;
```

### Python

#### Don't

```python options={"locale":"en-US"}
def run():
    """Retry colour until it works."""
```

#### Do

```python options={"locale":"en-US"}
def run():
    """Retry color until it works."""
```

### JSON

#### Don't

```json options={"locale":"en-US"}
{ "description": "Retry colour until it works" }
```

#### Do

```json options={"locale":"en-US"}
{ "description": "Retry color until it works" }
```

### YAML

#### Don't

```yaml options={"locale":"en-US"}
# Retry colour until it works.
summary: Retry colour until it works
```

#### Do

```yaml options={"locale":"en-US"}
# Retry color until it works.
summary: Retry color until it works
```

### TOML

#### Don't

```toml options={"locale":"en-US"}
# Retry colour until it works.
```

#### Do

```toml options={"locale":"en-US"}
# Retry color until it works.
```

### Markdown

#### Don't

```md options={"locale":"en-US"}
Retry colour until it works.
```

#### Do

```md options={"locale":"en-US"}
Retry color until it works.
```

### CSS

#### Don't

```css options={"locale":"en-US"}
/* Retry colour until it works. */
.box { color: red; }
```

#### Do

```css options={"locale":"en-US"}
/* Retry color until it works. */
.box { color: red; }
```

### SCSS

#### Don't

```scss options={"locale":"en-US"}
// Retry colour until it works.
.box { color: red; }
```

#### Do

```scss options={"locale":"en-US"}
// Retry color until it works.
.box { color: red; }
```

### HTML

#### Don't

```html options={"locale":"en-US"}
<p title="Retry colour">Retry until it works</p>
```

#### Do

```html options={"locale":"en-US"}
<p title="Retry color">Retry until it works</p>
```

---
id: spell/known-typo
title: "No known typos in comments, docs and prose"
languages: [javascript, jsx, typescript, tsx, python, json, yaml, toml, markdown, css, scss, html]
fix: auto
stability: preview
since: 0.3.0
---

A word in a comment, a docstring, a prose string, Markdown text, HTML text or an HTML `alt`, `title`, `aria-label` or `placeholder`, or a string value of JSON, YAML or TOML is on the pack's list of known typos (`recieve`, `lenght`, `occured`). The fix writes the one correction the list gives, in the word's case.

**Why:** a typo in a message, a doc or a comment reads as carelessness and hides from search (`grep receive` misses `recieve`). The list holds only words that are no English word and come from exactly one common word, so the fix is safe to run unattended.

A string is read only when it holds whitespace and no interpolation; keys are names, not prose. URLs, email addresses, paths, dotted names, hashes, UUIDs, base64, code and identifiers (`snake_case`, `camelCase`, letters and digits mixed) are skipped, as is any word shorter than four letters. Template tags (`{% … %}`, `{{ … }}`, `{# … #}`) are code, not prose. A text of three words or more of which fewer than half are English words or known typos — lorem ipsum, a translated README, a comment in another language — is not checked: its words are not English typos.

```codelaw
native: spell/known-typo
```

## How to fix

1. Run `codelaw fix`: each typo becomes its correction.
2. For a word that is right where it stands (a product name, a quoted misspelling), excuse the line: `// codelaw-ignore spell/known-typo: <reason>`.

## Examples

### JavaScript

#### Don't

```js
// Retry until we recieve a response.
throw new Error("Unexpected lenght of the header");
```

#### Do

```js
// Retry until we receive a response.
throw new Error("Unexpected length of the header");
```

### JSX

#### Don't

```jsx
const Hint = () => <p title="Wrong lenght">Retry untill it works</p>;
```

#### Do

```jsx
const Hint = () => <p title="Wrong length">Retry until it works</p>;
```

### TypeScript

#### Don't

```ts
// The adress is checked first.
const message: string = "No adress given";
```

#### Do

```ts
// The address is checked first.
const message: string = "No address given";
```

### TSX

#### Don't

```tsx
const Empty = (): JSX.Element => <p>Nothing recieved yet</p>;
```

#### Do

```tsx
const Empty = (): JSX.Element => <p>Nothing received yet</p>;
```

### Python

#### Don't

```python
def parse(data):
    """Return the adress of the first record."""
```

#### Do

```python
def parse(data):
    """Return the address of the first record."""
```

### JSON

#### Don't

```json
{ "description": "Checks the lenght of a name" }
```

#### Do

```json
{ "description": "Checks the length of a name" }
```

### YAML

#### Don't

```yaml
# Runs untill the queue is empty.
summary: Clears the cache untill the next run
```

#### Do

```yaml
# Runs until the queue is empty.
summary: Clears the cache until the next run
```

### TOML

#### Don't

```toml
# The adress of the server.
description = "Checks the lenght of a name"
```

#### Do

```toml
# The address of the server.
description = "Checks the length of a name"
```

### Markdown

#### Don't

```md
The cache is cleared untill the next run.
```

#### Do

```md
The cache is cleared until the next run.
```

### CSS

#### Don't

```css
/* Keeps the lenght of a line short. */
.prose { max-width: 70ch; }
```

#### Do

```css
/* Keeps the length of a line short. */
.prose { max-width: 70ch; }
```

### SCSS

#### Don't

```scss
// Keeps the lenght of a line short.
.prose { max-width: 70ch; }
```

#### Do

```scss
// Keeps the length of a line short.
.prose { max-width: 70ch; }
```

### HTML

#### Don't

```html
<img src="map.png" alt="The adress on a map">
```

#### Do

```html
<img src="map.png" alt="The address on a map">
```

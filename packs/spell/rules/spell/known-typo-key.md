---
id: spell/known-typo-key
title: "No known typos in keys, classes and ids"
languages: [json, yaml, toml, css, scss, html]
fix: manual
fix_reason: "other files read the name — the code that reads a key, the markup that uses a class, the link to an id — so renaming it here alone breaks them"
stability: preview
since: 0.3.0
---

A name other files read holds a word on the pack's list of known typos: a JSON, YAML or TOML key, a CSS or SCSS class, id, custom property, `$variable`, mixin or function, or an HTML `id`. The name is split at case changes, `_`, `-`, `.` and digits, and each part of four letters or more is looked up. Each distinct name is reported once per file, where it is first declared.

**Why:** a misspelled key or class is copied into every file that reads it, and `grep receive` never finds it. It is cheapest to fix before it has readers, and it is never safe to fix in one file alone.

```codelaw
native: spell/known-typo-key
```

## How to fix

1. Find every reader of the name: search the project for it (`git grep -n recieve_all`), including code that builds it from parts.
2. Rename it here and in every reader in one change; for a key or class published to other projects, accept both names for a release before removing the old one.
3. For a name that is right as it stands (a field another system defines), excuse the line: `# codelaw-ignore spell/known-typo-key: <reason>`.

## Examples

### JSON

#### Don't

```json
{ "recieveTimeout": 30 }
```

#### Do

```json
{ "receiveTimeout": 30 }
```

### YAML

#### Don't

```yaml
recieve_timeout: 30
```

#### Do

```yaml
receive_timeout: 30
```

### TOML

#### Don't

```toml
[server]
recieve_timeout = 30
```

#### Do

```toml
[server]
receive_timeout = 30
```

### CSS

#### Don't

```css
.adress-card { padding: 1rem; }
```

#### Do

```css
.address-card { padding: 1rem; }
```

### SCSS

#### Don't

```scss
$line-lenght: 70ch;
```

#### Do

```scss
$line-length: 70ch;
```

### HTML

#### Don't

```html
<section id="adress-form"></section>
```

#### Do

```html
<section id="address-form"></section>
```

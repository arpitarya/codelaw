---
id: graph/no-unused-css-classes
title: CSS classes must be used somewhere
languages: [css]
scope: project
severity: warn
fix: suggest
fix_reason: "A class can be built at run time or used by markup outside the repo, so removing its rule needs a human or agent to accept the diff."
stability: preview
ported_from: [purgecss/unused-selector]
since: 0.1.0
---

Every class a stylesheet defines appears in the project's HTML, JSX, templates or strings.

**Why:** a rule for a class no markup uses is dead weight every page downloads, and it makes the stylesheet harder to change safely.

```codelaw
graph:
  class:
    defined_in: css
message: "`.$CLASS` is defined in `$FILE` but nothing in the project uses it"
fix:
  - else:
    do: [remove-rule]
```

## How to fix

1. Search for the class: a name built at run time (`"btn-" + kind`) counts only through its static prefix.
2. If markup outside the repo uses it, add the file to `[graph] keep` or list it in the package's `style` or `exports`.
3. Otherwise delete the rule, or the class from its selector list.

## Examples

### CSS

#### Don't

```css files
/* src/styles.css */
.card { padding: 1rem; }
.legacy-banner { color: red; }
<!-- index.html -->
<div class="card"></div>
```

#### Do

```css files
/* src/styles.css */
.card { padding: 1rem; }
<!-- index.html -->
<div class="card"></div>
```

---
id: scss/no-import
title: "No `@import`"
languages: [scss]
fix: suggest
fix_reason: "`@use` namespaces what it loads and loads it once, so a bare swap can change what resolves"
stability: preview
since: 0.3.0
---

`@import` is not used for Sass files: Dart Sass deprecates it and removes it in 3.0. Load a file with `@use`, or re-export it with `@forward`. An import that stays a plain CSS `@import` (a `url()`, an `http://` address, a `.css` file, or one with a media query) is allowed.

**Why:** `@import` puts everything it loads in one global scope; two files can silently overwrite each other's variables and mixins, and a file imported twice is compiled twice.

```codelaw
native: scss/no-import
```

## How to fix

1. Replace `@import "file"` with `@use "file"`; the members are then reached as `file.$name`, `file.mixin()`.
2. Use `@use "file" as *` to keep the old unprefixed names while migrating.
3. In an index file that only gathers other files, use `@forward`.

## Examples

### SCSS

#### Don't

```scss
@import "variables";
@import "mixins";
```

#### Do

```scss
@use "variables";
@use "mixins";
```

## Differences from the source

Plain CSS imports are not reported. The fix is offered for a single quoted path; it is `suggest` because `@use` changes how the file's members are named.

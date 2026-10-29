---
id: js/flag-marker
title: "An inline flag marker agrees with the registry"
languages: [javascript, jsx, typescript, tsx]
scope: project
severity: error
fix: auto
stability: preview
options:
  presets:
    type: "string[]"
    description: "`[flags] presets`, for this rule only: the SDKs whose reads codelaw knows (launchdarkly, unleash, split, openfeature, growthbook, flagsmith)"
  accessors:
    type: "string[]"
    description: "`[flags] accessors`, for this rule only: calls with one `$NAME` slot that read a flag"
  inline:
    type: "string"
    description: "`[flags] inline`, for this rule only: `allow` or `registry-only`"
  warn_days:
    type: "integer"
    description: "`[flags] warn_days`, for this rule only: how many days before `expires` a read warns"
since: 0.5.0
---

An inline marker registers a flag where it is read, when no `.codelaw/flags.toml` entry does:

```text
// codelaw-flag new-checkout expires 2027-02-28 owner @payments ticket PAY-12 keep on reason <the rest of the line>
```

`expires` is a date (`YYYY-MM-DD`) or `never`, which needs a `reason`; `keep` is `on` or `off`. The rule is on when the root config has a `[flags]` table (SR-PROJECT-GRAPH decision 11). It reports:

| the marker | finding | fix |
|---|---|---|
| disagrees with the flag's registry entry | `warn` | `auto`: the marker is rewritten to the registry's values — the registry wins |
| marks a flag also marked elsewhere with other values | `error` | by hand: register the flag once in `.codelaw/flags.toml` |
| cannot be read, has no `expires`, or `never` without a `reason` | `error` | by hand |
| marks a flag with no read left | `info` | `auto`: the marker is removed |
| any marker, under `[flags] inline = "registry-only"` | `error` | `auto` when the registry has the flag: the marker is removed |

**Why:** two places saying when a flag goes will disagree, and the one nobody reads wins by accident. The registry is the one place; a marker is a convenience that must say the same.

```codelaw
graph:
  flags: markers
message: "This flag marker disagrees with the registry or cannot be used"
fix:
  do: [fix-marker]
```

## How to fix

1. Run `codelaw fix`: a marker that disagrees with the registry is rewritten to match it, and a marker of a flag with no read left is removed.
2. A flag marked in several places with different values: add one `[[flag]]` entry for it to `.codelaw/flags.toml` with the values that are right, and delete the markers (or let `codelaw fix` rewrite them).
3. An unreadable marker: write it as `codelaw-flag <name> expires YYYY-MM-DD [owner …] [ticket …] [keep on|off] [reason …]`.

## Examples

### JavaScript

#### Don't

```js files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.js
// codelaw-flag beta expires 2027-01-31 owner @web
export const on = isFeatureOn("beta");
```

#### Do

```js files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.js
// codelaw-flag beta expires 2027-02-28 owner @web
export const on = isFeatureOn("beta");
```

### JSX

#### Don't

```jsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.jsx
// codelaw-flag beta expires 2027-01-31 owner @web
export const Beta = () => (isFeatureOn("beta") ? <New /> : null);
```

#### Do

```jsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.jsx
// codelaw-flag beta expires 2027-02-28 owner @web
export const Beta = () => (isFeatureOn("beta") ? <New /> : null);
```

### TypeScript

#### Don't

```ts files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.ts
// codelaw-flag beta expires 2027-01-31 owner @web
export const on = isFeatureOn("beta");
```

#### Do

```ts files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.ts
// codelaw-flag beta expires 2027-02-28 owner @web
export const on = isFeatureOn("beta");
```

### TSX

#### Don't

```tsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.tsx
// codelaw-flag beta expires 2027-01-31 owner @web
export const Beta = () => (isFeatureOn("beta") ? <New /> : null);
```

#### Do

```tsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-02-28"
// src/beta.tsx
// codelaw-flag beta expires 2027-02-28 owner @web
export const Beta = () => (isFeatureOn("beta") ? <New /> : null);
```

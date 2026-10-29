---
id: js/flag-expiry
title: "Every feature flag read is registered, with an owner and a date it goes"
languages: [javascript, jsx, typescript, tsx]
scope: project
severity: error
fix: suggest
fix_reason: "Keeping one branch of a flag removes the other branch's code; a person or agent accepts the diff."
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

Each call that reads a feature flag names a flag that is registered — in `.codelaw/flags.toml` or by an inline `codelaw-flag` marker — and that has not expired on the as-of date. The rule is on when the root config has a `[flags]` table (SR-PROJECT-GRAPH decision 11), and reads nothing until `[flags]` names a preset or an accessor.

```toml
[flags]
presets   = ["launchdarkly"]                 # launchdarkly, unleash, split, openfeature, growthbook, flagsmith
accessors = ["isFeatureOn($NAME)", "flags.get($NAME, $$$)", "useFeature({ key: $NAME })"]
inline    = "allow"                          # or "registry-only"
warn_days = 14
```

A function whose parameter flows unchanged into an accessor's `$NAME` is an accessor too — a team's wrapper over LaunchDarkly, Unleash or Split, and a wrapper of that wrapper — found through the project graph, across files and imports; `codelaw flags accessors` lists them all. A flag's name is a string literal, or a constant, enum member or object key the graph resolves; a name computed at run time is reported as `info`, uncertain, never an error.

| at a read of a flag that is | finding |
|---|---|
| not registered | `error` |
| registered, expiring within `warn_days` of the as-of date | `warn` |
| registered, past its `expires` on the as-of date | `error` |
| registered with `keep = "on"` or `"off"` (decided) | `warn`, with the fix |
| registered with `expires = "never"` and a `reason` | nothing |
| read with a computed name | `info`, uncertain |

Dates are judged on the as-of date (`--as-of`, `CODELAW_AS_OF`, else `HEAD`'s commit date), never the clock; with no as-of date only unregistered and decided flags are reported. codelaw never asks a flag service whether a flag is live (L5): the registry is the truth.

**Why:** a flag left in the code after its rollout is a branch nobody tests, and every one makes the next change slower. A flag with an owner and a date is one someone removes.

```codelaw
graph:
  flags: reads
message: "A feature flag read here is unregistered, expiring, expired or decided"
fix:
  do: [keep-branch]
```

## How to fix

1. Unregistered: add the flag to `.codelaw/flags.toml` (`[[flag]] name, owner, ticket, expires`), or mark it inline above the read: `// codelaw-flag <name> expires YYYY-MM-DD owner @team`.
2. Expiring or expired: decide which branch stays. Set `keep = "on"` or `keep = "off"` on its registry entry and run `codelaw fix --unsafe`: an `if` or a conditional whose whole condition is the read keeps that branch. Where the read is anything else (`&&`, `!`, a variable), keep the branch by hand.
3. When the flag has no read left, `codelaw fix` removes its registry entry (`data/flag-registry`).
4. A flag meant to stay (a kill switch): `expires = "never"` with a `reason`.

## Examples

### JavaScript

#### Don't

```js files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.js
if (isFeatureOn("new-checkout")) {
  renderNew();
} else {
  renderOld();
}
```

#### Do

```js files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.js
renderNew();
```

### JSX

#### Don't

```jsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.jsx
export const Checkout = () => (isFeatureOn("new-checkout") ? <NewCheckout /> : <OldCheckout />);
```

#### Do

```jsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.jsx
export const Checkout = () => (<NewCheckout />);
```

### TypeScript

#### Don't

```ts files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.ts
export function total(cart: Cart): number {
  if (isFeatureOn("new-checkout")) {
    return newTotal(cart);
  } else {
    return oldTotal(cart);
  }
}
```

#### Do

```ts files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.ts
export function total(cart: Cart): number {
  return newTotal(cart);
}
```

### TSX

#### Don't

```tsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.tsx
export const Checkout = (): JSX.Element => (isFeatureOn("new-checkout") ? <NewCheckout /> : <OldCheckout />);
```

#### Do

```tsx files options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
// src/checkout.tsx
export const Checkout = (): JSX.Element => (<NewCheckout />);
```

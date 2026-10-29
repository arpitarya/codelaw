# Cases: js/flag-expiry

### case: an unregistered flag is an error, and has no fix

```js options={"accessors":["isFeatureOn($NAME)"]}
// src/a.js
export const on = isFeatureOn("beta");
```

```expect
src/a.js:1:19 js/flag-expiry
```

### case: a wrapper of a wrapper over an SDK, through imports and an alias

```ts options={"presets":["launchdarkly"]}
// src/ld.ts
export const client = init();
// src/flags.ts
import { client } from "./ld";
export function isOn(name: string) { return client.variation(name, false); }
// src/feature.ts
import { isOn } from "./flags";
export const enabled = (flag: string) => isOn(flag);
// src/app.ts
import { enabled as on } from "./feature";
export const a = on("new-checkout");
```

```expect
src/app.ts:2:18 js/flag-expiry
```

### case: a name from an enum in another module resolves

```ts options={"accessors":["isFeatureOn($NAME)"]}
// src/names.ts
export enum Flag { Beta = "beta" }
// src/app.ts
import { Flag } from "./names";
export const on = isFeatureOn(Flag.Beta);
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2030-01-01"
keep = "off"
```

```expect
src/app.ts:2:19 js/flag-expiry
```

### case: a computed name is info, uncertain, never an error

```js options={"accessors":["isFeatureOn($NAME)"]}
// src/a.js
export const on = (k) => isFeatureOn(`flag-${k}`);
```

```expect
src/a.js:1:26 js/flag-expiry
```

### case: a registered flag with no as-of date and no decision is quiet

```js options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
owner = "@web"
expires = "2027-01-01"
// src/a.js
export const on = isFeatureOn("beta");
```

```expect
(none)
```

### case: a decided flag read inside a larger condition is reported without a fix

```js options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "on"
// src/a.js
if (ready && isFeatureOn("beta")) {
  go();
}
```

```expect
src/a.js:1:14 js/flag-expiry
```


### case: a branch that declares a name is not unwrapped

```js options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "on"
// src/a.js
if (isFeatureOn("beta")) {
  const x = 1;
  go(x);
}
```

```expect
src/a.js:1:5 js/flag-expiry
```


### case: `keep = "off"` with no else removes the statement

```js options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "off"
// src/a.js
start();
if (isFeatureOn("beta")) {
  go();
}
end();
```

```expect
src/a.js:2:5 js/flag-expiry
```

```fixed
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
keep = "off"
// src/a.js
start();
end();
```

### case: a kill switch that never expires, with a reason, is quiet

```js options={"accessors":["isFeatureOn($NAME)"]}
// .codelaw/flags.toml
[[flag]]
name = "kill"
expires = "never"
reason = "the payments kill switch"
// src/a.js
export const on = isFeatureOn("kill");
```

```expect
(none)
```

# Cases: js/flag-marker

### case: one flag marked in two places with different dates

```js options={"accessors":["isFeatureOn($NAME)"]}
// src/a.js
// codelaw-flag beta expires 2027-01-01 owner @web
export const a = isFeatureOn("beta");
// src/b.js
// codelaw-flag beta expires 2027-02-01 owner @web
export const b = isFeatureOn("beta");
```

```expect
src/a.js:1:4 js/flag-marker
src/b.js:1:4 js/flag-marker
```

### case: under `registry-only`, a marker the registry covers is removed

```js options={"accessors":["isFeatureOn($NAME)"],"inline":"registry-only"}
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
// src/a.js
// codelaw-flag beta expires 2027-01-01
export const a = isFeatureOn("beta");
```

```expect
src/a.js:1:4 js/flag-marker
```

```fixed
// .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
// src/a.js
export const a = isFeatureOn("beta");
```

### case: a marker of a flag no longer read is removed

```js options={"accessors":["isFeatureOn($NAME)"]}
// src/a.js
export const x = 1; // codelaw-flag gone expires 2027-01-01
```

```expect
src/a.js:1:24 js/flag-marker
```

```fixed
// src/a.js
export const x = 1;
```

### case: a marker that cannot be read

```js options={"accessors":["isFeatureOn($NAME)"]}
// src/a.js
// codelaw-flag beta expires
export const a = isFeatureOn("beta");
```

```expect
src/a.js:1:4 js/flag-marker
```

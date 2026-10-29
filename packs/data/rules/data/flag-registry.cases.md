# Cases: data/flag-registry

### case: `never` without a reason, and a key a flag does not have

```toml options={"accessors":["isFeatureOn($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "kill"
expires = "never"
colour = "red"
# src/a.js
export const on = isFeatureOn("kill");
```

```expect
.codelaw/flags.toml:3:1 data/flag-registry
.codelaw/flags.toml:4:1 data/flag-registry
```

### case: a flag registered twice

```toml options={"accessors":["isFeatureOn($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "beta"
expires = "2027-01-01"
[[flag]]
name = "beta"
expires = "2027-02-01"
# src/a.js
export const on = isFeatureOn("beta");
```

```expect
.codelaw/flags.toml:4:1 data/flag-registry
```

### case: while a read computes its flag's name, an unread entry is uncertain and kept

```toml options={"accessors":["isFeatureOn($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "old"
expires = "2027-01-01"
# src/a.js
export const on = (k) => isFeatureOn(k + "-flag");
```

```expect
.codelaw/flags.toml:1:1 data/flag-registry
```


### case: a registry that is not TOML

```toml options={"accessors":["isFeatureOn($NAME)"]}
# .codelaw/flags.toml
[[flag]
name = "x"
# src/a.js
export const on = isFeatureOn("x");
```

```expect
.codelaw/flags.toml:1:1 data/flag-registry
```

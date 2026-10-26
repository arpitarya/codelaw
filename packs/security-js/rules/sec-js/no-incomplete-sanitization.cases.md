# sec-js/no-incomplete-sanitization — cases

### case: one-pass stripping of ../

```js
const p = name.replace(/\.\.\//g, "");
```

```expect
1:11 sec-js/no-incomplete-sanitization
```

### case: a global regular expression

```js
const s = v.replace(/</g, "&lt;");
```

```expect
(none)
```

### case: replace of &

```js
s.replace("&", "&amp;");
```

```expect
1:1 sec-js/no-incomplete-sanitization
```

### case: javascript:

```js
u.replace(/javascript:/gi, "");
```

```expect
1:1 sec-js/no-incomplete-sanitization
```

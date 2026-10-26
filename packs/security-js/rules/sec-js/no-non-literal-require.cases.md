# sec-js/no-non-literal-require — cases

### case: a template with no value is a literal

```js
require(`./a`);
```

```expect
(none)
```

### case: a computed dynamic import

```js
const m = await import(`./locale/${lang}.js`);
```

```expect
1:17 sec-js/no-non-literal-require
```

### case: import()

```js
const m = await import(name);
```

```expect
1:17 sec-js/no-non-literal-require
```

### case: a literal

```js
require("./a");
```

```expect
(none)
```

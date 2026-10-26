# sec-js/no-unsafe-regex — cases

### case: a pattern from a request

```js
const re = new RegExp(req.query.q);
```

```expect
1:12 sec-js/no-unsafe-regex
```

### case: a fixed pattern

```js
const re = /^[a-z]+$/;
```

```expect
(none)
```

### case: a star group

```js
const r = /(\w+\s?)+$/;
```

```expect
1:11 sec-js/no-unsafe-regex
```

### case: RegExp from the body

```js
RegExp(req.body.pattern);
```

```expect
1:1 sec-js/no-unsafe-regex
```

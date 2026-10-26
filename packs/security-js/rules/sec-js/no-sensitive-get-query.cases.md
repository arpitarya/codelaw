# sec-js/no-sensitive-get-query — cases

### case: koa

```js
const t = ctx.query.token;
```

```expect
1:11 sec-js/no-sensitive-get-query
```

### case: a harmless key

```js
const p = req.query.page;
```

```expect
(none)
```

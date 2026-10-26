# sec-js/no-header-injection — cases

### case: set

```js
res.set("X-Next", req.query.next);
```

```expect
1:1 sec-js/no-header-injection
```

### case: header

```js
res.header("Location", req.body.to);
```

```expect
1:1 sec-js/no-header-injection
```

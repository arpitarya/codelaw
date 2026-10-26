# sec-js/jwt-verify — cases

### case: an algorithm list

```js
jwt.verify(t, k, { algorithms: ["HS256"] });
```

```expect
(none)
```

### case: the none algorithm

```js
const o = { algorithms: ["none"] };
```

```expect
1:13 sec-js/jwt-verify
```

### case: decode with no verify in the file

```js
const c = jwt.decode(token);
```

```expect
1:11 sec-js/jwt-verify
```

### case: three arguments with no algorithms

```js
jwt.verify(t, k, { audience: "a" });
```

```expect
1:1 sec-js/jwt-verify
```

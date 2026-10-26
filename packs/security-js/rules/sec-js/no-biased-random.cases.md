# sec-js/no-biased-random — cases

### case: a power of two is not biased

```js
const b = crypto.randomBytes(1)[0] % 16;
```

```expect
(none)
```

### case: readUInt32BE

```js
const n = crypto.randomBytes(4).readUInt32BE(0) % 10;
```

```expect
1:11 sec-js/no-biased-random
```

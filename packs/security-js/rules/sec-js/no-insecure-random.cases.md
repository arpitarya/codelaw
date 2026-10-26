# sec-js/no-insecure-random — cases

### case: a numeric code has no mechanical fix

```js
const otp = Math.floor(Math.random() * 1000000);
```

```expect
1:7 sec-js/no-insecure-random
```

### case: pseudoRandomBytes

```js
const b = crypto.pseudoRandomBytes(16);
```

```expect
1:11 sec-js/no-insecure-random
```

### case: a name that is not a secret

```js
const jitter = Math.random() * 100;
```

```expect
(none)
```

### case: an object property

```js
const o = { sessionId: Math.random().toString(16) };
```

```expect
1:13 sec-js/no-insecure-random
```

### case: an assignment

```js
this.token = Math.random().toString(36).substring(2);
```

```expect
1:1 sec-js/no-insecure-random
```

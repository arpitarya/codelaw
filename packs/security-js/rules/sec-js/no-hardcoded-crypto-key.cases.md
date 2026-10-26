# sec-js/no-hardcoded-crypto-key — cases

### case: jwt.sign with a literal secret

```js
const t = jwt.sign({ id }, "shhh");
```

```expect
1:11 sec-js/no-hardcoded-crypto-key
```

### case: createHmac with an environment key

```js
crypto.createHmac("sha256", process.env.KEY);
```

```expect
(none)
```

### case: an IV literal

```js
crypto.createCipheriv("aes-256-gcm", key, "000000000000");
```

```expect
1:1 sec-js/no-hardcoded-crypto-key
```

### case: createHmac literal

```js
crypto.createHmac("sha256", "secret");
```

```expect
1:1 sec-js/no-hardcoded-crypto-key
```

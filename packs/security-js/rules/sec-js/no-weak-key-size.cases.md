# sec-js/no-weak-key-size — cases

### case: a weak curve has no mechanical fix

```js
crypto.generateKeyPairSync("ec", { namedCurve: "secp192k1" });
```

```expect
1:36 sec-js/no-weak-key-size
```

### case: P-256

```js
crypto.generateKeyPairSync("ec", { namedCurve: "P-256" });
```

```expect
(none)
```

### case: generateKeyPair

```js
generateKeyPair("rsa", { modulusLength: 512 }, cb);
```

```expect
1:26 sec-js/no-weak-key-size
```

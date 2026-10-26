# sec-js/no-weak-hash — cases

### case: sha1 in createHmac

```js
crypto.createHmac("sha1", key).update(d);
```

```expect
1:1 sec-js/no-weak-hash
```

### case: sha256

```js
crypto.createHash("sha256");
```

```expect
(none)
```

### case: crypto.subtle

```js
crypto.subtle.digest("SHA-1", data);
```

```expect
1:1 sec-js/no-weak-hash
```

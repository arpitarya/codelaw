# sec-js/password-hash-effort — cases

### case: bcrypt with a low cost

```js
const h = await bcrypt.hash(password, 4);
```

```expect
1:17 sec-js/password-hash-effort
```

### case: a fast hash of a password

```js
const h = crypto.createHash("sha256").update(password).digest("hex");
```

```expect
1:11 sec-js/password-hash-effort
```

### case: bcrypt.hashSync

```js
bcrypt.hashSync(password, 8);
```

```expect
1:1 sec-js/password-hash-effort
```

### case: pbkdf2 callback form

```js
crypto.pbkdf2(password, salt, 100000, 64, "sha512", cb);
```

```expect
1:1 sec-js/password-hash-effort
```

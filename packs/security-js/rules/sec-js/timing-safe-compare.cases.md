# sec-js/timing-safe-compare — cases

### case: a check for undefined is not a comparison of secrets

```js
if (token === undefined) {}
```

```expect
(none)
```

### case: !== has no mechanical fix

```js
if (sig !== expectedSignature) {}
```

```expect
1:5 sec-js/timing-safe-compare
```

### case: a confirm-password field is not a secret check

```js
if (password !== confirmPassword) {}
```

```expect
(none)
```

### case: a signature

```js
if (signature === expected) {}
```

```expect
1:5 sec-js/timing-safe-compare
```

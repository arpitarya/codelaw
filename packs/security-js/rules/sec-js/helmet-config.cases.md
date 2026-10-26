# sec-js/helmet-config — cases

### case: hsts

```js
helmet({ hsts: false });
```

```expect
1:1 sec-js/helmet-config
```

### case: a configured policy

```js
helmet({ hsts: { maxAge: 1 } });
```

```expect
(none)
```

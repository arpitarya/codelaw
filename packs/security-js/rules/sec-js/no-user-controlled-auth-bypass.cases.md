# sec-js/no-user-controlled-auth-bypass — cases

### case: a header

```js
if (req.headers["x-admin"]) { go(); }
```

```expect
1:1 sec-js/no-user-controlled-auth-bypass
```

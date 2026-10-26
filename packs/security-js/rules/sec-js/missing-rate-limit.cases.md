# sec-js/missing-rate-limit — cases

### case: a client call is not a route

```js
api.post("/auth/login", data);
```

```expect
(none)
```

### case: a limiter in the file

```js
const loginLimiter = rateLimit({});
app.post("/login", handler);
```

```expect
(none)
```

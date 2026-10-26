# sec-js/express-session-secret — cases

### case: secure false

```js
session({ secret: key, cookie: { secure: false } });
```

```expect
1:1 sec-js/express-session-secret
```

### case: cookie-session keys

```js
cookieSession({ keys: ["a"] });
```

```expect
1:1 sec-js/express-session-secret
```

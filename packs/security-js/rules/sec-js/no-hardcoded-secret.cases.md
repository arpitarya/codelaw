# sec-js/no-hardcoded-secret — cases

### case: a private key header in a template

```js
const pem = `-----BEGIN RSA PRIVATE KEY-----\nMIIE...`;
```

```expect
1:13 sec-js/no-hardcoded-secret
```

### case: a short placeholder is not flagged

```js
const password = "x";
```

```expect
(none)
```

### case: a cookie name is not a secret

```js
const TOKEN_COOKIE_NAME = "my_app_session_token";
```

```expect
(none)
```

### case: a GitHub token

```js
const t = "ghp_abcdefghijklmnopqrstuvwxyz0123456789";
```

```expect
1:11 sec-js/no-hardcoded-secret
```

### case: an object password

```js
const cfg = { password: "hunter2hunter2" };
```

```expect
1:15 sec-js/no-hardcoded-secret
```

### case: a Stripe key

```js
const k = "sk_live_abcdefghijklmnopqrstuvwx";
```

```expect
1:11 sec-js/no-hardcoded-secret
```

# sec-js/cookie-flags — cases

### case: options that lack sameSite have no mechanical fix

```js
res.cookie("sid", sid, { httpOnly: true, secure: true });
```

```expect
1:1 sec-js/cookie-flags
```

### case: all three flags

```js
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "strict" });
```

```expect
(none)
```

### case: koa

```js
ctx.cookies.set("sid", id);
```

```expect
1:1 sec-js/cookie-flags
```

### case: fastify

```js
reply.setCookie("sid", id);
```

```expect
1:1 sec-js/cookie-flags
```

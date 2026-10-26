# sec-js/no-open-redirect — cases

### case: with a status

```js
res.redirect(302, req.query.to);
```

```expect
1:1 sec-js/no-open-redirect
```

### case: NextResponse

```js
NextResponse.redirect(searchParams.get("next"));
```

```expect
1:1 sec-js/no-open-redirect
```

# sec-js/no-token-in-web-storage — cases

### case: an assignment

```js
sessionStorage.authToken = value;
```

```expect
1:1 sec-js/no-token-in-web-storage
```

### case: a harmless key

```js
localStorage.setItem("sessionStart", String(Date.now()));
```

```expect
(none)
```

### case: bracket

```js
localStorage["jwt"] = t;
```

```expect
1:1 sec-js/no-token-in-web-storage
```

### case: sessionStorage

```js
sessionStorage.setItem("refresh_token", t);
```

```expect
1:1 sec-js/no-token-in-web-storage
```

# sec-js/no-document-cookie-token — cases

### case: session

```js
document.cookie = "session=" + id;
```

```expect
1:1 sec-js/no-document-cookie-token
```

### case: append

```js
document.cookie += "jwt=" + t;
```

```expect
1:1 sec-js/no-document-cookie-token
```

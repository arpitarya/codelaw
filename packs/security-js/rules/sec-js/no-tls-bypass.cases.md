# sec-js/no-tls-bypass — cases

### case: the environment switch

```js
process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
```

```expect
1:1 sec-js/no-tls-bypass
```

### case: the safe value

```js
process.env.NODE_TLS_REJECT_UNAUTHORIZED = "1";
```

```expect
(none)
```

### case: strictSSL

```js
request({ url, strictSSL: false });
```

```expect
1:16 sec-js/no-tls-bypass
```

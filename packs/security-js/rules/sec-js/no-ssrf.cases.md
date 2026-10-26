# sec-js/no-ssrf — cases

### case: a template built from the request

```js
await axios.get(`https://${req.query.host}/status`);
```

```expect
1:7 sec-js/no-ssrf
```

### case: a constant URL

```js
await fetch("https://api.example.com/v1");
```

```expect
(none)
```

### case: axios.get

```js
axios.get(req.body.url);
```

```expect
1:1 sec-js/no-ssrf
```

### case: http.get

```js
http.get(req.query.u, cb);
```

```expect
1:1 sec-js/no-ssrf
```

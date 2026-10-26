# sec-js/no-cleartext-http — cases

### case: localhost is fine

```js
fetch("http://localhost:3000/api");
```

```expect
(none)
```

### case: a template

```js
axios.get(`http://${host}/status`);
```

```expect
1:1 sec-js/no-cleartext-http
```

### case: axios.post

```js
axios.post("http://pay.example-bank.com/", body);
```

```expect
1:1 sec-js/no-cleartext-http
```

### case: an https URL

```js
fetch("https://a.example-bank.com");
```

```expect
(none)
```

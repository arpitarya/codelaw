# sec-js/no-secret-in-log — cases

### case: a template with the token

```js
console.log(`token is ${token}`);
```

```expect
1:1 sec-js/no-secret-in-log
```

### case: a message that only mentions the word

```js
console.log("token expired");
```

```expect
(none)
```

### case: a bare variable

```js
console.error(apiKey);
```

```expect
1:1 sec-js/no-secret-in-log
```

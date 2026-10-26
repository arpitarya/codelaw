# sec-js/no-log-injection — cases

### case: console.log

```js
console.log(req.body.name);
```

```expect
1:1 sec-js/no-log-injection
```

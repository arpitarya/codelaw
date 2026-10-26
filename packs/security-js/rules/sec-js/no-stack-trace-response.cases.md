# sec-js/no-stack-trace-response — cases

### case: an object holding the error

```js
res.json({ ok: false, error: err });
```

```expect
1:1 sec-js/no-stack-trace-response
```

### case: a message string

```js
res.send("failed");
```

```expect
(none)
```

### case: json

```js
res.json(err);
```

```expect
1:1 sec-js/no-stack-trace-response
```

### case: a nested stack

```js
res.status(500).json({ stack: err.stack });
```

```expect
1:1 sec-js/no-stack-trace-response
```

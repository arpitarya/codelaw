# sec-py/no-bind-all-interfaces — cases

### case: uvicorn

```py
uvicorn.run(app, host="0.0.0.0")
```

```expect
1:1 sec-py/no-bind-all-interfaces
```

### case: socket

```py
s.bind(("0.0.0.0", 80))
```

```expect
1:1 sec-py/no-bind-all-interfaces
```

### case: loopback

```py
s.bind(("127.0.0.1", 80))
```

```expect
(none)
```

### case: from the environment

```py
app.run(host=HOST)
```

```expect
(none)
```

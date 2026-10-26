# sec-py/logging-listen — cases

### case: listen

```py
logging.config.listen(9999)
```

```expect
1:1 sec-py/logging-listen
```

### case: dictConfig

```py
logging.config.dictConfig(c)
```

```expect
(none)
```

### case: another listen

```py
server.listen(5)
```

```expect
(none)
```

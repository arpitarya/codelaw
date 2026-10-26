# sec-py/no-log-injection — cases

### case: a request value

```py
def v(request):
    logger.info("login %s", request.POST["u"])
```

```expect
2:5 sec-py/no-log-injection
```

### case: a message only

```py
def v(request):
    logger.warning(request.GET["u"])
```

```expect
2:5 sec-py/no-log-injection
```

### case: a constant

```py
def v(request):
    logger.info("login %s", "x")
```

```expect
(none)
```

### case: not a logger

```py
def v(request):
    other.info("login %s", request.POST["u"])
```

```expect
(none)
```

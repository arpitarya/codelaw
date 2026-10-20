# py/no-tunable-literals — cases

### case: default parameter

```py
def connect(host, retries=5):
    pass
```

```expect
1:19 py/no-tunable-literals
```

### case: module constant is the config point

```py
TIMEOUT = 30
```

```expect
(none)
```

### case: zero and one are fine

```py
get(url, timeout=0)
loop(max_items=1)
```

```expect
(none)
```

### case: unrelated names

```py
resize(width=640)
```

```expect
(none)
```

### case: local assignment

```py
def f():
    max_size = 1024
```

```expect
2:5 py/no-tunable-literals
```

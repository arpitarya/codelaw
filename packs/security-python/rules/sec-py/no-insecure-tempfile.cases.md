# sec-py/no-insecure-tempfile — cases

### case: mktemp

```py
tempfile.mktemp(suffix=".txt")
```

```expect
1:1 sec-py/no-insecure-tempfile
```

### case: a fixed /tmp path

```py
open("/tmp/report.txt", "w")
```

```expect
1:6 sec-py/no-insecure-tempfile
```

### case: mkstemp

```py
tempfile.mkstemp()
```

```expect
(none)
```

### case: a path outside /tmp

```py
open("/var/data/report.txt", "w")
```

```expect
(none)
```

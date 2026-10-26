# sec-py/no-hardcoded-secret — cases

### case: a password assignment

```py
password = "s3cr3tvalue"
```

```expect
1:1 sec-py/no-hardcoded-secret
```

### case: a keyword argument

```py
connect(host, password="s3cr3tvalue")
```

```expect
1:15 sec-py/no-hardcoded-secret
```

### case: a placeholder is short

```py
password = ""
```

```expect
(none)
```

### case: an environment read

```py
password = os.environ["PASSWORD"]
```

```expect
(none)
```

### case: an AWS key

```py
key = "AKIAIOSFODNN7EXAMPLE"
```

```expect
1:7 sec-py/no-hardcoded-secret
```

### case: a name that is not a secret

```py
title = "A long title string"
```

```expect
(none)
```

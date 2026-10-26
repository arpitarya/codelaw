# sec-py/no-exec-eval — cases

### case: a literal string

```py
eval("1 + 1")
```

```expect
(none)
```

### case: an f-string

```py
exec(f"import {name}")
```

```expect
1:1 sec-py/no-exec-eval
```

### case: exec with a namespace

```py
exec(code, {})
```

```expect
1:1 sec-py/no-exec-eval
```

### case: a method named eval

```py
model.eval()
```

```expect
(none)
```

### case: a name

```py
compile(source, 'f.py', 'exec')
```

```expect
1:1 sec-py/no-exec-eval
```

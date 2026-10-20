# py/function-uses-loop-variable — cases

### case: every read is reported

```py
for i in range(3):
    def f():
        return i + i
```

```expect
3:16 py/function-uses-loop-variable
3:20 py/function-uses-loop-variable
```

### case: key functions run immediately

```py
for i in range(3):
    rows.sort(key=lambda r: r[i])
```

```expect
(none)
```

### case: map runs immediately

```py
for i in range(3):
    list(map(lambda x: x + i, xs))
```

```expect
(none)
```

### case: comprehension variable

```py
fs = [lambda: i for i in range(3)]
```

```expect
1:15 py/function-uses-loop-variable
```

### case: parameter with the same name

```py
for i in range(3):
    f = lambda i: i
```

```expect
(none)
```

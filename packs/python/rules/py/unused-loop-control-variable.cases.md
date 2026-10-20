# py/unused-loop-control-variable — cases

### case: one of two targets

```py
for key, value in pairs:
    print(value)
```

```expect
1:5 py/unused-loop-control-variable
```

```fixed
for _key, value in pairs:
    print(value)
```

### case: used after the loop: no rename

```py
def f(xs):
    for x in xs:
        pass
    return x
```

```expect
2:9 py/unused-loop-control-variable
```

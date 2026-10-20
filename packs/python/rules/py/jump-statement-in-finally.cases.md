# py/jump-statement-in-finally — cases

### case: break inside a loop inside finally

```py
try:
    pass
finally:
    for x in y:
        break
```

```expect
(none)
```

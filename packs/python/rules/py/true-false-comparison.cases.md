# py/true-false-comparison — cases

### case: comparing with False

```py
if done == False:
    wait()
```

```expect
1:4 py/true-false-comparison
```

```fixed
if not done:
    wait()
```

### case: compound operand is parenthesised

```py
if a or b != True:
    stop()
```

```expect
1:9 py/true-false-comparison
```

```fixed
if a or not b:
    stop()
```

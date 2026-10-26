# sec-py/torch-load-weights-only — cases

### case: a bare load

```py
torch.load(path)
```

```expect
1:1 sec-py/torch-load-weights-only
```

### case: map_location only

```py
torch.load(path, map_location="cpu")
```

```expect
1:1 sec-py/torch-load-weights-only
```

### case: weights_only=False

```py
torch.load(path, weights_only=False)
```

```expect
1:1 sec-py/torch-load-weights-only
```

### case: weights_only=True

```py
torch.load(path, weights_only=True)
```

```expect
(none)
```

# sec-py/ml-unsafe-load — cases

### case: allow_pickle

```py
np.load(path, allow_pickle=True)
```

```expect
1:1 sec-py/ml-unsafe-load
```

### case: no pickle

```py
np.load(path)
```

```expect
(none)
```

### case: joblib

```py
joblib.load(path)
```

```expect
1:1 sec-py/ml-unsafe-load
```

### case: pandas

```py
pd.read_pickle(path)
```

```expect
1:1 sec-py/ml-unsafe-load
```

### case: keras

```py
keras.models.load_model(p, safe_mode=False)
```

```expect
1:1 sec-py/ml-unsafe-load
```

### case: keras safe

```py
keras.models.load_model(p)
```

```expect
(none)
```

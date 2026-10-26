# sec-py/no-unsafe-deserialize — cases

### case: pickle.load

```py
import pickle

pickle.load(f)
```

```expect
3:1 sec-py/no-unsafe-deserialize
```

### case: dill

```py
dill.loads(b)
```

```expect
1:1 sec-py/no-unsafe-deserialize
```

### case: marshal

```py
marshal.loads(b)
```

```expect
1:1 sec-py/no-unsafe-deserialize
```

### case: shelve.open

```py
shelve.open(path)
```

```expect
1:1 sec-py/no-unsafe-deserialize
```

### case: pickle.dumps is fine

```py
pickle.dumps(obj)
```

```expect
(none)
```

### case: json is fine

```py
json.load(f)
```

```expect
(none)
```

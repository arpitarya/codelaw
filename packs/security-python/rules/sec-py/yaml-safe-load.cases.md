# sec-py/yaml-safe-load — cases

### case: a bare load

```py
import yaml

yaml.load(stream)
```

```expect
3:1 sec-py/yaml-safe-load
```

```fixed
import yaml

yaml.safe_load(stream)
```

### case: an unsafe Loader

```py
yaml.load(stream, Loader=yaml.Loader)
```

```expect
1:1 sec-py/yaml-safe-load
```

```fixed
yaml.safe_load(stream)
```

### case: a positional FullLoader

```py
yaml.load(stream, yaml.FullLoader)
```

```expect
1:1 sec-py/yaml-safe-load
```

```fixed
yaml.safe_load(stream)
```

### case: SafeLoader

```py
yaml.load(stream, Loader=yaml.SafeLoader)
```

```expect
(none)
```

### case: safe_load

```py
yaml.safe_load(stream)
```

```expect
(none)
```

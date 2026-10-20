# py/module-import-not-at-top — cases

### case: sys.path change before imports

```py
import sys
sys.path.insert(0, 'lib')
import local_module
```

```expect
(none)
```

### case: os.environ change before imports

```py
import os
os.environ['X'] = '1'
import other
```

```expect
(none)
```

### case: conditional imports

```py
try:
    import ujson as json
except ImportError:
    import json
import os
```

```expect
(none)
```

### case: dunders and docstring

```py
"""Doc."""
__all__ = []
__version__ = '1'
import os
```

```expect
(none)
```

### case: imports inside functions are fine

```py
x = 1
def f():
    import os
```

```expect
(none)
```

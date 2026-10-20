# py/star-import-usage — cases

### case: names in __all__

```py
from os.path import *
__all__ = ["join", "local"]
local = 1
```

```expect
2:12 py/star-import-usage
```

### case: submodule bound by a relative star import

```py
from .events import *
print(events.__all__)
```

```expect
(none)
```

### case: builtins

```py
from os.path import *
print(len([]))
```

```expect
(none)
```

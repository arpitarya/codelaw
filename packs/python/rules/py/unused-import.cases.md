# py/unused-import — cases

### case: listed in __all__

```py
from .core import load

__all__ = ["load"]
```

```expect
(none)
```

### case: redundant alias re-exports

```py
from .core import load as load
```

```expect
(none)
```

### case: used only in a string annotation

```py
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .models import User


def name(user: "User") -> str:
    return ""
```

```expect
(none)
```

### case: future imports are used by being there

```py
from __future__ import annotations
```

```expect
(none)
```

### case: one of several names

```py
from os.path import join, split

print(join("a", "b"))
```

```expect
1:27 py/unused-import
```

```fixed
from os.path import join

print(join("a", "b"))
```

### case: import with a fallback in except

```py
try:
    import ujson
except ImportError:
    ujson = None
```

```expect
(none)
```

### case: last import of a name wins

```py
import re
import re
```

```expect
2:8 py/unused-import
```

### case: function-level import

```py
def f():
    import json
    return 1
```

```expect
2:12 py/unused-import
```

```fixed
def f():
    return 1
```

### case: used inside a function

```py
import json


def f():
    return json.dumps({})
```

```expect
(none)
```

### case: importing __all__ defines the exports

```py
from io import __all__
```

```expect
(none)
```

### case: class-level import is an attribute

```py
class C:
    from os.path import join
```

```expect
(none)
```

### case: submodule loaded for an aliased package

```py
import multiprocessing as mp
import multiprocessing.connection

mp.connection.wait([])
```

```expect
(none)
```

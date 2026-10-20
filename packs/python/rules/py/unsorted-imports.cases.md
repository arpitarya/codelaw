# py/unsorted-imports — cases

### case: same module ordered by first name

```py
from multiprocessing import pool
from multiprocessing import Pool as ProcessPool
```

```expect
1:1 py/unsorted-imports
```

```fixed
from multiprocessing import Pool as ProcessPool
from multiprocessing import pool
```

### case: members and merging

```py
from collections import deque, OrderedDict
from collections import abc
```

```expect
1:1 py/unsorted-imports
```

```fixed
from collections import OrderedDict, abc, deque
```

### case: relative imports furthest first

```py
from . import a
from .. import b
```

```expect
1:1 py/unsorted-imports
```

```fixed
from .. import b
from . import a
```

### case: a comment moves with its import

```py
import sys
# json is fast enough
import json
```

```expect
1:1 py/unsorted-imports
```

```fixed
# json is fast enough
import json
import sys
```

### case: blocks are separate

```py
import sys

x = 1

import os
```

```expect
(none)
```

### case: nested blocks

```py
def f():
    import sys
    import os
```

```expect
2:5 py/unsorted-imports
```

```fixed
def f():
    import os
    import sys
```

### case: future first

```py
import os
from __future__ import annotations
```

```expect
1:1 py/unsorted-imports
```

```fixed
from __future__ import annotations

import os
```

### case: case-insensitive module order

```py
import apple
import Zebra
import requests
```

```expect
1:1 py/unsorted-imports
```

```fixed
import apple
import requests
import Zebra
```

### case: already sorted with sections

```py
import os
import sys

import requests

from . import local
```

```expect
(none)
```

### case: isort: skip

```py
import sys  # isort: skip
import os
```

```expect
(none)
```

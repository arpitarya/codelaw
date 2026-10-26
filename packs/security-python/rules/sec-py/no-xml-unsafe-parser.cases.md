# sec-py/no-xml-unsafe-parser — cases

### case: ElementTree through an alias

```py
import xml.etree.ElementTree as ET

ET.fromstring(text)
```

```expect
3:1 sec-py/no-xml-unsafe-parser
```

### case: ElementTree imported from xml.etree

```py
from xml.etree import ElementTree

ElementTree.parse(p)
```

```expect
3:1 sec-py/no-xml-unsafe-parser
```

### case: defusedxml

```py
import defusedxml.ElementTree as ET

ET.parse(p)
```

```expect
(none)
```

### case: minidom

```py
from xml.dom import minidom

minidom.parseString(text)
```

```expect
3:1 sec-py/no-xml-unsafe-parser
```

### case: lxml

```py
from lxml import etree

etree.fromstring(text)
```

```expect
3:1 sec-py/no-xml-unsafe-parser
```

### case: an unrelated etree

```py
from mylib import etree

etree.parse(text)
```

```expect
(none)
```

### case: an unrelated parse

```py
parser.parse(text)
```

```expect
(none)
```

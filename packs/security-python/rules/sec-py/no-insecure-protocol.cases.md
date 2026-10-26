# sec-py/no-insecure-protocol — cases

### case: an import

```py
import telnetlib
```

```expect
1:1 sec-py/no-insecure-protocol
```

### case: a from import

```py
from ftplib import FTP
```

```expect
1:1 sec-py/no-insecure-protocol
```

### case: SNMP

```py
CommunityData("public")
```

```expect
1:1 sec-py/no-insecure-protocol
```

### case: ssh

```py
import paramiko
```

```expect
(none)
```

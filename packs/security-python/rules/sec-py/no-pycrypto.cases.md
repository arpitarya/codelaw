# sec-py/no-pycrypto — cases

### case: pycrypto

```py
from Crypto.Cipher import AES
```

```expect
1:1 sec-py/no-pycrypto
```

### case: import

```py
import Crypto
```

```expect
1:1 sec-py/no-pycrypto
```

### case: Cryptodome

```py
from Cryptodome.Cipher import AES
```

```expect
(none)
```

### case: cryptography

```py
from cryptography.fernet import Fernet
```

```expect
(none)
```

# sec-py/no-xmlrpc — cases

### case: ServerProxy

```py
xmlrpc.client.ServerProxy(url)
```

```expect
1:1 sec-py/no-xmlrpc
```

### case: a server

```py
SimpleXMLRPCServer(("localhost", 8000))
```

```expect
1:1 sec-py/no-xmlrpc
```

### case: requests

```py
requests.post(url)
```

```expect
(none)
```

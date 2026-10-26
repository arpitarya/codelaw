# sec-py/paramiko-host-key — cases

### case: auto add

```py
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
```

```expect
1:1 sec-py/paramiko-host-key
```

### case: reject

```py
client.set_missing_host_key_policy(paramiko.RejectPolicy())
```

```expect
(none)
```

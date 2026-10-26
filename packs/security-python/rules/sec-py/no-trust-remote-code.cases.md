# sec-py/no-trust-remote-code — cases

### case: from_pretrained

```py
AutoModel.from_pretrained(n, trust_remote_code=True)
```

```expect
1:1 sec-py/no-trust-remote-code
```

### case: false

```py
AutoModel.from_pretrained(n, trust_remote_code=False)
```

```expect
(none)
```

### case: a dataset

```py
load_dataset("x", trust_remote_code=True)
```

```expect
1:1 sec-py/no-trust-remote-code
```

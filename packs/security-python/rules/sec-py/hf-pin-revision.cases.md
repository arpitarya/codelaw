# sec-py/hf-pin-revision — cases

### case: pinned

```py
model = AutoModel.from_pretrained(name, revision=rev)
```

```expect
(none)
```

### case: unpinned

```py
tok = AutoTokenizer.from_pretrained(name)
```

```expect
1:7 sec-py/hf-pin-revision
```

### case: hub download

```py
hf_hub_download(repo_id, filename)
```

```expect
1:1 sec-py/hf-pin-revision
```

### case: dataset

```py
load_dataset("org/data")
```

```expect
1:1 sec-py/hf-pin-revision
```

### case: a local path is still flagged

```py
AutoModel.from_pretrained("./model")
```

```expect
1:1 sec-py/hf-pin-revision
```

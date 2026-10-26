# sec-py/fastapi-cors — cases

### case: wildcard with credentials

```py
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_credentials=True)
```

```expect
1:1 sec-py/fastapi-cors
```

### case: wildcard without credentials

```py
app.add_middleware(CORSMiddleware, allow_origins=["*"])
```

```expect
(none)
```

### case: listed origins

```py
app.add_middleware(CORSMiddleware, allow_origins=["https://a.example"], allow_credentials=True)
```

```expect
(none)
```

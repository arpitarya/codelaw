# json/no-duplicate-keys — edge cases

### case: same key in different objects

```json
{
  "a": { "id": 1 },
  "b": { "id": 2 }
}
```

```expect
(none)
```

### case: escaped spelling of the same key

```json
{
  "id": 1,
  "\u0069d": 2
}
```

```expect
3:3 json/no-duplicate-keys
```

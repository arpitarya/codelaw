# json/tsconfig-strict — edge cases

### case: extends a base config

```json
{
  "extends": "@tsconfig/node22/tsconfig.json",
  "compilerOptions": { "outDir": "dist" }
}
```

```expect
(none)
```

### case: strict turned off

```json
{
  "compilerOptions": {
    "strict": false
  }
}
```

```expect
3:15 json/tsconfig-strict
```

```fixed
{
  "compilerOptions": {
    "strict": true
  }
}
```

### case: no compilerOptions

```json
{
  "include": ["src"]
}
```

```expect
1:1 json/tsconfig-strict
```

```fixed
{
  "compilerOptions": {
    "strict": true
  },
  "include": ["src"]
}
```

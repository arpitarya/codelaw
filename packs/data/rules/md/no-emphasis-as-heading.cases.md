# md/no-emphasis-as-heading — edge cases

### case: emphasis inside a sentence

````md
Use **only** this.

*Note: it is slow.*
````

```expect
(none)
```

### case: emphasis around a link

````md
*For version 1, see [the old docs](v1.md)*
````

```expect
(none)
```

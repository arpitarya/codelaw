# md/no-bare-urls — edge cases

### case: already linked or in code

````md
See <https://a.example>, [b](https://b.example), `https://c.example` and <a href="https://d.example">d</a>.
````

```expect
(none)
```

### case: closing parenthesis

````md
Docs (https://example.com/a_(b)).
````

```expect
1:7 md/no-bare-urls
```

````fixed
Docs (<https://example.com/a_(b)>).
````

### case: other schemes and rejected links

````md
Repository: git+https://github.com/a/b.git

By [Ann Lee](https://github.com/Ann Lee).
````

```expect
(none)
```

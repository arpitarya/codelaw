# md/link-image-reference-definitions — edge cases

### case: comment idiom and shortcut use

````md
[//]: # (a comment)

See [docs].

[docs]: https://example.com
````

```expect
(none)
```

### case: unused definition of a fragment

````md
Text.

[end]: #end
````

```expect
3:1 md/link-image-reference-definitions
```

````fixed
Text.

````

### case: defined twice

````md
See [a].

[a]: https://one.example
[a]: https://two.example
````

```expect
4:1 md/link-image-reference-definitions
```

````fixed
See [a].

[a]: https://one.example
````

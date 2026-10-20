# md/reference-links-images — edge cases

### case: shortcuts and task lists

````md
- [x] done
- [ ] todo

See [CommonMark].
````

```expect
(none)
```

### case: labels ignore case

````md
Read [the spec][Spec].

[spec]: https://spec.commonmark.org/
````

```expect
(none)
```

### case: definition of a fragment

````md
Jump to [the end][end].

## End

[end]: #end
````

```expect
(none)
```

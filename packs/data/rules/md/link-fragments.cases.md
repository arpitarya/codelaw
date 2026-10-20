# md/link-fragments — edge cases

### case: repeated headings, HTML anchor, line anchor

````md
# Notes

## Usage: `x`

## Usage: `x`

<a id="custom"></a>

[a](#usage-x-1) [b](#custom) [c](#L10) [d](#top)
````

```expect
(none)
```

### case: wrong case

````md
# My Title

[a](#My-Title)
````

```expect
3:1 md/link-fragments
```

### case: reference definition

````md
# Title

[a][x]

[x]: #nowhere
````

```expect
5:1 md/link-fragments
```

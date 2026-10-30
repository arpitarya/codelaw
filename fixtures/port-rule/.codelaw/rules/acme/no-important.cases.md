# acme/no-important — cases

Written fresh from the rule's documented behaviour; none is copied from stylelint's tests.

### case: no space before the bang

```css
a { color: teal!important; }
```

```expect
1:16 acme/no-important
```

### case: inside a media query

```css
@media print {
  nav { display: none !important; }
}
```

```expect
2:23 acme/no-important
```

### case: a custom property's value

```css
:root { --gap: 4px !important; }
```

```expect
1:20 acme/no-important
```

### case: the word in a comment or a string is not a declaration's

```css
/* never use !important here */
a::after { content: "!important"; }
```

```expect
(none)
```

# Cases: scss/double-slash-comment-empty

### case: a trailing empty comment

```scss
.a {
  color: red; //
}
```

```expect
2:15 scss/double-slash-comment-empty
```

```fixed
.a {
  color: red;
}
```

### case: a doc comment

```scss
///
.a {
  color: red;
}
```

```expect
(none)
```

### case: a url is not a comment

```scss
.a {
  background: url(//example.com/a.png);
}
```

```expect
(none)
```

### case: a comment with text

```scss
// text
.a {
  color: red;
}
```

```expect
(none)
```

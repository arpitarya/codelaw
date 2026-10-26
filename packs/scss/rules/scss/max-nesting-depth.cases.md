# Cases: scss/max-nesting-depth

### case: three levels

```scss
.a {
  .b {
    .c {
      .d {
        color: red;
      }
    }
  }
}
```

```expect
(none)
```

### case: a media query does not count

```scss
.a {
  @media print {
    .b {
      .c {
        .d {
          .e {
            color: red;
          }
        }
      }
    }
  }
}
```

```expect
6:11 scss/max-nesting-depth
```

### case: a flat sheet

```scss
.a {
  color: red;
}
```

```expect
(none)
```

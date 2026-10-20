# html/missing-doctype — cases

### case: a fragment is not a document

```html
<div>
  <p>Partial</p>
</div>
```

```expect
(none)
```

### case: a malformed doctype still counts

```html
<!DOCTYPE>
<html lang="en"></html>
```

```expect
(none)
```

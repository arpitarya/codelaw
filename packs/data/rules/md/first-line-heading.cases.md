# md/first-line-heading — edge cases

### case: HTML title and leading comment

````md
<!-- generated -->
<h1 align="center">Tool</h1>

Text.
````

```expect
(none)
```

### case: front matter title

````md
---
title: Tool
---

Text.
````

```expect
(none)
```

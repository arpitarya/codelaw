# md/single-h1 — edge cases

### case: front matter title

````md
---
title: Guide
---

# Guide
````

```expect
5:1 md/single-h1
```

### case: setext headings count

````md
Guide
=====

Usage
=====
````

```expect
4:1 md/single-h1
```

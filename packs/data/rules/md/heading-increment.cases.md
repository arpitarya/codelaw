# md/heading-increment — edge cases

### case: front matter title

````md
---
title: Guide
---

### Setup
````

```expect
5:1 md/heading-increment
```

````fixed
---
title: Guide
---

## Setup
````

### case: subsections move with the fixed heading

````md
# A

### B

#### C

## D
````

```expect
3:1 md/heading-increment
```

````fixed
# A

## B

### C

## D
````

### case: first heading may start deep

````md
## A

### B
````

```expect
(none)
```

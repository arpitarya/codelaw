# Cases: spell/inconsistent-term

### case: across files, the spelling the project writes less is reported

```md
<!-- docs/a.md -->
The colour of a canceled order is gray.
<!-- docs/b.md -->
A canceled order keeps its colour.
<!-- src/order.js -->
// The cancelled order is gray; its color is kept.
export const order = 1;
```

```expect
src/order.js:1:8 spell/inconsistent-term
src/order.js:1:37 spell/inconsistent-term
```

```fixed
<!-- docs/a.md -->
The colour of a canceled order is gray.
<!-- docs/b.md -->
A canceled order keeps its colour.
<!-- src/order.js -->
// The canceled order is gray; its colour is kept.
export const order = 1;
```

### case: on a tie, the American spelling, in the word's case

```md
<!-- a.md -->
Organise the work.
<!-- b.md -->
Organize the work.
```

```expect
a.md:1:1 spell/inconsistent-term
```

```fixed
<!-- a.md -->
Organize the work.
<!-- b.md -->
Organize the work.
```

### case: one spelling throughout is not reported

```md
<!-- a.md -->
The cancelled order and the colour.
<!-- b.md -->
Cancelled twice, the colour stays.
```

```expect
(none)
```

### case: names are not read, only prose

```js
// src/a.js
// A canceled order is kept.
export const cancelledOrders = [];
const cancelled = "cancelled";
```

```expect
(none)
```

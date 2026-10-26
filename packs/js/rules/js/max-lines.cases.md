# js/max-lines — cases

### case: blank lines and comments are not counted

```js options={"max":2}
a();

// one
/* two
   three */
b();
```

```expect
(none)
```

### case: blank lines counted

```js options={"max":4,"skip_blank_lines":false}
a();



b();
```

```expect
5:1 js/max-lines
```

### case: comments counted

```js options={"max":2,"skip_comments":false}
a();
// one
b();
```

```expect
3:1 js/max-lines
```

### case: a line of code and a comment is code

```js options={"max":1}
a(); // note
b(); /* note */
```

```expect
2:1 js/max-lines
```

### case: reported at the first line over the limit

```js options={"max":2}
a();
b();
  c();
d();
```

```expect
3:3 js/max-lines
```

### case: a JSX comment is one comment line

```jsx options={"max":4}
const a = (
  <div>
    {/* note */}
  </div>
);
```

```expect
(none)
```

### case: the import block

```ts options={"max":2,"skip_imports":true}
import a from "a";
import { b } from "b";
import type { C } from "c";
use(a, b);
use2();
```

```expect
(none)
```

### case: imports counted by default

```ts options={"max":2}
import a from "a";
import { b } from "b";
use(a, b);
```

```expect
3:1 js/max-lines
```

### case: a file at the limit passes

```js options={"max":3}
a();
b();
c();
```

```expect
(none)
```

### case: an empty file

```js options={"max":0}

```

```expect
(none)
```

### case: a file with CRLF line endings

```js options={"max":1}
a();
b();
```

```expect
2:1 js/max-lines
```

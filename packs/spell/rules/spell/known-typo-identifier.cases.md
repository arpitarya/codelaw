# Cases: spell/known-typo-identifier

### case: a local name is renamed at its declaration and every use

```js
export function f() {
  const recieveAll = 1;
  return recieveAll + 1;
}
```

```expect
2:9 spell/known-typo-identifier
```

```fixed
export function f() {
  const receiveAll = 1;
  return receiveAll + 1;
}
```

### case: an exported name is reported, not renamed

```ts
export const recieveTimeout = 30;
```

```expect
1:14 spell/known-typo-identifier
```

### case: a Python module-level name and a parameter are not renamed

```py
lenght = 1


def f(adress):
    return adress
```

```expect
1:1 spell/known-typo-identifier
4:7 spell/known-typo-identifier
```

### case: a shorthand property names a key too

```js
export function f() {
  const recieve = 1;
  return { recieve };
}
```

```expect
2:9 spell/known-typo-identifier
```

### case: uses and imports are not reported

```js
import { recieve } from "./x.js";

export const y = recieve();
```

```expect
(none)
```

### case: a class member is reported where it is declared

```ts
export class Queue {
  recieve(): void {}
}
```

```expect
2:3 spell/known-typo-identifier
```

### case: the case of each part is kept

```js
export function f() {
  const RECIEVE_LENGHT = 1;
  return RECIEVE_LENGHT;
}
```

```expect
2:9 spell/known-typo-identifier
```

```fixed
export function f() {
  const RECEIVE_LENGTH = 1;
  return RECEIVE_LENGTH;
}
```

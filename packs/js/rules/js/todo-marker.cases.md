# js/todo-marker — cases

### case: a ticket and a date, in either order

```js
// TODO(ABC-123, 2027-01-31): later
// TODO(2027-01-31, ABC-123): later
/* FIXME(ABC-9) */
```

```expect
(none)
```

### case: a bare marker, a name without a ticket, a marker mid-sentence

```js
// TODO: later
// HACK(arpit) shim
const note = 1; // the TODO list is in the wiki
```

```expect
1:4 js/todo-marker
2:4 js/todo-marker
```

### case: a marker inside a block comment

```js
/**
 * FIXME: the cache never expires
 * See the FIXME above.
 */
function f() {}
```

```expect
2:4 js/todo-marker
```

### case: the ticket is not required

```js options={"require_ticket":false}
// TODO: later
```

```expect
(none)
```

### case: the date is required

```js options={"require_date":true}
// TODO(ABC-1): later
// TODO(ABC-1, 2027-01-31): later
```

```expect
1:4 js/todo-marker
```

### case: a date that is not a calendar day

```js
// TODO(ABC-1, 2027-02-30): later
```

```expect
1:4 js/todo-marker
```

### case: another tag list and ticket pattern

```js options={"tags":["XXX"],"ticket_pattern":"#[0-9]+"}
// TODO: ignored, it is not a tag here
// XXX(#42): fine
// XXX(ABC-1): not this project's ticket
```

```expect
3:4 js/todo-marker
```

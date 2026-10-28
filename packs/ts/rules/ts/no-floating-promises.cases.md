# Cases: ts/no-floating-promises

### case: a rejection handler handles it

```ts
async function save(): Promise<void> {}
save().catch(() => {});
save().then(() => {}, () => {});
```

```expect
(none)
```

### case: a then without a rejection handler still floats

```ts
async function save(): Promise<void> {}
save().then(() => {});
```

```expect
2:1 ts/no-floating-promises
```

### case: a call that returns no Promise is not judged

```ts
function log(): void {}
log();
```

```expect
(none)
```

### case: a thenable that is not Promise-shaped is not judged

```ts
declare const job: { then: number };
declare function start(): typeof job;
start();
```

```expect
(none)
```

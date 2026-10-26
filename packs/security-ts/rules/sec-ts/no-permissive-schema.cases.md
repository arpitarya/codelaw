# sec-ts/no-permissive-schema — cases

### case: z.any has no mechanical fix

```ts
const Body = z.object({ data: z.any() });
```

```expect
1:31 sec-ts/no-permissive-schema
```

### case: class-validator validate with the option

```ts
import { validate } from "class-validator";
await validate(dto, { forbidUnknownValues: true });
```

```expect
(none)
```

### case: class-validator validate without it

```ts
import { validate } from "class-validator";
await validate(dto);
```

```expect
2:7 sec-ts/no-permissive-schema
```

### case: valibot any

```ts
const s = v.object({ x: v.any() });
```

```expect
1:25 sec-ts/no-permissive-schema
```

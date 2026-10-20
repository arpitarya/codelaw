---
id: json/tsconfig-strict
title: "tsconfig.json turns on strict mode"
languages: [json]
files: ["**/tsconfig.json", "**/tsconfig.*.json"]
fix: suggest
fix_reason: "turning on strict mode can surface type errors the code then has to fix"
stability: preview
since: 0.1.0
---

A `tsconfig.json` sets `"strict": true` in `compilerOptions`. A config that `extends` another and doesn't set `strict` itself is left alone, since it may inherit it.

**Why:** Strict mode turns on the checks that catch `null` and `undefined` mistakes and implicit `any`; without it the compiler lets most of those bugs through.

```codelaw
native: json/tsconfig-strict
```

## Examples

### JSON

#### Don't

```json path=tsconfig.json
{
  "compilerOptions": {
    "target": "es2022",
    "module": "nodenext"
  }
}
```

#### Do

```json path=tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "target": "es2022",
    "module": "nodenext"
  }
}
```

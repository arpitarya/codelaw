---
id: sec-js/upload-limits
title: "multer has size limits and a file filter"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which sizes and types are allowed is a decision for the author"
stability: preview
security: { cwe: [434, 770], owasp: ["A06:2025"], severity: medium }
since: 0.3.0
---

`multer(…)` is given both `limits` and `fileFilter`.

**Why:** Without a size limit one request fills the disk or memory, and without a filter any file type is stored and may later be served as a page or run (CWE-434, CWE-770).

```codelaw
match:
  all:
    - pattern: multer($$$ARGS)
    - any:
        - not:
            has:
              kind: pair
              stopBy: end
              regex: '^["'']?limits["'']?\s*:'
        - not:
            has:
              stopBy: end
              any:
                - kind: pair
                  regex: '^["'']?fileFilter["'']?\s*:'
                - kind: shorthand_property_identifier
                  regex: '^fileFilter$'
message: "`multer` has no `limits` or no `fileFilter`"
```

## How to fix

1. Set `limits: { fileSize, files }` to what the feature needs.
2. Write a `fileFilter` that accepts only the types the feature handles, and check the file's content, not its name or `mimetype` alone.
3. Store uploads outside the web root, under a name the server picks.

## Examples

### JavaScript

#### Don't

```js
const upload = multer({ dest: "uploads/" });
```

#### Do

```js
const upload = multer({
  dest: "uploads/",
  limits: { fileSize: 1000000, files: 1 },
  fileFilter: (req, file, cb) => cb(null, file.mimetype === "image/png"),
});
```

### JSX

#### Don't

```jsx
const upload = multer({ dest: "uploads/" });
```

#### Do

```jsx
const upload = multer({
  dest: "uploads/",
  limits: { fileSize: 1000000, files: 1 },
  fileFilter: (req, file, cb) => cb(null, file.mimetype === "image/png"),
});
```

### TypeScript

#### Don't

```ts
const upload = multer({ dest: "uploads/" });
```

#### Do

```ts
const upload = multer({
  dest: "uploads/",
  limits: { fileSize: 1000000, files: 1 },
  fileFilter: (req, file, cb) => cb(null, file.mimetype === "image/png"),
});
```

### TSX

#### Don't

```tsx
const upload = multer({ dest: "uploads/" });
```

#### Do

```tsx
const upload = multer({
  dest: "uploads/",
  limits: { fileSize: 1000000, files: 1 },
  fileFilter: (req, file, cb) => cb(null, file.mimetype === "image/png"),
});
```

---
id: sec-js/no-electron-insecure
title: "No insecure Electron window setting"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "the window's preload and renderer code may rely on the setting; a setting set to `true` (such as `nodeIntegration`) has no mechanical fix and is reported as manual"
stability: preview
security: { cwe: [79, 829], owasp: ["A02:2025"], severity: high }
since: 0.3.0
---

`webSecurity`, `contextIsolation` and `sandbox` are not `false`, and `nodeIntegration`, `nodeIntegrationInWorker`, `allowRunningInsecureContent` and `enableRemoteModule` are not `true`.

**Why:** Each switch removes a wall between web content and the machine: with Node integration on or context isolation off, one XSS bug in a window is code execution on the user's computer (CWE-79, CWE-829).

```codelaw
match:
  any:
    - pattern:
        context: '({ webSecurity: $V })'
        selector: pair
    - pattern:
        context: '({ contextIsolation: $V })'
        selector: pair
    - pattern:
        context: '({ sandbox: $V })'
        selector: pair
    - pattern:
        context: '({ nodeIntegration: $W })'
        selector: pair
    - pattern:
        context: '({ nodeIntegrationInWorker: $W })'
        selector: pair
    - pattern:
        context: '({ nodeIntegrationInSubFrames: $W })'
        selector: pair
    - pattern:
        context: '({ allowRunningInsecureContent: $W })'
        selector: pair
    - pattern:
        context: '({ enableRemoteModule: $W })'
        selector: pair
constraints:
  V: { regex: '^false$' }
  W: { regex: '^true$' }
message: "This Electron setting removes a security boundary"
fix:
  - replace: { target: $V, with: "true" }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const win = new BrowserWindow({ webPreferences: { contextIsolation: false } });
```

#### Do

```js
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

#### Fixed

```js
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

### JSX

#### Don't

```jsx
const win = new BrowserWindow({ webPreferences: { contextIsolation: false } });
```

#### Do

```jsx
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

#### Fixed

```jsx
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

### TypeScript

#### Don't

```ts
const win = new BrowserWindow({ webPreferences: { contextIsolation: false } });
```

#### Do

```ts
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

#### Fixed

```ts
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

### TSX

#### Don't

```tsx
const win = new BrowserWindow({ webPreferences: { contextIsolation: false } });
```

#### Do

```tsx
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

#### Fixed

```tsx
const win = new BrowserWindow({ webPreferences: { contextIsolation: true } });
```

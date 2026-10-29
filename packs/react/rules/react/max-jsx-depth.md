---
id: react/max-jsx-depth
title: "JSX elements nest no deeper than a maximum"
languages: [jsx, tsx]
fix: manual
fix_reason: "moving a nested part into a component means choosing its name and its props; that is the author's call"
stability: preview
options:
  max: { type: "integer", default: 6, description: "the most JSX elements that may nest inside one another" }
ported_from: ["eslint-plugin-react/jsx-max-depth"]
since: 0.5.0
---

JSX elements nest at most `max` deep (6 by default). Every element and fragment counts, a self-closing one too, and nesting goes through `{…}` — an element returned from a `.map()` callback inside `<ul>` is one level below the `<ul>`. A callback does not reset the count. The rule reports once for each branch that passes the limit, on the first element past it: `JSX is nested 8 deep (max 6)`, the number being how deep that branch goes.

**Why:** Deep markup hides what is a component and what is just layout, and an indent at that depth leaves little room for the content. Naming a subtree makes it testable and reusable.

```codelaw
native: react/max-jsx-depth
```

## How to fix

1. Take the subtree that starts at the reported element and move it into a component with a name that says what it is (`<ProfileMenu />`).
2. Pass in what the subtree reads as props.
3. A wrapper `<div>` that only carries a class can often be a prop on its child, or go away.

## Examples

### JSX

#### Don't

```jsx options={"max":2}
const Page = () => (
  <main>
    <section>
      <ul>
        <li>one</li>
      </ul>
    </section>
  </main>
);
```

#### Do

```jsx options={"max":2}
const Items = () => (
  <ul>
    <li>one</li>
  </ul>
);
const Page = () => (
  <main>
    <Items />
  </main>
);
```

### TSX

#### Don't

```tsx options={"max":2}
const Page = (): JSX.Element => (
  <main>
    <section>
      <ul>
        <li>one</li>
      </ul>
    </section>
  </main>
);
```

#### Do

```tsx options={"max":2}
const Items = (): JSX.Element => (
  <ul>
    <li>one</li>
  </ul>
);
const Page = (): JSX.Element => (
  <main>
    <Items />
  </main>
);
```

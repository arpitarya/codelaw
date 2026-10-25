# @codelaw/rules-redux-toolkit

codelaw's Redux Toolkit rules — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on npm only, as
`@codelaw/rules-redux-toolkit`: Redux is a JavaScript library.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:redux-toolkit/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` adds it when `package.json` depends on `@reduxjs/toolkit` or
`redux`.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `redux-toolkit/recommended` | 12 | written from the Redux Style Guide: its Priority A rules as errors, its Priority B rules as warnings |

<!-- /docs-sync:presets -->

There is no official Redux Toolkit ESLint plugin, so **these rules are
codelaw's own** (the decider's ruling of 2026-10-24, option (a)): each is
written fresh from one or more rules of the official
[Redux Style Guide](https://redux.js.org/style-guide/), and no rule has a
`ported_from:`. A Style Guide rule is guidance; only those with a
deterministic match in the code ship ([L5](../../records/0007_LAW-5-no-model-in-the-engine.md)).

Several rules first find the file's reducers: a `createSlice` case reducer,
a builder's `addCase`/`addMatcher`/`addDefaultCase` callback (both update an
Immer draft), or a hand-written `(state, action)` function that tests
`action.type`.

| rule | fix | Style Guide section |
|---|---|---|
| `redux-toolkit/no-mutating-reducer` | manual | A · Do Not Mutate State |
| `redux-toolkit/no-side-effects-in-reducer` | manual | A · Reducers Must Not Have Side Effects |
| `redux-toolkit/no-non-serializable-state` | manual | A · Do Not Put Non-Serializable Values in State or Actions |
| `redux-toolkit/no-legacy-create-store` | manual | B · Use Redux Toolkit for Writing Redux Logic |
| `redux-toolkit/prefer-create-slice` | manual | B · Use Redux Toolkit for Writing Redux Logic; Use Immer for Writing Immutable Updates |
| `redux-toolkit/no-blind-payload-return` | manual | B · Reducers Should Own the State Shape |
| `redux-toolkit/slice-key-names` | manual | B · Name State Slices Based On the Stored Data |
| `redux-toolkit/no-sequential-dispatch` | manual | B · Avoid Dispatching Many Actions Sequentially; Model Actions as Events, Not Setters |
| `redux-toolkit/prefer-hooks-api` | manual | B · Use the React-Redux Hooks API |
| `redux-toolkit/object-shorthand-mapdispatch` | manual | B · Use the Object Shorthand Form of mapDispatch with connect |
| `redux-toolkit/no-object-selector` | manual | B · Call useSelector Multiple Times in Function Components |
| `redux-toolkit/no-immutable-js` | manual | B · Use Plain JavaScript Objects for State |

Every fix is `manual`: each rewrites how state is shaped or updated, which
changes what the rest of the code reads.

## Not written, and why

The Style Guide's other Priority A and B rules are judgment about a design,
not a pattern a check can find:

| Style Guide rule | why there is no rule |
|---|---|
| A · Only One Redux Store Per App | the stores are spread over the whole project, and importing the store directly is sometimes right (tests, non-React entry points); a file-level check sees neither |
| A · Do Not Put Non-Serializable Values in … Actions | an action's payload is made at the dispatch site, often from a variable; only the state's literals are checked (`no-non-serializable-state`) |
| B · Structure Files as Feature Folders with Single-File Logic | a layout choice; a folder named `reducers/` doesn't show whether the logic is split |
| B · Put as Much Logic as Possible in Reducers | needs to know which component code is state logic |
| B · Organize State Structure Based on Data Types, Not Components | whether `loginScreen` is a component or data is a naming judgment |
| B · Treat Reducers as State Machines | which transitions are invalid is the app's own state model |
| B · Normalize Complex Nested/Relational State | whether nested data is relational needs its meaning |
| B · Keep State Minimal and Derive Additional Values | whether a field is derived from another needs its meaning |
| B · Write Meaningful Action Names | whether a name is meaningful is a judgment; a fixed list of bad names would be arbitrary |
| B · Allow Many Reducers to Respond to the Same Action | advice to allow something; nothing to find |
| B · Evaluate Where Each Piece of State Should Live | the choice between Redux and component state is the app's |
| B · Connect More Components to Read Data from the Store | how far down to read is a performance judgment |
| B · Use Static Typing | the language of a file, not a pattern in it; codelaw's `ts` pack covers TypeScript |
| B · Use the Redux DevTools Extension for Debugging | `configureStore` turns it on; turning it off in production is deliberate |

---
type: Handoff
name: W-41
description: "Arpit, 2026-10-25: existing React context use only warns; any new createContext or useContext is an error — a react/no-context rule plus a grandfathering baseline."
item: W-41
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-41 — old React context warns, new React context fails

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider rules what "new" means | `createContext` and `useContext` both | ✅ 2026-10-25 — [compare](../compare/ci-profiles-and-dated-exceptions.compare.md) fork 5 |
| 2 | Add `react/no-context` to the react pack, with cases | the rule | ⏳ |
| 3 | Document the adoption recipe (below) in the rule's page and USAGE | old uses warn, new ones fail | ⏳ |
| 4 | Fixture: a repo with two context files; the recipe; then a new `useContext` → exit 1, untouched old ones → warnings | proof the recipe works end to end | ⏳ |

**Arpit, 2026-10-25:** *"wherever we were using React context, that's okay. It should just throw a warning. But if there is a new implementation, it shouldn't be allowed and it should throw an error."*

**Model:** Sonnet — a rule with a written spec and cases; the grandfathering is existing baseline behaviour.

## Goal and design

**The rule `react/no-context`** (default severity `error`; not in `react/recommended` — it is a team choice):
- Flags `createContext(…)` and `useContext(…)` imported from `react` (named, aliased, or `React.createContext` / `React.useContext` on the default or namespace import). A local function named `useContext` is not flagged.
- Out of scope, named in the rule page: React 19 `use(SomeContext)` (can't tell a context from a promise without types) and `<Ctx.Provider>` / `<Ctx>` JSX.
- Fix: `manual` — the steps name the team's alternative (an option `use_instead`, text, default "props or the app's state store").

**The adoption recipe — no new engine code** (SR-BASELINE decisions 1–2 already do this: `warn_from` with no deadline → warning; a new use → error):

```bash
# codelaw.toml: [rules] "react/no-context" = "error"
codelaw baseline add --rule react/no-context \
  --warn-from 2026-10-25 --on-touch keep --reason "legacy context, grandfathered"
```

- Every file using context today: **warning**, forever (no deadline) — or until a date, once W-40 lands (`--expires`).
- A **new file** using context: **error** (no baseline entry).
- **One more** `useContext` in an old file: **every** use in that file errors ("3 in this file; the baseline allows 2").
- `--on-touch keep`: editing an old file for other reasons doesn't force a migration.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `codelaw test packs/react` green with cases: named, aliased, `React.*`, shadowed local, `use()` not flagged | ⏳ |
| 2 · | the fixture in step 4 gives the stated exit codes | ⏳ |
| 3 · | `codelaw explain react/no-context` shows the recipe | ⏳ |

## Blockers

None.

## Record affected

**No SR affected** — a pack rule (its page is its spec, SR-RULE-FORMAT) using SR-BASELINE as it stands. SR-RULE-PACKS' rule counts, if they are stated, move by one.

## Hazards

- **The count hole** (SR-BASELINE's veto result): with `on_touch = keep`, removing one `useContext` and adding another in the same file nets out and passes. Accepted; say so on the rule page.
- With W-40's `require_expiry = true` this baseline needs an `expires`.

## Reference

[compare doc](../compare/ci-profiles-and-dated-exceptions.compare.md) · [SR-BASELINE](../../records/0104_baseline-and-deprecations.md) decisions 1–2, 5

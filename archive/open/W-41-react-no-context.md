---
type: Handoff
name: W-41
description: "Arpit, 2026-10-25: existing React context use only warns; any new createContext or useContext is an error — a react/no-context rule plus a grandfathering baseline."
item: W-41
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), sub-agent"
ball: done
---

# W-41 — old React context warns, new React context fails

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider rules what "new" means | `createContext` and `useContext` both | ✅ 2026-10-25 — [compare](../compare/ci-profiles-and-dated-exceptions.compare.md) fork 5 |
| 2 | Add `react/no-context` to the react pack, with cases | the rule | ✅ 2026-10-25 — `native: react/no-context` in `react.rs`; 4 Don't/Do pairs, unit test `react::tests::no_context` |
| 3 | Document the adoption recipe (below) in the rule's page and USAGE | old uses warn, new ones fail | ✅ 2026-10-25 — the page's summary, step 2 of *How to fix* and *Adopting it…*; USAGE *Without AI* |
| 4 | Fixture: a repo with two context files; the recipe; then a new `useContext` → exit 1, untouched old ones → warnings | proof the recipe works end to end | ✅ 2026-10-25 — `fixtures/react-no-context/`, run by `crates/codelaw-cli/tests/react_no_context.rs` |

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
| 1 · | `codelaw test packs/react` green with cases: named, aliased, `React.*`, shadowed local, `use()` not flagged | ✅ `codelaw test packs/react`: 22 rules · 305 checks · 0 failed; the aliased, namespace, shadowed-local, other-module and `use()` cases are `react::tests::no_context` (a manual rule's page examples can't hold a Do that only differs by import) |
| 2 · | the fixture in step 4 gives the stated exit codes | ✅ `cargo test -p codelaw-cli --test react_no_context`: 1 passed — before the baseline 3 errors, exit 1; `baseline add` exit 0 ("2 entries, 3 findings allowed"); untouched 3 warnings, exit 0; a new file → its use errors, exit 1; one more use in `Header.jsx` → "(2 in this file; the baseline allows 1)", exit 1 |
| 3 · | `codelaw explain react/no-context` shows the recipe | ✅ the summary and *How to fix* step 2 carry the `codelaw baseline add … --warn-from <today> --on-touch keep` command; asserted in the same test |

## Blockers

None.

## Record affected

**No SR affected** — a pack rule (its page is its spec, SR-RULE-FORMAT) using SR-BASELINE as it stands. SR-RULE-PACKS' rule counts, if they are stated, move by one.

**As landed (2026-10-25):** SR-BASELINE is unchanged. [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 8 gains a sub-bullet (a team-policy rule sits in no preset; `react/no-context` and its recipe) and now owns `fixtures/react-no-context` and `crates/codelaw-cli/tests/react_no_context.rs` (two rows in the register's Ownership table); [SR-ARCHITECTURE](../../records/0108_architecture.md) 15b gains a sub-bullet for the matcher. The corpus count is `react/no-context` = 10 (zustand and jotai make context — the rule is a ban, so these are true positives).

## Hazards

- **The count hole** (SR-BASELINE's veto result): with `on_touch = keep`, removing one `useContext` and adding another in the same file nets out and passes. Accepted; say so on the rule page.
- With W-40's `require_expiry = true` this baseline needs an `expires`.

## Reference

[compare doc](../compare/ci-profiles-and-dated-exceptions.compare.md) · [SR-BASELINE](../../records/0104_baseline-and-deprecations.md) decisions 1–2, 5

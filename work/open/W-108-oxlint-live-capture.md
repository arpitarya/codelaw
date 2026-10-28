---
type: Handoff
name: W-108
description: "Arpit, 2026-10-27 (voice docket): run the oxlint type-aware live capture W-107 left to the decider, and make the adapter's fixture and test hold on real output."
item: W-108
filed: 2026-10-28
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-108 — oxlint type-aware: the live capture

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Run oxlint's type-aware mode on W-107's fixture, in a throwaway folder outside the repo | real output, not output built from the docs | ✅ 2026-10-28 (Cowork, a scratch folder in the Cowork VM) — below |
| 2 | Put the captured `stdout.json` in `crates/codelaw-adapters/tests/fixtures/oxlint-type-aware/` | the test reads what oxlint prints | ⏳ |
| 3 | Make `oxlint_type_aware_findings_keep_their_typescript_ids` hold on it — the three differences below | the adapter proven on real output | ⏳ |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** *"You run it, create a work item for it."* — on W-107's inbox row (run the capture, or "skip it").

**Model:** Sonnet.

## The capture (2026-10-28)

`npm add -D oxlint oxlint-tsgolint typescript@7` in a fresh folder outside the repo (nothing installed in codelaw; L11), W-107's fixture copied in (`.oxlintrc.json` with `options.typeAware`, `src/index.ts`), the `tsconfig.json` from the inbox row, then `npx oxlint --format=json src/index.ts` (exit 1). Resolved: **oxlint 1.90.0**, **oxlint-tsgolint 7.0.2005**, **typescript 7.0.2** (`typescript@7` resolves to 7.0, not 7.1, today). `filename` is relative, so no `@ROOT@` substitution is needed.

```json
{ "diagnostics": [{"message": "Unexpected `await` on a non-Promise value","code": "unicorn(no-unnecessary-await)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/unicorn/no-unnecessary-await.html","help": "Consider removing the `await`","filename": "src/index.ts","labels": [{"span": {"offset": 192,"length": 5,"line": 8,"column": 3}}]},
{"message": "Promises must be awaited, add void operator to ignore.","code": "typescript(no-floating-promises)","severity": "error","url": "https://oxc.rs/docs/guide/usage/linter/rules/typescript/no-floating-promises.html","help": "The promise must end with a call to .catch, or end with a call to .then with a rejection handler, or be explicitly marked as ignored with the `void` operator.","filename": "src/index.ts","labels": [{"label": "This unhandled promise-like value has type `Promise<string>`.","span": {"offset": 182,"length": 6,"line": 7,"column": 3}},{"span": {"offset": 182,"length": 6,"line": 7,"column": 3}}]},
{"message": "Unexpected `await` of a non-Promise (non-\"Thenable\") value.","code": "typescript(await-thenable)","severity": "warning","url": "https://oxc.rs/docs/guide/usage/linter/rules/typescript/await-thenable.html","help": "Remove `await` if the value is synchronous, or change the expression to return a Promise or Thenable before awaiting it.","filename": "src/index.ts","labels": [{"label": "This expression is not Promise-like","span": {"offset": 198,"length": 2,"line": 8,"column": 9}},{"span": {"offset": 192,"length": 5,"line": 8,"column": 3}}]}],
              "number_of_files": 1,
              "number_of_rules": 111,
              "threads_count": 4,
              "start_time": 0.036492625
            }
```

## Where it differs from W-107's fixture (the test fails on it as written)

1. **A third finding:** `unicorn(no-unnecessary-await)` (warning) on `await` — on in oxlint 1.90.0's default `correctness` set. The test asserts `d.len() == 2`.
2. **`await-thenable`'s span:** two labels now — the first, labelled *"This expression is not Promise-like"*, on `42` (offset 198, length 2), the second on `await` (192, 5). The fixture had one span on `await 42` (192, 8), which the test asserts.
3. **`await-thenable`'s help:** *"Remove `await` if the value is synchronous, or change the expression to return a Promise or Thenable before awaiting it."* — the test asserts the old *"Remove unnecessary `await`."*
4. No `causes` or `related` fields (the fixture had both, empty) — the adapter must not require them.

`no-floating-promises` matches the fixture exactly.

## Definition of done

- The fixture's `stdout.json` is this capture; the mapping test asserts what it holds (the three findings, which label the adapter takes as the primary span, the new help line), and still passes `cargo test -p codelaw-adapters`.
- If the adapter's choice of span changes, SR-LINTER says which label it takes.

## Record affected

SR-LINTER decision 2 (the oxlint adapter), only if the span rule changes; otherwise no SR affected.

## Hazards

- oxlint changes its defaults and help text between versions; the test's comment names the captured versions so the next mismatch is read as drift, not a bug.

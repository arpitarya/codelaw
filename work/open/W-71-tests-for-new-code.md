---
type: Handoff
name: W-71
description: "Arpit, 2026-10-26: every new or edited function in JS, TS and Python must have a unit test and at least 60% line coverage (configurable); untouched old functions are exempt; a .codelawignore file exempts the rest."
item: W-71
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-71 — a unit test for every new or edited function (JS, TS, Python)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider confirms **diff-scoped**: the rules judge only functions the change set adds or edits | old code is exempt with no baseline file | 🔴 decider decides |
| 2 | Work out per function whether it is **new, edited or untouched**, against the base revision, by token stream | "edited" ignores formatting, moves and comments | ⏳ |
| 3 | `graph/untested-function`: a new or edited function that no test file reaches is an error | "every new function has a test" | ⏳ |
| 4 | `graph/function-coverage`: a new or edited function under **60%** line coverage (`min`) in an lcov report is an error | "and that test really runs it" | ⏳ |
| 5 | `.codelawignore` exempts files and single functions, with optional `expires` / `by` / reason | the escape hatch, visible in `codelaw exceptions` | ⏳ |
| 6 | Flags, config, cache key, opt-in preset `codelaw:tests`, docs | usable in CI, pre-commit and the agent hook | ⏳ |

**Arpit, 2026-10-26:** *"if a new function is added or if a file is being written, it should have a unit test for it. And by default, it should have a coverage of around 60%, which can be configured through options. It should be there for JS, TS, Python. But if there are older functions, then no need to have this rule. But there is an exception to it. If somebody is editing that function, then this rule again gets applied."*

**Arpit, 2026-10-26 (rulings):** scope is *"every function unless called out in a separate `.codelawignore` file"*; coverage is **per function**. On new-vs-old he asked for a recommendation: **diff-scoped** (below), awaiting his yes.

**Model:** Opus — new engine machinery (a function-level change set, an lcov reader, a new consumer file) across three languages and six records.

## Goal

### The one idea

- **Only new and edited functions are judged.** A function nobody touched is never reported, whatever its coverage.
- **"Edited" is decided per function, not per file.** Touching one function in a 40-function legacy file asks for a test for that one function only.
- **Two checks, one story:** a test must *reach* the function (static, always), and that test must *cover* it (from a report, when one is given).

### Step 1 — why diff-scoped (recommended), not baseline + touch

| | **diff-scoped** (recommended) | baseline + touch |
|---|---|---|
| how old code is exempt | the rule looks only at the change set | `codelaw baseline add` freezes every untested function today |
| setup | none | a baseline file with one entry per untested function — thousands in a legacy repo |
| editing an old function | it is in the change set → judged | needs `on_touch_scope = "function"` (SR-BASELINE decision 3, specified, **not built**) |
| plain `codelaw check` | judges nothing; the header says so | reports every grandfathered function as `info` |
| legacy debt visible / burn-down | no | yes (`baseline history`) |
| prior art | SonarQube "coverage on new code", Codecov patch, diff-cover | ESLint bulk suppressions, Betterer |

- **The verdict is still the same everywhere** ([L5](../../records/0007_LAW-5-no-model-in-the-engine.md)): the change set is an explicit input (`--changed-since`, `--staged`, the hook's file), never inferred.
- **Baseline stays possible later:** `on_no_change_set = "all"` makes the rules judge every function, and a baseline then freezes them — the same code, no second design.

### Step 2 — new, edited or untouched

- **Base content per changed file:** `--changed-since <ref>` → the merge base's copy; `--staged` → `HEAD`'s; the agent hook → `HEAD`'s. Read with `git show` (`git.rs` already has `files_at`). No copy at base → **a new file: every function in it is new.**
- **Functions are matched by qualified name** (`Invoice.total`, `module.helper`):
  - **new** — no function of that name at base (a rename is new);
  - **edited** — its `codelaw_lang::token_stream` (W-28) differs from base, so reformatting, comment edits and moving it do not count;
  - **untouched** — same tokens; never judged.
- **What counts as a function:** declarations, methods (getters/setters included), and arrow or function expressions bound to a name (`const f = () => …`, a class field, an object property); Python `def` / `async def` at module or class level. **Not:** anonymous callbacks (`items.map(x => …)`), nested `def`s (judged through the function that holds them), `.d.ts`, overload signatures, abstract methods, test files themselves, generated files.
- **No change set** (plain `check`, no git): nothing is judged; `text`/`agent` output opens with `new code: no change set; test rules not evaluated`, as the as-of header does (SR-BASELINE decision 4).

### Step 3 — `graph/untested-function`

- **A function is reached** when a test file (the graph's presets: Jest/Vitest test files, pytest `test_*.py` / `*_test.py`, unittest `test*.py`) refers to it by **graph identity**: imported under any alias, through a barrel or re-export, as `from pkg.mod import f` or `mod.f`. A method is reached when its class is.
- **A function that cannot be imported** (a JS function the module doesn't export): reached when a test file imports its module. Whether the test actually runs it is step 4's job.
- **Reported on the function's name:** "`computeTax` is new and no test reaches it (add one in `orders.test.ts`, or exempt it in `.codelawignore`)".
- **Runs everywhere, fast:** no report needed, so it also runs in the pre-commit hook and the agent hook, while the agent is still writing.

### Step 4 — `graph/function-coverage`

- **The input is an lcov file**, the one format all three ecosystems write: Jest (`coverageReporters: ["lcov"]`), Vitest (`coverage.reporter: ["lcov"]`), c8, coverage.py (`coverage lcov`), pytest-cov (`--cov-report=lcov`). lcov is plain text, so codelaw reads it itself (L11: no new dependency).
- **Per-function line coverage:** of the lcov `DA` lines inside the function's span *as codelaw parses it now*, the share with hits > 0. codelaw does not trust the tool's own `FN` names, so all tools are judged the same way.
- **Reported:** "`computeTax` is 42% covered (5 of 12 lines; min 60)". A function with no instrumented lines (type-only) is skipped.
- **No report given:** not evaluated, and the header says `coverage: none; not evaluated` — unless `require_report = true`, which makes it an error.
- **A report that does not match the file** (it doesn't name the file, a `DA` line is past the end of the file, or lands on a line that is blank or a comment now): one warning, `codelaw/stale-coverage`, on that file, and no coverage verdict for it.

### Options (both rules unless noted)

| option | type | default | meaning |
|---|---|---|---|
| `scope` | `"all"` \| `"public"` | `"all"` (Arpit's ruling) | `public`: exported (JS/TS); no leading `_`, or in `__all__` (Python) |
| `min` | int 0–100 | `60` | coverage rule: least line coverage per function |
| `mode` | `"function"` \| `"patch"` | `"function"` (Arpit's ruling) | `patch`: one finding when all new and edited functions' lines together are under `min` (diff-cover's measure) |
| `require_report` | bool | `false` | coverage rule: no report is an error, not a skip |
| `on_no_change_set` | `"skip"` \| `"all"` | `"skip"` | `all`: judge every function (for a baseline roll-out) |
| `ignore_names` | regex | none | function names exempt, e.g. `"^(main|__repr__)$"` |

### Step 5 — `.codelawignore`

At the project root, one entry per line; read at load, as the baseline folder is:

```text
# a whole file or folder: every function in it is exempt
src/generated/**
scripts/*.py
# one function: <path>#<qualified name>
src/orders/list.tsx#formatRow
app/billing.py#Invoice._legacy_total  expires=2027-03-01 by=lead@example.com: replaced in ORD-1500
```

- **Globs are `.gitignore`'s** (the `globset` codelaw already uses); `#name` picks one function.
- **The dated keys are the inline ignore's** (`expires`, `warn_days`, `by`, a reason after `:`), so SR-BASELINE decision 9 applies unchanged: a warning before `expires`, an error on and after it.
- **Every entry is listed by `codelaw exceptions`** (W-46), so the escape hatch is never invisible.
- **An entry that matches nothing** is `codelaw/stale-ignore` (`warn`, `auto`: `codelaw fix` removes it).
- **It covers these two rules only.** Making it a general per-rule ignore file is a separate decision — a backlog row if wanted.
- The inline `// codelaw-ignore graph/untested-function: <why>` above a function keeps working too.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · ruling | The decider confirms diff-scoped (step 1) | 🔴 |
| 2 · change set | new / edited / untouched per function for `--changed-since`, `--staged` and the hook, by token stream; a reformat-only commit judges no function; a rename is new; a new file's functions are all new | ⏳ |
| 3 · `untested-function` | JS, TS (incl. JSX/TSX), Python; graph identity through aliases and re-exports; fixtures for each "not a function" case above | ⏳ |
| 4 · `function-coverage` | lcov from Jest, Vitest and coverage.py fixtures; `min`, `mode`, `require_report`; `codelaw/stale-coverage` on a mismatched report; absolute and relative `SF:` paths | ⏳ |
| 5 · `.codelawignore` | globs, `#function`, dated keys, `codelaw exceptions` rows, `codelaw/stale-ignore` with its fix | ⏳ |
| 6 · surfaces | `--coverage <path>` and `[coverage] report = "…"`; the MCP `check` tool takes `coverage`; the two headers; JSON `data: { covered, lines, pct, min }` | ⏳ |
| 7 · cache | the key gains the report's hash and the base content (SR-CACHE decision 1); the cold/warm test passes | ⏳ |
| 8 · fixes ([L4](../../records/0006_LAW-4-every-rule-has-a-fix.md)) | both rules `manual`, `fix_reason:` "a test needs judgment", `## How to fix` steps; the `codelaw-fix` skill carries them | ⏳ |
| 9 · preset + docs | opt-in `codelaw:tests` (not in `recommended`); USAGE "tests for new code" section; CHANGELOG | ⏳ |
| 10 · budgets ([L8](../../records/0010_LAW-8-performance-budgets.md)) | CI's `perf` run with a change set: no budget exceeded | ⏳ |

## Blockers

- **DoD 1 — the decider's yes on diff-scoped.** Everything else follows from it; the inbox row names it.

## Record affected

- **[SR-PROJECT-GRAPH](../../records/0103_project-graph.md)** — the two graph rules and the "a test file reaches this function" query.
- **[SR-BASELINE](../../records/0104_baseline-and-deprecations.md) decision 3** — the change set gains functions (new / edited by token stream); decision 9's dated keys apply to `.codelawignore`.
- **[SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 1** — `ENGINE_RULES` gains `graph/untested-function`, `graph/function-coverage`, `codelaw/stale-coverage`, `codelaw/stale-ignore`.
- **[SR-CONFIG](../../records/0110_config.md)** — `[coverage]` and `.codelawignore`.
- **[SR-CLI-MCP](../../records/0115_cli-and-mcp.md)** — `--coverage`, the MCP parameter, the two header lines.
- **[SR-CACHE](../../records/0109_cache.md) decision 1** — the key's new inputs.
- **[SR-CONSUMER-LAYOUT](../../records/0111_consumer-layout.md)** — `.codelawignore` is a consumer file; if `init` ever writes one, from a template ([L10](../../records/0012_LAW-10-consumer-templates.md)).

## Hazards

- **The engine never runs tests.** It reads a report the consumer's runner wrote ([L5](../../records/0007_LAW-5-no-model-in-the-engine.md); L8). A report from an older commit is the main way to get a wrong verdict: generate it in the same CI job, before `codelaw check`.
- **lcov has no content hash,** so a stale report is only *mostly* caught (step 4's three signals). Say so in USAGE.
- **Static "reached" is not "tested":** a test that imports a function and never calls it passes step 3. Step 4 is the real check; step 3 is the fast early warning.
- **`mock.patch("pkg.mod.f")` and other string references** are not graph identity, so they do not count as reaching `f`. Document it; don't guess from strings.
- **`scope = "all"` is noisy in hooks:** an agent's new private helper asks for a test at once. That is Arpit's ruling; `.codelawignore` and `scope = "public"` are the exits.
- **Shallow CI clones** have no merge base: `--changed-since` already warns (`git.rs` `is_shallow`); the rules then judge nothing, and the header says why.
- **Source maps:** coverage of TS compiled to JS must be mapped back. Jest and Vitest write lcov against the `.ts` source; a raw `tsc` + c8 run without source maps does not. Fixtures cover the first two; the third is unsupported, and USAGE says so.

## Reference

- **Prior art:** SonarQube's default *Sonar way* gate — new-code coverage ≥ 80%; Codecov patch coverage; diff-cover `--fail-under` (Cobertura, Clover, JaCoCo, lcov; compares against `origin/main`, staged and unstaged by default); Jest `coverageThreshold`, coverage.py `fail_under` (whole-project, not new code).
- **Formats:** lcov `SF` / `DA` / `FN` / `FNDA`; Istanbul `coverage-final.json` (`statementMap`, `fnMap`, `s`, `f`); coverage.py JSON (per-function regions since 7.6.0). **Only lcov is in scope**; the two JSON readers are a follow-up (serde_json is ratified); Cobertura XML needs an XML parser, which L11 has not ratified.
- **Reuses:** `git.rs` (`changed_since`, `staged`, `files_at`, `is_shallow`); `codelaw_lang::token_stream` (W-28); the graph's test-file presets and identity resolution (as `graph/no-deprecated-usage` uses them); SR-BASELINE decision 9's dated keys; W-46's exceptions report.
- **Unlocks:** step 2 is exactly what SR-BASELINE decision 3's unbuilt `on_touch_scope = "function"` needs.

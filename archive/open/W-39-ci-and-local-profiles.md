---
type: Handoff
name: W-39
description: "Arpit, 2026-10-25: the same rule warns on a laptop and fails in CI (console, debugger), and some globs never error or never warn — profiles, a per-glob severity remap, and js/no-console."
item: W-39
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), 2026-10-25: closed — merged, CI perf within budget"
ball: done
---

# W-39 — warn on a laptop, fail in CI; globs that never error or never warn

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider rules the forks | the design below | ✅ 2026-10-25 — [compare](../compare/ci-profiles-and-dated-exceptions.compare.md) forks 1–3 |
| 2 | Write SR-CONFIG decisions 10–11 and the §1/decision 3/4 lines, and the schema, in one change | the record is the spec | ✅ 2026-10-25 — with the cites in SR-LINTER 5, SR-AGENT-SURFACES (hook), SR-CLI-MCP 2/4/5, SR-EDITOR 3, SR-ARCHITECTURE 4, SR-RULE-FORMAT 2, SR-RULE-PACKS 7 |
| 3 | Build profiles: selection, `[profiles.<name>]`, the header, `config --print` | console warns locally, fails in CI | ✅ 2026-10-25 — `--profile` on the common flags, `hook` and `config`; the warm client forwards `CI` and `CODELAW_PROFILE` |
| 4 | Build `remap` in `[[overrides]]` | a glob that never errors, or never warns | ✅ 2026-10-25 — `check`, `fix`, `hook`, MCP and the editor |
| 5 | Add `js/no-console` to the js pack with cases | the rule the ask is about | ✅ 2026-10-25 — native (`js_console.rs`), `suggest`; corpus count 38 |
| 6 | Docs: USAGE, README config example, CHANGELOG, GLOSSARY (`profile`, `remap`) | users can find it | ✅ 2026-10-25 |

**Arpit, 2026-10-25:** *"while I'm starting my dev server or doing it in my local, having debugger or console logs is fine. But when we are running it in CI/CD pipeline, it should throw an error. In local, it should give you a warning … for certain files it should never throw an error. For certain files, it should never throw a warning."*

**Model:** Opus — it changes SR-CONFIG's precedence and touches L5's inputs; no test catches a wrong precedence choice. Close call: with this file as the spec, Sonnet could build steps 3–5.

## Goal

```toml
[profiles.local.rules]
"js/no-console"  = "warn"
"js/no-debugger" = "warn"

[profiles.ci.rules]
"js/no-console"  = "error"
"js/no-debugger" = "error"

[[overrides]]                      # never fails the run
files = ["scripts/**"]
remap = { error = "warn" }

[[overrides]]                      # silent
files = ["**/*.generated.ts"]
remap = { error = "off", warn = "off", info = "off" }

[[overrides]]                      # strict: no warnings, only errors
files = ["src/payments/**"]
remap = { warn = "error" }
```

## Design (becomes SR-CONFIG decisions 10–11)

**Profiles**
- `[profiles.<name>]` holds `rules` and `overrides`, the same shapes as the top level. Root-only.
- **Which profile:** `--profile <name>` > `CODELAW_PROFILE` > `CI` set and not `""`, `false` or `0` → `ci` > `local`. A named profile the config doesn't define is a usage error (exit 2) — except `ci` and `local`, which may be absent (no change).
- **Per surface:** CLI and CI as above · **`codelaw hook` and MCP default to `ci`** (ruling 2) · LSP defaults to `local`. Each still honours `--profile` / `CODELAW_PROFILE`.
- **Precedence** (SR-CONFIG decision 4): defaults → `.editorconfig` → root → nested → **profile `rules`** → `[[overrides]]` → **profile `overrides`** → env → flags.
- **Visible:** `text`/`agent` output open with `profile ci (CI=true)`; `json` carries `profile: { name, source }`; `config --print` names it as a source.
- **L5:** env is already an input (decision 9); the profile is part of the resolved config, so of the cache key.

**`remap`**
- A table on any `[[overrides]]` entry (top level or profile): keys `error`, `warn`, `info`; values `error`, `warn`, `info`, `off`.
- Applied **last** — after the rule's severity and after the baseline's outcome — so an expired baseline entry in a `{ error = "warn" }` glob is a warning.
- **Never remaps `codelaw/*` diagnostics** (config, stale baseline, unknown suppression, rule failure): those can't be hidden.
- Two matching overrides: later wins per key, as for `rules`.

**`js/no-console`**
- Flags calls on the global `console` (`console.log(…)`, `console["log"](…)`), not a shadowed local `console`. Option `allow = ["warn", "error"]`.
- Fix: `suggest` — delete the statement when the call is a whole expression statement; otherwise `manual`.
- In `js/recommended` at `warn`. The README example shows the profiles above.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `CI=true codelaw check` exits 1 on a `console.log`; without `CI`, exit 0 with a warning; `--profile local` under `CI=true` exits 0 | ✅ `crates/codelaw-cli/tests/profiles.rs` `warn_on_a_laptop_fail_in_ci`; by hand on a `codelaw:js/recommended` repo, `console.log` is `warn` (local) and `error` (`CI=true`) |
| 2 · | `codelaw hook` on the same file reports an error without `CI` set | ✅ same test: `hook a.py` → `error`, exit 2; `hook --profile local` → `warn` |
| 3 · | each `remap` example above behaves as commented; `codelaw/stale-baseline` is not remapped | ✅ `profiles.rs` `remap_never_errors_never_warns_or_only_errors`: `scripts/` warns, `gen/` silent, `payments/` errors, an expired entry in `scripts/` warns, `codelaw/stale-baseline` stays `error` |
| 4 · | `config --print --for <f>` names the profile and its source | ✅ `profile = "ci"  # CI=1 (codelaw.toml:4)`; `remap.<sev>` lines name their override |
| 5 · | `cargo test -p codelaw-config --test schema` green with the new keys; `codelaw test packs/js` green with `no-console` cases | ✅ schema 4/4; `codelaw test packs/js` 73 rules · 1255 checks · 0 failed; corpus 73 rules · 2154 checks · 0 failed |
| 6 · | L8 budgets unchanged (`codelaw bench`) | ✅ 2026-10-25: CI's `perf` on `ubuntu-24.04` — run 40210463091 (`eda3607`, W-40 merged) and run 40214600609 (`bb589ee`, W-39–W-43 merged): every budgeted operation within budget (`check_warm_10k` p95 217.0 ms of 331, `check_cold_10k` 9750.2 of 13118, `startup` 1.6 of 3.0, `peak_memory_10k` 48.9 MiB of 72) |

## Blockers

None.

## Record affected

SR-CONFIG new decisions 10 (profiles) and 11 (`remap`), plus §1's section list, decision 3's owners and decision 4's precedence. SR-LINTER decision 5 cites them. SR-AGENT-SURFACES / SR-CLI-MCP: the hook's and MCP's default profile.

## Hazards

- **Don't touch SR-CONFIG §1 before the schema:** `crates/codelaw-config/tests/schema.rs` parses §1's TOML and its `[x] # SR-…` list. Record, schema and loader land in one change.
- A pipeline without `CI` set (some Jenkins setups) runs `local`. The README says to pass `--profile ci` there.
- Two caches: warm processes must key on the profile, or `hook` (ci) and LSP (local) serve each other's verdicts.

## Reference

[compare doc](../compare/ci-profiles-and-dated-exceptions.compare.md) · [ESLint `no-console`](https://eslint.org/docs/latest/rules/no-console)

---
type: Handoff
name: W-26
description: "Arpit, 2026-10-22: the repo review — the README, CHANGELOG and pack docs make claims the binary and the packs no longer match."
item: W-26
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-24
updated_by: "Claude (Opus 5.5), W-25/W-26 close-out 2026-10-24"
ball: agent
---

# W-26 — Generate the public docs' numbers and tables from the binary and the packs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A script (`scripts/docs-sync.py --check/--write`) that counts rules and fix classes per pack from the rule files and rewrites the README's and the pack READMEs' figures between markers | the counts were hand-fixed to 419 on 2026-10-22 and will drift again; the registry trigger becomes a CI check | ✅ 2026-10-24 |
| 2 | The README's command table from `codelaw --help` — one row per subcommand, `graph`, `config`, `match`, `hook`, `sync-agents`, `baseline`, `deprecate`, `daemon`, `cache stats/clean` included; the npm/Python API section names `format` and `rulesFor`/`rules_for` | a consumer sees every command that ships | ✅ 2026-10-24 |
| 3 | Move the roadmap rows that landed to ✅: agent hooks and `init --agents` (W-10), project-wide rules and baselines (W-11, W-13), SARIF (W-16) | the roadmap is the truth of what is built | ✅ 2026-10-24 |
| 4 | `CHANGELOG.md` *Unreleased*: add what landed and is not there — W-10 (`hook`, `sync-agents`, `init --agents`), W-12 (`target:` rules), W-14 (adapters), W-15 (`daemon`, warm sockets), W-16 (`config`, `match`, SARIF, `--timings`), W-19 (`bench`), W-23 (`.editorconfig`, `--import-config`, ignore markers) and the rules added since 2026-10-20 | a release note that matches the release | ✅ 2026-10-24 |
| 5 | Say how a consumer turns on the three `graph/*` rules that are in no preset — a `*/project` preset per pack, or a README row per pack | three shipped rules stop being unreachable | ✅ 2026-10-24 |

**Arpit, 2026-10-22:** *"review the repo, create work items to fix whatever was broken"*

**Model:** Sonnet: counting and rewriting between markers; the one design choice (preset or README row for `graph/*`) is small and named in SR-RULE-PACKS decision 8.

## Goal

Every number, command and roadmap row in `README.md`, `CHANGELOG.md` and `packaging/README.md` is produced from, or checked against, the tree — and `ci.yml` fails when they drift.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · counts | `scripts/docs-sync.py --check` passes in `ci.yml`'s `lint` job and fails when a rule file is added without re-running `--write` | ✅ 2026-10-24 |
| 2 · commands | every subcommand in `codelaw --help` has a README row; the check compares them | ✅ 2026-10-24 |
| 3 · roadmap | no roadmap row says "designed" for a record the register marks `built: yes` | ✅ 2026-10-24 |
| 4 · changelog | *Unreleased* names every item `work/IMPLEMENTATION.md` dates after the last release entry | ✅ 2026-10-24 |
| 5 · graph | `graph/no-unused-exports` (js, python) and `graph/no-unused-css-classes` are reachable from a documented preset or a README row | ✅ 2026-10-24 |

## Blockers

None. **Closed 2026-10-24** (W-25/W-26 close-out): every step and DoD holds; `check-work-system.py --self-test` and `docs-sync.py --check` pass.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 8 (step 5) · [SR-WORK-REGISTRY](../../records/0060_WORK-registry.md) §3 (`README.md`'s and `CHANGELOG.md`'s rows: the trigger becomes a check) · [SR-WORK-DOCS](../../records/0055_WORK-docs.md) decision 10 if the check is made a task obligation. Otherwise **no SR affected**.

## Hazards

- **Numbers in prose:** the README states counts in four places (`:11`, `:47`, `:189`, `:389` at `f75c848`) and the fix-class table in a fifth; the markers must cover all of them or the script rewrites one and leaves the rest.
- **The pack READMEs state counts too** — the same script, the same markers.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- The 2026-10-22 review's audit: js 140, python 99, html 72, css 71, data 37 rules; fixes auto 67 / suggest 68 / manual 284 (frontmatter count over `packs/*/rules/**/*.md`).

---
type: Handoff
name: W-98
description: "Arpit, 2026-10-27 (W-36 gap G7): an llms.txt and a machine-readable rule catalogue generated from docs/rules/."
item: W-98
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5), W-98 closed"
ball: done
---

# W-98 — `llms.txt` and a rule catalogue agents can fetch

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `scripts/rule-docs.py` also writes `llms.txt` (index) and `llms-full.txt` (every rule page) and a `rules.json` catalogue (id, pack, languages, fix kind, options schema, one-line do/don't) | agents and search can read codelaw's rules with no install | ✅ |
| 2 | `--check` keeps them current in CI; linked from the README | they never drift | ✅ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Haiku.

Why: Prettier, Ruff and oxc serve `llms.txt`; none serves a per-rule schema'd catalogue ([proposal](../proposals/linter-for-ai-positioning.md) §3).

## Definition of done

- ✅ The three files are generated, deterministic, checked in CI, and list all rules with their fix kind.

## Closed — 2026-10-27 (Claude Code, Opus 5.5)

- **Where they are:** `llms.txt` at the repo root (the llmstxt.org form, 5.6 KB: the docs, the 17 packs with their npm/PyPI names and presets, the engine rules); `docs/rules/llms-full.txt` (0.9 MB: one configuration preamble, then each of the 651 rules — facts, what it checks, options table, how to fix, examples and its other sections, headings demoted, links absolute — then the 23 engine rules) and `docs/rules/rules.json` (674 entries: `id`, `title`, `packs`, `npm`, `pypi`, `languages`, `fix`, `fix_reason`, `severity`, `stability`, `since`, `scope`, `presets`, `options` as the rule file declares them, `dont`/`do` one-liners, `fix_steps`, `example` {language, dont, do}, `ported_from`, `security`, `page`, `source`). The two big files sit beside the pages so `llms.txt` stays short (the hazard).
- **One-line don't/do:** the first line of the first example's don't (or do) that the other side lacks; null where there is none (59 don'ts, 51 dos — a deletion, a reorder), and `title` already states the rule as what the code does.
- **Generated:** by `scripts/rule-docs.py` in the same pass as `docs/rules/` (`build()`); `--check` covers them, so CI's lint step (renamed to say so) holds them. A second `--write` writes 0 files.
- **Linked:** README *Rule packs*, USAGE *Configure*; SR-WORK-REGISTRY §3 has an `llms.txt` row, and the `docs/rules/` row notes the two files.
- **Not done, not asked:** `lockmaster.toml` has no `regen` list and its `[checks]` do not run `rule-docs.py --check` (CI does); unchanged.

## Record affected

SR-WORK-REGISTRY (the generated files) — no SR on behaviour.

## Hazards

- Size: `llms-full.txt` for 652 rules is large; keep `llms.txt` short.

---
type: Handoff
name: W-140
description: "Arpit, 2026-10-30 (voice): anyone can ask for a rule from ESLint, Pylint or any tool that codelaw lacks, and an agent writes it into their project — plus a way to see how many people want each one."
item: W-140
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-140 — Port any rule on demand, and count who wants it

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A skill, `codelaw-port-rule`: "I want ESLint's no-nested-ternary in codelaw" → checks `ported_from` first; if codelaw has it, says how to turn it on | no duplicate rules | ⏳ |
| 2 | Otherwise it reads the source rule's docs and tests, writes the rule fresh in `.codelaw/rules/`, `ported_from` set, `preview` | the rule in the user's project, the same hour | ⏳ |
| 3 | The source's test cases become codelaw cases; the rule counts only when `codelaw test` passes them all | the test runner decides, not the agent | ⏳ |
| 4 | After a port, the skill asks "share this rule with codelaw?" — yes opens a pre-filled issue tagged with the source rule | a demand signal people choose to send | ⏳ |
| 5 | A maintainer script counts public `.codelaw/rules` files by `ported_from` and the shared issues per rule | which ports to take into the first-party packs | ⏳ |

**Arpit, 2026-10-30 (voice):** *"It is just to copy or migrate a rule that doesn't exist in code law today, but if somebody wants to use it, they can just say that this is a rule that is in ESLint, or this is a rule that is there in PyLint, and any tool, and I want it in code law. So write a rule for it."* → on counting: *"how can I know … how many people use that rule?"* — both ways (the share prompt and the code search): *"Yes."* → *"file it."*

**Model:** Opus (Claude Code): a skill and its subagent in the skills package, plus one maintainer script.

## Goal

**The skill** (`templates/agents/`, so it ships in the skills package — SR-AGENT-SURFACES decision 8):

1. Takes `<tool>/<rule>` (any tool: ESLint and its plugins, typescript-eslint, Ruff, Pylint, flake8 plugins, stylelint, Biome, markdownlint, html-validate, Semgrep, …) or a link to the rule's page.
2. Looks for a first-party rule whose `ported_from:` names it; found → the preset or `[rules]` line that turns it on, and stops.
3. Not found → reads the source's documentation and test cases (read only: never runs them), then follows the `codelaw-author-rule` loop (SR-RULE-AUTHORING decision 2): the source's invalid cases are the Don'ts, its valid cases the Dos, a fix class chosen (L4; the source's own autofix suggests `auto` or `suggest`), `stability: preview`, `ported_from: [<tool>/<rule>]`.
4. Writes `.codelaw/rules/<namespace>/<name>.md` from a template (L10); a project namespace, never a first-party one.
5. Done when `codelaw test <id>` is green on every case; the user reviews and merges.
6. Cannot express it (needs types not yet asked for, or engine code) → says which, and offers the share prompt as a request instead.

**The demand signal** — codelaw sends nothing by itself (L5: no network in the engine; no telemetry):

- **Share prompt:** after a port, *"share this rule with codelaw?"* — yes opens a pre-filled issue on `arpitarya/codelaw` from `.github/ISSUE_TEMPLATE/rule_request.yml` (a `ported-rule` label, the title `port: <tool>/<rule>`, the rule file attached), through `gh` or the browser; no is the end of it.
- **Count:** `scripts/ported-rule-demand.py` — `gh search code` for `ported_from:` in `.codelaw/rules/` across public GitHub, plus the `ported-rule` issues; prints one row per source rule (repos, issues, first seen). Maintainer-run, never in CI, never in the engine.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · skill | `codelaw-port-rule` and a context-isolated subagent, in `templates/agents/`, synced by `sync-agents` | ⏳ |
| 2 · check first | an already-ported rule is found by `ported_from`, also through MCP (`rules_for_path` or a `find_rule` lookup) | ⏳ |
| 3 · port | three end-to-end runs recorded: an ESLint core rule, a Pylint check, a stylelint rule — each green under `codelaw test` | ⏳ |
| 4 · share | `rule_request.yml` gains the ported-rule fields; the skill's prompt opens it pre-filled | ⏳ |
| 5 · count | `scripts/ported-rule-demand.py` with a `--format json` table; documented in RELEASING.md or CONTRIBUTING.md | ⏳ |
| 6 · records, docs | SR-AGENT-SURFACES (the skill), SR-RULE-AUTHORING (the port loop), SR-RULE-FORMAT (`ported_from` for a project rule); USAGE | ⏳ |

## Blockers

None.

## Record affected

[SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md) · [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md) · [SR-RULE-FORMAT](../../records/0112_rule-format.md).

## Hazards

- **Licences (L2):** read behaviour, write fresh. Pylint is GPL-2.0 and other sources may be copyleft: never paste their tests or docs — write new cases from what the rule does. From permissive sources, a case reused verbatim keeps its notice; prefer writing new ones.
- **Never execute** a source rule, its config or its tests (L5) — the agent reads them.
- The share prompt is opt-in, every time; nothing is sent without a yes.
- A ported rule that W-139 later ships first-party: the skill's step 2 then finds it, and the project rule can be deleted — say so in the rule's doc.

## Reference

W-139 (first-party ports) · W-94 (`codelaw migrate`, which maps built-in rules through `ported_from`) · W-133 (the review-comments skill, the same authoring loop) · `.github/ISSUE_TEMPLATE/rule_request.yml`

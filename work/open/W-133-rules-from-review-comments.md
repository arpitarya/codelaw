---
type: Handoff
name: W-133
description: "Arpit asked 2026-10-29 (Cowork, voice) for a skill that reads repeated PR review comments on any platform and turns them into codelaw rules, with his approval at each step."
item: W-133
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-133 — Rules from review comments, on every review platform

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read merged-PR comments through one adapter per platform | GitHub, GitLab, Bitbucket, Azure DevOps alike | ⏳ |
| 2 | Group what reviewers keep repeating | the team's unwritten rules, with evidence | ⏳ |
| 3 | Propose each as a rule or as guidance; the decider approves | nothing turns on by itself | ⏳ |
| 4 | Build approved rules as previews, with a baseline | new code obeys from day one | ⏳ |

**Arpit, 2026-10-29:** platforms — *"All of them."* · *"Repeats three times or more. Reviewers, it could be one person as well. And I want both of those to be configurable. Window, I also want that to be configurable."*

**Model:** Opus (Claude Code): a skill with four adapters and a judgement step.

## Goal

[The proposal](../proposals/deps-flags-markers-reviews.md) §7.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · adapters | `gh`, `glab`, Bitbucket Cloud and Data Center REST (`curl`), `az repos`; detected from the remote, self-hosted via `[reviews] platform, url`; one comment shape | ⏳ |
| 2 · thresholds | `[reviews] min_repeats = 3, min_reviewers = 1, window = "6m"` | ⏳ |
| 3 · the skill | collect → cluster → propose (evidence, rule vs guidance) → author via the rule-authoring flow → `codelaw test` → `preview` + baseline; approval at each step; `codelaw: rule` fast-track | ⏳ |
| 4 · package | in the skills package (whole agent-facing set, W-77); SR-AGENT-SURFACES | ⏳ |

## Blockers

None.

## Record affected

SR-AGENT-SURFACES (the skill) · SR-RULE-AUTHORING (the hand-off) · SR-CONFIG if `[reviews]` lives in `codelaw.toml`.

## Hazards

- **No `codelaw` command may fetch** (L5 decision 3's reopen trigger): the agent runs the platform CLIs; the engine never does.
- If SR-CONFIG refuses unknown sections, `[reviews]` is either added to the schema (nothing in the engine reads it) or read from `.codelaw/reviews.toml` — decide and record.
- Tokens stay in the platform CLIs or the environment; never written to a file.
- `window = "6m"` was the agent's default; the decider named no value.

## Reference

[Proposal](../proposals/deps-flags-markers-reviews.md) §7 · W-77 (skills package)

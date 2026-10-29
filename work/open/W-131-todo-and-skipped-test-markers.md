---
type: Handoff
name: W-131
description: "Arpit asked 2026-10-29 (Cowork, voice) for TODOs with a configurable ticket and date, and skipped tests that need a reason and an expiry."
item: W-131
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-131 — TODOs with tickets and dates; skipped tests with reasons and expiries

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Check TODO markers in every L3 language with comments | every TODO has its ticket, date if asked | ⏳ |
| 2 | Error once a written date has passed | TODOs cannot rot quietly | ⏳ |
| 3 | Forbid focused tests; require a reason and expiry on skipped ones | no test silently off | ⏳ |

**Arpit, 2026-10-29:** *"The to-do needs a ticket and a date … But I want the ticket and the date configurable, whether I want it as mandatory thing or not."* · *"Skipped test need a reason and expiry. I like that as well."*

**Model:** Sonnet (Claude Code): comment and call-pattern rules on parsers that exist.

## Goal

[The proposal](../proposals/deps-flags-markers-reviews.md) §4.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · todo | `[todo] tags, require_ticket, require_date, ticket_pattern, warn_days`; `TAG(ticket, date)` either order; JS, TS, Python, CSS, SCSS, HTML, YAML, TOML, Markdown | ⏳ |
| 2 · focus | `.only`, `fit`, `fdescribe` → error, suggest fix | ⏳ |
| 3 · skip | Jest/Vitest/Mocha skips, `@pytest.mark.skip`/`skipif`/`xfail`, `unittest.skip`; reason from the framework argument or a `SKIP(…)` comment above; `[tests.skip] require_reason, require_expiry, require_ticket` | ⏳ |
| 4 · records, tests, docs | SR-CONFIG `[todo]`, `[tests.skip]`; expiry on SR-BASELINE decision 9's semantics; fixtures per language; L8 | ⏳ |

## Blockers

None.

## Record affected

SR-CONFIG (`[todo]`, `[tests.skip]`) · SR-RULE-PACKS (rules per language pack).

## Hazards

- Dates come from the as-of date, never the clock (L5).
- `require_ticket` defaults `true`, `require_date` `false` (proposal §4) — a repo full of bare TODOs gets them all at once: the baseline exists for that; mention it in the rule doc.

## Reference

[Proposal](../proposals/deps-flags-markers-reviews.md) §4

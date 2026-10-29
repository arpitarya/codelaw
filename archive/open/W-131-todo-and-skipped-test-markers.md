---
type: Handoff
name: W-131
description: "Arpit asked 2026-10-29 (Cowork, voice) for TODOs with a configurable ticket and date, and skipped tests that need a reason and an expiry."
item: W-131
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Sonnet 5.5)"
ball: agent
---

# W-131 — TODOs with tickets and dates; skipped tests with reasons and expiries

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Check TODO markers in every L3 language with comments | every TODO has its ticket, date if asked | ✅ (2026-10-30) |
| 2 | Error once a written date has passed | TODOs cannot rot quietly | ✅ (2026-10-30) |
| 3 | Forbid focused tests; require a reason and expiry on skipped ones | no test silently off | ✅ (2026-10-30) |

**Arpit, 2026-10-29:** *"The to-do needs a ticket and a date … But I want the ticket and the date configurable, whether I want it as mandatory thing or not."* · *"Skipped test need a reason and expiry. I like that as well."*

**Model:** Sonnet (Claude Code): comment and call-pattern rules on parsers that exist.

## Goal

[The proposal](../proposals/deps-flags-markers-reviews.md) §4.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · todo | `[todo] tags, require_ticket, require_date, ticket_pattern, warn_days`; `TAG(ticket, date)` either order; JS, TS, Python, CSS, SCSS, HTML, YAML, TOML, Markdown | ✅ (2026-10-30) |
| 2 · focus | `.only`, `fit`, `fdescribe` → error, suggest fix | ✅ (2026-10-30) |
| 3 · skip | Jest/Vitest/Mocha skips, `@pytest.mark.skip`/`skipif`/`xfail`, `unittest.skip`; reason from the framework argument or a `SKIP(…)` comment above; `[tests.skip] require_reason, require_expiry, require_ticket` | ✅ (2026-10-30) |
| 4 · records, tests, docs | SR-CONFIG `[todo]`, `[tests.skip]`; expiry on SR-BASELINE decision 9's semantics; fixtures per language; L8 | ✅ (2026-10-30) — see the L8 line below |

## Blockers

None.

## Record affected

SR-CONFIG (`[todo]`, `[tests.skip]`) · SR-RULE-PACKS (rules per language pack).

## Hazards

- Dates come from the as-of date, never the clock (L5).
- `require_ticket` defaults `true`, `require_date` `false` (proposal §4) — a repo full of bare TODOs gets them all at once: the baseline exists for that; mention it in the rule doc.

## Reference

[Proposal](../proposals/deps-flags-markers-reviews.md) §4

## Landed 2026-10-30 (Claude Code, Sonnet 5.5)

- **Engine:** `crates/codelaw-native/src/markers.rs` (the eight `*/todo-marker` rules: one `TodoMarker` over the comment syntax of each language) and `test_markers.rs` (`js/no-focused-tests`, `js/skipped-test`, `py/skipped-test`); config `[todo]` and `[tests.skip]` in `codelaw-config` (struct, loader checks, schema, `config --print`).
- **Records:** SR-CONFIG decision 11a (the tables, the syntax, the semantics), SR-RULE-PACKS (the `markers` presets and the rules), `records/README.md` ownership rows.
- **Decisions where the proposal was silent:** each language's rule lives in its own pack, one id per language (`js/todo-marker` covers JS, JSX, TS, TSX; `yaml/`, `toml/`, `md/` in the data pack); opt-in `<ns>/markers` presets, in no `recommended`; Markdown markers are `<!-- -->` comments only (a TODO in prose is prose); a marker's tag must open a comment line; unknown items in the parentheses (an owner) are ignored; a malformed date is its own finding; `[tests.skip]` also takes `ticket_pattern` and `warn_days`; for a skip the ticket and expiry are always from the `SKIP` comment, only the reason may come from the framework; an expiring marker or skip is a warning; no `--as-of` date means only the missing and malformed are reported.
- **Tests:** unit tests in both modules, `crates/codelaw-cli/tests/markers.rs` (dates through the binary), cases sidecars, `codelaw test` on the six packs.
- **L8:** the rules are opt-in and read only comment nodes (and, for SCSS and CSS, one scan of the file), so no budgeted workload runs them; no new budget was measured or loosened. Not run: the corpus (the shared debug binary was another agent's build).

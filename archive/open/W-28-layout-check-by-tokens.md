---
type: Handoff
name: W-28
description: "Arpit, 2026-10-22: the layout-rule check must be codelaw's own definition, not whatever Biome happens to do."
item: W-28
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-24
updated_by: "Claude (Opus 5.5), cloud session 2026-10-24: corpus snapshots, W-28 re-run"
ball: agent
---

# W-28 — Decide "layout" by codelaw's own tokens, not by the formatter's output

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Define a layout rule in codelaw's terms: a rule whose `Don't` and every `Do`/`Fixed` have the **same token stream** through codelaw's own parser — every leaf token's kind and text equal, whitespace and other extras ignored, file by file | the test no longer depends on which engine formats, or on its version | ✅ `codelaw_lang::token_stream` |
| 2 | `codelaw test` runs that comparison instead of `format(Don't) == format(Do)`; the message still names SR-FORMATTER decision 3 | one deterministic answer per rule, the same on every machine (L5 spirit, L8 cheap) | ✅ |
| 3 | Keep the formatter comparison as a **second, advisory** check that prints a warning when the two disagree — that is exactly the case where the formatter changes meaning | a formatter that changes meaning is caught instead of trusted | ✅ |
| 4 | Re-run the five packs; record the result in SR-FORMATTER decision 3 and B-045's note | the two rules of 2026-10-22 are judged by the new definition | ✅ (pre-W-31 layout; re-run on the split packs at merge) |

**Arpit, 2026-10-22:** *"biome is not the right thing to reference. build how we design it. make the decision and create the work item"*

**Model:** Opus: the token comparison touches `codelaw-test` and `codelaw-lang`'s parse API; the judgment is in which extras count as layout for each of the nine grammars (comments do not: a comment is a token with text).

## The decision (this session's, 2026-10-23 — confirmed by the decider 2026-10-24)

**Arpit, 2026-10-24 (docket):** *"Confirm both"* — a rule that only changes quotes, semicolons or trailing commas passes with the advisory warning; CSS `not(` vs `not (` is layout under codelaw's tokens (B-045).

- **Layout is what the parser does not see.** A fix that leaves every token's kind and text unchanged changes only layout; that rule belongs to the formatter and is refused. A fix that changes any token changes meaning or structure and is a lint rule.
- **The formatter is evidence, not the judge.** `format(Don't) == format(Do)` stays as an advisory cross-check: when the tokens differ but the formatted texts are equal, the formatter changed meaning — that is reported, not treated as "layout".
- **Consequences on the two rules of 2026-10-22:** `css/media-query-no-invalid` (`not(` is a function token; `not (` is an identifier and a parenthesis) has different tokens, so it is a lint rule — and the advisory check reports that Biome changes its meaning (B-045). `js/no-irregular-whitespace`: between tokens a no-break space is an extra, so that part is layout and stays the formatter's; inside a template, a regular expression, JSX text or a comment it is token text, so the narrowed rule of 2026-10-22 stands.

## Goal

A rule is accepted or refused as a layout rule by codelaw's own parse of its examples, identically on every machine and under every formatter version.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · definition | SR-FORMATTER decision 3's "How" states the token comparison; SR-RULE-AUTHORING decision 4 names the message | ✅ |
| 2 · test | `a_layout_rule_is_refused` and a new `a_token_change_is_not_a_layout_rule` pass; a rule whose `Don't`/`Do` differ only in whitespace fails with the decision-3 message; one differing only inside a string literal passes | ✅ |
| 3 · advisory | a rule whose tokens differ but whose formatted texts are equal prints a warning naming the formatter and B-045, and does not fail | ✅ 2026-10-24: `RuleReport::warnings` (`crates/codelaw-test/src/lib.rs`), never a failure; the decider confirmed the behaviour |
| 4 · packs | `codelaw test packs/<pack>` passes for all five; the record's "Measured" note is rewritten against the new definition | ✅ 2026-10-24 on the nine W-31 packs: 419 rules, 0 failed, 0 warnings (`css` 71 · `data` 37 · `html` 72 · `js` 72 · `node` 10 · `python` 99 · `react` 21 · `stencil` 16 · `ts` 21); SR-FORMATTER's note says so |

## Progress

- **2026-10-24 (Opus 5.5, worktree `worktree-agent-afc9f1fca5c536088`):** built. `codelaw_lang::token_stream` (new `tokens.rs`): pre-order visible nodes with depth and kind, leaves with exact text; comments are tokens; words no node holds become `text` tokens (Markdown inline); depth carries Python/YAML indentation; embedded regions compared in their own language. `codelaw-test`'s `layout_only` now fails on equal token streams and warns (new `RuleReport::warnings`, rendered, never a failure) when tokens differ but the formatted texts agree. SR-FORMATTER decision 3, SR-RULE-AUTHORING decision 4, SR-LINTER decision 1 (pointer), the glossary and B-045 updated.
- **Findings:** (1) codelaw's CSS grammar parses `@media not(a)` and `not (a)` to the same tree, so the item's expectation that the two have different tokens is wrong for codelaw's parser; `not(` lives in the rule's `.cases.md`, outside the layout check, so nothing fails — pinned by `the_css_grammar_does_not_see_not_paren`. (2) Quotes, semicolons and trailing commas are tokens: a rule that only changes them now passes the refusal and draws the advisory warning instead — the decider may want that to fail. (3) Rule tests that reuse one rule id with different matchers interfere when run in parallel (the engine appears to keep compiled matchers by id for the process; not investigated); the tests now take distinct ids.

## Blockers

None. **Closed 2026-10-24:** DoD 1–4 hold; the decider confirmed both consequences (docket). W-23's archive and the `decider-rulings-2026-10-22` branch's handling of the two rules are the starting state; this item changes the test, not the rules.

## Record affected

[SR-FORMATTER](../../records/0105_formatter.md) decision 3 · [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md) decision 4 · [SR-LINTER](../../records/0106_linter.md) if it defines "extra"/trivia per grammar.

## Hazards

- **Comments:** tree-sitter marks comments as extras; this definition counts them as tokens (a rule that deletes a comment is not layout). Say so in the record.
- **Templates and regions:** compare a multi-file or embedded-region snippet region by region, as the current check does file by file.
- **[L8](../../records/0010_LAW-8-performance-budgets.md):** the token walk is per example at `codelaw test` time, never in `check`.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `crates/codelaw-test/src/lib.rs` `layout_only()` and `tests.rs` `a_layout_rule_is_refused`.

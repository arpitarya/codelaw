---
type: Handoff
name: W-100
description: "Arpit, 2026-10-27 (W-36 gap G8): promote rules from preview to stable by pinned corpus runs."
item: W-100
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Claude Code (Opus 5.5)"
ball: done
---

# W-100 — Promote rules out of `preview` by corpus

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The promotion criterion in a record: a rule is `stable` after N pinned corpus repos with zero false positives against its source tool's verdict, and its fix verified | "every rule is preview" stops being an objection | ✅ ratified 2026-10-27 as proposed — [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md) decision 6a; `scripts/corpus.json` `promotion.ratified` |
| 2 | The corpus CI job reports per-rule agreement; a script proposes promotions; the change lands as a diff to `stability:` | promotion is evidence, not opinion | ✅ `scripts/corpus.py promote` (report; `--write --run <url>` flips `stability:` and records the run, refused while unratified); CI's `corpus` job writes the report to its summary. Per-rule agreement *with the source tool* is not built: that tool in CI is an L11 ruling (W-99's) |
| 3 | First batch promoted; README FAQ updated | users see stable rules | ✅ 2026-10-28: 42 `css` rules `stable`, on [CI run 40622778075](https://github.com/arpitarya/codelaw/actions/runs/40622778075) (green on main at `644a37d`, its `corpus` job included); README FAQ's first sentence, roadmap row, USAGE and CHANGELOG updated |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** *"Ratify."* — decision 6a as proposed: ≥ 3 corpus repos and ≥ 10 files; no false positive, zero findings counting; the fix held level 4; ≥ 1 release in preview. Put with it: 326 of 620 preview rules pass at these values, all through zero findings, and every rule already carries its own tests.

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: a platform team's first objection is "every rule is `preview`" (README FAQ) ([proposal](../proposals/linter-for-ai-positioning.md) §6).

## Definition of done

- ✅ At least one pack's first batch is `stable` with the corpus run that proved it named in each rule's history. — 42 `css` rules carry `stability: stable`; each is in `scripts/corpus.json` `promotion.promoted` with `{"on": "2026-10-28", "run": "https://github.com/arpitarya/codelaw/actions/runs/40622778075"}`; their pages under `docs/rules/css/` say `stable`.

## Record affected

SR-RULE-FORMAT (`stability`) · SR-RULE-PACKS.

## Hazards

- Comparing with the source tool needs it installed in CI — L11, as W-99.

## Progress — 2026-10-27 (Claude Code, Opus 5.5)

**Built.** `scripts/corpus.py promote [--dir D] [--pack P] [--rules] [--write --run <CI run URL>]` reads each preview rule's front matter, its count in `packs/<pack>/corpus.toml`, the fetched repos, and `scripts/corpus.json`'s new `promotion` block (`criterion`, `ratified`, `reviewed`, `promoted`), and reports per pack which rules pass and why each other is held. `--write` refuses while `promotion.ratified` is null. `--self-test` checks the block and the verdict (nine cases). CI's `corpus` job runs `promote` after the counts hold and writes it to the job summary. No network, nothing downloaded: run on the cached `target/corpus`.

**Why no rule is promoted.** Decision 6 says "a corpus run with no unexpressible false positives" and names no number and no evidence; the item's own step 1 leaves N open. The criterion is now proposed in SR-RULE-AUTHORING decision 6a — **min_repos 3, min_files 10, zero findings counts as no false positive, one published release in preview** — and waits on the decider.

**At those values, on the cached corpus (2026-10-27):** 326 of the 620 preview rules in the 14 corpus packs pass, every one through a zero-finding count; 190 rules with findings need a review first; stencil (one corpus repo: ionic-framework), security-js and security-ts (since 0.3.0, unreleased) pass nothing. With `zero_findings_is_evidence = false`, none passes until reviews land.

## The ask (in OPEN-WORK's inbox)

Rule on decision 6a's four values: say **"ratify 6a as proposed"**, or give the numbers (e.g. "min_repos 2", "zero findings is not evidence"). Then an agent writes `promotion.ratified`, runs `promote --write --run <the green corpus run's URL>` for one pack (css: 42 pass today), and the DoD is met.

## Closed — 2026-10-28 (Claude Code, Opus 5.5)

**Dry run first** (`corpus.py --dir target/corpus promote --pack css --rules`, criterion as ratified): **42 of 72 css preview rules pass**, every one through a zero-finding count on the six css corpus repos (httpx, starlette, html5-boilerplate, normalize.css, bootstrap, djangoproject.com). **30 are held for one reason: they have findings and `promotion.reviewed` has no verdict on them** (e.g. `css/no-important` 17 708, `graph/no-unused-css-classes` 16 001, `css/color-function-alias-notation` 1 454, `css/alpha-value-notation` 366, down to single findings in `css/at-rule-no-vendor-prefix`, `css/declaration-property-value-no-unknown`, `css/selector-no-vendor-prefix`, `css/declaration-block-no-duplicate-properties`). None was held on repos, files or releases.

**Then** `promote --pack css --write --run https://github.com/arpitarya/codelaw/actions/runs/40622778075` and `rule-docs.py --write` (43 pages: the 42 rule pages and `rules.json`), re-run after merging main (W-62 was changing css rules in parallel). SR-RULE-AUTHORING's Consequences updated (first batch promoted).

## Left (not this item's DoD; follow-ups)

1. Done: the ruling (`promotion.ratified`) and the first batch (css, 42 rules).
2. The next packs: `promote --pack <p> --write --run <green run URL>`, pack by pack.
3. Reviews for rules with findings (`promotion.reviewed`), pack by pack — only if 6a keeps the human review.
4. Source-tool agreement in CI — after L11 ratifies the source tools (W-99's ruling); it then replaces the review.
5. Whether "history" should also be in the rule file (a front-matter field is an SR-RULE-FORMAT change); today it is `promotion.promoted` in `scripts/corpus.json`.

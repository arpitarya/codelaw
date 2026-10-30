---
type: Handoff
name: W-149
description: "Found on review 2026-10-30 (Cowork): main's CI has been red since the 10-30 batch landed — the corpus job fails in exactly the six packs that gained W-131's marker rules (css, data, scss, html, python, js)."
item: W-149
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-149 — CI red on `main`: six packs' corpus snapshots

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Run `scripts/corpus.py run --pack <p> --update` for css, data, scss, html, python, js; read each diff | the moved counts, named | ⏳ |
| 2 | If the moves are the new marker rules' findings (`*/todo-marker`, `*/flag-marker`, `*/skipped-test` …), commit the snapshots; if any other count moved or an invariant broke, fix the rule instead | `corpus <pack>` green | ⏳ |
| 3 | One clean `ci` run on `main`, its id here; W-126 then has its Rust- and pack-change runs to measure | `main` green | ⏳ |

**Asked 2026-10-30 (Arpit, text):** *"Review the open work file. Check the status on each and every work item. Update all of them to their latest status."* — found while checking W-126's runs.

**Model:** Sonnet (Claude Code): snapshot updates read against the rules that caused them.

## What the runs show (GitHub Actions API, 2026-10-30 10:30 IST)

| run | commit | red jobs |
|---|---|---|
| 40977728005 (nightly) | `7abbe20` | none — the last green run |
| 40996512721 | `d6869dc` (batch wrap-up) | every build's pack tests (`deps` example) — fixed by `3f9bf44` |
| 41002263638 | `6b0b948` | lint (the guard replay) — fixed by `c89f820`; `corpus` css, data, scss, html, python, js |
| 41002310815 | `c89f820` | `corpus` css, data, scss, html, python, js |

- **Why these six (a reading, to confirm in step 1):** they are exactly the packs holding W-131's/W-130's marker rules (js 8, python 4, data 3, css, scss, html 1 each); `react` and `ts` hold none and pass. `d7b7b612` added two of them to the scss *fixture* snapshots, not the corpus ones; `packs/css/corpus.toml` last changed in W-62.
- The annotation says: *"an invariant broke on a real file, or a finding count moved (rerun scripts/corpus.py run --pack js --update and review the diff)"*.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · green | `ci` green on `main`, run id recorded | ⏳ |
| 2 · counted | each new rule's corpus count in its pack's `corpus.toml`, any surprise (a count in the hundreds) named here | ⏳ |

## Blockers

None.

## Record affected

None — snapshots only (SR-RULE-AUTHORING's corpus check), unless a rule needs a fix.

## Hazards

- Never `--update` a count without reading why it moved: that is how a false positive gets baked in.
- Landing more on a red `main` hides the next break; this goes first.

## Reference

W-131, W-130 (`archive/open/`) · [W-126](W-126-fast-pipeline.md) · `scripts/corpus.py`

---
type: Handoff
name: W-89
description: "Arpit, 2026-10-27 (voice docket): PyPI day 4 of 4 — at most four new projects a day, one item per day, the command ready to paste."
item: W-89
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-89 — PyPI day 4 (Sat 2026-10-31): the three security rule packs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `twine upload` the three security packs at 0.2.0 — the version npm got (W-85, 2026-10-27) | `codelaw-rules-security-{js,html,python}` exist | 🔴 Arpit |
| 2 | A trusted publisher on each | CI can upload them | 🔴 Claude in Chrome, once Arpit is logged in to PyPI (ruled 2026-10-27) |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** *"Yes."* — on running days 1–4 as filed, with Claude adding the trusted publishers through Chrome.

**Arpit, 2026-10-27 (voice docket):** *"create work items, one work item to publish all NPM packages and then multiple work items, four packages a day but make sure all the packages are accounted in this."* Earlier: *"create a work item just for publishing four packages a day for PyPI … add the command in the OpenWork file so that I can just copy it and paste it every day … don't count today's day"*, and *"Include these SCSS and security packs as well."*

**Version: 0.2.0**, the one npm got on 2026-10-27 (W-85). These are not in the v0.2.0 tag; take the wheels from the same local build npm was published from (`…/scratchpad/pub/dist/pypi/`), copied into `dist/pypi/` before the day — that scratch folder does not survive a reboot.

## The command (repo root, on the Mac, not before Sat 2026-10-31)

```bash
# the 0.2.0 wheels of the build npm was published from (2026-10-27), copied into dist/pypi/ first
twine upload --skip-existing dist/pypi/codelaw_rules_security_{js,html,python}-0.2.0-py3-none-any.whl
```

## Trusted publisher, per project

`https://pypi.org/manage/project/<name>/settings/publishing/` → GitHub: owner `arpitarya` · repository `codelaw` · workflow `publish.yml` · environment `pypi`. Or ask Claude (Cowork, with Chrome) to add them — done that way for six projects on 2026-10-27.

## The whole set (every name accounted for)

| day / item | PyPI projects created | version |
|---|---|---|
| live before 2026-10-27 | `codelaw-rules-css`, `-data`; `codelaw-rules-{css,data,html,js}-fux-index` | 0.2.0 |
| W-86 · Wed 10-28 | `codelaw-rules-{html,js,python,react}` | 0.2.0 (v0.2.0 tag) |
| W-87 · Thu 10-29 | `codelaw-rules-{react,scss,python,security-js}-fux-index` | 0.2.0 |
| W-88 · Fri 10-30 | `codelaw-rules-security-{html,python}-fux-index`; `codelaw-skills`, `codelaw-rules-scss` | 0.2.0; W-85's version |
| W-89 · Sat 10-31 | `codelaw-rules-security-{js,html,python}` | W-85's version |
| CI, through its pending publisher | `codelaw` | at the Release (W-86 step 3) |

npm: the nine still missing are W-85; the 15 Fux index packages are live and get their trust in W-85 too. Nothing else in `package.py names` or `fux-index.py names` is unpublished.

## Definition of done

- Each project shows its version on pypi.org; each *Publishing* page lists `arpitarya/codelaw` · `publish.yml` · `pypi`.

## Record affected

No SR affected.

## Hazards

- PyPI answers **429** past four new projects in a day — wait a day; `--skip-existing` makes the retry safe.
- A project with no trusted publisher refuses CI's token before "already exists" is checked, so the publisher goes on the same day as the upload.

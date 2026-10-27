---
type: Handoff
name: W-87
description: "Arpit, 2026-10-27 (voice docket): PyPI day 2 of 4 — at most four new projects a day, one item per day, the command ready to paste."
item: W-87
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-87 — PyPI day 2 (Thu 2026-10-29): four Fux index projects

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `twine upload` four 0.2.0 Fux index wheels already in `dist/fux-index/pypi/` | `codelaw-rules-{react,scss,python,security-js}-fux-index` exist | 🔴 Arpit |
| 2 | A trusted publisher on each | CI can upload them once W-34 teaches `publish.yml` | 🔴 Arpit, or Claude in Chrome |

**Arpit, 2026-10-27 (voice docket):** *"create work items, one work item to publish all NPM packages and then multiple work items, four packages a day but make sure all the packages are accounted in this."* Earlier: *"create a work item just for publishing four packages a day for PyPI … add the command in the OpenWork file so that I can just copy it and paste it every day … don't count today's day"*, and *"Include these SCSS and security packs as well."*


## The command (repo root, on the Mac, not before Thu 2026-10-29)

```bash
twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_{react,scss,python,security_js}_fux_index-0.2.0-py3-none-any.whl
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

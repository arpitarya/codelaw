---
type: Handoff
name: W-86
description: "Arpit, 2026-10-27 (voice docket): PyPI day 1 of 4 — at most four new projects a day, one item per day, the command ready to paste."
item: W-86
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-86 — PyPI day 1 (Wed 2026-10-28): the four W-38 rule packs, then the v0.2.0 Release

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `twine upload` the four 0.2.0 wheels already in `dist/pypi/` (built from the v0.2.0 tag, byte-identical to CI's) | `codelaw-rules-{html,js,python,react}` exist | 🔴 Arpit |
| 2 | A trusted publisher on each of the four | CI can upload them | 🔴 Arpit, or Claude in Chrome |
| 3 | Publish the GitHub Release `v0.2.0` on W-37's tag (`08b7158`, CI green there) | CI creates `codelaw` on PyPI through its pending publisher, skips everything already uploaded, attaches the checksums | 🔴 Arpit |

**Arpit, 2026-10-27 (voice docket):** *"create work items, one work item to publish all NPM packages and then multiple work items, four packages a day but make sure all the packages are accounted in this."* Earlier: *"create a work item just for publishing four packages a day for PyPI … add the command in the OpenWork file so that I can just copy it and paste it every day … don't count today's day"*, and *"Include these SCSS and security packs as well."*

**The Release needs only this day.** v0.2.0's `dist` holds `codelaw` and six PyPI packs (`css`, `data`, `html`, `js`, `python`, `react`) — after step 2 all six have publishers, and `codelaw`'s pending publisher (registered 2026-10-25, still listed 2026-10-27) creates it. Days 2–4 are not in v0.2.0's `dist`. The tag's commit `08b7158` had a green `ci.yml` (run 40194930874), so the red `main` (W-84) does not stop this Release. The `dist` artifact lives 30 days from 2026-10-25.

## The command (repo root, on the Mac, not before Wed 2026-10-28)

```bash
twine upload --skip-existing dist/pypi/codelaw_rules_{html,js,python,react}-0.2.0-py3-none-any.whl
# after the four trusted publishers:
awk '/^## 0\.2\.0/{f=1;next} /^## /{f=0} f' CHANGELOG.md > /tmp/v0.2.0-notes.md
gh release create v0.2.0 -R arpitarya/codelaw --verify-tag --title "v0.2.0 — the rule packs, one per ecosystem" --notes-file /tmp/v0.2.0-notes.md
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
- After step 3, W-38 (an agent) checks `pip install codelaw`, `uvx codelaw` and the eight-platform install from the registry, then writes it down.

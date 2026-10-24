---
type: Handoff
name: W-30
description: "Arpit, 2026-10-24: the first publish of the 7 PyPI projects (14 wheels) at 0.2.0, in W-31's layout, by hand from his machine, from CI's dist artifact."
item: W-30
filed: 2026-10-24
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-24
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: arpit
---

# W-30 — Publish the 7 PyPI projects for the first time, at 0.2.0, from Arpit's machine

## At a glance

> **Re-scoped 2026-10-24 by W-31:** PyPI carries `codelaw` and six packs — `js`, `react`, `html`, `css`, `python`, `data` — and skips 0.1.0: its first release is 0.2.0, the first in the nine-pack layout, from that release's `dist`. Every `13`, `six` and `0.1.0` below reads as `14`, `seven` and `0.2.0`.

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The names are free: `codelaw` and `codelaw-rules-{js,react,html,css,python,data}` | nobody else owns them | 🟡 all seven are a 404 on PyPI's JSON API (2026-10-24); a 404 is not a guarantee — the first upload is the proof |
| 2 | The 0.2.0 `dist` (W-31 step 7: bumped, pushed, CI green) | the wheels hold the same binary as npm's 0.2.0 (L6 decision 2) | 🟡 waits on W-31 |
| 3 | `twine check`, then upload all 13 wheels with Arpit's account token | the six projects exist, owned by Arpit | ⏳ Arpit runs it |
| 4 | Add a trusted publisher to each of the six (now existing) projects: `arpitarya` / `codelaw` / `publish.yml` / environment `pypi` | every later release comes from CI, no token | ⏳ Arpit, on pypi.org |
| 5 | `RELEASING.md` §One time step 4: "pending publishers" becomes "trusted publishers on the existing projects", and the first PyPI publish is by hand like npm's | the release doc matches what was done | ⏳ agent |

**Arpit, 2026-10-24:** *"Create another work item to publish Python packages. Give me the command … so I can do the first publish from local. I have already configured token for my whole system."*

**Model:** NONE for steps 1–4 (Arpit); Sonnet for step 5 (one doc and one record sentence).

## The commands (from the repo root on the Mac, after W-29's step 3)

```bash
# the same dist as W-29 — do not download a second one
ls dist/pypi/                                   # 8 codelaw-*.whl + 5 codelaw_rules_*.whl

# 3 · check, then upload with the token in ~/.pypirc ([pypi] username = __token__)
python3 -m pip install --user twine             # if not installed (Apache-2.0)
python3 -m twine check dist/pypi/*.whl
python3 -m twine upload dist/pypi/*.whl

#   or with uv (reads the token from the environment, not ~/.pypirc):
#   UV_PUBLISH_TOKEN=pypi-… uv publish dist/pypi/*.whl

# check: every project is live at 0.1.0
for p in codelaw codelaw-rules-js codelaw-rules-react codelaw-rules-html codelaw-rules-css codelaw-rules-python codelaw-rules-data; do
  curl -s "https://pypi.org/pypi/$p/json" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d['info']['name'],d['info']['version'],len(d['urls']),'files')"
done
```

## Goal

The six PyPI projects exist at the release version with all 13 wheels, uploaded from the same checked `dist` as npm, and each trusts `publish.yml` so no later release needs a token.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · live | the six projects show the version; `codelaw` carries 8 wheels, each pack 1 | ⏳ |
| 2 · works | in a fresh venv: `pip install codelaw && codelaw --version`; `uvx codelaw --version` | ⏳ |
| 3 · trusted | each project's *Publishing* page lists `arpitarya/codelaw` · `publish.yml` · `pypi` | ⏳ |
| 4 · recorded | `RELEASING.md` step 4 rewritten; SR-PACKAGES §Consequences says the first PyPI publish was by hand too; `IMPLEMENTATION.md` row | ⏳ |

## Blockers

W-31 step 7: the 0.2.0 bump and a green CI run on it (W-24).

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) §Consequences — one sentence: the first PyPI publish was by hand, from CI's `dist` (it now says only npm's is). `RELEASING.md` §One time step 4.

## Hazards

- **A file is permanent on PyPI.** A filename can never be uploaded twice, even after deleting the release; a mistake means a new version.
- **Upload all 13 in one command.** A partial upload leaves platforms without a wheel; `twine upload --skip-existing dist/pypi/*.whl` finishes an interrupted one.
- **The token:** an account-wide token is needed to *create* projects; after step 4, revoke it or replace it with project-scoped ones — publishing no longer needs it.
- **twine is a build tool under [L2](../../records/0004_LAW-2-foss-dependencies.md)** only if a repo script installs it; typed by hand it is not. If `RELEASING.md` names it, declare it in `tests/licences.rs` like `uv`.
- **One release, both registries:** the same `dist` and version as W-29, the same day.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- [`RELEASING.md`](../../RELEASING.md) §One time; W-29 (the shared `dist`).

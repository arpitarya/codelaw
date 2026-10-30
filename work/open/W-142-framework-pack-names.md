---
type: Handoff
name: W-142
description: "Arpit, 2026-10-30 (voice): the five Python framework packs W-139 builds (Django, FastAPI, Airflow, NumPy, pandas) need their PyPI names bound before release."
item: W-142
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-142 — PyPI names for the five framework packs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-139 has the five packs ready to release | names bound when they are needed | 🟡 waits on W-139 |
| 2 | Claude, in Chrome: pypi.org → *Publishing → Add a pending publisher*, once per name; Arpit checks and submits | `publish.yml` can create each project | 🔴 Arpit submits |

**Arpit, 2026-10-30 (voice):** *"Give each framework its own pack."* · on the names: *"with anything that you recommend."*

**Model:** Cowork with Claude in Chrome; Arpit submits.

## The names (all free on PyPI, checked 2026-10-30 08:30)

| pack | PyPI | its Fux index (L6 decision 12) |
|---|---|---|
| Django | `codelaw-rules-django` | `codelaw-rules-django-fux-index` |
| FastAPI | `codelaw-rules-fastapi` | `codelaw-rules-fastapi-fux-index` |
| Airflow | `codelaw-rules-airflow` | `codelaw-rules-airflow-fux-index` |
| NumPy | `codelaw-rules-numpy` | `codelaw-rules-numpy-fux-index` |
| pandas | `codelaw-rules-pandas` | `codelaw-rules-pandas-fux-index` |

Form values for each: owner `arpitarya` · repository `codelaw` · workflow `publish.yml` · environment `pypi`.

**PyPI only:** L6 ships `python` and `security-python` on PyPI alone (*"each registry carries only the packs its users install"*); these five follow them. Said by voice as also on npm (`@codelaw/rules-django` …) — not filed; say so if npm is wanted, and L6's table changes with it.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | ten pending publishers saved | ⏳ |
| 2 | the release that ships the packs creates the five projects | ⏳ |

## Blockers

🟡 W-139.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) · L6 decision 11 (W-139 writes the amendment) · RELEASING.md.

## Hazards

- A pending publisher reserves nothing; bind close to the release.
- [W-150](W-150-publish-dev-server-plugins.md) (was B-063): Fux-index packages are not uploaded by `publish.yml` yet.

## Reference

[W-139](W-139-rule-parity-build.md) · [W-134](W-134-deps-pypi-pending-publishers.md) (the same steps for `deps`)

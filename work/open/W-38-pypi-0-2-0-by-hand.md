---
type: Handoff
name: W-38
description: "Arpit, 2026-10-25: the whole PyPI side of 0.2.0 in one item, blocked on Arpit while he uploads the rule packs by hand (PyPI lets him create four new projects a day), then the trusted publishers and the GitHub Release that brings `codelaw` itself from CI. Absorbs W-30, W-31's step 7 (PyPI), W-04's step 5 and W-03's step 6."
item: W-38
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-38 — PyPI 0.2.0, by hand, over two or three days

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Upload the six rule-pack wheels by hand from the 0.2.0 `dist` — PyPI creates at most four new projects a day | the six projects exist, owned by Arpit | 🔴 2 of 6: `codelaw-rules-css` and `-data` are live; `html`, `js`, `python`, `react` left |
| 2 | Add a trusted publisher to each of the six projects | later releases come from CI, with no token | 🔴 Arpit, on pypi.org |
| 3 | Publish the GitHub Release `v0.2.0` — on the tag W-37 created | CI creates `codelaw` on PyPI through its pending publisher, skips the six packs and everything npm already has, and attaches the checksums | 🔴 Arpit: `gh release create v0.2.0` |
| 4 | Check it as a user: a fresh venv, `pip install codelaw && codelaw --version`, `uvx codelaw --version`, and the install smoke from the registry on the eight platforms | PyPI works for real | ⏳ an agent, after step 3 |
| 5 | Write it down: an `IMPLEMENTATION.md` row, `RELEASING.md` §One time step 4 marked done, the README's preview note | the docs say PyPI is live | ⏳ an agent, after step 4 |

**Arpit, 2026-10-25:** *"All codelaw npm packages are published. The PyPI
packages will take 2 to 3 days because there is a limit of 4 packages per day to
be published by hand. … For PyPI packages to be published, create just one work
item, merge everything into it, and keep it blocked until and while I'm trying
to publish the PyPI packages."*

**Model:** NONE for steps 1–3 (Arpit's hand on pypi.org and `gh`); Sonnet for
steps 4–5.

## What this item absorbed (each archived the same day)

| from | what came here |
|---|---|
| [W-30](../../archive/open/W-30-first-pypi-publish.md) | everything: the hand uploads, the trusted publishers, its definition of done |
| [W-31](../../archive/open/W-31-registry-pack-split.md) | step 7's PyPI half and the `gh release create v0.2.0` (its npm half is W-37) |
| [W-04](../../archive/open/W-04-release-pipeline.md) | step 5, the first release on both registries |
| [W-03](../../archive/open/W-03-rule-packs.md) | step 6, publishing the packs (its *later* presets are B-048) |
| the inbox row filed 2026-10-20 | the ruling of 2026-10-25: hand-upload the six packs, then bind |

## The commands (from the repo root on the Mac)

```bash
# 1 · the four packs still to create — four new projects a day at most
#     (the 0.2.0 wheels, built by `build-packs.py --release`, byte-identical to CI's)
twine upload --skip-existing dist/pypi/codelaw_rules_{html,js,python,react}-0.2.0-py3-none-any.whl

# 2 · each of the six projects: Settings → Publishing → add a trusted publisher
#     owner arpitarya · repository codelaw · workflow publish.yml · environment pypi

# 3 · after W-37 has made the v0.2.0 tag and npm is live
gh release create v0.2.0 -R arpitarya/codelaw --title "v0.2.0 — nine-pack layout, PyPI" --notes-file <notes>
```

- **Step 3 must wait for step 2 on all six.** The release's PyPI job uploads with
  `skip-existing`, but a project with no trusted publisher refuses the token
  before "already exists" is ever checked.
- **`codelaw` itself is never uploaded by hand.** Its pending publisher
  (registered 2026-10-25) is used up by the first CI upload, which creates it.

## Definition of done

- The seven PyPI projects show 0.2.0: `codelaw` with its eight platform wheels,
  each pack with one wheel.
- Each project's *Publishing* page lists `arpitarya/codelaw` · `publish.yml` · `pypi`.
- Step 4's checks pass; step 5's rows are written.

## Hazards

- **PyPI's new-project limit answers 429** — wait a day; `--skip-existing` makes
  the retry safe.
- **The `dist` artifact lives 30 days** from its CI run; past that, re-run that
  CI run before step 3 (RELEASING.md §Every release step 5).

**Records affected:** none changed by this item; RELEASING.md §One time step 4
is marked done at step 5.

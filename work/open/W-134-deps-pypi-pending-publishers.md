---
type: Handoff
name: W-134
description: "Arpit, 2026-10-30 (voice): the deps pack's two PyPI names as pending trusted publishers, as their own item, done just before the deps release."
item: W-134
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-134 — PyPI: pending trusted publishers for the `deps` pack

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-128 is ready to release | the names are bound when they are needed | ✅ W-128 built and closed 2026-10-30; the next release (after 0.4.0) carries `deps` |
| 2 | Claude, in Chrome: pypi.org → *Your account → Publishing → Add a pending publisher*, filled twice; Arpit checks and submits | `publish.yml` can create both projects | 🔄 `codelaw-rules-deps` saved 2026-10-30 10:20 IST (Claude in Chrome, on Arpit's ask); the Fux index refused — *"A pending trusted publisher matching this configuration has already been registered for a different [project]"* — until the first project exists |

**Arpit, 2026-10-30 (voice):** *"just before the deps release."* · *"I'm just talking about PyPI publishing and NPM publishing. Have those as two separate work items."*

**Model:** Cowork with Claude in Chrome — a browser form on Arpit's signed-in account; he submits.

## The form, twice

| field | value |
|---|---|
| PyPI project name | `codelaw-rules-deps`, then `codelaw-rules-deps-fux-index` |
| Owner | `arpitarya` |
| Repository | `codelaw` |
| Workflow | `publish.yml` |
| Environment | `pypi` |

## Found 2026-10-30 (Cowork) — a pending publisher alone does not publish

- **`publish.yml` skips a PyPI project that does not exist yet**, except `codelaw` (its `exists` step; RELEASING.md: *"its first upload is by hand"*). So the pending publisher only works if `publish.yml` also uploads `codelaw-rules-deps` (one line beside the `codelaw` exception), or its first version goes up by hand: `gh run download <ci-run-id> -R arpitarya/codelaw -n dist -D dist && twine upload dist/pypi/codelaw_rules_deps-<v>-py3-none-any.whl`.
- **Not before the next release:** the `deps` rules are native and post-date `v0.4.0` (`7b855499` is not in it), and `main` still says 0.4.0 — a wheel published now would claim a version whose engine lacks its rules.
- **All 24 existing codelaw PyPI projects already have an active trusted publisher** (checked on *Publishing*, 2026-10-30).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | Both pending publishers saved on PyPI | ⏳ |
| 2 | The first release with the `deps` pack creates both projects | ⏳ |

## Blockers

None — W-128 closed 2026-10-30. Do it just before the next release.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · RELEASING.md.

## Hazards

- A pending publisher reserves nothing: someone else can take the name first — do it close to the release, as ruled, but not after it.
- B-063: the Fux-index packages are not uploaded by `publish.yml` yet; the second name may stay pending until B-063 lands.

## Reference

[W-128](W-128-deps-pack-and-vulnerabilities.md) · [proposal](../proposals/deps-flags-markers-reviews.md) §2.1 · [W-135](W-135-deps-npm-first-publish.md)

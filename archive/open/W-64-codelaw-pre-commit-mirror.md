---
type: Handoff
name: W-64
description: "Arpit, 2026-10-26 (docket on W-50): a codelaw-pre-commit mirror repo so the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does."
item: W-64
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-64 — `codelaw-pre-commit`: the pre-commit framework installs codelaw by version

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Create the repo `codelaw-pre-commit` with a `.pre-commit-hooks.yaml`: hooks `codelaw-check`, `codelaw-fix`, `codelaw-format`, `language: python`, `additional_dependencies: ["codelaw==<version>"]` | `rev: v0.2.0` in a consumer's config installs the PyPI wheel — no Rust build, nothing on `PATH` | ⏳ |
| 2 | Tag it per codelaw release; `publish.yml` (or a scheduled job) opens the bump | the mirror follows every release | ⏳ |
| 3 | USAGE and W-50's recipes point to it; the `language: system` hooks stay for teams with codelaw already installed | both ways documented | ⏳ |

**Arpit, 2026-10-26 (docket on W-50):** *"PATH now, mirror repo later"*.

**Model:** Sonnet — a small repo after Ruff's pattern, with a test install.

## Goal

A consumer's `.pre-commit-config.yaml`:
```yaml
- repo: https://github.com/arpitarya/codelaw-pre-commit
  rev: v0.2.0
  hooks: [{ id: codelaw-check }]
```

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `pre-commit run --all-files` in a fresh repo installs codelaw (0.4.0 today) from PyPI and runs it | ⏳ |
| 2 · | a codelaw release produces a matching mirror tag | ⏳ |

## Blockers

W-38 is done: `codelaw` is on PyPI (checked 2026-10-29).

## Record affected

SR-CLI-MCP (`init --git-hooks` docs) · L6 is unaffected: the mirror publishes no package, it pins one.

## Hazards

- The mirror's version must equal codelaw's — a bump that lags gives users an old engine.

## Reference

[ruff-pre-commit](https://github.com/astral-sh/ruff-pre-commit) · [pre-commit: creating new hooks](https://pre-commit.com/#creating-new-hooks) · [W-50](W-50-git-hooks.md)

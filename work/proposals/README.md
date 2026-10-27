---
type: Index
description: "Index of parked, undecided ideas, each with a graduation trigger."
---

# `work/proposals/` — parked ideas

Ideas worth keeping that are **not being built now** and **not yet decided**.
Every proposal names its **graduation trigger**. Every file here also carries
one `B-nnn` row in [`../BACKLOG.md`](../BACKLOG.md), filed and deleted in the
same change as the proposal.

Frontmatter:

```yaml
---
type: Proposal
title: <idea>
description: <one line>
status: proposed        # proposed | graduated | rejected
timestamp: <ISO 8601>
---
```

| doc | idea | graduation trigger | status |
|-----|------|--------------------|--------|
| [write-time-guidance](write-time-guidance.md) | `hook --pre` for every agent with a pre-write hook; the brief cut by what is specific to the repo (W-05) | ruled 2026-10-27 → built by W-93 | graduated |
| [linter-for-ai-positioning](linter-for-ai-positioning.md) | codelaw beside ESLint, Prettier, Biome, Ruff, oxc: claims, README changes, gaps G1–G8 (W-36) | ruled 2026-10-27 → W-94–W-102 | graduated |
| [type-aware-rules](type-aware-rules.md) | type-aware TS rules and TS errors through the project's own TypeScript 7.1 API, a Node helper (W-101) | ruled 2026-10-27 → W-104–W-107 | graduated |

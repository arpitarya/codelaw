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
| [guidance-experiment-redesign](guidance-experiment-redesign.md) | W-102's experiment redesigned: a no-codelaw baseline, tasks screened for temptation by the agent, the check off the guidance arms and on as arm D, ≥ 5 runs per cell, a pre-registered bound instead of "zero in 60" (W-118) | ruled 2026-10-30 → W-102 reruns | graduated |
| [type-aware-rules](type-aware-rules.md) | type-aware TS rules and TS errors through the project's own TypeScript 7.1 API, a Node helper (W-101) | ruled 2026-10-27 → W-104–W-107 | graduated |
| [deps-flags-markers-reviews](deps-flags-markers-reviews.md) | the `deps` pack (vulnerable from merged on/off advisory sources, outdated, licence, lockfile sync, drift, runtime), flag expiry, TODO/skip markers, complexity and CODEOWNERS, rules from review comments | ruled 2026-10-29/30 → W-128–W-133 | graduated |
| [rule-parity](rule-parity.md) | every rule of ESLint, typescript-eslint, TSLint, Ruff, Biome, stylelint, html-validate, markdownlint and the plugins, counted and matched: 3,431 upstream, 653 covered, 1,690 to port in tranches A–E; the formatter measured against Prettier (W-138) | ruled 2026-10-30 (build all, 1,745) → W-139; W-141–W-145 | graduated |

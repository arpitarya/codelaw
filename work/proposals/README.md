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
| [type-aware-rules](type-aware-rules.md) | type-aware TS rules and TS errors through a typescript-go helper | the decider reopens it, or TypeScript 7.1 ships its API | proposed |

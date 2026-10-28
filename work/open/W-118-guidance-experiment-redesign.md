---
type: Handoff
name: W-118
description: "Arpit, 2026-10-29 (voice): a research document — why W-102's pilot could not separate guidance from the check, a new design, its downsides and what must be settled before the next run."
item: W-118
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: cowork
---

# W-118 — Research: redesign the guidance-vs-check experiment

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Why the pilot failed: A = B = C at 2/20, both breaks on tasks whose request asks for the violating code (t02 `console.log`, t04 unused parameters); 18 tasks tempted no arm | the diagnosis, from the pilot's data | ⏳ |
| 2 | A new design: tasks where the tempting mistake is not requested; a true baseline with no codelaw at all; enough runs to separate a real difference | a run that can answer the question | ⏳ |
| 3 | Downsides and open questions: cost, how many tasks and runs, how "tempting" is judged without bias, model and CLI versions pinned | what to settle first | ⏳ |
| 4 | A proposal in `work/proposals/` for the decider; on his ruling, W-102 runs it | the next run, or a reason not to | ⏳ |

**Arpit, 2026-10-29 (voice docket):** *"Let's post a research document on how do you want to design it again. With the downsides and details on why the current one failed, or the previous one failed, and what needs to be improved in next use case. Or what is something that you need to figure out in the next use case. So probably create a research work item."*

**Model:** Cowork — research (SR-WORK-OPEN-QUEUE rule 37a).

Source: [W-102](W-102-guidance-vs-check-experiment.md) (the pilot, `experiments/guidance-vs-check/`).

## Definition of done

- A proposal covering steps 1–3 is filed and the decider has picked a design (or no rerun).

## Record affected

No SR affected — research; the README line it informs is W-102's.

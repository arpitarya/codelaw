---
type: Handoff
name: W-36
description: "Arpit, 2026-10-25: research codelaw as the linter and formatter for AI — before, while and after the AI writes, and with no AI at all — that is as easy to pick up as ESLint and Prettier or Biome; what it must match, what it can claim, and who it is for."
item: W-36
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-36 — research: codelaw as the linter and formatter for AI, and for everyone without it

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Map what ESLint + Prettier, Biome, Ruff and oxlint do at each of the four moments — before, while and after an AI writes, and with no AI — including the AI features they ship today | an honest picture of where codelaw is ahead and where it is behind | ⏳ |
| 2 | List what makes those tools easy to adopt — one-line install, migrating an old config, an editor extension, pre-commit, a CI action, ignore comments — and mark what codelaw has | the adoption gaps, each one fileable | ⏳ |
| 3 | Decide which claims codelaw can prove: speed against Biome and ESLint on the same three pinned repos, rule coverage, one config for nine languages | marketing that survives a reader checking it | ⏳ |
| 3a | Measure the claim "with Fux and good docs an agent never breaks a rule": an agent writes code with Fux and the rule docs available, then `codelaw check` counts what it broke, on the same tasks with and without Fux | whether a check is still needed when the guidance is good — the README's "rules it can't skip" line, proven or withdrawn | ⏳ |
| 4 | Name who picks it and why — a team lead, a platform team, a team where agents write much of the code — and their objections | who the README speaks to | ⏳ |
| 5 | Write the proposal: README and usage-guide changes, a comparison page if it earns one, and the gaps as items; Arpit picks | a decision | 🔴 Arpit decides |

**Arpit, 2026-10-25:** *"I want codelaw to be a linter, prettier for AI — before,
while, after, and even without AI. Just like how ESLint, Biome works, it can work
that way as well. I want a work item which can research all these things."*

**Model:** **Cowork**, Opus — a `research` item, never Claude Code
([SR-WORK-OPEN-QUEUE](../../records/0051_WORK-open-queue.md) rule 37a). No code.

**Researched 2026-10-27 (Cowork, Opus 5.5):** steps 1–4 done and step 3a designed — [the proposal](../proposals/linter-for-ai-positioning.md) (sourced incumbent tables, claims kept and dropped, gaps G1–G8, the experiment). Step 5: the decider rules R1–R4 (inbox row filed 2026-10-27).

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md). **Closed 2026-10-27:** README fixed in the same change (R1); gaps filed as W-94–W-100, type-aware research W-101, the experiment W-102.

## Already done, and where the rest lives

- **The README already tells this story** (2026-10-25, the same ask): the story,
  the four moments, an ESLint/Prettier/Biome comparison table, and a separate
  `USAGE.md`. This item checks those claims and sharpens them; it does not
  restart them.
- **The while moment's design is W-05** (`brief_for_task`, `hook --pre`, which
  agents can inspect a write before it lands). Step 1 here reads W-05's matrix;
  it does not repeat it.
- **The README copy rule is W-35's:** say what ships. The while moment stays
  labelled until W-05's check is built.

## Questions to answer

- **Before:** does any of the four ship a way for an agent to ask for the rules
  that apply to a file (an MCP server, an agent-readable rule list)? Cite the
  version and date.
- **While:** does any refuse an edit before it reaches disk, or only report after?
- **After:** fix on save, fix in a hook, fix in CI — what each one's default is.
- **Without AI:** editor extension, pre-commit, a GitHub Action, SARIF, baseline
  or suppressions files — the table stakes a team expects on day one.
- **Migration:** what an ESLint, Prettier, Biome or Ruff user has to do to switch,
  and how much of that `init` and the planned `codelaw-migrate` skill
  ([SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md)) already cover.
- **Proof:** which numbers `codelaw bench --repo` can produce for a fair
  side-by-side, and on which machine
  ([SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md)).

**Arpit, 2026-10-25 (voice):** *"when an AI writes the code, you don't actually need codelaw. If you have a tool like Fux and there is a very good documentation around it, it's never going to break it"* → after the counter-argument (guidance is probabilistic, enforcement is deterministic): *"I got your point. Let's go ahead and add those lines."* Step 3a settles it with data: a reliable zero means the README's line goes.

## Definition of done

- A sourced table per step 1 and step 2: every cell names a doc, a release note or
  a run, with its date.
- A proposal in `work/proposals/` with the README and USAGE changes it wants, the
  claims it drops, and each gap it finds as a candidate item.
- An inbox row for Arpit's pick.

## Hazards

- **A competitor table goes stale fast**, and a wrong cell about another project is
  worse than none: every cell carries its source and date.
- **Speed claims need the same machine and the same repos**, run the same day; a
  number from a loaded laptop is not a finding (LESSONS).
- **Never claim an unbuilt moment** — W-35's rule.

**Records affected:** no SR affected — a proposal. Its outcome may amend
SR-AGENT-SURFACES or SR-CLI-MCP, each by its own item.

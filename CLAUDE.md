# CLAUDE.md — the agent contract for codelaw

This file is binding. It carries **pointers, never rules**: every rule lives in
exactly one record under [`records/`](records/README.md) and nothing here
restates one ([SR-LAW-0](records/0002_LAW-0-authority.md)). Read this, then
[`work/INTERVIEW.md`](work/INTERVIEW.md), before your first substantive change.

## Law zero — the records are always up to date

Stated once in [SR-LAW-0](records/0002_LAW-0-authority.md) decision 1a — three
obligations; the third cannot be enforced and is the only thing covering
coherence. The gate for the first two is [SR-WORK-OWNERSHIP](records/0058_WORK-ownership.md)
decisions 3 and 6. Install its `commit-msg` hook once:

```bash
ln -sf ../../scripts/sr-guard.sh .git/hooks/commit-msg
```

## Landing on main — lockmaster

**An agent session never commits to `main`** — the kit's `block-main.py`
refuses it (SR-WORK-SESSION decision 15). Work goes through
[lockmaster](https://github.com/arpitarya/lockmaster), which lands claimed
paths one change at a time after the checks in [`lockmaster.toml`](lockmaster.toml):

```bash
# `lockmaster` = PYTHONPATH=~/my_programs/lockmaster/src python3 -m lockmaster  (until it is on PyPI)
lockmaster start W-nn --paths <path> …   # branch + worktree + claim
lockmaster submit -m "W-nn: …"           # commit the claimed paths and land them
lockmaster queue · status · doctor
```

The hooks pinned in `.claude/lockmaster.lock` are lockmaster's kit: change them
in `~/my_programs/lockmaster/kit/`, then `lockmaster install .` — an edit made
here is reported by `lockmaster doctor` and overwritten by the next install.

## Triage first — a human-blocked queue stops the session

**Before any work, read [`work/OPEN-WORK.md`](work/OPEN-WORK.md)** and ask: *is
any item agent-closable right now?* If none is, the session's **first** output
is the blocked-on-the-decider list and it then **stops**. What the first output must
name, the inbox, the 5-day rule, the `↳ blocks:` sub-row and which balls an
agent may pick from are [SR-WORK-OPEN-QUEUE](records/0051_WORK-open-queue.md)
rules 32–34 and 39–45. **Claude Code never picks from the `research` section**
(rule 37a). **Two strikes → a gate**: [SR-WORK-SESSION](records/0054_WORK-session.md)
decision 13.

## Where the state of play lives

| you want | read |
|---|---|
| what to work on next | [`work/OPEN-WORK.md`](work/OPEN-WORK.md) — **the single live queue** |
| the queue explained one item at a time, with a recommendation, rulings written back | `/docket` — [`.claude/skills/docket/SKILL.md`](.claude/skills/docket/SKILL.md) |
| what has been named and nobody is doing | [`work/BACKLOG.md`](work/BACKLOG.md) |
| the judgment behind the current state | [`work/INTERVIEW.md`](work/INTERVIEW.md) |
| what has actually shipped | [`work/IMPLEMENTATION.md`](work/IMPLEMENTATION.md) |
| the spec for anything | [the register](records/README.md) |
| a closed fork and its reopen-trigger | [`work/compare/`](work/compare/README.md) |
| a word you don't recognise | [`docs/GLOSSARY.md`](docs/GLOSSARY.md) |
| why a command fails on *this* machine | [`work/MACHINE.md`](work/MACHINE.md) |
| the stages a feature moves through | [SR-WORK-LIFECYCLE](records/0053_WORK-lifecycle.md) |
| what a session owes before it ends, and how long an answer may be | [SR-WORK-SESSION](records/0054_WORK-session.md) |
| what `work/NOW.md` holds — only what is live | [SR-WORK-NOW](records/0061_WORK-now.md) |
| what happens to worktrees and lock files when a session ends | [SR-WORK-CLEANUP](records/0062_WORK-cleanup.md) |
| what a task owes the documentation | [SR-WORK-DOCS](records/0055_WORK-docs.md) |
| what to do when you cannot proceed without the decider | [SR-WORK-BLOCKERS](records/0056_WORK-blockers.md) |
| whether a claim may be grounded in an archived doc | [SR-WORK-ARCHIVE](records/0057_WORK-archive.md) |
| which record owns a module | [`records/README.md`](records/README.md) §Ownership |
| which file governs what | [SR-WORK-GOVERNANCE](records/0059_WORK-governance.md) |
| the whole set of rules about how work is done | the **WORK** records `0051`–`0062` |

Any work item that starts, finishes, blocks or is descoped updates
`OPEN-WORK.md` **in the same change as the work**. The record is the spec;
OPEN-WORK is the state; `IMPLEMENTATION.md` is the record of what landed.

## The laws

Each law is stated in exactly one record and is **not restated here**:

- **L0** — SRs are the only source of truth; Law records outrank every other record — [SR-LAW-0](records/0002_LAW-0-authority.md)
- **L1** — a retired record is archived, never deleted — [SR-LAW-1](records/0003_LAW-1-retired-records-archived.md)
- **L2** — every dependency is free, open source and permissive; nothing paid, nothing copyleft — [SR-LAW-2](records/0004_LAW-2-foss-dependencies.md)
- **L3** — nine languages — JS, TS, Python, JSON, YAML, TOML, Markdown, CSS, HTML — each alone or in any combination — [SR-LAW-3](records/0005_LAW-3-languages.md)
- **L4** — every rule has a fix — `auto` wherever safe, otherwise `suggest` or `manual`, and a skill carries every `manual` fix — [SR-LAW-4](records/0006_LAW-4-every-rule-has-a-fix.md)
- **L5** — no model in the engine, and no network in it: `codelaw deps refresh` is the one networked command, outside the engine; works the same with or without an agent — [SR-LAW-5](records/0007_LAW-5-no-model-in-the-engine.md)
- **L6** — one release — `codelaw` on PyPI and on npm — each a complete engine, identical output; rules as fifteen packs, thirteen on npm (`@codelaw/rules-*`) and nine on PyPI (`codelaw-rules-*`); npm sub-packages only as `@codelaw/*` — [SR-LAW-6](records/0008_LAW-6-python-and-node-packages.md)
- **L7** — always both a linter and a formatter, for every L3 language — [SR-LAW-7](records/0009_LAW-7-linter-and-formatter.md)
- **L8** — performance budgets — set from measurement, never exceeded, never loosened — [SR-LAW-8](records/0010_LAW-8-performance-budgets.md)
- **L9** — `work/MACHINE.md`'s budget section is always followed — 5 GB of disk per project, all agents combined; every GB past it needs the decider's approval — [SR-LAW-9](records/0011_LAW-9-machine-budget.md)
- **L10** — every file codelaw writes into a consumer's repo comes from a template in `templates/`; code only fills slots — [SR-LAW-10](records/0012_LAW-10-consumer-templates.md)
- **L11** — no dependencies, in any ecosystem or scope, unless the decider ratifies each one by name in `DEPENDENCIES.md` — [SR-LAW-11](records/0013_LAW-11-no-dependencies.md)

[SR-LAWS](records/0001_LAWS.md) assigns the handles and routes each to its
record; that is all it does. Add a law by adding a record in the law range and
a line here, on the decider's ruling named in the record.

## Documentation (required)

Three rule groups, one home — [SR-WORK-DOCS](records/0055_WORK-docs.md): how a
doc is written (decisions 1–5); who may edit the agent-steering files, this one
included, and under what two obligations (6–8 — **read before editing
`CLAUDE.md`**); the documents every task updates before it is done (10). The
registry is [SR-WORK-REGISTRY](records/0060_WORK-registry.md) §3.

**The shared memory between sessions is [`work/`](work/README.md)**; `docs/`
holds what the project *is*. Read `work/README.md` once; it is the map.

## Session continuity (required)

The worklog entry (a chat-only session counts), the interview kept current
*during* the session, `work/NOW.md` ([SR-WORK-NOW](records/0061_WORK-now.md)), the transition markers and
how long an answer may be — [SR-WORK-SESSION](records/0054_WORK-session.md).

## Blockers stop the session (required)

A blocker is a file — [`work/BLOCKED.json`](work/BLOCKED.json) — and the
session stops. Its shape, the four values, the prohibition on working around
one and the hooks that enforce it — [SR-WORK-BLOCKERS](records/0056_WORK-blockers.md).

## Archive is not evidence

One archive, [`archive/`](archive/README.md), at the repo root. What may be
named, what may never be cited — [SR-WORK-ARCHIVE](records/0057_WORK-archive.md).

## Hard-won build knowledge

Dated lessons live in [`work/LESSONS.md`](work/LESSONS.md), newest first.
Three that bind hard enough to name here: **re-derive `git status` right before
staging** — concurrent sessions commit too; **ground truth over prose** — a doc
repeating another doc is not a second source; **a loaded machine produces a
clean, localised anomaly** that reads like a finding — say what you are
running.

## Layout

```
work/        THE SHARED MEMORY between sessions — start at work/README.md
records/     THE STANDING RECORDS — the register, the template, the laws, the WORK records
docs/        WHAT THE PROJECT IS — the glossary
templates/   EVERY FILE CODELAW WRITES INTO A CONSUMER'S REPO — one template each, registry in templates/README.md (L10)
archive/     THE ONE ARCHIVE — everything retired, mirroring the live tree
crates/      THE ENGINE — one Cargo workspace (Cargo.toml); each crate's owner is in the register
fixtures/    test repos the law checks run on
packs/       THE FIFTEEN RULE PACKS — each folder is its package on npm, PyPI or both (pack.rs FIRST_PARTY_PACKS)
packaging/   the npm launcher and API, the PyPI module — every manifest is generated (SR-PACKAGES)
scripts/     the commit guard, the inbox check, the version and the release scripts
.github/     ci.yml builds and checks every package; publish.yml uploads them — RELEASING.md
.claude/     hooks and their wiring; skills/docket — the queue walk-through
```

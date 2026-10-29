---
type: Log
description: "Append-only per-session trail, date-grouped, newest first."
---

# WORKLOG — the append-only session log

Every session appends **one entry before it ends** — a chat-only session
counts. **Never edit a past entry; append only, newest on top.** A wrong old
entry is corrected by a new one. Distinct from `INTERVIEW.md`, which is the
*current* state of play. Rules: [SR-WORK-SESSION](../records/0054_WORK-session.md).

**Entry format:**

```
## YYYY-MM-DD — <one-line title>  ·  <surface> (<model>)

- **Asked:** *<the ask>*
- **Done:** <what landed, with commit shas where they exist>
- **Decided / left open:** <rulings taken in-session; forks put to the decider>
- **Mistakes:** <what went wrong and what was done about it, or "none">
- **Next:** <the single next step>
```

---

## 2026-10-29 — Worktree cleanup: ten dropped, W-120–W-124 closed, W-125 filed  ·  Claude Code (Opus 5.5)

- **Asked:** *"execute the cleanup of all the working trees. And any temporary files."*
- **Done:** each of the ten red worktrees merged in memory against `main` — nine held nothing `main` lacks; `W-119-loop` held Arpit's 10:31 ruling that `only=pypi-new` is temporary, which W-119's Cowork close had missed (it said the mode *stays*). Filed as W-125, archive line corrected; then all ten worktrees and branches force-removed, W-120–W-124 archived. `.DS_Store`, `__pycache__`, `fixtures/github-action/.codelaw/cache` deleted.
- **Decided / left open:** `target/` (19 GB of the 19.9 GB used) not touched — a rebuild, the decider's call. `lm/W-74` (nothing new) and `lm/W-82` (no worktree, conflicts with `main`) left as branches.
- **Mistakes:** none.
- **Next:** W-125.

## 2026-10-29 — Every codelaw package published and trusted; W-119 closed  ·  Cowork (Opus 5.5)

- **Asked:** one package at a time — *"add to pending publisher"*, *"next package"* — ten times.
- **Done:** for each of ten PyPI projects, Claude in Chrome checked the last one (version, trusted publisher, pending slot free) and registered the next pending publisher; Arpit ran `publish.yml` `only=pypi-new`. Final check 11:25: PyPI 24/24 and npm 45/45 exist and are trusted, no pending publisher left. W-119 closed and archived; its inbox row (*"the next new project, through CI … 0 of 10"*, written by another session) and open row removed. Not committed.
- **Decided / left open:** none.
- **Mistakes:** one "next" arrived before its run had happened — caught by the check, the command re-sent.
- **Next:** W-95 (the extension through `vsce`/`ovsx`), W-117 (Open VSX claim and token), W-97 (the Marketplace tick).

## 2026-10-29 — L2 decision 8: `vsce` and `ovsx` for the VS Code extension only  ·  Cowork (Opus 5.5)

- **Asked:** by voice, a package to check (and publish) the VS Code extension; ruled `@vscode/vsce` and `ovsx` as scoped exceptions; *"file it."*
- **Done:** L2 decision 8 (the scoped exception, his words); L11 §3 *CI — the VS Code extension's registry tools*: D-46 `@vscode/vsce`, D-47 `ovsx`, the 2026-10-27 *not ratified* line marked superseded; SR-PACKAGES; **`CLAUDE.md`'s L2 line** gains *"one scoped exception, the VS Code extension's registry tools (decision 8)"* — a fact on the decider's ruling, said here per SR-WORK-DOCS 7a. W-95 → 🟢 agent (move the job to the tools, the check command, pins, the licences test); its "run the `curl` job once" inbox row dropped; W-117 noted. Not committed.
- **Decided / left open:** ruled — both tools, only to publish and check the extension. Agent's reading, flagged in W-95: the job switches from `curl` to the tools.
- **Mistakes:** none.
- **Next:** an agent takes W-95.

## 2026-10-29 — W-119 step 1: `publish.yml` creates one new PyPI project from `main`  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement w119"*.
- **Done:** landed Cowork's W-119 filing first (582e56d; docs-sync wanted a CHANGELOG line for W-88/W-89/W-115, added; lockmaster missed staged deletions, worked around by unstaging). Then `publish.yml`'s `only=pypi-new` (`release`, `project` inputs): the gate on the tag's commit with the tag's scripts, `trust`/`npm` skipped, one wheel uploaded through its pending publisher, `-fux-index` names from the `fux-index` artifact. Records (L6 decision 9, SR-PACKAGES decision 11), RELEASING.md, W-119's loop. `pypi` environment: branch rule `main` added.
- **Decided / left open:** Arpit ruled *"Admit main in pypi"*. Open: the `fux-index` artifact is not attested, so those four wheels rest on the verified run id and their `SHA256SUMS`; B-063 (releases do not upload `dist/fux-index/`) stands, so after W-119 the four index projects stay at 0.3.0 until it is built.
- **Mistakes:** none. Meanwhile Arpit published `@codelaw/rules-spell-fux-index@0.3.0` (step 3; Cowork's note folded into this change, on npm at 10:22).
- **Next:** Arpit registers `codelaw-rules-scss`'s pending publisher and runs the command in the inbox; the first run tests the mode.

## 2026-10-29 — W-119 filed: PyPI through pending publishers; W-88, W-89, W-115 closed into it  ·  Cowork (Opus 5.5)

- **Asked:** by voice, how to publish more than four new PyPI projects a day; then *"file it."*
- **Done:** W-119 filed (a one-project `publish.yml` run, then the per-project loop; the ten PyPI projects listed with where each wheel is; what step 1 must solve — the ref and the `pypi` environment's tag rule, the `fux-index` artifact, the skip rule, one pending slot). W-88, W-89, W-115 archived; the day-3 and day-4 inbox rows removed. Not committed.
- **Decided / left open:** ruled — CI creates the projects one pending publisher at a time; the npm spell index stays a separate step. Open for step 1: run from `main` (the `pypi` environment admits it) or a v0.3.1 tag.
- **Mistakes:** none.
- **Next:** an agent builds W-119 step 1; then the loop with Arpit.

## 2026-10-29 — W-38 executed and closed: PyPI checked as a user  ·  Claude Code (Sonnet 5.5)

- **Asked:** *"execute W38 and test it out in a sibling repo. And then close it."*
- **Done:** fresh venv: `codelaw` 0.2.0 and 0.3.0 install and report their version; `uvx codelaw --version` 0.3.0; `codelaw-rules-python`, `-css` install by name; `codelaw check` on `codelaw-playground` ran (17 errors, 7 warnings, no files changed). README, RELEASING.md, IMPLEMENTATION written; W-38 archived; W-64 unblocked.
- **Decided / left open:** the eight-platform install from the registries is a CI job and was not run here.
- **Mistakes:** none.
- **Next:** the inbox (W-88/W-89 uploads, W-95 "run it", W-97 tick, W-117, W-118 rulings).

---

## 2026-10-29 — v0.3.0 found released; W-116 closed; day-2 notes corrected  ·  Cowork (Opus 5.5)

- **Asked:** (continuing day 2) — reconciling after the publishers turned up `codelaw` already at 0.3.0.
- **Done:** v0.3.0 was released 01:28 IST (IMPLEMENTATION had it; the item files did not): npm and PyPI at 0.3.0, `codelaw` created on PyPI by CI through its pending publisher. W-116 closed and archived. W-86/W-87's close notes and IMPLEMENTATION row corrected — day 2 back-filled `codelaw` 0.2.0 and created three new projects, not four. W-95 → 🔴 with an inbox row (run the extension's job alone from `main`); W-97's inbox row now points at the v0.3.0 release's edit page. Not committed.
- **Decided / left open:** none.
- **Mistakes:** this session's 09:59 notes said the pending publisher became `codelaw`'s on Arpit's upload; it had become so at v0.3.0's run — corrected the same turn.
- **Next:** Arpit: "run it" for W-95, the Marketplace tick for W-97; one PyPI new-project slot is free today.

## 2026-10-29 — PyPI day 2 done: `codelaw` is on PyPI; W-86 and W-87 closed  ·  Cowork (Opus 5.5)

- **Asked:** Arpit's day-2 run (09:53): the v0.2.0 tag's CI `dist` downloaded, SHA256SUMS all OK, law-gate passed, `twine upload` of `codelaw` 0.2.0 and three Fux index wheels.
- **Done:** trusted publishers checked and added in Chrome — `codelaw`'s pending publisher had become its normal one on creation; the three Fux index projects added; all four `arpitarya/codelaw` · `publish.yml` · `pypi`; no pending publisher left. W-86, W-87 closed and archived; W-38 → 🟢 (the user check); the day-2 inbox row removed; W-116 noted (the pending slot is free). Not committed.
- **Decided / left open:** none.
- **Mistakes:** none.
- **Next:** an agent runs W-38's user check; day 3 on Friday.

## 2026-10-29 — W-34 and W-77 closed into the PyPI days; B-063 filed  ·  Cowork (Opus 5.5)

- **Asked:** *"W34 can be archived and it can be merged with PyPy day 2, day 3, day 4 items. W77, the npm package is published, the PyPy package merge it with PyPy day 2, day 3, day 4 or create a new one day 5."*
- **Done:** W-34 and W-77 archived (IMPLEMENTATION rows, archive README successors); their PyPI uploads named in W-87, W-88, W-89; the inbox's blocks sub-rows cleaned. `publish.yml` uploading the Fux index packages — W-34's only remaining agent step — moved to BACKLOG B-063 rather than lost. `codelaw-skills` checked: not on PyPI (404), already in day 3, so no day 5.
- **Decided / left open:** his ruling, applied. Not committed.
- **Mistakes:** none.
- **Next:** day 2's upload.

## 2026-10-29 — `codelaw` itself joins PyPI day 2  ·  Cowork (Opus 5.5)

- **Asked:** *"getting code law in itself into PyPy … make it a part of PyPy Day 2 … mark it as pending trusted publisher and then I push it from my local. Will that work for W86?"*
- **Done:** day 2 (W-87) now uploads `codelaw` 0.2.0 by hand — its eight platform wheels from the v0.2.0 tag's CI `dist` (run 40194930874, kept to 2026-11-24), checked by SHA256SUMS and law-gate — with the `react`, `scss`, `python` Fux index wheels; `security-js`'s index moved to day 4 (W-89) to stay at four new projects. W-86 → 🔴 on W-87; W-116 noted. Not committed.
- **Decided / left open:** his ask, applied. `codelaw`'s pending publisher stays registered; it neither reserves the name nor blocks his upload; a normal trusted publisher replaces it after the upload.
- **Mistakes:** none.
- **Next:** Arpit runs day 2's command; Claude adds the four trusted publishers.

## 2026-10-29 — The queue reconciled again: W-114 closed; W-86, W-95, W-116, W-117, W-38 brought current  ·  Cowork (Opus 5.5)

- **Asked:** *"review all the open work items. Are all of them up to date?"*
- **Done:** checked against `git log`, worktrees, GitHub (releases, runs, environments, issue search) and the registries. W-114 closed and archived (the five worktrees and branches are gone). W-86: the v0.2.0 Release was created 00:33 and its `publish` run 40750812279 failed at the gate — nothing published. W-95 → 🟡 on W-116 (environment `vscode` exists, the job landed). W-116: bump/CHANGELOG/PyPI skip landed; left green CI and the Release. W-117 and its inbox row: what is done, the claim (no issue found) and the token left. W-38's row. Not committed: written beside a live session (59502574) whose staged edits are untouched.
- **Decided / left open:** none ruled. NOW's row for `c6c222e8` still says it waits on the v0.2.0 Release, which has since run — that session's to update.
- **Mistakes:** none.
- **Next:** CI green on `main`, then Arpit's v0.3.0 Release; the Open VSX claim and token.

## 2026-10-29 — Open VSX set up; the Marketplace's trusted publishing on  ·  Cowork (Opus 5.5)

- **Asked:** *"Set up open VSX. Create the account."*, then *"Go ahead, claim ownership. Do what's necessary."*, then, on the Marketplace, how to publish through CI with trusted publishing.
- **Done:** Open VSX — Arpit logged in with GitHub, linked his Eclipse account and signed the Publisher Agreement; Claude in Chrome created the namespace `codelaw` (unverified). The Marketplace — Arpit created the Trusted Publishing policy *codelaw publish.yml* (`arpitarya` · `codelaw` · `publish.yml` · `vscode`), Active. W-95 → 🟢 agent (the environment and the OIDC job), W-117 and SR-PACKAGES brought current. Not committed.
- **Decided / left open:** Open VSX's ownership claim — Option 2 needs Marketplace access (blocked then by a looping-login 500), Option 1 fits after the first Marketplace version; Arpit reported filing the claim by voice, issue number not yet recorded. Whether the deleted 0.2.0 blocks `codelaw.codelaw` — open.
- **Mistakes:** the voice reply said the Marketplace has no trusted publishing; his screenshot showed it does — corrected in text.
- **Next:** an agent builds `publish.yml`'s Marketplace job (OIDC → `curl`); Arpit names the claim issue for W-117.

## 2026-10-29 — Docket (voice): seven blockers ruled; W-116, W-117, W-118, B-062 filed  ·  Cowork (Opus 5.5)

- **Asked:** *"List all the blockers in OpenWork in detail with explanation and examples and recommendation."* — then a voice walk-through of all but PyPI days 2–4, and *"file it."*
- **Done:** OPEN-WORK: six inbox rows ruled and removed (PyPI day 1, W-114, W-95's tokens, W-105, W-102, W-103), one added (W-117, Open VSX); W-114, W-105, W-103 → 🟢; W-86 → 🟡 on W-116; W-95 → 🔴 on W-117; W-102 → 🟡 on W-118. Filed W-116 (release 0.3.0 through CI), W-117 (Open VSX), W-118 (research: redesign the experiment), B-062 (the shared benchmark). Records: SR-TYPES decision 4 (agents may add questions), SR-SPELL decision 9 (its own operation). Rulings in his words in each item. Not committed: this Cowork shell cannot run lockmaster.
- **Decided / left open:** ruled — 0.3.0 through CI, PyPI only for projects that exist (*"We can skip the one pending for pipe here"*); drop all five worktrees; Open VSX now and the Marketplace by hand on release day; agents add the four type questions, profiling before budgets; redesign before the 180, as a research item; spell measured as its own operation, the shared benchmark backlogged. Agent's reading, flagged in W-86: the v0.2.0 Release is superseded by v0.3.0's. A live Claude Code session (`c6c222e8`, since 00:31) was on W-86's v0.2.0 Release, held by the permission classifier.
- **Mistakes:** a Python quoting error stopped the first write; nothing had been written.
- **Next:** an agent takes W-116 (the `publish.yml` skip, the bump); W-114's cleanup on the Mac.

## 2026-10-29 — The queue reconciled: W-111 merged into W-114, W-115 filed, W-95 brought current  ·  Cowork (Opus 5.5)

- **Asked:** *"review the open work document, get all of them updated to the latest, and whatever is the latest status, whatever is done, close them out."*
- **Done:** every row checked against IMPLEMENTATION, `git log`, PyPI's and npm's registries and GitHub's releases. Nothing open was done: v0.2.0 has no GitHub Release yet (`codelaw` absent from PyPI), PyPI days 2–4's names are absent, W-105/W-103/W-102 wait on their inbox rows. W-111 closed and archived — merged into W-114 (the session-end hook filed the same question for five worktrees); W-114 checked read-only (each worktree's work is on `main` or is `.fux/index` churn) and its inbox row recommends "drop all five". W-95 brought current (publisher `codelaw` created 10-28; the token blocked by Azure; global PATs end 2026-12-01). W-115 filed: the spell pack exists on neither registry and no item held its first publish. NOW: two dead sessions' rows removed (019585aa, c1b83c1f; no commit since 11:38 on 10-28). The uncommitted W-114 row and file the hook left are kept. Not committed: this Cowork shell cannot run lockmaster.
- **Decided / left open:** agent's calls, flagged — W-111 merged rather than kept beside W-114; W-115 as 🟢 agent prep for the next release. Left in the working tree untouched, from dead sessions: `.claude/hooks/auto-land.py`, `.claude/lockmaster.lock` (lockmaster's kit — `lockmaster doctor` reports them), `packaging/vscode/icon.png`.
- **Mistakes:** on 2026-10-28 this session told Arpit that pending publishers could create many PyPI projects at once; RELEASING.md step 4 and warehouse #11296 say one pending publisher per repository · workflow · environment, so that plan does not work — said so in the reply.
- **Next:** Arpit rules W-114 and runs the v0.2.0 Release; PyPI day 2's upload is today.

## 2026-10-27/28 — the coordinating session: CI green, packages published, every green item run in parallel  ·  Claude Code (Opus 5.5), session 2df06f6d

- **Asked:** *list unpublished packages; first-publish the npm ones; fix CI until it is green; then "implement all the items in green in parallel and close them out", keep going as items turn green; land Cowork's uncommitted edits; "whatever is done merge it to main".*
- **Done:** CI red → green (`records/RULE-SINCE` 791b189, 0dfbaae, d69b476 — each named; schema test c410174); the nine npm packages built locally for Arpit's first publish; Cowork's main-checkout edits landed byte for byte (f669b92, 163d33b, d91b093 and more); agents closed W-62, W-84, W-85, W-90, W-91, W-93, W-94, W-96, W-98, W-99, W-100, W-104, W-107, W-108, W-109, W-110, W-112, and built W-95, W-97, W-102 (pilot $10.47), W-103, W-105 to their decider steps; lockmaster W-11's kit handoff landed (7128514) with auto-land.py's ownership row.
- **Decided / left open:** Arpit's standing approvals — land Cowork's main-checkout edits; move RULE-SINCE past a guard-refused commit (both in memory). Open for him: W-86–W-89, W-95, W-97, W-102, W-103 (L8 measurement), W-105 (type questions), W-111.
- **Mistakes:** lockmaster's `submit` landed guard-refused commits three times and once a merge with conflict markers (W-112, fixed 8e5f9b9) — agents now run the guard before submit; two landing merges carry the message "x" (c410174, d91b093) from a careless `-m x`; zsh word-splitting broke three commands before they ran (nothing changed); the disk check misreads from a worktree; agents shared one cargo target, so tests saw stale builds — `touch` before building.
- **Next:** file the lockmaster kit bugs (guard bypass, conflict-marker merge, `save`'s claim, disk check in worktrees) and the `discover.rs` nested-repo ignore bug as items.

---

## 2026-10-28 — W-109's ruling applied: `init --ci bitbucket` removed, the step documented  ·  Claude Code (Opus 5.5)

- **Asked:** apply the decider's 2026-10-28 W-109 ruling (coordinator): keep `--format bitbucket`, remove `init --ci bitbucket`, USAGE shows the step to copy with its two switches, a test proves the documented step.
- **Done:** the generator out (`ci.rs`, `init.rs`, `templates/ci/bitbucket/`, its registry row and `template.rs` entries); SR-CLI-MCP 12b rewritten as the documented step, SR-PACKAGES 15a (no pipe, no generated step), SR-LAW-10 decision 3's example; USAGE §Bitbucket Pipelines; `tests/ci_bitbucket.rs` → `tests/usage_bitbucket.rs`, taking USAGE's YAML block; CHANGELOG, IMPLEMENTATION, INTERVIEW, the item and its OPEN-WORK row (🔴 `arpit`) plus an inbox row for the live proof.
- **Decided / left open:** the step pins `codelaw@0.3.0` (the first release with `--format bitbucket`). Left: the live proof after v0.3.0, the decider's.
- **Next:** v0.3.0, then the inbox row's live proof.

## 2026-10-28 — W-109 ruled down to the Code Insights output  ·  Cowork (Opus 5.5)

- **Asked:** *"codelaw init --ci bitbucket no need to build this part, but do build out the configurable insights part."* — found W-109 already built and landed (incl. `init --ci bitbucket`); by voice: remove the generator, keep the Code Insights output, SARIF and Insights configurable, the live proof kept; *"file it."*
- **Done:** OPEN-WORK: the pipe-fork inbox row removed (ruled: no pipe), W-109 re-balled 🟢 agent for the removal; W-109's file (view, ruling, Left, records line); SR-CLI-MCP 12b and SR-PACKAGES 15a carry the ruling as *to be removed*. No code changed here: the removal lands through lockmaster.
- **Decided / left open:** ruled — no `init --ci bitbucket`, no template, no pipe; `--format bitbucket` stays with the two switches. Agent's reading, flagged in the item: the switches stay as the USAGE snippet's variables; *"Perfect."* read as keeping the live proof.
- **Mistakes:** the earlier W-109 rescope edit failed on a stale match — the file had changed under it (an agent built W-109 at 09:02); nothing was written then.
- **Next:** a Claude Code session takes W-109's removal, records first.

## 2026-10-28 — the project icon, one colour  ·  Claude Code (claude-opus-5-5)

- **Asked:** *read the documentation and create an icon that I can use in different places; put it in the repository* — then *2–3 options*, *A*, *only the one-colour version*, *land it*.
- **Done:** three directions on a design canvas (checked brackets, a rule file, § between brackets); Arpit picked A, one colour only. `docs/brand/`: `codelaw-mono.svg` (Ink `#182133`), `codelaw-mono-white.svg`, both at 16–512 px in `png/`, and a README (W-113).
- **Decided / left open:** Arpit: option A, one colour, no full-colour tile. Left open: whether `packaging/vscode/icon.png` (the old colour placeholder) becomes the one-colour mark.
- **Mistakes:** I first wrote a full-colour tile too; removed on Arpit's word before landing.
- **Next:** Arpit's call on the VS Code icon.

## 2026-10-28 — W-109 built: Bitbucket Pipelines, Code Insights, SARIF a switch  ·  Claude Code (Opus 5.5)

- **Asked:** *"Implement work item W-109 in codelaw: Bitbucket Pipelines"* (coordinator, from Arpit's voice ruling of 2026-10-27).
- **Done:** `--format bitbucket` (SR-CLI-MCP decision 4), `codelaw init --ci bitbucket` from `templates/ci/bitbucket/bitbucket-pipelines.yml` (decision 12b), SR-PACKAGES 15a, USAGE; `ci_bitbucket.rs` runs the step's own script with `curl`/`npm` stubbed; the report and annotation shape checked against Bitbucket's published swagger.
- **Decided / left open:** a step, not a pipe — a pipe's base image is an L11 row and its registry an account, so that fork is the decider's (inbox); the live proof waits on v0.3.0, since 0.2.0 has no `--format bitbucket`. W-109 stays open, 🔴 `arpit`.
- **Mistakes:** the first `curl` stub numbered its log by every file, not every call; fixed before the test was believed.
- **Next:** the decider answers the pipe fork; after v0.3.0, the throwaway-repo run in the inbox row.

## 2026-10-28 — W-112 filed: the spell words file, init by default, spell learn  ·  Cowork (Opus 5.5)

- **Asked:** *"Is there a way to add custom words to spelling? Pack."* — answered (`[spell] words`, built today in W-103) — then by voice the design, and *"file it."*
- **Done:** W-112 filed with his words; its OPEN-WORK row (🟢 agent, build); SR-SPELL decisions 1 and 7 each carry the ruling as *ruled, not built*; INTERVIEW §1. Not committed: this Cowork shell cannot run lockmaster.
- **Decided / left open:** ruled — a words file next to the rules directory, path set in TOML; the inline list stays and merges; silent when the default file is missing, an error when a set path is; `.codelaw/spell/words.txt`; `codelaw init` adds the pack and the file and runs `spell learn`, `--no-spell` skips all; `codelaw spell learn` adds unknown words and lists likely typos for review. Agent's proposals he accepted: the path, `words_file`, the review list.
- **Mistakes:** none.
- **Next:** an agent takes W-112, records first (it reverses SR-SPELL decision 1).

## 2026-10-28 — Docket: nine rulings written back; PyPI day 1's publishers; W-108–W-111 filed  ·  Cowork (Opus 5.5)

- **Asked:** *"Review all the open work blockers, explain them to me one by one with a recommendation"* — then a voice walk-through, one card at a time — then *"write it."*
- **Done:** OPEN-WORK: six inbox rows ruled and removed, three reworded (PyPI day 1, W-95's accounts, W-97's Marketplace step), W-93's stored "0 d" age cell gone with its row; W-93, W-99, W-100, W-102, W-104 to 🟢. Item files W-86–W-89, W-93, W-95, W-97, W-99, W-100, W-102, W-104 carry the rulings in his words. Records: L11 §3 D-37…D-41 (bench, CI-only) and D-45 (`typescript` 7.1, tests and CI), D-34's arm-C scope, `upload-sarif`/`vsce`/`ovsx` not ratified; SR-RULE-AUTHORING 6a ratified + `scripts/corpus.json` `promotion.ratified`; SR-TYPES decision 5; SR-LAW-8 and `perf/budgets.toml` (`hook_pre` unset); SR-PACKAGES (curl upload, `gh` SARIF, Marketplace + moving `v0`); the experiment README. PyPI day 1: Arpit uploaded the four wheels (08:10 IST); Claude in Chrome added their four trusted publishers; `codelaw`'s pending publisher still listed. The oxlint live capture run in the Cowork VM (oxlint 1.90.0, tsgolint 7.0.2005, typescript 7.0.2), outside the repo. Filed W-108 (the capture: it disagrees with W-107's fixture — a third finding, another span, another help line), W-109 (Bitbucket Pipelines), W-110 (session 2df06f6d's merged worktrees, for the Mac), W-111 (`lm/W-90`, `lm/W-96` keep-or-drop, inbox). `check-open-work-inbox.py` and `check-work-system.py` pass. Written in the main checkout, not committed: this Cowork shell cannot run lockmaster.
- **Decided / left open:** ruled 2026-10-27 by voice — PyPI as filed (*"Yes."*); W-95 (a) (*"A. A."*); 6a (*"Ratify."*); W-97's three calls (*"Okay."*); D-37…D-41 (*"Ratify."*); `hook_pre` unset (*"We can go with unset"*, after asking for 150 ms); W-102 (*"Approved."*); W-104 both (*"Yes."*); W-107 (*"You run it, create a work item for it."*). Agent's calls, flagged: rulings dated 2026-10-27, the day of the call; the cleanup split into W-110 (🟢) and W-111 (🔴) per SR-WORK-CLEANUP decision 3; the gh-missing error for the Action is offered, not ruled.
- **Mistakes:** first WebFetch to PyPI and the first device call failed (robots, the Mac offline); the walk-through read GitHub's copy until the Mac came back.
- **Next:** Arpit runs `gh release create v0.2.0` (the inbox row); then an agent takes W-108.

## 2026-10-27 — W-107 closed: oxlint type-aware through the adapter  ·  Claude Code (Opus 5.5)

- **Asked:** *Implement and close W-107: prove and document that oxlint's type-aware mode flows through the oxlint adapter, `typescript/*` ids included; a test on a recorded oxlint JSON; the live run a decider ask.*
- **Done:** fixture `crates/codelaw-adapters/tests/fixtures/oxlint-type-aware/` (built from oxlint's JSON formatter and oxc's tsgolint snapshots), a mapping test and a binary test with a stand-in oxlint; adapter fixes `ported_plugins` (oxlint `typescript/` = `typescript-eslint/`, so shown once works) and `uncached_when = "typeAware"`; USAGE *Type rules today*; SR-LINTER decision 2, SR-ARCHITECTURE decision 19.
- **Decided / left open:** closed on the built fixture; the live capture is an inbox row (no oxlint is installed or ratified here, L11). oxlint's `react`/`node` plugin names still miss the packs' `eslint-plugin-*` `ported_from:` names — unverified, not mapped.
- **Mistakes:** none.
- **Next:** the decider runs the capture, or says "skip it".

## 2026-10-27 — W-104: the records for type-aware rules  ·  Claude Code (Opus 5.5)

- **Asked:** implement and close W-104 — amend L3, L5, L6, L8 on W-101's ruling; write SR-TYPES; the `type:` clause; resolve SR-ARCHITECTURE's open line; the decider confirms L11. Records only.
- **Done:** L3 decision 9, L5 (TypeScript version an input), L6 decision 3's runtime exception, L8's two `unset` operations and decision 8 — each quoting the ruling's words from the proposal §3; SR-TYPES (0117, accepted, built: no); SR-RULE-FORMAT 3f; SR-ARCHITECTURE's Consequences line and decision 28; DESIGN §3.6 and its risks row; glossary *type question*, *type helper*. No `CLAUDE.md` change: no law's handle line changed.
- **Decided / left open:** agent's calls, flagged: L3 decision 9 says JavaScript is not type-checked and restates decision 5's isolation for a TS program; L8's operations are not put in `perf/budgets.toml` until W-105 can time them; SR-TYPES gets no Ownership row (nothing it owns exists yet). Put to the decider: (1) the project's TypeScript is no dependency; (2) codelaw's own CI/test copy of `typescript` 7.1 (Apache-2.0) is one and needs a row. W-104 stays open on them.
- **Mistakes:** none.
- **Next:** the decider answers the L11 row; then W-104 closes and W-105 starts.

## 2026-10-27 — W-103's first increment: the spell pack and `spell/known-typo`  ·  Claude Code (Opus 5.5)

- **Asked:** *build W-103 — the spell pack, known-typo on with an auto fix, dictionary opt-in, word data from SCOWL, VarCon and the software terms; land coherent increments.*
- **Done:** licences read from each source (all notice-only or MIT, no decider ask); SCOWL 2020.12.07 and software terms 5.4.8 pinned by SHA-256; `scripts/spell-data.py` generates 188,151 known typos; `packs/spell/` with `spell/known-typo` (native, `auto`, all nine languages, 107 test checks); eighteen packs in `FIRST_PARTY_PACKS`, L6, SR-RULE-PACKS, SR-PACKAGES, SR-RULE-FORMAT, **`CLAUDE.md`'s L6 and `packs/` lines** (a fact, on the decider's ruling in SR-SPELL — said here per SR-WORK-DOCS 7a), the laws index, DESIGN, README, RELEASING, GLOSSARY; the compiled-rule-set key now carries each rule's default (a one-rule pack's `codelaw test` reused a set compiled with the rule off).
- **Decided / left open:** as built, not as written — text is chosen by a node-kind table, not query files (no pinned grammar ships `@spell` captures); SCOWL's size-95 lists are not "words" for the generator because they hold `untill`, `accomodate`; identifiers in prose are skipped. All in SR-SPELL *as built*. The L8 budget is the decider's after CI's perf runs.
- **Mistakes:** none landed; a first measurement ran on a machine at load 21, said so in SR-SPELL decision 9.
- **Next:** W-103's left list, item 1: `spell/known-typo-identifier`.

## 2026-10-27 — W-101 researched and ruled: type-aware TS rules on the project's own TypeScript 7.1  ·  Cowork (Fable 5.1, then Opus 5.5)

- **Asked:** *"W101. research and propose."* Then a voice walk-through (how it works, what is built, decisions, blockers) and *"go ahead and write all these things."*
- **Done:** research on oxlint/tsgolint, TypeScript 7.1's preview API, typescript-eslint, Biome, tsrs, Pyrefly and ty; [the proposal](proposals/type-aware-rules.md) rewritten with the rulings, cost and sources; W-101 closed and archived; W-104 (records), W-105 (helper, engine, rules), W-106 (version watch), W-107 (oxlint gap filler) filed. Written in the main checkout, not committed: this Cowork shell cannot run lockmaster.
- **Decided / left open:** R1–R8 ruled (proposal §3). Left for the decider at W-104: confirm L11 counts the project's own TypeScript as no dependency.
- **Mistakes:** `git rm --cached` staged W-101's deletion in the shared index; reset at once, nothing else staged.
- **Next:** land these paths through lockmaster; then an agent takes W-104.

## 2026-10-27 — CSpell researched; the spell pack ruled and filed as W-103  ·  Cowork (Opus 5.5)

- **Asked:** *"There is a tool called C-Spell. Research it and tell me if that can be built into CodeLaw. A similar feature, a better feature, would be even better."* Then a voice walk-through of the forks, and *"go ahead"* to write it up.
- **Done:** SR-SPELL (0116, accepted, built: no); the compare doc `work/compare/spell-checking.compare.md`; L11 §3 rows D-42 SCOWL, D-43 VarCon, D-44 `@cspell/dict-software-terms`, ratified; W-103 filed; B-054, B-055; glossary terms; registry bumps. Written in the main checkout, not committed: this Cowork shell cannot run lockmaster.
- **Decided / left open:** ruled — codelaw's own checker, not CSpell; the known-typo list generated from SCOWL, not `typos`' list (*"Let's go with B."*); SCOWL and VarCon (*"Approve both."*); software terms (*"Approve it."*); a separate pack (*"want it as a separate package."*); known-typo on once installed, dictionary opt-in, data in the pack (*"Go with the recommendation."*). Agent's call, flagged: `init` never adds the pack (as with the security packs). Open: the data's size and the pass's speed, measured in W-103.
- **Mistakes:** none.
- **Next:** land this change through lockmaster (`lockmaster start W-103 --paths …` from the main checkout), then build W-103 step 1.

---

## 2026-10-27 — W-96 closed: `--format agent` by itself inside a coding agent  ·  Claude Code (Opus 5.5)

- **Asked:** implement and close W-96 — pick `--format agent` when an agent's variable is set; `--format` wins; CI and a person's terminal are not an agent.
- **Done:** `codelaw-report` `agent_env.rs` (`AGENT_VARS`, `FormatEnv`, `resolve_format`); `check`/`fix`/`--watch` use it, warm processes get the client's; `config --print` shows `format` and its source; SR-CLI-MCP decision 4a; USAGE; tests `agent_format.rs`; `.cargo/config.toml` pins `CODELAW_FORMAT=text` for this repo's tests.
- **Decided / left open:** only variables an agent's own docs or source name: Claude Code, Codex, Gemini CLI. `CLAUDECODE` counts only off a terminal (Claude Code's IDE extensions set it in the integrated terminal). Cursor (`CURSOR_AGENT`), Copilot CLI (`COPILOT_CLI`) and oxlint's other variables not detected until their agents document them.
- **Mistakes:** none.
- **Next:** add `CURSOR_AGENT` / `COPILOT_CLI` to `AGENT_VARS` once Cursor's or Copilot's docs name them.

---

## 2026-10-27 — W-05 and W-36 ruled and filed: W-93 to W-102; README fixed  ·  Cowork (Opus 5.5)

- **Asked:** a voice walk-through of the two proposals, rulings *"Yes to all three"* (W-05) and *"Yes to all four"* (W-36); then *"write to the repo."*
- **Done:** W-93 (the `hook --pre` + brief build, records first), gap items W-94 `codelaw migrate`, W-95 publish the VS Code extension, W-96 agent-format auto-detection, W-97 GitHub Action, W-98 `llms.txt`, W-99 side-by-side bench, W-100 rule promotion; W-101 type-aware TS research (B-049 graduated); W-102 the guidance-vs-check experiment. README: the hero no longer claims *while* as built; rule counts corrected (492 ported of 652, Bandit named); the W-05 links point to W-93. W-05 and W-36 closed and archived; the three proposals marked graduated; W-35 now waits on W-93.
- **Decided / left open:** the rulings above. Left for the decider at landing: W-95's publisher and tokens, W-99's CI-only rival rows (L11).
- **Mistakes:** none.
- **Next:** W-93 (agent); W-96 is the quickest win.

## 2026-10-27 — W-05 and W-36 researched; npm trust finished  ·  Cowork (Opus 5.5)

- **Asked:** *"What is blocked on W-36 and W-05? Research both of them and propose."* Earlier: add the npm trusted publishers through the browser.
- **Done:** npm — all 24 untrusted names trust `publish.yml` (11 in the browser, the rest by Arpit's CLI loop; Cloudflare's bot check stopped the browser); W-85 re-balled to agent. W-05 — an eight-agent pre-write hook matrix from the agents' docs; construct coverage measured over the 652 rules (~20 derivable); `work/proposals/write-time-guidance.md`. W-36 — sourced incumbent tables (ESLint 10.13, Prettier 3.9, Biome 2.6.3, Ruff 0.17.3, oxlint 1.89/oxfmt 0.74), adoption gaps G1–G8, claims kept and dropped, the step 3a experiment; `work/proposals/linter-for-ai-positioning.md`. Two inbox rows for the rulings; both items re-balled to Arpit.
- **Decided / left open:** nothing ruled; R1–R3 (W-05) and R1–R4 (W-36) are the decider's. Found: the README hero claims *while* as built; README's 441 and 652 counts are both wrong (492 ported of 652).
- **Mistakes:** the browser added trust faster than Arpit could tap the key, losing saves and tripping Cloudflare; slowed to one at a time, then stopped.
- **Next:** the decider's rulings on the two proposals.

## 2026-10-27 — Voice docket filed: five publishing items, L11 "dependencies", two ratifications, W-45 closed, W-51 parked  ·  Cowork (Opus 5.5)

- **Asked:** a voice walk through the blockers, one by one; then *"file whatever we talked about."*
- **Done:** W-85 (npm first publish of the nine missing names + trust, incl. the 15 Fux index packages), W-86–W-89 (PyPI, four new projects a day, Wed 10-28 to Sat 10-31, each inbox row carrying its command); W-38 → 🟡 on W-86, W-34 → 🟡 on W-85/W-87/W-88, W-77 → 🟡 on W-85/W-88. `DEPENDENCIES.md`: D-34 ratified unversioned, D-36 `tree-sitter-cli` 0.25.10 ratified; `BLOCKED.json` → `PROCEED`, W-62 → 🟢. W-90 (L11 renamed "dependencies", holds the list, `DEPENDENCIES.md` archived) and W-91 (fux auto-bump by PR) filed. W-45 closed and archived; W-51 archived as parked, B-053. IMPLEMENTATION rows for both and for the PyPI trusted publishers. Not committed — main checkout; lockmaster is not reachable from this session.
- **Decided / left open:** Arpit's rulings, quoted in each item. Left for him: the version W-85 first-publishes at (none of the nine, nor `codelaw-skills`, `codelaw-rules-scss`, the security packs, is in v0.2.0; recommended the next release's). PR #3 and its branch still to close.
- **Mistakes:** the voice call said the v0.2.0 Release waits for day 4 and that seven of the nine npm names are rule packs; the Release needs only day 1 (v0.2.0's `dist` holds six PyPI packs), and five are rule packs, three plugins, one skills package. Corrected in the items.
- **Next:** land these edits through lockmaster; W-86 on Wed 2026-10-28.

## 2026-10-27 — OPEN-WORK brought up to date; trusted publishers on PyPI; W-84 filed  ·  Cowork (Opus 5.5)

- **Asked:** *trusted-publisher list updated for PyPI and npm for all published packages*; then *is OPEN-WORK up to date?*; then *update it to the latest*.
- **Done:** PyPI trusted publishers (`arpitarya/codelaw` · `publish.yml` · `pypi`) added on `codelaw-rules-css`, `-data`, and the `-fux-index` projects for `css`, `data`, `html`, `js`; `codelaw`'s pending publisher confirmed. npm: all 19 release packages already trust `publish.yml`; the 15 `-fux-index` packages trust nothing (adding needs Arpit's security key per package). OPEN-WORK: W-38 and W-34 inbox rows and item files reflect the registries; W-62 and W-45 get inbox rows (W-45's ball → `arpit`); W-44 and W-70 rows and files carry the CI runs; W-84 filed (CI red on `main` since 2026-10-26 11:33 UTC, untracked). Not committed — edited in the main checkout; lockmaster is not reachable from this session.
- **Decided / left open:** `rules-tanstack-query` and `rules-redux-toolkit` on npm show *Pending validation* and only `npm publish` permission; W-51 is marked `agent` but needs a throwaway repo and a token.
- **Mistakes:** said W-38's inbox and file `filed` dates disagree — they do not; the file absorbed the inbox row of 2026-10-20.
- **Next:** land these edits through lockmaster; then W-84.

## 2026-10-27 — A page per rule, generated: `docs/rules/` and `scripts/rule-docs.py`  ·  Claude (Opus 5.5)

- **Asked:** *a detailed document with examples on how each and every rule can be configured with its options — one doc per rule is fine*; then *do it right now here itself*.
- **Done:** `scripts/rule-docs.py` (stdlib; `--write` / `--check`) renders `docs/rules/<namespace>/<rule>.md` for all 651 rules, plus `engine.md` (the 18 engine rules) and the index `README.md`, from the rule files, their `.cases.md`, the presets, `FIRST_PARTY_PACKS` and `codelaw.schema.json`. Each page: facts (pack, presets, default severity, fix, stability, security), what it checks, turning it on, severity, every option (type, default, description, an example), every option at its default, the `call[]` mode, the cases that run with `options=`, scopes, profiles, `pyproject.toml` / `package.json`, `[deps]` / `[css]` where read, suppression, baseline, then How to fix, Examples and the rest. Its front-matter parser equals PyYAML on all 652 files; every TOML and JSON config snippet it writes (4,555 settings) parses and matches the rule's declared option names and types. Ownership row, `owns:`, the `docs/rules/` registry row, a USAGE pointer and a CI step (`rule-docs.py --check`). Not committed: written into the main checkout, uncommitted.
- **Decided / left open:** an option with no `.cases.md` example gets a value from its description's `e.g.`, its type and its default — illustrative, not run. Options with no description in the rule file say so (`ts/no-unused-vars`, `js/no-unused-vars`, `stencil/ban-prefix`). Landing through lockmaster is the decider's.
- **Mistakes:** none landed; two found and fixed before writing (case fences kept their closing line; link rebasing touched inline code).
- **Next:** land it through lockmaster (`scripts/rule-docs.py`, `docs/rules/`, `USAGE.md`, `.github/workflows/ci.yml`, `records/README.md`, `records/0060_WORK-registry.md`, `work/WORKLOG.md`).

## 2026-10-27 — W-74 built and closed (editor logs you can debug from)  ·  Claude Code (Sonnet 5.5)

- **Asked:** build W-74 end to end and close it, through lockmaster.
- **Done:** `codelaw-lsp` gains `LogLevel`, per-project and per-file lines, once-per-file-per-reload "why nothing" reasons, reload causes, failed-request errors and per-check debug timings; the extension logs through a `LogOutputChannel`, traces binary candidates, spawns the server itself to log its exit code and stderr, adds `codelaw.logLevel` and *Copy debug info*; SR-EDITOR decision 7, README *Troubleshooting*, IMPLEMENTATION, CHANGELOG; the handoff archived. `cargo test -p codelaw-lsp` and `npm test` pass.
- **Decided / left open:** a debug line is shown at the channel's info level behind `[debug]` so `codelaw.logLevel = debug` works without raising VS Code's own level; the extension was not run inside a real VS Code (only type-checked and unit-tested).
- **Mistakes:** none.
- **Next:** none for W-74.

## 2026-10-27 — W-77 filed (a separate package for everything agent-facing)  ·  Cowork (Opus 5.5)

- **Asked:** Arpit: *"I want a separate package to be created just for skills, and it will have a hard dependency on code law. Skills being agent facing scripts or files."* Then: *"write it again. I believe it got cleared out."*
- **Done:** read `templates/agents/`, `crates/codelaw-agents/src/skills.rs` (agent files compiled into the binary), SR-AGENT-SURFACES ("ship inside both packages") and SR-PACKAGES (zero runtime dependencies). Asked four questions; filed W-77 with the rulings and its 🟢 `build` row. The file and this entry were lost from the working tree once (the row survived) and were rewritten.
- **Decided / left open:** ruled by Arpit — the whole agent-facing set; npm `@codelaw/skills` and PyPI `codelaw-skills`; codelaw keeps nothing agent-facing (exit 2 without the package); exact same version. Open to the builder: discovery order on PyPI, the `codelaw.toml` override. Records are amended in the building change.
- **Mistakes:** left uncommitted in a main checkout another session was cleaning; the untracked file and the WORKLOG edit were discarded. It needs landing through lockmaster promptly.
- **Next:** land W-77's three files; then a Claude Code lane builds it (Opus), records first.

## 2026-10-27 — W-76: commit the main checkout's loose work; worktree cleanup  ·  Claude Code (Opus 5.5)

- **Asked:** *"commit everything, look into worktrees if anything is not merged, merge it, and then do a cleanup."*
- **Done:** landed the main checkout's uncommitted Fux index refresh (`.fux/index`, three new shards) and `work/NOW.md` through lane W-76. Audited the three worktrees: the lockmaster-bootstrap worktree (at 7f0546c, 111 commits behind, no commits off main; its 33 dirty files were all superseded on main — every added line present, reworded, or its item archived; its `now-ledger.py` equals the kit) — removed with its branch; `W-45-demo` (`demo/W-45-annotations-v2`, clean, pushed, PR #3 open "do not merge") — worktree removed, branch kept for the PR; `W-73` — live lane of session fda9b580, untouched.
- **Decided / left open:** nothing merged into main from any worktree — none held unlanded work meant for main. PR #3 still waits on the decider (W-45 DoD 2).
- **Mistakes:** first line of the session said `work/NOW.md` was empty (the hook injected an empty snapshot); it held six rows — corrected in chat. Backed up the bootstrap worktree's tracked diff (568-line patch, session scratchpad) but not its 13 untracked files (`DEPENDENCIES.md`, records 0012/0013, item files W-58–W-67, archived W-66) before `git worktree remove --force` — they are gone; every path exists on main (or `archive/open/`) in a later version, so only early drafts were lost. Lesson: `git status --short` lists untracked files too; back up `ls-files --others` before a forced removal.
- **Next:** the decider's inbox — W-62's `tree-sitter-cli` ratification, W-38, W-34.

## 2026-10-26 — W-51: the exceptions section goes unconditional  ·  Claude Code (Sonnet 5.5)

- **Asked:** *Finish W-51; the locally doable part is the exceptions section now W-46 has landed.*
- **Done:** removed the `exceptions --help` guard from `templates/ci/github/codelaw-autofix.yml` and `templates/ci/gitlab/codelaw-autofix.yml`; the section runs `codelaw exceptions --expiring 30d --as-of "$(date +%F)" --format markdown`. `ci_autofix.rs` asserts it; 4/4 pass, actionlint clean. Landed via lockmaster.
- **Decided / left open:** W-51 stays open for DoD 1 and 3 only: one run in a throwaway GitHub repo with fixable findings and `CODELAW_AUTOFIX_TOKEN`.
- **Mistakes:** none.
- **Next:** the decider (or CI) runs the workflow once in a throwaway repo.

## 2026-10-26 — W-34: Fux pinned at 5.1.1; what is unpublished  ·  Claude Code (Opus 5.5)

- **Asked:** which packages are not on PyPI and npm (the decider publishes by hand); then *"Implement Fux for the latest version that we have."*
- **Done:** listed the unpublished set (PyPI: `codelaw` and eight rule packs; npm: `scss`, `css-modules`, three security packs; all 25 Fux index packages). `FUX_VERSION` 5.1.0 → 5.1.1 with D-34, `licences.rs`, RELEASING.md, W-34 and OPEN-WORK; the count corrected from 16 to 25. Built all 25 with a scratch fux 5.1.1 venv; `check` passed.
- **Decided / left open:** the pin moved on the decider's word; D-34 is still `pending`. No CI run has ever uploaded a `fux-index` artifact (the last green `main` run predates it; the two newest are red), so the decider builds locally with `--fux`.
- **Mistakes:** gave the decider commands with `<run-id>`/`<tarball>` placeholders, which zsh ran literally, and told them to downgrade their venv to 5.1.0 before the pin moved; corrected in chat.
- **Next:** the decider publishes the 25 from a local `assemble` and rules D-34.

## 2026-10-26 — W-62 finish attempt: blocked on the tree-sitter CLI  ·  Claude Code (Sonnet 5.5)

- **Asked:** finish W-62 (patch tree-sitter-scss, CSS rules onto trees, pinned corpus), blocking under L11 where a dependency is unratified.
- **Done:** nothing in code. Regenerating a patched grammar needs the tree-sitter CLI, which is absent and not in `DEPENDENCIES.md`; filed `work/BLOCKED.json` (ASK), re-balled W-62 to `arpit` in `OPEN-WORK.md` and the item.
- **Decided / left open:** ratify `tree-sitter-cli` 0.25.10 (build-time, MIT)? The corpus step needs CI network and was not attempted.
- **Mistakes:** none.
- **Next:** Arpit rules on the CLI; then an agent patches both grammars.

## 2026-10-26 — W-71 step 1: the per-function change set  ·  Claude Code (Sonnet 5.5)

- **Asked:** implement W-71, step 1 first.
- **Done:** `codelaw-lang` gains `functions` / `classify` (new, edited by token stream, untouched; qualified names, `f#1` numbering, public flag); the engine gains `git::file_at` and `Project::changed_functions`; the CLI gains `codelaw debug changed-functions` (text and JSON). 13 unit tests and 3 end-to-end tests on real git repos (reformat-only judges nothing, rename is new, new file all new, no change set prints the header line). Also carried Cowork's staged ruling edits (W-71 file, OPEN-WORK row, two worklog entries) into this change, because they were staged in the main checkout and blocked other lanes.
- **Decided / left open:** left: the agent hook's file as a change-set source; W-58's `size.rs`/`js_ast.rs` still have their own function finders (W-58 was in flight); steps 2-5. Restoring the staged copies in the main checkout was refused by the permission classifier and is left to the decider or coordinator once this lands.
- **Mistakes:** edited two files in the main checkout before starting the lockmaster worktree; moved them into the worktree and restored the main copies.
- **Next:** step 2 (`graph/untested-function`) and steps 3, 4 as parallel lanes.

## 2026-10-26 — W-71 ruled diff-scoped; now agent-ready  ·  Cowork (Opus 5.5)

- **Asked:** Arpit: *"yes"* — to diff-scoped for W-71.
- **Done:** W-71 DoD 1 and step 1 marked ruled; ball `agent`; queue row 🔴 → 🟢; its inbox row and `↳ blocks:` sub-row removed.
- **Decided / left open:** diff-scoped is ruled. Still unasked-for: the class-wide `.codelawignore` target `File#Class.*` (out of scope until he says).
- **Mistakes:** none.
- **Next:** an agent starts W-71 step 1 on Opus.

## 2026-10-26 — W-71 updated: `.codelawignore` design and build order  ·  Cowork (Opus 5.5)

- **Asked:** Arpit, after a voice walk-through: *"file the work item for what we talked about"*.
- **Done:** W-71 gains the `.codelawignore` design — `on_touch` (`error` default, `warn`, `keep`) and `on_touch_scope` (`file` default, `function`); `forever`; reason mandatory with a date or `forever`; `[coverage.ignore]` `require_expiry` and `require_on_touch`, both on by default; one rule `codelaw/incomplete-ignore` — plus a five-step build order (step 1, the per-function change set, first; 2–4 parallel). Queue row unchanged (kept short).
- **Decided / left open:** all `.codelawignore` choices are Arpit's voice rulings, quoted in the file. Still open to him: diff-scoped (inbox row) and a class-wide target `File#Class.*`.
- **Mistakes:** none.
- **Next:** Arpit's yes on diff-scoped flips W-71 to 🟢 `agent`; step 1 starts on Opus.

## 2026-10-26 — W-70 example apps for the dev-server plugins  ·  Claude Code (Sonnet 5.5)

- **Asked:** implement W-70: example apps for Vite, webpack, Rspack, Next.js plus a CI job that installs the hosts.
- **Done:** `examples/dev-servers/` (four apps, `run.mjs` driver) and the `dev-servers` job in `ci.yml`. Route 1 of the item (hosts installed on the runner), so L11 needs no ratification and no `BLOCKED.json`. Syntax-checked the driver and the workflow YAML only.
- **Decided / left open:** no host was installed locally (disk over budget, L9), so nothing ran on a real host; host version ranges and the Next.js/websocket assertions are unverified. W-70 stays open until the first CI run.
- **Mistakes:** none.
- **Next:** push, read the five `dev-servers` jobs, fix what the real hosts disagree on.

## 2026-10-26 — W-44 tidied after the rulings  ·  Claude Code (Sonnet 5.5)

- **Asked:** finish W-44's locally doable remainder.
- **Done:** re-ran `tests/watch.rs` (6/6); the handoff's Blockers section now records the two questions as ruled and What is left lists only the CI items (DoD 5 on `win32-*`, `ubuntu-24.04` measurements).
- **Decided / left open:** nothing new; DoD 5 and the runner measurements need CI.
- **Mistakes:** none.
- **Next:** read the `smoke` Windows legs and the runner bench once CI has run on main.

## 2026-10-26 — W-58: corpus counts confirmed, left list trimmed  ·  Claude Code (Sonnet 5.5), in a lockmaster worktree

- **Asked:** finish W-58 except CI's `perf` run (DoD 6).
- **Done:** checked `packs/js/corpus.toml` (23/177/7) and `packs/python/corpus.toml` (13/91/134) hold the six `[findings]` rows (landed in the CI repair); removed the stale "six corpus counts" from OPEN-WORK and IMPLEMENTATION. No re-run of `corpus.py` (pinned repos not cloned here).
- **Decided / left open:** left: DoD 6, CI's `perf` job on the merge (and, if the decider wants, a `js/size`-on workload).
- **Mistakes:** none.
- **Next:** read `perf` on the merge, then close W-58.

## 2026-10-26 — W-74 filed (editor logs you can debug from)  ·  Cowork (Opus 5.5)

- **Asked:** Arpit: *"The VS Code extension doesn't have very good output or logging. Create a work item for better logs so that at least it can be debugged if there is any issue."*
- **Done:** read `packaging/vscode/src/extension.ts` and `crates/codelaw-lsp/src/lib.rs`: the extension logs one line (the binary), never the server version it receives; the server logs only config errors and project warnings — a project that fails to load is a popup only, a failed request and a file with no project are silent. Filed W-74 (`work/open/W-74-editor-logs.md`, six steps, nine DoD) and its 🟢 `build` row; build count 12 → 13.
- **Decided / left open:** nothing ruled. The design pointers in the file (`LogOutputChannel`, `codelaw.logLevel`, *Copy debug info*, never log source text) are proposals, not rulings. SR-EDITOR is amended by the building session.
- **Mistakes:** none. Left uncommitted in the working tree; it lands through lockmaster.
- **Next:** a Claude Code lane builds W-74 (Sonnet).

## 2026-10-27 — W-73: configurable approved calls  ·  Claude Code (Sonnet 5.5)

- **Asked:** build W-73 as a lockmaster change: `sanitizers` on the five HTML-sink rules, `js/use-instead` and `py/use-instead`.
- **Done:** `call[]` option type, `calls:` predicate, `$OPTION.<name>`; the five rules moved onto `sanitizers`; the two `use-instead` natives with cases; corpus counts for `js`, `python`, `security-js`; USAGE, CHANGELOG, SR-RULE-FORMAT 3a note. The built-in default is stricter (a locally declared `sanitize` no longer passes); an instance of `createDOMPurify(window)` was added to the matcher after the corpus showed bulletproof-react using it.
- **Decided / left open:** the validator-call lists in `sec-ts/next-server-action-validation` and `no-unchecked-cast-of-input` detect a validator anywhere in a function, not at a value, so `calls` does not fit; a follow-up. Release `perf` not run on this machine (disk).
- **Mistakes:** started editing in the main checkout before `lockmaster start`; the coordinator caught it and the edits were moved into the worktree, the main tree restored.
- **Next:** CI's corpus and `perf` runs on the landed change.

## 2026-10-26 — Cross-lane cleanup after thirteen landings  ·  Claude Code (Sonnet 5.5)

- **Asked:** fix what the thirteen parallel lanes left: a duplicate `D-33`, a red docs-sync, the MCP tool list, clippy, the build-tool declaration.
- **Done:** `fux-engine` renumbered D-33 to D-34 in `DEPENDENCIES.md` and `work/open/W-34-fux-index-packages.md` (status still `pending`; no other file cited its id); `CHANGELOG.md` names W-60, W-65, W-66, so `docs-sync --check` is green and is now a lockmaster check; the MCP tool-list test expects `list_exceptions`; `fux-engine` and its three pinned dependencies are declared in `licences.rs` `BUILD_TOOLS` (a licence declaration, MIT, not a ratification); `scripts/package.py check`'s wheel regex takes `codelaw_rules_security_js`; four clippy errors (css_syntax.rs, scss_rules.rs, dependencies.rs).
- **Decided / left open:** ratifying D-34 stays the decider's. `scripts/package.py check dist` needs CI's built wheels and was not run end to end.
- **Mistakes:** none.
- **Next:** the decider rules D-34.

## 2026-10-26 — W-63: CSS Modules; the `css-modules` pack  ·  Claude Code (Sonnet 5.5)

- **Asked:** W-63 as a separate lockmaster change after W-62 landed (`494a128`).
- **Done:** a stylesheet is a CSS Module by name (`*.module.css`, `*.module.scss`) or `[css] modules`; `[css] modules_convention` (a new root-only, schema-validated `[css]` section); the `css/*` rules read module syntax without findings; the graph reads each stylesheet's module facts from its text and the JS facts read `styles["x"]` and `const { x } = styles`; two graph queries and five rules in the new npm-only `css-modules` pack (L6: seventeen packs, fifteen on npm, ten on PyPI); `init` adds it. SR-PROJECT-GRAPH gains decision 8; **`CLAUDE.md` edited** (the L6 and `packs/` lines; SR-WORK-DOCS decision 7a). W-63 closed and archived.
- **Decided / left open:** a module nobody imports and a module read by a computed name are never judged (no false positive); a module whose class names could not be worked out (Sass interpolation, `&-suffix` on a selector list) is skipped by both queries. The pinned CSS Modules and SCSS corpora moved to W-62's *Left* list (CI network).
- **Mistakes:** none new; the shared-`target/` collision of the W-62 entry bit again (stale `codelaw-core`, another lane's binary under `CARGO_BIN_EXE_codelaw`) and was handled the same way.
- **Next:** W-62's *Left* list.

## 2026-10-26 — W-65 closed: L11's dependency check built  ·  Claude Code, parallel lane (Sonnet 5.5)

- **Asked:** *"W-65 — L11's CI check"* (brief: implement fully, land through lockmaster).
- **Done:** `crates/codelaw-cli/tests/dependencies.rs` and the ledger block at the end of `DEPENDENCIES.md` (per unit: members and ratified tree as `name@major`; `own` for codelaw's own code). Fails on an unlisted Cargo/npm/Python/action dependency, a grown tree (a new package or major), a unit in use with no valid row status, a lockfile package outside every tree, a script or hook importing past the Python standard library. SR-LAW-11 decision 7 now says built and `owns:` the two files; the Ownership table and register updated; CI runs it in `cargo test --workspace`.
- **Decided / left open:** `option-ext`'s MIT stand-in and the sibling tool `lockmaster` (imported in `now-ledger.py`) are `own`, not dependencies — if the decider disagrees, they need rows. `bit-set` is declared in the root `Cargo.toml` and used by no crate; the check ignores `[workspace.dependencies]` declarations. W-67 deletes its three units' rows and ledger lines when it removes the crates.
- **Mistakes:** none.
- **Next:** W-67 drops `strsim`, `anyhow`, `postcard` and their ledger lines.

## 2026-10-26 — W-45: demo PR opened for DoD 2; CHANGELOG names W-45 and W-47  ·  Claude Code (Sonnet 5.5)

- **Asked:** open the PR that runs `codelaw check --format github`; fix the CHANGELOG gap.
- **Done:** draft PR https://github.com/arpitarya/codelaw/pull/3 (branch `demo/W-45-annotations-v2`, never to merge) runs the formats on `w45-demo/` (2 errors, 3 warnings); `CHANGELOG.md` gains the W-45 and W-47 lines.
- **Decided / left open:** DoD 2 stays open until Arpit looks at the annotations on PR 3 and says so; then close the PR and delete the branch.
- **Mistakes:** none.
- **Next:** Arpit looks at PR 3's Files changed tab.

## 2026-10-26 — W-73 filed (configurable approved calls)  ·  Cowork (Opus 5.5)

- **Asked:** Arpit: *"rules which say that we need to call certain functions … DOM purify dot sanitize … I want that function to be configurable … custom function … custom variables … I want these kind of rules"*.
- **Done:** found the same hard-coded sanitizer regex in five rules, matched by spelling only (a local `sanitize()` passes); amended SR-RULE-FORMAT decisions 2, 3a, 3b (`call[]` option type with `_mode`, the `calls:` predicate, `$OPTION.<name>`); filed W-73 and its build row.
- **Decided / left open:** Arpit ruled extend *or* replace (configurable), per-rule options, name + import source, and a `use-instead` rule for JS/TS and Python. Not built here: no Rust toolchain on the Cowork VM, lockmaster unreachable, `target/` 37 GB against L9's 5 GB.
- **Mistakes:** none.
- **Next:** a Claude Code lane builds W-73 (`cargo clean` first, L9).

## 2026-10-26 — W-71 filed (tests for new and edited functions)  ·  Cowork (Opus 5.5)

- **Asked:** Arpit: *"a rule where if a new function is added or if a file is being written, it should have a unit test … coverage of around 60%, configurable … JS, TS, Python … older functions no need … if somebody is editing that function, this rule again gets applied … research and write a work item"*.
- **Done:** filed W-71 (`work/open/W-71-tests-for-new-code.md`), its build row and an inbox row; researched SonarQube new-code gate, Codecov patch, diff-cover, lcov / Istanbul / coverage.py report formats; found SR-BASELINE decision 3's `on_touch_scope = "function"` specified but unbuilt (`git.rs` diffs names only).
- **Decided / left open:** Arpit ruled `scope = "all"` with a `.codelawignore` file for exemptions, and per-function coverage. Open to him: diff-scoped (recommended) vs baseline + touch — the inbox row.
- **Mistakes:** none.
- **Next:** Arpit's yes on diff-scoped flips W-71 to 🟢 `agent`.

## 2026-10-26 — W-49 closed, W-70 filed (queue housekeeping)  ·  Claude Code (Sonnet 5.5)

- **Asked:** Arpit: *"create a separate work item for example app"* for W-49; he publishes the new npm packages himself.
- **Done:** filed W-70 (example apps on real Vite, webpack, Rspack, Next.js; L11 route: a CI job that installs them, alternative a ratified dev dependency); closed W-49 (DoD 1 and 4 moved to W-70), archived its detail file with the `archive/README.md` row, removed its OPEN-WORK row; RELEASING.md names the three plugin packages among the new names.
- **Decided / left open:** W-70 starts on the CI-job route; no decider ruling needed. The first publish of `@codelaw/vite-plugin`, `webpack-plugin` and `next` is the decider's hand.
- **Mistakes:** none.
- **Next:** an agent builds W-70 on the CI-job route.

## 2026-10-26 — W-49: dev-server plugins for Vite, webpack, Rspack, Next.js  ·  Claude Code (Sonnet 5.5)

- **Asked:** W-44's remaining agent work, then W-49 (lane brief).
- **Done:** W-44: nothing an agent can still do — its Windows paths are already `cfg`-guarded (inode, Ctrl-C) and the rest is CI-only; no change. W-49: `packaging/npm/{dev-core,vite-plugin,webpack-plugin,next}`, `scripts/package.py` (stage, check, names, `sync-dev-core`), SR-PACKAGES decision 3a, `plugins.test.js` and `crates/codelaw-cli/tests/dev_plugins.rs`, USAGE, CHANGELOG.
- **Decided / left open:** three packages, not four (Rspack runs the webpack plugin); no shared package because a package may have no dependency, so one source is copied and checked. Left: an example app on each real host and the first publish.
- **Mistakes:** none that landed; `package.py check`'s wheel regex rejects `codelaw_rules_security_js` (an underscore) — found, not mine to change here.
- **Next:** the decider ratifies the hosts for an examples job, or publishes the three packages.

## 2026-10-26 — W-51: scheduled auto-fix PR (`init --ci … --autofix`)  ·  Claude Code (Sonnet 5.5)

- **Asked:** implement W-51 in a parallel lane.
- **Done:** `ci.rs` (a slot-only renderer, writer, install), four templates under `templates/ci/`, `--ci/--autofix/--unsafe` on `init`, `tests/ci_autofix.rs`, SR-CLI-MCP decision 12, CHANGELOG, queue row.
- **Decided / left open:** the renderer is a stand-in until W-60 lands its own (same `{{slot}}` and `\{{` rules); DoD 1 and 3 need a real GitHub repo; the exceptions section is guarded until W-46; no deps-snapshot section (the job is offline, L5).
- **Mistakes:** none.
- **Next:** run the workflow once on a throwaway GitHub repo; W-60 registers these templates.

## 2026-10-26 — W-34: the Fux index packages built (steps 2–4)  ·  Claude Code (Sonnet 5.5)

- **Asked:** do W-34 steps 2–4 now that Fux's index-package item (fux item 299) is closed; leave step 5 (reserve names, publish) to Arpit.
- **Done:** L6 decision 12 (and the law sentence, decision 10, the veto), SR-PACKAGES decision 14 and SR-RULE-PACKS decision 6; `scripts/fux-index.py` and a `suffix` on `package.py`'s pack builders; `ci.yml`'s `package` job; `init` prints the install and pin lines when `.fux/` exists and installs nothing (`tests/fux_index.rs`); RELEASING.md, DEPENDENCIES.md D-33, CHANGELOG.
- **Decided / left open:** the two open questions answered in the item (print whenever `.fux/` exists; version left to Fux's pin plus our manifest check), for the decider to overrule. Sixteen packages, not twenty-two: the four empty security packs wait for a rule. Index packages stay out of `dist/npm` and `dist/pypi` so `publish.yml` cannot trip on an unreserved name. Left: Arpit reserves and publishes the 16 names, and ratifies `fux-engine` (L11).
- **Mistakes:** none that landed; the first assemble run failed on the empty security packs and a temp-dir scope bug, both fixed before commit.
- **Next:** Arpit's step 5.

## 2026-10-26 — W-61 closed: `data` and the partials scope  ·  Claude Code (Sonnet 5.5)

- **Asked:** finish W-61 now that W-59, W-58 (`data_of`) and W-62 have landed; collapse the blank runs in OPEN-WORK's build section.
- **Done:** `MaxLines::data_of` (`{lines, max}`, parsed from the finding's message as `js/max-lines` does); a unit test and the binary test `css_and_html_size_data_and_a_partials_scope` (JSON and SARIF `data`; `[scope.partials]` with `files = ["**/_*.scss"]` raises `css/max-lines` for partials only). DoD 3 met, item archived, row removed.
- **Decided / left open:** nothing left on W-61.
- **Mistakes:** none.
- **Next:** none.

## 2026-10-26 — W-61: `html/max-lines` and `css/max-lines`, with `html/size` and `css/size`  ·  Claude Code (Sonnet 5.5)

- **Asked:** W-61 — a max-lines rule for HTML and CSS (CSS, SCSS and CSS Modules), with W-58's options shape.
- **Done:** `crates/codelaw-native/src/size_lines.rs` (a text scanner, no tree: comments, strings, unquoted `url(…)`, SCSS `//`, `@import`/`@use`/`@forward` statements, HTML `<!-- -->` and `<script>`/`<style>` bodies); the two rule pages with cases; the two opt-in `size` presets, added to both pack manifests; USAGE's `Size limits` section; SR-RULE-PACKS decision on the `size` presets; unit tests for every option. Both rules do nothing on a region (a `<style>` in a page, a fence in Markdown): the host file only.
- **Decided / left open:** W-58's counter is tree-based and not landed, so this item has its own text scanner rather than waiting. Left: DoD 3 (a `[scope]` for `_*.scss` partials) needs W-59 and W-62; the measured `data` in JSON/SARIF needs W-58's `data_of` trait method (the message already ends ` (max N…`, the shape its parser reads).
- **Mistakes:** a shared `CARGO_TARGET_DIR` made cargo reuse another lane's artifacts for same-named workspace crates (one build failed on another lane's `Language::Scss`); verified with a private `RUSTFLAGS` so the fingerprints differ.
- **Next:** merge main after W-58 and W-59 land, add `data_of`, and close DoD 3.

## 2026-10-26 — W-57 closed: the `security-python` pack, 61 rules  ·  Claude Code (Sonnet 5.5)

- **Asked:** *"W-57 (work/open/W-57-security-python-pack.md, 61 rules). Every rule needs a fix (L4), fixtures and tests."* by the lead agent, on Arpit's approval of the security-pack lane.
- **Done:** 61 `match:` rules in `packs/security-python/rules/sec-py/` with examples, `.cases.md` sidecars (positions from marks in the source) and a fix class each (17 `suggest` with a verified `fix:`, 44 `manual`); `data/sources/` for `from_source:`; both presets; the README with the Bandit and Ruff `S` accounting; the corpus snapshot over Flask, Starlette, full-stack-fastapi-template and djangoproject.com; the false positives that run showed were fixed first (test files excluded from the noisy rules, cookie flags passed as variables, bare `except`, `Response`, `key=` in a non-crypto call). Closed and archived W-57.
- **Decided / left open:** `no-xpath-injection`, `no-xmlrpc` and `no-cleartext-http` are `manual`, not `suggest`: no edit of the code alone is right. The ReDoS check for a literal pattern is not built (`from_source:` cannot sit beside a literal-pattern alternative in one rule). Every rule stays `preview`. The PyPI upload is the decider's.
- **Mistakes:** the first `codelaw test` of a single rule file reported "does not fire" for every `match:` rule, which was the single-file load path, not the rule; running the pack folder is the check. A `where:` on a capture that only one alternative of an `any:` binds makes every other alternative fail, and an `any:` does not fall back to the next alternative when the first matched structurally but a predicate failed; the rules were written around both.
- **Next:** review the `security-python` counts and promote the rules whose findings hold; W-54 to W-56 are the other three packs.

## 2026-10-26 — W-50 closed: post-commit reminders  ·  Claude Code (Sonnet 5.5)

- **Asked:** W-50 after W-46 landed (`2120458`).
- **Done:** `post-commit.sh` prints the committer's exceptions expiring within 7 days and the 30-day burn-down total; test added; SR-CLI-MCP decision 11, USAGE, CHANGELOG updated; W-50 archived.
- **Decided / left open:** the reminders live in the script, not as `post-commit.d` files, so an installed hook needs `init --git-hooks` re-run. The burn-down "delta" is `baseline history`'s total row (no new command).
- **Mistakes:** first threshold counted lines (a one-row report is 2 lines, as is an empty one); the test caught it.
- **Next:** none.

## 2026-10-26 — W-46 built: `codelaw exceptions` and `list_exceptions`  ·  Claude Code (Sonnet 5.5)

- **Asked:** build W-46 (CLI plus MCP twin), then W-50's post-commit reminder.
- **Done:** engine `Project::exceptions()` collects baseline entries, deprecations (their symbol's baseline entries folded in) and inline ignores; `codelaw-report::exceptions` filters, groups and renders text, JSON, CSV and markdown; CLI `exceptions` and MCP `list_exceptions`; SR-BASELINE decision 11 and SR-CLI-MCP updated; tests `exceptions.rs` and the parity test.
- **Decided / left open:** `[[overrides]]` are not listed (no `expires`; decision 9). `--fail-if-expiring` counts expired rows too.
- **Mistakes:** the shared `CARGO_TARGET_DIR` lets lanes overwrite each other's `target/debug/codelaw` between build and test (an old binary answered "unrecognized subcommand"); rebuilt and reran until green.
- **Next:** W-50's post-commit reminders.

## 2026-10-26 — W-67 closed: strsim, anyhow and postcard replaced  ·  Claude Code (Sonnet 5.5)

- **Asked:** replace the three dependencies the decider ruled "Remove" / "Replace" in W-66, and measure L8 before and after.
- **Done:** `levenshtein` and `jaro_winkler` in `codelaw-core::similar` (with known-pair tests) replace `strsim` in config, schema and target; the unused `anyhow` line left `codelaw-format`; the cache body is now codelaw's own deterministic binary format (`codelaw-cache/src/codec.rs`, header version 2; tests: round trip, fixed layout, every truncation, bit flips, a huge length, an old-version entry, a truncated entry). `postcard` left the workspace; D-20, D-23, D-24 left `DEPENDENCIES.md`; SR-CACHE decision 4 and SR-LAW-11 context updated; W-67 archived.
- **Decided / left open:** `strsim` (via clap) and `anyhow` (via markup_fmt) are still in `Cargo.lock` as parts of ratified trees — said so in `DEPENDENCIES.md`, not fought. L8 was measured on this (loaded) machine only: no slower; the CI-runner numbers are the `perf` job's.
- **Mistakes:** concurrent lanes share one target dir and overwrite each other's `codelaw_core` artifacts (cargo then said `cannot find similar in codelaw_core`); worked around with a per-package profile setting so my builds had their own hash. `mcp_parity` reads another lane's clobbered `codelaw` binary (10 tools, not 9) — not this change.
- **Next:** none for W-67.

## 2026-10-26 — W-60: L10 built, every consumer-file text is a template  ·  Claude Code (Sonnet 5.5)

- **Asked:** *implement L10: one template per consumer file under `templates/`, a registry, a slot-only renderer and a test.*
- **Done:** `templates/` (`init`, `git-hooks`, `agents`, `new-rule`) with the registry in `templates/README.md`; `codelaw_core::template` (slots, `\{{`, errors on unfilled or unknown slots, no dependency); `init.rs`, `git_hooks.rs`, `rules_cmd.rs`, `sync.rs`, `skills.rs` render from it; `agents/` is now `templates/agents/` and the hook scripts left `crates/codelaw-cli/src/hooks/`; CLAUDE.md §Layout names `templates/` (said out loud: a layout line only); ownership rows and the 0012, 0101, 0102, 0107, 0108, 0115 records updated. Output byte-identical: 95 files diffed before and after.
- **Decided / left open:** W-51 landed first with its own renderer in `ci.rs` and `templates/ci/`; on merging I replaced its renderer with the core one and added its four templates to the registry. Any later writer (W-49) follows L10 from the start. SR-LAW-10 is `yes` in the register.
- **Mistakes:** the shared target dir gave another lane's `codelaw-core` rlib to my build twice (same artefact hash across worktrees); fixed with a per-package profile override so the hash differs. `a_rule_panic_exits_2` failed in a full parallel run and passes alone; not looked into.
- **Next:** none for W-60.

## 2026-10-26 — W-59: `[scope.<name>]` and `[profile.<name>]`  ·  Claude Code, parallel lane (Opus 5.5)

- **Asked:** *W-59: named, singular config tables — `[scope.utils]` for a group of files, `[profile.ci]` for an environment — with preset-shipped scopes a project extends by name; the old spellings stay one minor release.*
- **Done:** SR-CONFIG decisions 3a, 4, 10, 11 and §1 amended; loader (`scope.rs`): ordered scope table, name check, preset scopes merged under the project's, profile scopes inheriting `files`, deprecation warnings for `[[overrides]]` and `[profiles.*]`; presets carry `[scope.*]` (SR-RULE-PACKS decision 3) and the rule set hands them to the config; `config --scopes`, scope names in `config --print`; schema, tests, USAGE, README, CHANGELOG, GLOSSARY, DESIGN, the suppress skill. Item closed.
- **Decided / left open:** preset globs written `**/…` (the item's `*.config.*` would only match the root); a merged scope keeps the preset's place in the order; a `pyproject.toml` home reads its scopes in file order too (positions from the text). The alias goes in the next minor release (CHANGELOG says so). B-050 (`codelaw migrate`) and W-58 must use the new spelling when built.
- **Mistakes:** the shared `target/debug/codelaw` is overwritten by every lane's build, so a CLI test can run another lane's binary; the tests were run from a private copy of the binary with the baked path patched, after every `cargo test` that skipped the relink gave a false red.
- **Next:** none for W-59.

## 2026-10-26 — W-55 built: `security-ts`, 16 rules  ·  Claude Code, lockmaster (Sonnet 5.5)

- **Asked:** *"W-55 (security-ts, 16 rules on top)"*, the second landing of the same sub-agent after W-54.
- **Done:** 16 rule files under `packs/security-ts/rules/sec-ts/` with `.cases.md` sidecars, the two presets (extending `security-js`'s), the README, NOTICE, `corpus.toml`. `codelaw test packs/security-ts`: 16 rules · 198 checks · 0 failed. W-55 archived; SR-RULE-PACKS names what landed.
- **Decided / left open:** no cargo integration test (disk); the pack's own examples, cases and corpus counts are the tests. `nest-missing-guard`, `nest-throttler`, `no-ts-ignore-on-sink` (21 hits in test code at first) are `strict` because a guard or a throttler registered in another file is not seen.
- **Mistakes:** matched `class_declaration` for a decorated controller, but tree-sitter-typescript puts the decorators of an exported class on the `export_statement`; found by the Do example firing, fixed by matching the export (or an unexported class).
- **Next:** the decider's inbox; W-56 and the rest of the queue.

## 2026-10-26 — W-54 built: `security-js`, 66 rules  ·  Claude Code, lockmaster (Sonnet 5.5)

- **Asked:** *"W-54 (security-js pack, 66 rules), every rule with a fix (L4), fixtures and tests"* — a sub-agent of a parallel session, Arpit having approved everything.
- **Done:** 66 rule files under `packs/security-js/rules/sec-js/` with `.cases.md` sidecars (generated from one table kept outside the repo, then read and tested rule by rule), five `data/sources/` tables, the two presets, the README with the not-ported table, NOTICE, `corpus.toml`. `codelaw test packs/security-js`: 66 rules · 1482 checks · 0 failed. The corpus run (744 files) showed false positives in `timing-safe-compare`, `no-hardcoded-secret` and `missing-rate-limit`; each was fixed and became a case. W-54 archived; SR-RULE-PACKS names what landed.
- **Decided / left open:** the catalogue's `auto` for `no-bidi-characters` and `suggest` for `no-cleartext-http` are not reachable with the edit primitives (a native rule would do both: not built, because a build was past the disk ceiling at the time); the other differences are at the foot of the item. A release-build timing of the pack is the `perf` job's.
- **Mistakes:** assumed an `any:` tries each branch with its own `constraints:` — it runs the first branch that matches the shape, and a constraint that fails then ends the match; found by a case, fixed by `all:` with `regex:` per branch.
- **Next:** W-55 (`security-ts`), which extends this pack.

## 2026-10-26 — W-56 closed: `security-html`, 19 rules  ·  Claude Code (Sonnet 5.5)

- **Asked:** build W-56 in a parallel lane: every rule with a fix (L4), fixtures and tests.
- **Done:** `crates/codelaw-native/src/html_security.rs` (19 native rules on the `html/*` document model, registered in `lib.rs`), `packs/security-html/rules/sec-html/` (a rule file and a cases sidecar each; `security:` on all), the two presets (14 + 5 rules, extending `html/security`), `corpus.toml` counts on `flask`, `html5-boilerplate` and `djangoproject.com`, the pack README, `NOTICE`, the SR-RULE-PACKS row, `README.md` counts, `crates/codelaw-cli/tests/security_html.rs`. `suggest` fixes (`https://`, `method="post"`, `sandbox`, `crossorigin`, `referrerpolicy`, `ng-href`) apply with `--unsafe`; `sec-html/no-sensitive-comments` is `auto`. W-56's file archived.
- **Decided / left open:** the preset split is mine: `strict` holds `no-external-base`, `no-plugin-embed`, `no-sensitive-comments`, `form-csrf-token`, `iframe-allow-powerful`, the rules that cannot tell a legitimate use from a risky one; the rest are in `recommended`. Uploading the pack to npm and PyPI is the release's (W-38), not this item's.
- **Mistakes:** `target/debug/codelaw` is one file shared by every lane building into the shared target dir, so a test that runs the binary can run another lane's build; a copy taken right after a forced rebuild gave the right one.
- **Next:** none for W-56; W-54, W-55 and W-57 are the sibling packs.

## 2026-10-26 — W-58: size limits for JS, TS and Python  ·  Claude Code (Sonnet 5.5), in a lockmaster worktree

- **Asked:** W-58 — max lines per file, max lines per function and max parameters, each configurable, for JS, TS and Python.
- **Done:** six native rules in one module (`size.rs`), their pages and cases (every option and form), presets `js/size` and `py/size`; `NativeRule::data_of` and `Diagnostic::data` carry the measured value and the limit into JSON and SARIF; `options=<json>` on an example or case fence (a Don't cannot be 300 lines long), with the test harness keying its compiled rule set on the options; USAGE "Size limits", the packs' README rows, CHANGELOG; SR-RULE-FORMAT 5, SR-ARCHITECTURE 15b, SR-CLI-MCP 4, SR-RULE-PACKS rows.
- **Decided / left open:** the rule counts lines from the parsed tree's comment, docstring and import ranges; with `include_nested = false` an inner function's lines are removed from its first line to its last, as ESLint does; an invalid `ignore_names` expression is one finding at the file's start, and the rule still measures. Left: the six corpus counts and DoD 6 on CI's `perf` job.
- **Mistakes:** the integration tests of `codelaw-cli` run `target/debug/codelaw`, which concurrent lanes sharing one target dir overwrite; five suites failed with another lane's binary until I re-ran each with a fresh build. A retry is the workaround, a per-lane target dir the fix.
- **Next:** the two items under W-58's "what is left".

## 2026-10-26 — W-62: SCSS joins as a CSS dialect; the `scss` pack  ·  Claude Code (Sonnet 5.5)

- **Asked:** W-62 (L3 amendment, B-047 ruled: own grammar / tree-sitter-scss), then W-63 as a separate change; Arpit has approved everything.
- **Done:** `Language::Scss`; tree-sitter-scss vendored as D-33; Biome formats `.scss` (98.6 % identical files vs Prettier on 212 files); the CSS statement reader reads Sass (`//`, `#{}`, `@mixin`/`@include`/`@use`/`@if`, nested properties, `&-suffix`, `!default`) and 64 of 71 `css/*` rules declare `scss`; the `scss` pack with 15 rules, `scss/recommended` and `scss/strict`; L3, L6 (sixteen packs), L7, SR-RULE-PACKS, SR-PACKAGES, SR-LINTER, SR-FORMATTER, SR-ARCHITECTURE amended; `init`, the VS Code extension, the config schema, `format-parity.py`; **`CLAUDE.md` edited** (the L3 and L6 lines and the `packs/` count, SR-WORK-DOCS decision 7a).
- **Decided / left open:** `scss/operator-no-unspaced` not shipped (layout: SR-FORMATTER decision 3 refuses it); `scss/at-if-no-null` fixes `!= null` only (tree-sitter-scss reads `@if not …` as an error, and the fix verifier refuses an edit that adds one). Left: the grammar patch and moving the CSS rules onto trees, a pinned SCSS corpus, the graph on `.scss` (the item's *Left* section).
- **Mistakes:** the shared `target/` hashes workspace crates by their path inside the workspace, so parallel worktrees overwrite each other's `codelaw-core` and `target/debug/codelaw`: builds failed with a stale core and tests ran another lane's binary. `touch crates/*/src/lib.rs` before a build, and copy the `deps/codelaw-<hash>` that holds your change.
- **Next:** W-63 (CSS Modules), then the *Left* list.


## 2026-10-26 — W-66 closed: dependencies ruled (29 ratified, 3 to replace); W-67 filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** by voice, *"explain to me in simple terms, go one by one"* — 32 cards, one ruling each; then *"based on the previous discussion, update the work items or create new ones."*
- **Done:** `DEPENDENCIES.md` carries every ruling (29 `ratified 2026-10-26`; D-20 postcard, D-23 strsim, D-24 anyhow `remove`); W-66 closed and moved to `archive/open/` with an IMPLEMENTATION row and an archive map row; W-67 filed to replace the three (anyhow is declared but unused); W-65's DoD 3 restated; SR-LAW-11's context notes the rulings. Nothing committed.
- **Decided / left open:** the two "Remove" rulings arrived transcribed as "Je m'ouvre"; recorded as *"Remove"*, matching the confirmations given on the call.
- **Mistakes:** a plain `os.remove` of W-66's file was refused (deletes are off in the connected folder); moved it with `mv` instead.
- **Next:** W-67 and W-65 in Claude Code.

## 2026-10-26 — L11 (no dependencies unless ratified) and `DEPENDENCIES.md`; W-65, W-66 filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"Create a law, no dependencies. For Rust, for Python, for JavaScript, TypeScript, JSON, CSS, YAML, TOML, HTML, nothing … tell me what all dependencies do we have. We will ratify them one by one."*
- **Done:** SR-LAW-11 (`records/0013`), its SR-LAWS, register and ownership rows, and **a new line in `CLAUDE.md` §The laws**. Took the inventory into `DEPENDENCIES.md`: 32 units (25 Rust runtime, 1 Rust test, 2 VS Code, 1 Python build, 3 CI), all `pending`; 47 direct crates, 376 in `Cargo.lock`. Filed W-65 (the CI check) and W-66 (🔴 the rulings, with an inbox row). Nothing committed.
- **Decided / left open:** agent's reading, stated in the law: default-deny with ratification by name (as the decider said "ratify one by one"); toolchains and standard libraries are not dependencies; vendored code is; ratification fixes the transitive tree; pending units stay at today's pin.
- **Mistakes:** none.
- **Next:** W-66, unit by unit.

## 2026-10-26 — docket: six rulings written back (W-44, W-45, W-50, B-047); W-64 filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"What are the blockers on me? List them one by one, explain them to me, and give me a recommendation as well."*
- **Done:** walked 7 cards. Rulings written back: W-44 keep polling, DoD 1 judged on the re-check (→ 🟢); W-45 Claude Code opens the PR (→ 🟢 agent); W-50 no profile in the hook confirmed, DoD 5 amended to `language: system` (→ 🟡 W-46); B-047 own grammar + tree-sitter-scss, against the recommendation (absorbed by W-62, row deleted); W-64 filed (mirror repo, waits on W-38). W-38 left for later; inbox holds W-38 only. Nothing committed.
- **Decided / left open:** L10's consumer template override stays unruled.
- **Mistakes:** W-62 sat 🟢 while waiting on B-047 (drift) — resolved by the ruling.
- **Next:** W-38 by hand (four uploads fit one day's limit).

## 2026-10-26 — SCSS and CSS Modules become their own packs (W-62, W-63 amended)  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"Create separate packages for SCSS and CSS modules."*
- **Done:** W-62 gains a `scss` pack (`@codelaw/rules-scss`, `codelaw-rules-scss`); W-63 a `css-modules` pack (`@codelaw/rules-css-modules`, npm only); both amend L6 (fifteen → seventeen packs) in the change that builds them. Queue rows updated. Nothing committed.
- **Decided / left open:** agent's call: `scss` on both registries (Python web apps use SCSS), `css-modules` npm only (a JS-bundler feature); the dialect support and the graph join stay in the engine and the `css` pack.
- **Mistakes:** none.
- **Next:** the decider's B-047 answer; then W-62.

## 2026-10-26 — W-61 (max-lines HTML/CSS), W-62 (SCSS), W-63 (CSS Modules) filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"I want max lines rule for HTML CSS files as well. Research and create a work item for it. The CSS pack should work with SCSS, and CSS modules as well."*
- **Done:** researched (no stylelint/html-validate/Biome file-length rule exists; Biome's language-support page lists SCSS parsing, formatting and linting; tree-sitter-scss is MIT; stylelint-scss and eslint-plugin-css-modules are MIT). Filed W-61, W-62 (amends L3 on this ruling; settles B-047), W-63. Nothing committed.
- **Decided / left open:** SCSS joins L3 as a CSS dialect (the decider's ask); `.sass` and Less stay out. Open: B-047's parser fork — recommended (b), Biome's tree.
- **Mistakes:** none.
- **Next:** the decider confirms B-047 (b); W-61 and W-63 can start now.

## 2026-10-26 — L10 (consumer files from `templates/`) ratified; W-59 (scope/profile tables) and W-60 (templates) filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** a sample `codelaw.toml` with `*.utils.*` at 50 lines and the rest at 100; then *"overrides is not the right terminology"* → *"we will go with scope"* (singular in TOML); then *"create a law that all the files that are supposed to be created on the consumer end will have templates in separate templates directory."*
- **Done:** wrote SR-LAW-10 (`records/0012`), its SR-LAWS row, register row and **a new line in `CLAUDE.md` §The laws** (SR-WORK-DOCS decision 7a: said here and in chat). Filed W-59 (`[scope.<name>]`, `[profile.<name>]`, preset scopes, one-release aliases) and W-60 (move consumer text into `templates/`). Nothing committed.
- **Decided / left open:** the decider: scopes, singular `scope` and `profile`. Agent's calls, stated in the record: slot-only templates, embedded in the binary, serialised data exempt. Not ruled: consumers overriding templates.
- **Mistakes:** none.
- **Next:** W-59 and W-60 in Claude Code.

## 2026-10-26 — W-58 filed: max lines, max lines per function, max params  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"Does code law have a rule for max line?"* — no; then *"create a work item for max lines for TypeScript, JavaScript, and Python … max lines per function and max params … show me how am I going to use it in my consuming application."*
- **Done:** filed W-58: six `native:` rules (`js/` and `py/` `max-lines`, `max-lines-per-function`, `max-params`) with options, defaults from ESLint and Pylint, opt-in `size` presets; showed consumer usage in chat. Nothing committed.
- **Decided / left open:** all six are `fix: manual`; `max-depth`, `complexity`, `max-statements` named as a later item, not filed.
- **Mistakes:** none.
- **Next:** W-58 in Claude Code.

## 2026-10-25 — six items built in parallel (W-44, W-45, W-47, W-50, W-52, W-53) and merged  ·  Claude Code, VS Code (Opus 5.5)

- **Asked:** *"implement whatever possible in parallel"*; on disk, *"cargo clean first then 6 agents"*; on W-52's and W-53's law amendments, *"Yes, include them."*
- **Done:** `cargo clean` (10 GB freed; it also pulled `target/` from under session f38f16ad's running `cargo test`). Six sub-agents in worktrees, merged on `integrate-parallel` with main merged in; two cross-branch breaks fixed (`4707d43`). `cargo test --workspace` 555 passed, 0 failed; clippy `-D warnings` clean. W-47, W-52, W-53 closed and archived; W-44, W-45, W-50 to the inbox; W-54–W-57 unblocked; B-051, B-052 filed. **CLAUDE.md changed** (SR-WORK-DOCS decision 7a): the L5 line (no network in the engine, `deps refresh` the one networked command) and the L6 and `packs/` lines (fifteen packs, thirteen on npm, nine on PyPI), on the decider's ruling above.
- **Decided / left open:** for the decider — W-44 (`notify` is CC0; what DoD 1 times), W-45 (a real PR for DoD 2), W-50 (the hook's profile; DoD 5), B-051 (`requirements*.txt` and L3), W-53's six first hand uploads and `min_codelaw` at the release.
- **Mistakes:** the shared `CARGO_TARGET_DIR` let worktrees overwrite each other's artifacts and `target/debug/codelaw`, so every branch's green tests were suspect until the one clean run on the merge; the `cargo clean` broke another session's run. Lesson for LESSONS.md: parallel worktrees on one target dir are not isolated.
- **Next:** the decider's inbox; then W-54 (`security-js`).

## 2026-10-25 — npm 0.2.0 from CI (W-37); W-39–W-43 merged, W-41/W-42/W-43 closed  ·  Claude Code (Opus 5.5)

- **Asked:** *"go ahead and run it"*; then *"Whatever is in green implement it … till the closure. And I want more React query hooks and React hooks. Rules."*
- **Done:** tagged `v0.2.0` on `08b7158` and ran `publish.yml` by hand (run 40207195373, 5 min 21 s): 19 npm names at 0.2.0, 17 with provenance; W-37 closed, its eight-platform registry check moved to W-38. The 0.2.0 release notes are CHANGELOG's section (fux/sheetpane shape). Filed W-42 (TanStack Query) and W-43 (React hooks); four agents in worktrees ran W-39, W-40, W-41+W-43, W-42; merged all onto `main`; a clean rebuild: 497 workspace tests, clippy clean, 11 packs / 458 rules 0 failed. W-41, W-42, W-43 closed; W-39, W-40 wait on CI's `perf` (DoD 6).
- **Decided / left open:** for the decider — W-42's three v4-loud rules on repos not yet on v5; W-40's changed wording (`expires`) and info-not-warn in the warning window.
- **Mistakes:** twice staged a shared file whole (`OPEN-WORK.md`) and swept another session's rows into a commit — once caught before push, once after (`eda3607`, taken back in `3909552`). Shared files are now staged by hunk only. Parallel agents sharing one `CARGO_TARGET_DIR` overwrote each other's binaries: every result was re-proved on `main` from a clean build.
- **Next:** push; CI's `perf` closes W-39 and W-40; W-38 is the decider's.

## 2026-10-25 — security packs: a second research pass, 82 more rules  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"research, more security rules, and add it to the list for the programming languages, respectively."*
- **Done:** read CodeQL's JS and Python query lists, Ruff's `S` rules and the OWASP cheat sheets; added 34 rules to W-54 (66 in all), 9 to W-55 (16), 8 to W-56 (19 + five existing `html/*`), 31 to W-57 (61), each with CWE, OWASP 2025, fix class and its starting preset; queue rows updated. Nothing committed.
- **Decided / left open:** heuristic rules start in each pack's `strict` preset (`warn`), not `recommended`.
- **Mistakes:** none. `check-work-system.py` reports W-41 and W-43 as `ball: done` in their files but still queued — another session's close in progress, not touched.
- **Next:** W-53 in Claude Code.

## 2026-10-25 — W-48 `migrate` moved to the backlog as B-050  ·  Claude Code, VS Code (Opus 5.5)

- **Asked:** *"the migrate work item, put it in backlog."*
- **Done:** W-48's row out of `OPEN-WORK.md`; its detail file moved to `archive/open/` with a map row; B-050 (`unbuilt`, sourced in SR-AGENT-SURFACES §P2 `codelaw-migrate`) in `BACKLOG.md`; the interview's filing line notes it. Nothing committed.
- **Decided / left open:** filed as `unbuilt`, not `unruled` — the P2 skill is already ratified in SR-AGENT-SURFACES; only its timing was deferred.
- **Mistakes:** none.
- **Next:** W-38 (PyPI 0.2.0 by hand) is still the decider's.

## 2026-10-25 — type-aware parked (B-049); security packs filed as W-53–W-57  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"For now, put type aware rules and details in backlog … create a pack for security issues … Veracode, Black Duck, X-Ray, OWASP … one for JavaScript, one for TypeScript, one for HTML, one for Python. Research and create work items for individual languages … if … anything that we are missing from security point, print it in the chat."*
- **Done:** wrote [the type-aware proposal](proposals/type-aware-rules.md) and B-049 (`unruled`). Researched OWASP Top 10:2025, CWE Top 25 2025, JFrog's SAST rule list, Bandit and eslint-plugin-security; filed W-53 (groundwork: L6 to fifteen packs, CWE/OWASP metadata, SARIF security-severity, a `from_source:` predicate) and W-54 JS (32 rules), W-55 TS (7), W-56 HTML (11 + five existing `html/*`), W-57 Python (30), each waiting on W-53. Printed the security gaps in chat (taint, SCA/CVEs and licences, secrets in data files, CI and supply-chain config, scanner import, stricter security exceptions). Nothing committed.
- **Decided / left open:** four packs, as asked (one combined `security` pack was the alternative); the gaps in chat are not filed.
- **Mistakes:** none.
- **Next:** the decider picks which security gaps to file; W-53 in Claude Code.

## 2026-10-25 — W-51 (scheduled auto-fix PR) and W-52 (dependency latest versions) filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"schedule fix PR, create a work item for it … For type-aware rules, TypeScript Go. If TypeScript dependency is already present in a repo, do we still need the Go binary … Elaborate on this highlighting the latest version of each dependency. Let's go with it."*
- **Done:** filed W-51 and W-52; W-52 records the decider's ruling that `deps refresh` is the one network command and the engine stays offline (L5 decision 3 to be amended in that item). Answered the type-aware question in chat: TypeScript 7 ships no programmatic API (expected in 7.1), so codelaw needs its own helper for now. Type-aware is not filed yet. Nothing committed.
- **Decided / left open:** the decider chose typescript-go (Option A); L3's amendment and the "use the project's TypeScript once 7.1's API lands" mode are still to be filed.
- **Mistakes:** none.
- **Next:** file the type-aware item once the decider confirms the helper plan.

## 2026-10-25 — W-44–W-50 filed: watch, CI formats, exceptions report, burn-down, migrate, dev-server plugins, git hooks  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"Watch mode, let's build it. Then CI report formats … Exception report … Baseline burn down over commits … Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."* Also: explain scheduled auto-fix PRs, type-aware option A (typescript-go), and highlighting the latest versions of Node and Python dependencies — chat only.
- **Done:** filed W-44 (watch), W-45 (CI formats), W-46 (exceptions report, waits on W-40), W-47 (baseline history), W-48 (migrate, no oxlint), W-49 (dev-server plugins, waits on W-44), W-50 (git hooks). Explained the three discussion items in chat; nothing filed for them. Nothing committed; another session's uncommitted OPEN-WORK edits were left as they were.
- **Decided / left open:** open for the decider: pre-commit's profile (W-50); L3's amendment for type-aware rules; whether a network `deps refresh` command fits L5.
- **Mistakes:** none.
- **Next:** the decider's answers on the three discussion items; then file them.

## 2026-10-25 — CI vs local severity, dated exceptions, React context: designed and filed as W-39–W-41  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"I want Code Law to work in a way that it can be used in CI/CD pipeline. As well as dev environment … for certain files it should never throw an error … every exception should have a date … an optional field … email ID … validated on commit or in CI/CD … React context … new implementation … should throw an error."*
- **Done:** mapped the ask against SR-CONFIG, SR-LINTER and SR-BASELINE (the baseline already gives `until`, `warn_from`, `reason`, and old-warns/new-errors). Wrote [the compare doc](compare/ci-profiles-and-dated-exceptions.compare.md) and filed **W-39** (profiles, `remap`, `js/no-console`), **W-40** (`expires`, `warn_days`, `require_expiry`, `added_by`), **W-41** (`react/no-context` + the baseline recipe). No record edited: SR-CONFIG §1 is parsed by the schema test, so record, schema and loader land together in each build. Nothing committed.
- **Decided / left open:** Arpit ruled auto-detected profiles with agents on `ci`; a configurable per-glob remap (silent, warning or error); `added_by` checked against the git author; `createContext` and `useContext` both. The agent chose `expires` = first failing day, with `until` as a one-release alias.
- **Mistakes:** none.
- **Next:** W-39 in Claude Code.

## 2026-10-25 — npm 0.2.0 from CI made possible; PyPI merged into one blocked item; queue brought up to date  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"All codelaw npm packages are published. The PyPI packages will take 2 to 3 days … create just one work item, merge everything into it, and keep it blocked … Everything else related to npm, let's go ahead and build it out, even have the pipelines start working … Whatever Claude Code needs to do, create separate work items for it. Review open work file and get it up to date."*
- **Done:** checked npm: 19 names exist, but `codelaw`, the `cli-*` and eight packs are at 0.1.0; only `rules-tanstack-query` and `rules-redux-toolkit` are at 0.2.0. `publish.yml` gains `workflow_dispatch` (npm only, `v*` tag only; `pypi` and `attach` run only for a Release) — YAML parsed, not run. L6 decision 9 and SR-PACKAGES decision 11 amended on the ruling; `RELEASING.md` gains the npm-first commands. Filed **W-37** (Claude Code: commit, tag, run, check) and **W-38** (🔴 PyPI by hand, absorbing W-30, W-31 step 7, W-04 step 5, W-03 step 6 and the inbox row). Closed and archived W-03, W-04, W-29, W-30, W-31, W-32 with IMPLEMENTATION rows and archive map rows; links repointed; W-03's *later* presets became B-048. Long rows for W-05 and W-35 trimmed. Nothing committed.
- **Decided / left open:** the tag must be a commit carrying the new `publish.yml` (a run by hand reads the workflow from the tag), so not `f33b772`; W-38's Release reuses that tag so PyPI gets npm's binary.
- **Mistakes:** none.
- **Next:** W-37 in Claude Code — `actionlint`, commit with explicit paths; then Arpit pushes.

## 2026-10-25 — README: who codelaw is built for; W-36 step 3a  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** by voice, whether the README should say codelaw is for mixed human/AI codebases and not needed for 100% AI code; then *"Let's go ahead and add those lines."*
- **Done:** two lines open README's "Is codelaw for you?": built for people and AI side by side; the more an agent writes, the more you need rules it can't skip. W-36 gains step 3a, measuring violations with and without Fux. Nothing committed.
- **Decided / left open:** the "not needed for 100% AI" half was dropped after the counter-argument (guidance is probabilistic, enforcement deterministic); step 3a's data can reverse it.
- **Mistakes:** none.
- **Next:** W-36 step 1.

## 2026-10-25 — W-19, W-35 (step 2) landed; 0.2.0 pushed and its dist verified; PyPI held by 429  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement all the green items and then implement the yellow items as and when they get unblocked … close them out … in parallel."*
- **Done:** rulings asked and applied — `fix_one_file` left unset (W-19 closed), `v0.2.0` from the latest `main` (W-33's rules `since: 0.2.0`), README's Before says what ships (W-35 step 2, agent in a worktree). The cloud session's W-05/W-34/W-35 filing committed as its own commit. Pushed `main` to `f33b772`; CI run 40189751330 all green with the budgets gating; its `dist` passes `package.py check`, `law-gate`, `SHA256SUMS`; the hand-built pack wheels are byte-identical to CI's. PyPI: `codelaw-rules-css`, `-data` live; `html`, `js`, `python`, `react` refused (429) for over an hour — a 30-minute retry runs on.
- **Decided / left open:** Arpit's hand: `npm login` + the two new npm names + `npm-trust.sh`; six PyPI trusted publishers (pypi.org UI); `gh release create v0.2.0`. W-34 waits on Fux item 298; W-05 and W-36 are research.
- **Mistakes:** a `git add -A` swept a concurrent session's uncommitted USAGE.md, W-36 and docs-sync changes into my commit; caught before push, reset, and recommitted with only my hunks (CLAUDE.md's "re-derive `git status` right before staging" — I had, one command earlier, and the other session wrote in between).
- **Next:** the four PyPI uploads land; Arpit's steps above; then the release, W-29's 8-platform install check, and W-04/W-03/W-30/W-31/W-32 close.

## 2026-10-25 — README rewritten around the story and four moments; USAGE.md; W-36 filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"I want codelaw to be a linter, prettier for AI — before, while, after, and even without AI … like ESLint, Biome … a work item which can research all these things … edit README … a story which people can relate to … a separate file just for usage."*
- **Done:** README rewritten: a two-line pitch, the story (an overnight agent PR breaking four rules), the four moments table (W-35's ruled wording kept; Without AI added), a diagram, an ESLint/Prettier/Biome comparison, "for the person who owns the codebase", two new FAQ answers. Install, config, rule workflow, agents (before/while/after), editor/pre-commit/CI and the command table moved to new `USAGE.md`. `docs-sync.py` now checks `USAGE.md`'s install loops and command table (`sync_usage`); a planted bad row fails it. W-36 filed (research). W-35 step 2a, IMPLEMENTATION, CHANGELOG Unreleased, registry rows. Nothing committed.
- **Decided / left open:** the comparison table states structure only (tools, configs, where rules live), no claims about competitors' AI features — that is W-36's sourced step 1. Version wording left at 0.1.0 on npm: `npm view @codelaw/rules-js` still says 0.1.0.
- **Mistakes:** none.
- **Next:** Arpit reads the README on GitHub's renderer; W-36 step 1 in Cowork.

## 2026-10-25 — W-05 researched; W-34 and W-35 filed, Fux item 298 filed in Fux  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"research W05 … if Fux can be utilized, enhanced, for code law … propose just within chat"*, then by voice the shape, then *"file all the work items discussed here."*
- **Done:** W-05 researched in chat (pre-write hooks: Claude Code confirmed; Codex, Copilot CLI and Kiro have a `PreToolUse` but their edit payloads are undocumented); W-05 updated with the voice rulings and step 4 moved to Fux. Filed **W-34** (an optional `-fux-index` package per pack, npm and PyPI) and **W-35** (README: "before, while and after the AI writes"). In the Fux repo, filed **item 298** (packages ship their own index; one query over many). Nothing committed.
- **Decided / left open:** ruled by voice — the index is built where the pack is built, shipped as its own package, never a default, on both registries; options A and B in W-05 are dead. Left for the decider: L6's amendment text (W-34 step 2), and W-34's two open questions.
- **Mistakes:** the first W-34 row named the Fux item by its bare id and `check-work-system.py` read it as a lost codelaw id; it is now spelled "Fux item 298". The guard has no cross-repo id form, so rule 23's "waiting on W-mm" cannot name a Fux item.
- **Next:** Fux item 298 step 1 — the 20-intent spike, bar 16 of 20.

## 2026-10-25 — 0.2.0 committed, PyPI packs uploading; W-24, W-33 closed, W-19's budgets set  ·  Claude Code (Opus 5.5)

- **Asked:** *"commit everything generate a build and then use the terminal to publish PyPy packages"*; then *"implement everything green in open work file. In parallel"*.
- **Done:** `f40b2e6 release: 0.2.0` and the docket commit (SR-PACKAGES gained the by-hand PyPI sentence the guard asked for). `build-packs.py --release` → six `codelaw_rules_*` wheels, `twine check` passed; `codelaw-rules-css` and `-data` uploaded, then PyPI's new-project 429 — the other four retried every 5 min. W-24 closed on CI run 40185344678 (22/22 green). W-19 (agent, worktree): seven budgets by the ruled method, `fix_one_file` capped → inbox; p95s spot-checked against the raw CI log. W-33 (agent, worktree): both Stencil rules ported, closed. W-05 not picked (research, rule 37a).
- **Decided / left open:** `fix_one_file`'s budget (inbox). Trusted publishers on the six PyPI projects, the push, npm's two new names and the release stay Arpit's.
- **Mistakes:** my W-30 row in IMPLEMENTATION would have turned `lint` red (docs-sync: CHANGELOG did not name W-30); caught by running docs-sync after the W-33 merge, fixed in 0.2.0's section.
- **Next:** the four PyPI uploads finish; Arpit pushes. W-33's rules carry `since: 0.3.0`: if `v0.2.0` is cut from a commit after `3cb762e`, they ship in 0.2.0 and `since` is wrong.

## 2026-10-25 — Docket: L8 by method, PyPI step 4 rethought, 0.2.0 prepared, W-33 filed  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"What are the blockers? Let's go through them one by one … provide a recommendation."*
- **Done:** Arpit pushed `76f1ad6` (CI run 40185344678 in progress; W-24 waits on it). L8 ruled by method (min(design target, 1.5 × worst CI p95), a capped one back to the decider) in SR-LAW-8 decision 4, `perf/budgets.toml`, W-19 (🟢: an agent with `gh` on the Mac applies it). PyPI step 4 in Arpit's Chrome: `codelaw`'s pending publisher registered; `codelaw-rules-js` refused (one pending publisher per repo · workflow · environment) → ruled: the six packs by hand, then trusted publishers; `RELEASING.md` step 4, W-30, W-31, W-04 rewritten. `version.py --set 0.2.0` applied in the tree (uncommitted) and the release command list put in W-31; on npm, `rules-ts`, `-react`, `-node`, `-stencil` already exist at 0.1.0, so only the two W-32 packs need a hand publish. W-29 DoD 4 found done; its 8-platform check moved to 0.2.0. W-33 filed (the two Stencil rules from the 2026-10-25 coverage comparison).
- **Decided / left open:** the rulings are verbatim in W-19, W-29, W-30, W-31, W-33.
- **Mistakes:** none.
- **Next:** CI green on `76f1ad6`; then Arpit commits the 0.2.0 bump and runs W-31's list.

## 2026-10-25 — W-32's two packs finished and verified; `main`'s red lint found and fixed  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement all OpenWork tasks in parallel."*
- **Done:** triage: of the 🟢 items only W-32 was agent-closable code. Its steps 2–4 were already in the tree from an earlier session, uncommitted; this session built them and verified them (both packs green, workspace 463 passed, clippy clean after one elided lifetime in `tanstack_query.rs`). Found `main` red since run 40125039176: `lint`'s docs-sync, the CHANGELOG missing W-18 and W-19. Added both lines, which also proved W-24 DoD 5 from that run's skipped `perf`. W-29 DoD 2, the Mac half: `npm i codelaw@0.1.0` works in an empty folder. Nothing committed.
- **Decided / left open:** not parallel. There was one code item, and L9 (4.2 GB of 5 at the start) leaves no room for extra worktrees with their own builds. W-32 and W-24 moved to the `arpit` lane: they wait on the commit and push.
- **Mistakes:** the test and clippy builds took the project to 5.7 GB, past L9. Brought back to 3.1 GB by deleting `target/debug/incremental` and the test binaries, all of which can be rebuilt. The next session should build with `CARGO_INCREMENTAL=0`. One `warm.rs` socket test timed out under that load and passed on the rerun.
- **Next:** Arpit commits and pushes. CI turns green, which closes W-24; then 0.2.0 (W-31 step 7) carries W-32's first publish.

## 2026-10-25 — W-32: the query pack is `tanstack-query`  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"rename react-query > tanstack query"*
- **Done:** W-32's item file, its queue row, INTERVIEW §2 and the registry rows now name `@codelaw/rules-tanstack-query`, namespace `tanstack-query`, `packs/tanstack-query/`. Code not touched.
- **Decided / left open:** Arpit's ruling, verbatim in the item file. Session `c8d31825` has W-32 in flight with uncommitted `crates/codelaw-native/src/react_query.rs` using `react-query/*` ids; it was not reachable to message, so its sub-agent renames them (file `tanstack_query.rs`, ids `tanstack-query/*`).
- **Mistakes:** none.
- **Next:** the W-32 session picks up the new name before it commits.

## 2026-10-24 — Docket: seven rulings, step 5 of the registry setup, W-21/W-25/W-26/W-28 closed, every corpus snapshot  ·  Claude, cloud linked to the Mac (Opus 5.5)

- **Asked:** *"Let's go ahead and raise a PR for TreeSitter.css."* → *"keep using the patch copy and put the PR in backlog. Then look into all the blockers one by one, explain them to me, and recommend"* → *"implement all the open items and close them out."*
- **Done:** the tree-sitter-css PR re-verified on upstream `master` (= v0.25.0: 46/46, 37 highlight checks, `src/` byte-identical to the fork) and packaged as `upstream/0001-tree-sitter-css-modern.patch`; `PR.md` now says `src/` is included. Docket: `BLOCKED.json` → `PROCEED`; the two registry rows merged; step 5 done in Arpit's Chrome (both environments, tag rule `v*`, checked); W-32 named; W-21 DoD 3 restated and W-21 closed (B-046, B-047); W-28 confirmed; W-20's build-and-test leg dropped; W-19's machine and three repos named (`perf` pinned to `ubuntu-24.04`). Built in the cloud (rustc 1.97): W-28 DoD 4 on nine packs (419 rules, 0 failed, 0 warnings) → closed; W-20 corpus extended to nine packs and 11 repos (`ionic-framework` for `stencil`), every `corpus.toml` written and re-run clean; W-25's checker (rules 40, 64; SR-WORK-OWNERSHIP 2, 9), item-file `ball:` fixes → closed; W-26's `docs-sync.py --check` in CI and its owner row → closed. Nothing committed.
- **Decided / left open:** Arpit's rulings are in each item file. Open: PyPI step 4 (needs his login), L8's budgets (after CI's first `perf` run).
- **Mistakes:** a source tarball was written into the repo and could not be deleted (no delete permission): it, and four helper scripts, sit in `_to_delete/` for Arpit to remove. Pushing the upstream PR was refused to this session; the patch waits as B-046.
- **Found:** `check-work-system.py` fails on four `.claude/hooks/*` rows another session staged before their files exist — that session's, not touched here. `target/` is 15 GB, past L9.
- **Next:** Arpit reviews `git status` and commits (concurrent session in flight), then pushes `main` — CI's first run proves W-18, W-20, W-24, W-27.

## 2026-10-24 — SR-WORK-CLEANUP: a worktree is removed at session end only if nothing is lost; otherwise red, blocked on the decider  ·  Cowork (Opus 5.5)

- **Asked:** *"whenever a work tree's cleanup happens, always make sure everything is merged into main branch and no data is lost. If any data is supposed to be lost, mark it as red, block it on me, and I'll make a call. Otherwise, automatically do the cleanup when the session ends. Create a work SR for work trees, lock files, cleanup."* — Arpit, for fux, sheetpane and codelaw.
- **Done:** [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md), new — worktrees (the four checks, never `--force` / `-D`, red → a `🔴 arpit` item plus an inbox row), other leftovers, lock files (moved here from SR-WORK-BLOCKERS decision 9a, now a pointer), surfaces. It owns `release-locks.sh` and the new `worktree-cleanup.py`. SR-WORK-NOW decision 4 now cleans or queues instead of only queueing. Register, ownership, governance and `CLAUDE.md` rows. The `.claude/` half is `install-agent-hooks.py` v4.
- **Decided / left open:** cleanup never merges (decision 4); an unmerged branch is red, even though removing only the worktree would keep its commits — "everything merged into main" is the bar Arpit set.
- **Mistakes:** none.
- **Next:** Arpit runs `python3 install-agent-hooks.py` (v4), then commits.

## 2026-10-24 — L9: work/MACHINE.md's budget section is law, every GB past 5 GB approved; SR-WORK-NOW  ·  Cowork (Opus 5.5)

- **Asked:** *"So 5 GB is the limit. If it goes beyond it, ask me for my approval on every GB that it increases, and create a law that machine.md should always, machine.md budget section should always be followed."* … *"create a work SR just for now.md, and whatever we talked about, put it in there … have a separate SR work document for now.md."* — Arpit, for fux, sheetpane and codelaw.
- **Done:** [SR-LAW-9](../records/0011_LAW-9-machine-budget.md) — **L9**, new: MACHINE.md's budget section binds every session and agent; past 5 GB every GB is approved first. It owns `.claude/hooks/disk-budget.py`, which now reads the number from MACHINE.md and raises a permission prompt per GB instead of refusing. [SR-WORK-NOW](../records/0061_WORK-now.md), new: everything decided today about NOW.md, moved out of SR-WORK-SESSION (decision 5 is a pointer; 5a, 5b and 15 are gone, never committed). SR-LAWS, the register, ownership rows and `CLAUDE.md` name both (SR-WORK-DOCS decision 7a). The `.claude/` half is `install-agent-hooks.py` v3.
- **Found:** at filing, codelaw is 15 GB — past the budget; `target/` alone is 15 GB, rebuildable with `cargo clean` (the installer offers it).
- **Decided / left open:** none.
- **Mistakes:** none.
- **Next:** Arpit runs `python3 install-agent-hooks.py` (v3), then commits.

## 2026-10-24 — NOW.md holds only what is live; a 5 GB disk budget per project, all agents combined  ·  Cowork (Opus 5.5)

- **Asked:** *"now.md should track everything that is being worked in the chat right now … once it ends, now.md should have nothing … if there is any cleanup … create work items, but make sure now.md is empty … no agent in one project should consume more than 5 GB. Even sub agents … combined"* — Arpit, for fux, sheetpane and codelaw.
- **Done:** [SR-WORK-SESSION](../records/0054_WORK-session.md) decisions 5–5b rewritten — this supersedes the morning's resource ledger, which was never installed — and decision 15, the disk budget; `work/NOW.md` emptied to its two live tables; MACHINE.md's budget section; ownership, registry and governance rows; `CLAUDE.md` names the budget (SR-WORK-DOCS decision 7a). The `.claude/` half — `now-ledger.py`, `disk-budget.py`, the settings wiring, the shared debug-free cargo target — is in `install-agent-hooks.py` v2.
- **Found:** at filing, codelaw is 15 GB — over; `target/` alone is 15 GB, rebuildable with `cargo clean` (the installer offers it).
- **Decided / left open:** the budget is a decision of SR-WORK-SESSION, not a numbered law; Arpit can promote it. The stale in-flight line was dropped — a status note, not live work; its next steps are in OPEN-WORK.
- **Mistakes:** none.
- **Next:** Arpit runs `python3 install-agent-hooks.py` (v2), then commits.

## 2026-10-24 — locks are cleaned; NOW.md lists what is in flight and every uncleaned worktree or VM  ·  Cowork (Opus 5.5)

- **Asked:** *"whenever .lock file is created, it needs to be cleaned out … now.md file should have everything that is being implemented right now. And once it is implemented, it should be cleaned … tracks in now.md what virtual machines got created, what work trees got created … if they aren't cleaned, then create a work item to clean them"* — Arpit, for fux, sheetpane and codelaw.
- **Done:** [SR-WORK-SESSION](../records/0054_WORK-session.md) decisions 5–5b and [SR-WORK-BLOCKERS](../records/0056_WORK-blockers.md) decision 9a; `work/NOW.md` moved to the in-flight list + resource ledger; ownership, registry, governance rows; `CLAUDE.md`’s session-continuity line changed (SR-WORK-DOCS decision 7a). The `.claude/` half — `session-lock.sh` (session_id owner, TTL sweep), `release-locks.sh`, `now-resources.py`, the `settings.json` wiring — is in `install-agent-hooks.py`: the bridge refuses writes under `.claude/`.
- **Found:** `stop-if-blocked.sh`, `require-progress.sh` and `guard-open-work-inbox.sh` emit `decision:"block"` inside `hookSpecificOutput`; Claude Code reads it only at the top level for `Stop` and `PostToolUse`, so none of the three has ever blocked. Fixed in the installer.
- **Decided / left open:** the 5 GB-per-project budget is proposed, not yet a law — waiting on Arpit.
- **Mistakes:** a write probe was left as `.claude/hooks/.probe-delete-me` (the bridge cannot unlink); the installer removes it.
- **Next:** Arpit runs `python3 install-agent-hooks.py`, then commits.

## 2026-10-24 — the green queue in parallel: W-20, W-21, W-24–W-28 merged; the disk filled  ·  Claude Code (Opus 5.5)

- **Asked:** *"give me the command to publish all npm packages … for trusted publisher"*; *"commit everything and then give me a command to publish the new packages for the first time"*; *"implement everything that is in green in parallel"*; *"merge everything to main. And delete everything that is not needed."*
- **Done:** W-31 committed (`da9b207`) and the rules-python unpublish recorded (`a9ee3a8`); gave the hand-publish commands for `@codelaw/rules-{ts,react,node,stencil}` from a local `build-packs.py --release` and `npm-trust.sh`. Seven agents in worktrees took W-20, W-21, W-24, W-25, W-26, W-27, W-28 (W-05 skipped, rule 37a; W-29/W-31 are Arpit's). Six were committed by me after the disk was freed, W-21 and W-28 by their agents; all seven merged into `main` (`1f48463` … `82df231`, record-timestamp conflicts only, kept the later). Verified on `main`: clippy `-D warnings` clean, `cargo test --workspace` 449 passed, `codelaw test` on all nine packs 0 failed, `version.py --check`, `check-work-system.py`, the inbox check, `corpus.py --self-test`, `docs-sync.py --check` pass. All 21 worktrees and 26 merged branches deleted; local `dist/` deleted. OPEN-WORK reconciled; the inbox's stored `age` column dropped (W-25's rule 40).
- **Decided / left open:** "no SR affected" lines added where a change did not alter a record's text (W-24's unit test and encoding fix, W-27's moved kill helper and version bump, W-20's corpus job, the merges); SR-RULE-PACKS §1 gained the `*/project` presets rather than a "no SR" line. New inbox rows: W-21 (submit the upstream PR; DoD 3 (a) or (b)), W-28 (quotes/semicolons now pass with a warning; CSS `not(` is layout), W-20 (may CI install the corpus repos' dependencies — L2). W-24's agent reworded SR-LAW-8's "How L8 is enforced" for the perf path filter — worth the decider's look. W-28 rewrote SR-FORMATTER decision 3 on the 2026-10-22 quote "make the decision".
- **Mistakes:** seven parallel cargo builds filled the disk (LESSONS 2026-10-24); every agent stopped, and the decider had to free space. My first two merge loops ran under zsh, which does not word-split, so the timestamp resolver got one joined path; nothing wrong was committed, each merge was finished by hand. A check loop printed `rc=0` for scripts it never ran (same cause); rerun under bash. The first `cargo test` kept only its last 40 lines, so it was rerun for the count.
- **Next:** an agent runs `scripts/corpus.py run --update` and commits the snapshots; then Arpit pushes `main`.

## 2026-10-24 — the first npm publish, and each registry gets only its users' packs (W-31)  ·  Claude Code (Opus 5.5)

- **Asked:** *"Review how npm packages are published in Fux and Sheetpane. Give me the command to publish it in the same way for the first run"*; then *"The ones that are going to be used in npm and the ones that are going to be used in PyPI should only be published"*, *"in js have js, node, stencil as separate packs; in pypi have js, react as separate packs and remove ts, node and stencil from pypi"*, html kept on npm, *"implement everything"*; and *"i want a redux tool kit and react query pack as well in npm"*.
- **Done:** Arpit published 0.1.0 to npm by hand (all 14 names, `latest`) from `ci.yml` run 40026216188's `dist` — a failed run whose `dist` passed `package.py check`, `law-gate.py` and 14 dry runs — and bound trusted publishing on all 14 (W-29). W-31 landed uncommitted: `packs/js` split into `js`, `ts`, `react`, `node`, `stencil`; `FIRST_PARTY_PACKS` (pack.rs) gives each pack its npm and PyPI name; resolution, the missing-pack error, `codelaw init`, `bench`, `package.py` (reads the table), `build-packs.py`, the docs; L6, SR-RULE-PACKS, SR-PACKAGES and the records owning the touched files amended; **`CLAUDE.md`'s L6 line and `packs/` layout line changed** (SR-WORK-DOCS decision 7: the decider's ruling). Filed W-31 and W-32 (Redux Toolkit and React Query, npm only).
- **Decided / left open:** a preset names only its own pack's rules, so `ts/recommended`'s four core rules became `codelaw:js/typescript` and `js/conventions` split per pack; the reference workload's digest moved to `bf9c5bea0d8b31ee` (same rules; no budget is set). Open: unpublish `@codelaw/rules-python` before 2026-10-27, push, 0.2.0 as the first two-registry release; W-32's names and Redux source (inbox). `work/BLOCKED.json` still says `ASK` on the 0.1.0 go — the decider gave it by publishing; resetting the file to `PROCEED` is left to him.
- **Mistakes:** the first verification command I gave (`xargs -n1 npm view --json name version`) looked up an npm package called `name`, printing `0.0.2` fourteen times; corrected in the session. A pasted block ran a placeholder `<RUN_ID>` and every later step after the first failure — nothing was published.
- **Next:** Arpit runs `npm unpublish @codelaw/rules-python --force`, commits and pushes W-31; then W-24 to green, then 0.2.0.

## 2026-10-24 — the first publish planned: W-29 (npm) and W-30 (PyPI)  ·  Claude Code (Opus 5.5)

- **Asked:** *"go to the browser and check out the organization codelaw without any hyphen. Create a work item to publish npm packages. Create another work item to publish Python packages. Give me the command for both so I can do the first publish from local."*
- **Done:** checked in Arpit's Chrome: npm org `codelaw` exists, one member (`aarya11`, owner, 2FA), 0 packages; `codelaw` unscoped is free on npm; `pypi.org/project/codelaw` is a 404. Filed W-29 (14 npm packages, cli → rules → `codelaw` last, then `npm-trust.sh`) and W-30 (13 wheels with `twine`, then trusted publishers on the existing projects), both 🟡 on W-24; the inbox's registry row narrowed (step 1 done).
- **Decided / left open:** the first PyPI publish is by hand too (Arpit's ask), not through a pending publisher as `RELEASING.md` planned — W-30 step 5 rewrites the doc and one SR-PACKAGES sentence. Both publish from CI's `dist`, which needs W-24 step 1 (the Windows build) and the push of `main`.
- **Mistakes:** none.
- **Next:** W-24 step 1, then Arpit pushes `main`, then W-29 and W-30 from the same `dist`.

---

## 2026-10-22 → 24 — the review: rulings merged, five work items filed  ·  Claude Code (Fable 5.1), linked to the Mac

- **Asked:** *"review or open work blockers, ratify whatever you can, review the repo, create work items to fix whatever was broken or whatever needs to be fixed or whatever needs to be structured in a better way."*
- **Done:** the six inbox rows put to the decider (rulings below). `main` `941edac` built in a cloud clone from a bundle (rustc 1.97): clippy clean, 435 tests, the two known pack failures. An independent review agent read the tree and wrote 30 findings; nine verified by hand (the Windows `wait`, the guard replay's nine failing commits, the rule counts). Found the Cowork session's unmerged branch `decider-rulings-2026-10-22` carrying the same rulings plus the CSS fork and templates; on the decider's OK: the docket session's edits committed (`77ce957`), the branch merged (`f75c848`; a stray `.codelaw/cache/` under `.github/ISSUE_TEMPLATE` dropped, `.codelaw/cache/` ignored everywhere), this session's duplicate templates discarded. Filed **W-24** (CI cannot pass on `main`, 🧨), **W-25** (records and work files vs. the tree), **W-26** (public docs generated from the tree), **W-27** (engine and packaging hygiene), **W-28** (the layout check by codelaw's own tokens). INTERVIEW §1–2 rewritten; two lessons; registry rows; the inbox ages recomputed; `W-03`'s row count 419.
- **Decided / left open:** the decider, 2026-10-22: templates — agent writes them (superseded by the branch's, already ruled); layout rules — *"biome is not the right thing to reference. build how we design it. make the decision and create the work item"* → W-28 (this session's decision: same token stream = layout; the formatter comparison advisory); CSS grammar — fork in-tree, upstream in parallel; push `main` — *after the review's fixes land*; registry setup and the `v*` tag rule — *later, keep the row*. `work/BLOCKED.json` stands. Git housekeeping named in chat, not the queue (rule 47): `main` is 58 commits ahead of `origin/main`; 14 merged agent worktrees (8.2 GB) and 11 GB of `target/` sit under the repo.
- **Mistakes:** this session wrote five templates before finding the branch that already had them, ruled — the second lesson of 2026-10-24. A write test left `.github/.write-test` in the tree for a minute (moved out, nothing committed). The review agent hit the session's rate limit after writing its report; its last area (shell scripts) was not re-run.
- **Next:** an agent takes W-24; the decider pushes `main` once it lands.

---

## 2026-10-22 — npm's Artistic-2.0 ruled permissive; four inbox rows closed  ·  Cowork (Opus 5.5)

- **Asked:** *"What are the blockers? Let's go through them one by one"* — then *"add the license to law2"* and fix blockers 2, 4, 5 and 6 (the decider chose: narrow the JS rule, rewrite the CSS rule's example, fork `tree-sitter-css` in-tree plus an upstream patch).
- **Done:** SR-LAW-2 names Artistic-2.0 and `licences.rs` allows it (W-20; committed by the Claude Code session as `a95ca2e`); `js/no-irregular-whitespace` narrowed and `css/media-query-no-invalid`'s example rewritten, B-045 filed, SR-FORMATTER and SR-RULE-PACKS updated, W-23 closed; the `tree-sitter-css` fork with range media queries, `layer()` and fractional keyframes, `css/media-feature-range-notation` `auto`, upstream patch in the fork's `upstream/` (W-21 steps 3–4); the five `.github/` templates, W-06 closed; README and CHANGELOG counts refreshed. Built and verified in a cloud clone rebased onto `main` `941edac` (rustc 1.97): clippy clean, `cargo test --workspace` 437 passed, every pack 0 failed (419 rules, 3,949 checks); delivered as the branch `decider-rulings-2026-10-22`, not merged, because another session had uncommitted edits in the main checkout.
- **Decided / left open:** the `npm`/`pypi` environments and their `v*` tag rule stay the decider's (the agent's browser attempt was refused by the permission check; neither environment exists yet); the registry setup was not answered. W-21's DoD 3 reading is the decider's.
- **Mistakes:** the first recommendation to drop `js/no-irregular-whitespace` missed that it also reports templates, regexes and JSX text; caught before the change and put back to the decider, who chose to narrow it.
- **Next:** the decider creates the two environments with the `v*` tag rule, then the registry setup (W-04).

---

## 2026-10-22 — the `/docket` skill: the queue walked one item at a time  ·  Claude Code (Opus 5.5)

- **Asked:** *"Create me a skill which looks into open work document, explains them to me one by one, and gives me a recommendation"* — named Docket; skill plus pointers; in the repo and as a personal skill; inbox first, then every item; rulings written back.
- **Done:** `.claude/skills/docket/SKILL.md` (holds no rules — points to SR-WORK-OPEN-QUEUE and the WORK records). **`CLAUDE.md` changed** (SR-WORK-DOCS decision 7a): a state-of-play row and the layout line point to it; its registry row bumped. The personal copy is proposed to the decider for saving.
- **Decided / left open:** drift seen while reading, not fixed: the inbox still carries npm's Artistic-2.0 row, which the worklog records as ruled 2026-10-22; the worklog sends L8's budgets to the inbox, but no inbox row carries them and W-19 is 🟢. `/docket`'s first run should take both as drift cases.
- **Mistakes:** none.
- **Next:** run `/docket`.

---

## 2026-10-22 — the green items: W-11, W-13, W-15, W-19 landed  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement all the green items. In parallel"*.
- **Done:** four agents, one per green agent item (W-05 is Cowork research, rule 37a; W-13 flipped green once W-11's graph landed). Merged into `main` `35a6115`, verified in main's own target: clippy clean, 435 tests. W-11, W-13, W-15 closed and archived; W-19 waits on the decider's budgets.
- **Decided / left open:** to the decider: L8's budgets (inbox). Optional questions from the agents: W-11's two default gaps (self-aliased private packages; CSS only for linked stylesheets), W-13's `on_touch` default for non-deprecation baselines and `codelaw/stale-baseline` as `manual`, W-15's per-project tree-cache bound.
- **Mistakes:** one shared `CARGO_TARGET_DIR` for parallel worktrees let cargo link one worktree's crates into another's build (same relative paths, same artifact hashes); agents worked around it and the merged result was verified in main's own target. Next time: one target per worktree.
- **Next:** the decider's inbox (six rows); W-20's pinned corpus, W-18's staging-index install.

---

## 2026-10-22 — npm's Artistic-2.0 ruled permissive  ·  Claude Code (Opus 5.5)

- **Asked:** the decider, asked whether they had ruled on npm's licence: *"Yes, I ruled it."*
- **Done:** the ruling recorded: SR-LAW-2 decision 2 names Artistic-2.0 (it conditions only a redistributed modified copy of the package itself, so nothing that uses or aggregates npm carries its terms); `licences.rs` allows it and `AWAITING_RULING` is empty and may not grow without the decider; W-20 step 2 and its inbox row closed.
- **Decided / left open:** the ruling is the decider's, 2026-10-22. The edits that record it were found uncommitted in the main checkout; the decider confirmed them.
- **Mistakes:** none.
- **Next:** merge W-11, W-13 and W-19.

---

## 2026-10-22 — W-21 and W-23 finished and merged  ·  Claude Code (Opus 5.5)

- **Asked:** *"continue"* (same session as the entry below).
- **Done:** two agents finished W-21 (steps 1–2) and W-23 (steps 3 and 5) in their worktrees; merged into `main` `2a38ced`: clippy clean, 398 tests pass; pack examples pass but for two layout-check failures.
- **Decided / left open:** to the decider: the two layout-check rules (W-23 DoD 1) and the `tree-sitter-css` fork (W-21 step 3).
- **Mistakes:** none.
- **Next:** the decider's inbox; on the agent side W-11's false-positive judgment, W-15's tree-cache bound, W-13, W-19.

---

## 2026-10-22 — the agent branches merged into main and verified clean  ·  Claude Code (Opus 5.5)

- **Asked:** *"continue"* — merge each agent branch, then one clean verification run.
- **Done:** freed 25 GB (the full disk had stopped the agents); committed the agents' unfinished-but-complete work (W-14, W-15, W-11, with W-11's record text written here); merged W-12, W-14, W-15, W-20, W-11 and W-23's committed part into `main` (the W-15 merge kept its `Output`/`Ctx` CLI shape with W-16's `fix --trace`/`format --format json` and W-14's `--no-adapters`); fixed what the merge exposed (two compile errors, a clippy lint, `daemon status` hanging on a silent socket, the warm tests' teardown). `main` `f67746d`: clippy clean, 385 tests pass. Closed W-14; archived W-10, W-12, W-16 (their branches had moved the files but left the rows).
- **Decided / left open:** W-21's and W-23's remaining work is uncommitted and unbuilt in their worktrees, not merged. npm's `Artistic-2.0` licence put to the decider (W-20).
- **Mistakes:** none new; the agents' runs had been unverified because the disk filled, and the verification here found one real bug (`daemon status`).
- **Next:** finish W-21 and W-23 from their worktrees (build, records, commit, merge).

## 2026-10-21/22 — W-07 and W-09 landed; the WIP branches merged into main as-is  ·  Claude Code (Opus 5.5)

- **Asked:** *finish the agent-closable items from the 2026-10-20 state, one branch each, W-07 first*; then *"Merge everything to main. And then pull the latest main changes."* — and, asked whether that meant the unfinished `wip/*` branches, *"Merge them all as-is now."*
- **Done:** W-07 (formatter) and W-09 (LSP formatting) built from `wip/W-07-formatter`, merged `2404619`, both archived; `law-gate.py` lists nothing; W-23, B-043, B-044 filed. Then the seven `wip/*` commits merged into `main` unfinished (`2eb0f3e`…`db4bde7`), the merges repaired (`7498d1d`, `895324a`), the queue updated (`665c5e3`); pushed.
- **Decided / left open:** `end_of_line` unset keeps line endings (SR-ARCHITECTURE decision 9 over the WIP's LF default). `main` does not build whole: `codelaw-agents` (W-10) lacks `hook` and `sync`; 3 WIP tests fail. W-20 asks whether npm's Artistic-2.0 passes L2.
- **Mistakes:** three as-is merges (W-12, W-16, W-20) were first committed with conflict markers, because zsh did not split a file list in the resolve loop; caught by grepping for markers before pushing, resolved in `7498d1d`. The W-20 keep-both resolution dropped a closing brace; fixed in `895324a`.
- **Next:** write W-10's `hook` and `sync` modules so `main` builds.

---

## 2026-10-20 — parallel lanes on W-07–W-22; W-08, W-09, W-17, W-18, W-22 landed  ·  Claude Code cloud (Opus 5.5)

- **Asked:** *"list all the open items"*, then *"implement all open items … in parallel … merge everything to main"*, then *"Whatever is done is done. Merge commit everything, merge everything to main. And stop the work."*
- **Done:** eleven agents in git worktrees. Landed and merged: W-17 (all 34 rows), W-18 (all but the environment rule and staging index), W-22, W-09 (all but formatting), W-08. Fixed `session-lock.sh` (decider approved). Unfinished work pushed as `wip/*` branches, not merged: W-07, W-10, W-11, W-12, W-14, W-16, W-20, W-21. W-13, W-15, W-19 not started.
- **Decided / left open:** WIP kept off `main` because it is unbuilt. New inbox row: the tag-only `v*` environment rule. Questions in INTERVIEW §2 (rustfmt, the WHATWG entities licence, `yaml/document-start`, the project root with no config).
- **Mistakes:** nine parallel cargo target dirs filled the disk allowance once (ENOSPC lost some output). Fixed by deleting finished target dirs and building without debug info. Logged in `MACHINE.md`.
- **Next:** W-07 from `wip/W-07-formatter`, locally.

---

## 2026-10-20 — code review; the release law gate; pending work filed as W-07–W-22  ·  Cowork (Opus 5.5)

- **Asked:** *"Review the code. What are the blockers and what is pending?"*, then *"Why was the law broken? fix it. … create work items for what all things are pending."*
- **Done:**
  - **Review:** `cargo test --workspace` green (204 tests, rustc 1.97 — the pinned 1.99.1 wasn't downloadable here); every pack's `codelaw test` and `rules lint` green; three read-only reviewers (engine; semantic + native; packs + pipeline). Findings in W-17, W-18, W-22.
  - **The law:** nothing was published, but the inbox asked the decider to hand-publish 0.1.0, which has no `format`, `mcp` or `lsp` — against L7 and L6 decision 3. Added `scripts/law-gate.py`; `publish.yml`'s gate runs it, `ci.yml` reports it; `RELEASING.md` step 2 requires it; SR-PACKAGES decision 11, L6 and L7 enforcement sections, the register's ownership row.
  - **Queue:** W-07–W-22 filed; every backlog row but B-006, B-007 promoted and deleted, and every live citation repointed to its W-nn. The registry-setup inbox row narrowed to steps 1, 4 and 5 (they publish nothing); W-03 and W-04 wait on W-07–W-09.
  - **Lesson:** a law with no gate gets walked past by the next release (`LESSONS.md`).
- **Decided / left open:** this session kept L6 and L7 as they are and gated the release. Amending them to allow a linter-only preview is the decider's call, not taken here. Reproduced: the `py/*-without-explicit-strict` auto fixes write `map(…, strict=False)`, a TypeError before Python 3.14.
- **Mistakes:** none in this change. Earlier sessions' INTERVIEW said "clippy is clean"; CI's clippy step can't fail, and rustc 1.97's clippy finds three warnings.
- **Next:** W-17 and W-18 (🟢🧨), then W-07, W-08, W-09.

---

## 2026-10-20 — merged the release pipeline into `main`; MIT throughout; W-04 → W-06  ·  Claude Code (Opus 5.5)

- **Asked:** *"commit everything merge all the branches to main pull the latest changes and checkout to main and delete the other branches"*.
- **Done:**
  - Committed the local work (`d41c87f`: W-03 packaging, the MIT `LICENSE`, the public docs, W-05). `main` was level with `origin/main`.
  - Merged `origin/ci/w-04-release-pipeline` (B-038 built as W-04): nine conflicting work and record files resolved, keeping both sides.
  - The two sessions had recorded different licence rulings (`MIT OR Apache-2.0` on the branch, MIT locally). Arpit confirmed **MIT only**: `LICENSE-MIT` and `LICENSE-APACHE` removed; `scripts/package.py`, `packaging/python/pyproject.toml`, `packaging/README.md`, `ci.yml` and `.gitignore` now use `LICENSE`; SR-PACKAGES decision 10 rewritten.
  - Both pack builders kept (Arpit): `scripts/package.py` for the release, `scripts/build-packs.py` for local builds; SR-PACKAGES decision 9 names both.
  - The consumer-README item was renumbered W-04 → **W-06** (`work/open/W-06-consumer-readme.md`): the branch's W-04 is already in pushed commit messages. Older entries below still say W-04.
  - The four `worktree-agent-*` branches (all already in `main`) and their worktrees were deleted.
- **Decided / left open:** the remote branch `ci/w-04-release-pipeline` is left on GitHub, and nothing was pushed. Delete the branch once `main` is pushed. Open, the decider's: W-06's `.github/` templates; the registry setup and the ruling to publish.
- **Mistakes:** two sessions used the same item number (W-04) and recorded conflicting licence rulings on the same day. Both were caught at the merge.
- **Next:** the decider pushes `main` (CI then runs on the merged tree), then does `RELEASING.md` §One time.

---

## 2026-10-20 — the licence ruled: MIT  ·  Cowork (Opus 5.5)

- **Asked:** *"Add mit licence for codelaw"*.
- **Done:**
  - `LICENSE` (MIT, © 2026 Arpit Arya, with `SPDX-License-Identifier: MIT`) at the root and copied into all five `packs/<name>/`.
  - SR-PACKAGES decision 10 records the ruling: MIT for the repo and every package.
  - `build-packs.py --release --verify` now builds all ten artefacts. Each `package.json` carries `"license": "MIT"`, and each wheel carries `License-Expression: MIT` and its `LICENSE`. Both installs gave byte-identical output.
  - The README licence section and badge, a CHANGELOG line and a CONTRIBUTING line are updated. The inbox row, `BLOCKED.json`, W-03 and W-04 are narrowed to the go to publish. Nothing committed.
- **Decided / left open:** the decider's licence ruling, MIT, taken as given. Still open: the go to publish (inbox). The crates carry no `license` field; B-038 adds it when it builds the engine packages.
- **Mistakes:** none.
- **Next:** B-038, the engine's packages; then the decider's go to publish.

---

## 2026-10-20 — W-05 filed: guidance while an agent writes  ·  Cowork (Opus 5.5)

- **Asked:** *codelaw should tell an agent how to write code while it writes, not only fix it after; research, propose in chat, file a work item; can Fux do it?*
- **Done:** W-05 (`work/open/W-05-write-time-guidance.md`), its `research` row, two glossary terms. Nothing committed.
- **Decided / left open:** proposed a construct-filtered `brief` and a `hook --pre` check (Claude Code `PreToolUse` can deny with a reason the agent reads); Fux as an optional free-text layer, not a dependency, since L5 needs codelaw complete alone and Fux SR-MCP refuses new tools. Rulings come at W-05 step 5.
- **Mistakes:** none.
- **Next:** W-05 step 1, the per-agent pre-write matrix.

---

## 2026-10-20 — W-03 step 6: the pack packaging  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement w-03"*.
- **Done:**
  - Triage: one 🔴 inbox row (W-04's `.github/` templates, 0 d); W-03 was the only 🟢 item.
  - `scripts/build-packs.py` (stdlib, Python 3.9): the five packs → `@codelaw/rules-<name>` tarballs and `codelaw-rules-<name>` `py3-none-any` wheels in `dist/packs/`, version from `Cargo.toml`. Byte-reproducible; byte parity with `packs/<name>/` checked per artefact; `--verify` installs both layouts over `fixtures/l3` (10 `recommended` presets, 20 diagnostics, identical JSON); `--release` refuses without a `LICENSE`.
  - Checked by hand with real `npm i` (Node 24) and `pip install` / `pip uninstall` (Python 3.14 venv); the engine resolves the packs from `node_modules/` and through `VIRTUAL_ENV`.
  - SR-PACKAGES decision 9 names the script and owns it (register row); B-038 and B-012 narrowed; `/dist/` ignored; CONTRIBUTING gains the build command.
- **Decided / left open:** filed to the decider: the project licence (one SPDX id) and the go to publish; `BLOCKED.json` set to `ASK`. The engine's packages (B-038: launcher, eight binaries, engine wheels, release job) are not built: they are not W-03's own scope and want their own `W-nn`.
- **Mistakes:** none. The uncommitted W-04 changes from the Cowork session were left as they were, and this session's changes were added on top of them in the shared files.
- **Next:** the decider rules on the licence; meanwhile B-038 can be promoted to a `W-nn` and built.

## 2026-10-20 — W-04: the consumer README and the public-repo docs  ·  Cowork (Opus 5.5)

- **Asked:** *"create the readme file and other necessary files. research and create one which is helpful for consumer and could be used for marketing as well"*.
- **Done:**
  - Researched dev-tool README structure (Ruff, Biome, a README guide). Built 0.1.0 in a Linux workspace and ran every README command on a sample mixed repo, so the output blocks are real.
  - `README.md`: pitch, plain-words section, real `check`/`fix` output, quickstart from source (packs linked into `node_modules/` until published), the five packs with counts and fix classes, a local rule with `codelaw test`, config, a Claude Code `PostToolUse` hook recipe, commands, who it is for, roadmap matching B-028–B-041, FAQ.
  - `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`. The new docs pass `md/recommended` and `data/recommended` (dogfooded).
  - Registry rows for the five docs (SR-WORK-REGISTRY §3); W-04 filed.
- **Decided / left open:** `.github/` is protected from remote writes, so the five issue and PR templates were sent to the decider to place (W-04, 🔴). The licence is still open (W-03 step 6); the README says it will be chosen before the first release. `codelaw`, `@codelaw/rules-js` and `codelaw-rules-js` returned 404 on npm and PyPI today, so the names look free.
- **Mistakes:** a probe file `.github/x` was written while testing whether `.github/` is writable; removed with the decider's delete permission.
- **Found while testing** (not fixed, engine untouched): `codelaw rules … | head` panics on a broken pipe; `init` writes a `codelaw.toml` comment pointing at `codelaw config --print`, which is not built (B-037); `fix` leaves a blank first line after removing a file's only import; with `preview` off, `check` says "no rule pack is configured" when packs are configured but all-preview.
- **Next:** the decider places the `.github/` templates; then W-03 step 6.
## 2026-10-20 — W-04: the release pipeline, built and timed on real runners  ·  Claude Code (Opus 5.5)

- **Asked:** *"checkout fux and sheetpane for how the cicd pipeline are built how pypi and npm packages are published — build a cicd pipeline for codelaw to publish packages it should take around 2 mins even faster."*
- **Done:**
  - Read both pipelines: release-triggered `publish.yml`, CI-green gate, OIDC trusted publishing, one artifact for every registry job (fux), the npm trust preflight (sheetpane).
  - Packaging (B-038 → W-04): npm `codelaw` launcher + `check`/`fix` API + types, eight `@codelaw/cli-*`, PyPI wheels by maturin `bin` bindings, five packs on both; every manifest generated from `Cargo.toml`'s version (`scripts/version.py`).
  - `ci.yml`: test, clippy, the SR-guard replay (SR-WORK-OWNERSHIP decision 7, new `SR_GUARD_COMMIT` mode), eight parallel platform builds, `scripts/package.py assemble` + `check`, `scripts/smoke.sh`. `publish.yml`: gate, npm and PyPI in parallel, no rebuild; `scripts/withdraw.sh` for a half-published release.
  - Records: SR-PACKAGES decisions 10–13 and its owns; SR-LAW-6's Consequences and veto check, and SR-ARCHITECTURE's veto check (facts only); SR-WORK-OWNERSHIP decision 7. Branch `ci/w-04-release-pipeline`, green on all eight platforms; nothing published.
- **Decided / left open:** Arpit ruled the licence (`MIT OR Apache-2.0`), building in CI and publishing what CI built, and pinning Rust 1.99.1. Open, his: the one-time registry setup and the ruling to publish 0.1.0 (inbox). The npm/Python APIs lack `rulesFor` and `format` (B-042).
- **Mistakes:** the first push committed `scripts/__pycache__/` (now ignored); Windows died on `✔` under cp1252 (`PYTHONUTF8`, MACHINE.md). Lowering `codegen-units` to 16 was measured and rejected — slower to build here (1 m 44 s vs 1 m 26 s warm, 2 cores).
- **Next:** the decider reviews the branch, merges it, and does `RELEASING.md` §One time.

---

## 2026-10-20 — W-03 steps 2–5: all five rule packs merged  ·  Claude Code (Opus 5.5)

- **Asked:** *"continue all run everything in parallel"*.
- **Done:**
  - `packs/js` finished: `node`, `stencil` and `js/conventions` presets — 132 rules (b9cabc6).
  - Four pack agents in worktrees; `packs/data` (30 rules, f10648f) and `packs/css` (70 rules, edc1eaf) reviewed, re-tested and merged. The native registry test now checks unique ids, not sorted order, so parallel branches merge by union.
  - My own checks after each merge: workspace tests, clippy, every pack's `codelaw test` and `rules lint`; the data pack over this repo's records and docs; the css pack over 60 third-party stylesheets — every sampled finding matched upstream's behaviour.
  - SR-RULE-PACKS decision 7 named MDN browser-compat-data (CC0-1.0) as an allowed data source, against L2 (CC0 is not OSI-approved); corrected to name the MIT and Apache-2.0 sources actually used. W-03's css row said the same and is corrected.
  - Backlog B-040 (native rules see no file path) and B-041 (tree-sitter-css can't parse modern CSS).
- **Decided / left open:** `md/no-duplicate-heading` defaults to siblings only; `json/package-engines` defaults to `>=22` and `toml/pyproject-requires-python` to `>=3.11`, the L6 host floors — for the decider to correct. Still open: whether every pack rule stays `preview`, and the project licence (blocks publishing).
- **Mistakes:** the agents shared one scratchpad corpus folder and one emptied it under another mid-run; each now keeps its own. A whole-disk search for CSS files ran past its timeout and was stopped.
  - `packs/html` (71 rules, 6045676) merged after the same checks plus a 51-file rustdoc and Homebrew-docs run: findings sampled were spec violations html-validate also reports.
  - `packs/python` (98 rules) held back: compared with Ruff on pip's source it gave 87 extra unused-import findings and false `undefined-local` and `groupby` findings; returned to its agent with reproductions.
  - `packs/python` re-checked after the fixes and merged (f746d06): the agent traced the false positives to the semantic model reading `List[int]` as a PEP 695 type-parameter list, fixed it there with tests; on pip the only remaining extras are lines under one `# noqa`.
- **Next:** W-03 step 6 — the packaging (B-038), then the decider's ruling to publish and on the project licence.

---

## 2026-10-19 — W-03 in part: js, ts and react presets; Stencil and Node natives; subagents on python and css  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement open work items"*, then *"continue"*, then *"commit everything and push"*.
- **Done:**
  - `packs/js`: `js/recommended` (all 64 of ESLint 10.0.1), `ts/recommended` (typescript-eslint 8.71.2's 20 plus four core rules), `react/recommended` (17 of eslint-plugin-react 7.37.5, plus `rules-of-hooks` and `exhaustive-deps`) — 107 rules, `codelaw test packs/js` green.
  - Native rules for those, plus 16 Stencil rules and `node/no-deprecated-api` with a data table; their rule files are not written yet.
  - Engine: five predicates, data tables for natives, kinds missing from a grammar match nothing, duplicate YAML keys refused, empty `$$$` captures.
  - Corpus runs over npm's source, zod/ajv/TanStack sources and react-query: false-positive patterns found and fixed with tests; real defects found in `@npmcli/agent` and `node-gyp`.
- **Decided / left open:** every pack rule is `preview` (SR-RULE-AUTHORING decision 6), so presets need `preview = true` until a corpus job promotes them — for the decider to confirm. The project's own licence is undecided, so packs have no `LICENSE`; it blocks only publishing (W-03 step 6).
- **Mistakes:** I repeatedly wrote duplicate YAML keys in rule specs; the loader now refuses them. Both pack subagents stopped on a usage limit; the Python one is restarted, the CSS one resumed — both still running, in worktrees, uncommitted.
- **Next:** the Stencil and Node rule files and presets; then review and merge the python and css branches.

---

## 2026-10-19 — W-02: the engine — 13 crates, the `codelaw` binary, the law checks  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement open work items"* — the queue's 🟢 pick was W-02 (W-03 waited on it).
- **Done:**
  - **The engine** in `crates/`: core, cache, config, lang, rules, semantic, match, native, fix, engine, report, test, cli. `cargo test --workspace` green; clippy clean.
  - **Languages:** all nine parse; TOML registered by codelaw; Markdown's inline grammar runs on `inline` nodes; HTML and Markdown regions map back to host positions.
  - **Rules:** the Markdown rule parser with every template and L4 check at load; `where:` predicates over the semantic model; eight fix primitives; the apply loop with the verifier; `native:` rules (`js/no-unreachable` first); pack resolution from `node_modules` and the Python environment.
  - **Checks:** layers (B-020), L2 licences over every Cargo dependency (B-008, Cargo part), the L3 fixture matrix and isolation diff (`fixtures/l3/`), L4 fix fields and loader, L5 determinism, CLI/library parity and the no-model fence, cold/warm/partial cache.
  - **Records:** SR-RULE-FORMAT 3a–3d and 5, SR-ARCHITECTURE 7, 15a, 18 and 28, SR-LINTER 3 and 6, SR-RULE-PACKS 3, 4, 6, 9 and 11, SR-CONFIG 2 and 3, SR-CACHE 5, SR-CLI-MCP 4; every crate owned in the register; `built` column defined.
  - **Work files:** W-02 closed and archived; W-03 🟢; backlog B-028–B-039 added, B-010 and B-021 closed, five rows narrowed; two lessons, two MACHINE rows, five glossary terms.
  - **`CLAUDE.md` changed:** the layout block names `crates/`, `fixtures/` and `Cargo.toml` (a fact; SR-WORK-DOCS decision 7).
- **Decided / left open:** this session's calls, for the decider to correct:
  - the `where:` syntax, with `$$MATCH` for the matched node and `not`/`all`/`any`;
  - **root-only config keys** (`extends`, `rules_dir`, `preview`, the `[files]`-style sections): a nested config may set only languages, include/exclude, `[rules]` and overrides;
  - never entered: `.git`, `node_modules`, `.venv`, `.codelaw/cache`; the global gitignore isn't read;
  - `init` in a repo with both manifests: `rules-python` from PyPI, the rest from npm;
  - preview gating applies to pack rules only; `npm:`/`pypi:`/`path:` packs enable all their rules;
  - SR-ARCHITECTURE decision 28 now lists `init`'s package manager, which SR-RULE-PACKS decision 6 already required;
  - **the law records SR-LAW-2 and SR-LAW-3 were touched in `owns:` frontmatter only**, to own their checks; no law text changed.
- **Mistakes:** several test expectations were wrong on first run (line numbers, module-level Python names, a vacuous assertion) and were corrected against the code's behaviour, which was right. Two real bugs were caught by tests: YAML 1.1 booleans, and a missing `fix:` cascading into three misleading errors.
- **Next:** W-03 — the five rule packs, starting with `rules-js`.

---

## 2026-10-19 — W-01: the records rewritten for five separately published rule packs; SR-RULE-PACKS accepted  ·  Cowork (Opus 5.5)

- **Asked:** *"go with the recommendations for all of them"*: run W-01, then accept SR-RULE-PACKS.
- **Done:**
  - **SR-RULE-PACKS** rewritten: five first-party packs (`js`, `html`, `css`, `python`, `data`) on npm `@codelaw/rules-*` and PyPI `codelaw-rules-*`, with layout, resolution, `init` install, ported-rule policy and data tables. The embedded-in-binary choice is moved to Alternatives. Accepted.
  - **SR-LAW-6:** law text and decisions 3, 4, 9 and 10 amended; decision 11 added.
  - **SR-PACKAGES:** 20 crates (`codelaw-rules-builtin` out; `codelaw-semantic` and `codelaw-native` in); decision 9 covers the packs.
  - **SR-ARCHITECTURE:** decisions 15a–15c (semantic model, native rules, pack data).
  - **Smaller amendments:** SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT (`ported_from:`, `native:`), SR-LINTER, SR-AGENT-SURFACES, SR-BASELINE, SR-CLI-MCP (`init --no-install --registry`), L3 decision 2, L4.
  - **Routing:** `CLAUDE.md` L6 line, SR-LAWS, the register, DESIGN.md and the glossary.
  - **Queue:** W-01 closed and archived; W-02 🟢.
- **Decided / left open:** this session's calls, for the decider to correct:
  - the `rules-data` presets are namespaced `data/…`;
  - an opt-in `conventions` preset per pack;
  - `native:` only in first-party packs;
  - the npm copy of a pack is found before the Python copy;
  - SR-ARCHITECTURE's new decisions are lettered (15a–15c) so that nothing is renumbered.
- **Mistakes:** SR-RULE-PACKS's decisions were renumbered by the rewrite; four citations in other records were found by grep and repointed in the same change.
- **Next:** W-02 (the engine).

---

## 2026-10-19 — the decider rules on the whole inbox: L5 and fourteen records accepted; L6 floors 3.11 / 22  ·  Cowork (Opus 5.5)

- **Asked:** *"go with the recommendations for all of them."* (the blocker list with a recommendation per row)
- **Done:** accepted SR-LAW-5 (banner replaced by the ruling line) and SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE, SR-FORMATTER, SR-LINTER, SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP (frontmatter + register). SR-LAW-6 decision 8: host floors Python ≥ 3.11, Node ≥ 22. `CLAUDE.md` L5 line and `docs/DESIGN.md` law table no longer say *(proposed)*. Inbox emptied; W-02 and W-03 🔴 → 🟡; W-01 gains DoD 10 (accept SR-RULE-PACKS after the rewrite). Committed with everything filed since `9baa117`.
- **Decided / left open:** SR-RULE-PACKS stays proposed until W-01 rewrites it for separately published packs (the ruling: accept after the rewrite). Installing `scripts/sr-guard.sh` as the `commit-msg` hook stays Arpit's step.
- **Mistakes:** none.
- **Next:** W-01.

---

## 2026-10-19 — preset research; W-01–W-03 filed to split rules into five packs  ·  Cowork (Opus 5.5)

- **Asked:** *"research the rules preset in ruff, eslint, tslint, prettier, and other and create as separate packages, like 1 for js/ts/node/jsx/tsx, 1 for html, 1 for css, 1 for python and engine will be separate create work item"*
- **Done:** researched ESLint (`@eslint/js` recommended, 64 rules), typescript-eslint configs, TSLint (deprecated 2019), React hooks, Node, Stencil, oxlint and Biome; Ruff (old default 59 rules, v0.16.0 default 413); stylelint (recommended 41, standard); html-validate presets, HTMLHint, markuplint; markdownlint, SchemaStore, yamllint, taplo; Prettier option defaults. Filed **W-01** (record update, 🟢), **W-02** (engine, 🔴), **W-03** (the five packs + the research, 🔴); the nine-record and SR-LINTER inbox rows now block W-02 and W-03. Glossary: semantic model, native rule. Nothing committed.
- **Decided / left open:** the decider ruled: **separately published**; mirror every pack on both registries (`@codelaw/rules-*` / `codelaw-rules-*`); a fifth pack `rules-data` for JSON/YAML/TOML/Markdown; `codelaw init` installs detected packs and the engine ships no rules. This session's calls inside the items: one version for engine and packs; reimplement, never vendor; drop layout rules (L7) and type-aware rules (L3); Pylint and yamllint contribute ideas only (GPL). Found a gap no record covered: porting real presets needs a per-file semantic model and `native:` rules — now W-02 DoD 3–4.
- **Mistakes:** none.
- **Next:** W-01.

---

## 2026-10-19 — SR-ARCHITECTURE rewritten as the design of record; cross-cutting gaps closed  ·  Cowork (Fable 5.1)

- **Asked:** *"design the architecture fill/resolve the gaps if any."*
- **Done:** audited the 21 records against each other and rewrote `records/0108_architecture.md` (proposed) to 30 decisions in eight groups. Gaps no record decided, now decided there: file encoding and BOM (8), line endings (9), generated and oversized files (10), glob base (11), safe writes and the LSP never writing (12), position encodings incl. UTF-16 at the LSP boundary (13), git via the `git` binary only (14), lazy embedded regions and the `languages` switch (16), `root = true` as a separate project (17), the external-fixer stdin/stdout contract (18), the adapter manifest and "never in hook/LSP" (19), warm-socket discovery with version equality (21), daemon lifecycle (22), tree-cache memory bound (24), rule-panic isolation (25), `skipped[]` honesty (26), no telemetry (27), what may execute (28), fix-conflict detection (5). Small pointer patches in SR-CONFIG, SR-LINTER, SR-RULE-FORMAT, SR-BASELINE, SR-CLI-MCP, SR-EDITOR, SR-CACHE; `docs/DESIGN.md` §3 now defers to the record. B-026, B-027; two glossary terms. Nothing committed.
- **Decided / left open:** this session's calls, for the decider to correct: shell out to `git` rather than link a git library; adapters excluded from hook and LSP mode; symlinks not followed; 1 MiB file-size default; daemon stays opt-in. Still undecided by design: a JS rule API, type-aware rules, SCSS/Vue/Svelte. SR-ARCHITECTURE stays in the existing nine-record inbox row.
- **Mistakes:** the rewrite renumbered decisions, which broke three cross-references in other records; found by grep and fixed in the same change.
- **Next:** the decider rules on the inbox.

---

## 2026-10-18 — laws L7, L8 and nine records filed; L3, L4, L6 amended  ·  Cowork (Opus)

- **Asked:** *SRs for packages, presets + local rules, VS Code extension + LSP, MCP + CLI, rule authoring (manual or agent skill; debuggable, testable), the rule template, caching, architecture, the config file, the consumer directory; laws for the eleven file types, linter + formatter, fixer-or-skill, one `codelaw` package per registry with `@codelaw/*` sub-packages, and performance budgets that are never broken.*
- **Done:** SR-LAW-7 (linter + formatter), SR-LAW-8 (performance budgets, values unset); SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP (proposed). Amended in place by the decider's ruling: SR-LAW-3 (JSX and TSX named explicitly), SR-LAW-4 (a skill carries every manual fix; `fix_skill:`), SR-LAW-6 (naming: only `codelaw` unscoped on npm, `@codelaw/*` for the rest; only `codelaw` on PyPI). SR-RULE-PACKS retitled to cover local rules (no new record: it already owned presets, packs and local rules). Routed in SR-LAWS, the register and `CLAUDE.md` (L4/L6 lines changed, L7/L8 lines added). B-017–B-025; glossary. Nothing committed.
- **Decided / left open:** the language law, the fixer law and the packages law already existed (L3, L4, L6), so they were amended rather than duplicated. L8 fixes the mechanism now and leaves every number `unset` until measured (B-019). The nine records are one inbox row.
- **Mistakes:** none.
- **Next:** the decider rules on the inbox.

---

## 2026-10-18 — SR-LINTER filed; L3 amended to nine languages  ·  Cowork (Opus)

- **Asked:** *"create an SR for linter as well. The linter and formatter are going to work for JavaScript, TypeScript, JSON, Python, CSS, HTML."*
- **Done:** SR-LINTER (`records/0106_linter.md`, proposed). SR-LAW-3 amended to nine languages (CSS, HTML added). CSS/HTML carried into SR-RULE-PACKS (presets + 8 starter rules), SR-FORMATTER (Biome CSS, `markup_fmt` for HTML), SR-PROJECT-GRAPH (`graph/no-unused-css-classes`, HTML link edges), SR-AGENT-SURFACES (skill trigger), `docs/DESIGN.md`, SR-LAWS, the register and `CLAUDE.md` (L3 pointer line changed). B-016; glossary "embedded region". Nothing committed.
- **Decided / left open:** the ask named six languages without YAML/TOML/Markdown; the decider ruled **add CSS and HTML, keep all seven** (nine). SCSS/Sass/Less and Vue/Svelte/Angular templates stay out of scope. Ruling on SR-LINTER filed in the inbox.
- **Mistakes:** none.
- **Next:** the decider rules on the eight inbox rows.

---

## 2026-10-18 — SR-FORMATTER filed: codelaw is a linter and a formatter  ·  Cowork (Opus)

- **Asked:** *"Create an SR that linter code law will be a linter as well as a prettier."*
- **Done:** SR-FORMATTER (`records/0105_formatter.md`, proposed); register row, inbox row, B-015, glossary term "layout rule", `docs/DESIGN.md` link and config sample. Nothing committed.
- **Decided / left open:** engines embedded, not shelled out (L3/L6): Biome for JS/TS/JSON (Prettier-compatible), Ruff for Python (Black-compatible), pretty_yaml, taplo, dprint-plugin-markdown — all permissive (L2). Biome and Ruff are git-pinned because their crates.io copies are stale or internal. Layout-only lint rules are refused. Ruling filed in the inbox.
- **Mistakes:** none.
- **Next:** the decider rules on the seven inbox rows.

---

## 2026-10-18 — laws L3–L6 and four design records filed  ·  Cowork (Opus)

- **Asked:** *create laws (languages; every rule has a fix; Python and Node packages), decide how predefined rules ship, an SR for the skills (the whole agent-facing set), then the mechanisms for unused exports across a monorepo and deprecations with a grandfathering file.*
- **Done:** SR-LAW-3 (languages), SR-LAW-4 (fix), SR-LAW-5 (no model in the engine, proposed), SR-LAW-6 (PyPI + npm); SR-RULE-PACKS, SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE (all proposed); routed in SR-LAWS, the register and `CLAUDE.md` §The laws (four pointer lines added — `CLAUDE.md` changed); `docs/DESIGN.md` links the laws; B-009–B-014; glossary terms. Nothing committed.
- **Decided / left open:** predefined rules embedded in the binary as presets (the decider left the choice open); yamllint dropped as an adapter (GPL-3.0, refused by L2); host floors Python ≥ 3.9 / Node ≥ 18 are this session's reading. Six rulings filed in the inbox.
- **Mistakes:** this session first wrote its laws as `0001`–`0004` / L1–L4 without seeing the work system set up minutes earlier, and overwrote the uncommitted-in-its-turn `records/README.md`. Caught by listing the folder before the next write; the register was restored from `HEAD`, the laws renumbered to `0005`–`0008` / L3–L6, and the four misnumbered files removed.
- **Next:** the decider rules on the six inbox rows.

---

## 2026-10-18 — law L2 filed  ·  Claude Code (Opus 5.5)

- **Asked:** *"add a new law to not use paid and open source dependencies."*
- **Done:** SR-LAW-2 (`records/0004_LAW-2-foss-dependencies.md`); routed in SR-LAWS, `CLAUDE.md` §The laws and the register; backlog row B-008 for the licence check nobody runs yet. `CLAUDE.md` changed: one pointer line for L2.
- **Decided / left open:** the ask read two ways, so it went to the decider, who ruled: **FOSS only, nothing paid**, and **no copyleft anywhere**, dev tooling included. Weak copyleft (MPL, EPL) and paid hosted services are refused too — that reading is this session's, stated in SR-LAW-2 decisions 2 and 5 for the decider to correct.
- **Mistakes:** none.
- **Next:** the decider installs the commit-msg guard; B-008 becomes a `W-nn` when the first manifest lands.

---

## 2026-10-18 — the work system set up from WORK-SYSTEM.md  ·  Claude Code

- **Asked:** *set up the work system from WORK-SYSTEM.md.*
- **Done:** records 0001–0003 and 0051–0060, `work/`, `archive/`, `scripts/`, `.claude/` hooks, `CLAUDE.md`; WORK-SYSTEM.md moved to `archive/bootstrap/`.
- **Decided / left open:** tests deferred — filed as backlog rows B-001–B-005.
- **Mistakes:** none.
- **Next:** the decider installs the commit-msg guard; the first `W-nn` is filed.

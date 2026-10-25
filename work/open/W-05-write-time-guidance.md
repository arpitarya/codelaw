---
type: Handoff
name: W-05
description: "Arpit, 2026-10-20: codelaw guides an agent while it writes code, not only after: a brief before writing, and a check on each write before it reaches disk."
item: W-05
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: cowork
---

# W-05 — Guide the agent while it writes, not only after

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Check which agents can inspect a write **before** it lands: Claude Code, Codex, Copilot, Kiro — the exact input and output fields of each | a pre-write row for the SR-AGENT-SURFACES matrix | ⬜ (Claude Code confirmed in chat, 2026-10-20) |
| 2 | Prototype `codelaw hook --pre` on the 0.1.0 binary: build the file as it would be after the edit, in memory, check it under its path, and report only **new** diagnostics | a timing against the one-file hook target (W-19) | ⬜ |
| 3 | Work out "construct → rules" from each rule's matcher root node kind, over all 401 pack rules | a coverage count; the list of `native:` rules that must declare their constructs | ⬜ |
| 4 | Fux spike (optional): one pack's installed `rules/` in `.fux/sources.yaml`, 20 intents, the right rule in the top 5 at least 16 times | evidence for or against Fux as the free-text layer | ⬜ moved to Fux item 298 step 1 (2026-10-25) |
| 5 | Draft the amendments: SR-AGENT-SURFACES (pre-write row, brief surface), SR-CLI-MCP (`brief`, `brief_for_task`, `hook --pre`), SR-LAW-8 (a `hook_pre` budgeted operation), glossary | records ready for the decider to rule on | ⬜ |
| 6 | On the ruling, file the build `W-nn` (it rides with W-08 MCP and W-10 hooks) | the feature built | ⬜ |

**Arpit, 2026-10-20:** *"It shouldn't be like that, okay, after I have written the code, then you go ahead and fix it. Well, that should be there, but even while writing the code, if it can tell AI agents that this is how you should write it, that's even better. … can we use Fux for it since it is already doing most of it"*

**Arpit, 2026-10-25 (voice), on the Fux layer:** the package author builds the index — *"rather than Fux trying to re-ingest it, it can search through multiple indexes … one Fux index present at the root of a project … another index present inside node modules package"* — and codelaw ships it as *"a separate package for Fux index. It shouldn't be added as a default one"*, `@codelaw/rules-js-fux-index`, on *"NPM as well as PyPI"*. So: **neither A nor B below** — the Fux half is Fux's item 298 and codelaw's W-34; the before/while/after story is W-35.

**Model:** Cowork — research; step 2's prototype may hand to Claude Code once step 1 is done.

## Goal

An agent learns codelaw's rules **at three moments**, and the first two are new:

| moment | what the agent has | what codelaw gives it | exact? |
|---|---|---|---|
| **plan** — before any code | target paths, and what it will write (functions, classes, exports, tests) | `brief_for_task(paths, constructs)`: the rules for those paths and those constructs, each with its one-line Do/Don't | yes — path, language and construct are inputs, not guesses |
| **write** — the edit is proposed but not on disk | the full proposed content | `hook --pre`: new violations only; an `error` blocks the write with the reason and the fix, so the agent rewrites before the file exists | yes — the same engine verdict (L5) |
| **after** — the file is on disk | the file | today's `PostToolUse` hook, `check`, `fix` | yes |

## What already exists, and the gap

- SR-AGENT-SURFACES already has **Instruct** (AGENTS.md, skills), **Query** (`rules_for_path`, `check_snippet`) and **Enforce** (post-edit hooks). Nothing built yet (W-08, W-10).
- **Gap 1:** `rules_for_path` returns every rule for a path. A JS file has 132 candidate rules, about 8,000 tokens at 60 each — too many to read before every function.
- **Gap 2:** every enforcement point runs after the bytes land. Claude Code's `PreToolUse` can see the proposed content and refuse the call with a reason Claude reads, so codelaw can check it first.

## Proposed design (to be confirmed by steps 1–5)

1. **`codelaw brief --for <paths> [--constructs function,class,export,import,test,async,…]`**, and its MCP twin `brief_for_task`. It filters by path, language and construct. The construct vocabulary is a fixed list, mapped to rules from each matcher's root node kind. No free text and no ranking, so the same input always gives the same rules.
2. **`codelaw hook --pre`** reads the hook payload (Write: `file_path` + `content`; Edit: `file_path` + `old_string`/`new_string`), builds the post-edit buffer in memory and checks it as that path. It reports only diagnostics that are not already in the file on disk.
   - New `error` → deny, with `--format agent` lines and each fix (L4) as the reason.
   - New `warning` → allow, plus `additionalContext`.
   - `auto` fixes are **named, not applied** through `updatedInput`, so the code the agent believes it wrote is the code on disk.
3. **Fux is an optional layer for free text and precedent, never a dependency.** Rule files are Markdown (SR-RULE-FORMAT), which Fux already indexes: the repo's own rules as a plain folder, packaged rules in place through a package source (F1 below). An agent with both servers asks codelaw for the law (`brief_for_task`) and Fux for precedent (`fux_search "debounced fetch hook"` → rules + ADRs + examples). Neither engine imports the other.

## What the Fux integration needs built

**Arpit, 2026-10-20:** *"I don't want code law to set up a new directory where the rules are put. If they are part of package, then they should remain part of package."* No generated rule cards, and no copy of a pack anywhere.

- **The repo's own rules** (`.codelaw/rules/`) are already Markdown in the repo: one plain line in `.fux/sources/dirs`. Nothing to build.
- **Packaged rules stay in the package.** Fux reads them where the package manager put them, located **by package name, never by path**. This extends the pattern Fux already uses for its plugins (Fux SR-PLUGINS: declared, pinned to a version and digest, found with `importlib.util.find_spec`, refused loudly when missing or drifted).

| # | piece | where | why |
|---|---|---|---|
| F1 | a **package source**: a committed line naming the package, its registry, version, digest and subfolder, written as a purl, e.g. `pkg:npm/%40codelaw/rules-js@0.1.0#rules` or `pkg:pypi/codelaw-rules-python@0.1.0#codelaw_rules_python/rules` | Fux, new SR | a path into `node_modules/` or `.venv/lib/python3.x/site-packages/` differs by OS, Python version and venv name, and pins nothing, so two machines would build different indexes (Fux L4) |
| F2 | resolve by each ecosystem's own rule: Node module resolution (walking `node_modules` up to the root, as Fux's `.fux/fux` shim already does) and Python's `find_spec`. The resolved path goes in gitignored `.fux/runtime/`; the committed index stores only the purl | Fux, both readers | the committed index stays machine-free; the Node reader reading a PyPI package looks the path up instead of resolving Python |
| F3 | `fux setup --pin` re-pins a package source after an upgrade; a drifted install is a loud error naming the install line | Fux | upgrades are explicit, never silent |
| F4 | `codelaw init` **prints** the purl lines for the packs `codelaw.toml` enables; it never edits Fux's files | codelaw | nobody hand-writes purls |
| F5 | optional `see:` on a rule, linking the ADR or doc behind it; Fux turns it into a `ref` edge for `fux_related` | codelaw, SR-RULE-FORMAT | the reason behind a rule is one hop away |
| F6 | the `codelaw-write` skill: if a `fux` MCP server is registered, `fux_search` for examples; codelaw's verdict wins any conflict | codelaw | the agent joins the two answers |

**Arpit, 2026-10-20:** packaged rules (`rules-js`, `rules-python`, …) must not be duplicated in the **consumer** repository. Nothing is copied under F1–F3. **Ruled 2026-10-25: neither option below** — the index is built once in codelaw's CI and shipped as its own optional package (W-34); the multi-index query is Fux item 298. The two options as they were weighed:

- **A — committed**, like every other Fux source: the consumer repo carries index statistics for the pack (never its text, Fux L3), and they change on every pack upgrade.
- **B — local only** (proposed): package sources are indexed into gitignored `.fux/runtime/` after install, and a query merges that local segment with the committed index. The consumer repo carries **zero** pack-derived bytes, only the purl line. Costs a new Fux design: a merged two-segment query whose document statistics stay deterministic for a given pinned install.

**F1–F3 are a Fux feature — now Fux item 298** (2026-10-25); F4 moves to W-34. They need a Fux SR and the decider's sign-off in Fux's own queue, and they are generic (any package that ships docs), not codelaw-specific. **Until they land**, a repo can list `node_modules/@codelaw/rules-js/rules` as a plain dir. That works for npm but pins nothing.

## Why not build this inside Fux

- **L5 and SR-CLI-MCP:** codelaw is complete with no agent and no other tool. A brief that needs Fux would break that.
- **A verdict must be exact** (SR-AGENT-SURFACES decision 4). Fux ranks; a ranking may suggest, never block.
- **Fux's own records:** its MCP surface is three tools by decision, with "no new tools" (Fux SR-MCP decision 2). A new tool or adapter there needs its own SR and the decider's sign-off. That belongs in Fux's queue, not here.

## Definition of done

- Step 1's matrix covers each of the five agents in SR-AGENT-SURFACES, with a source link per row.
- Step 2's timing and step 4's run (if run) are written up with their inputs.
- Step 5's amendments are filed `proposed`, with an inbox row for the ruling.

## Hazards

- A pre-write check that is slow makes every edit slow; it must fit the one-file hook target, or it ships off by default.
- An Edit hook sees only a fragment, so the check must rebuild the whole file. A fragment check would give false positives.
- Hook input fields are each agent's API and change between versions; the SR-AGENT-SURFACES veto already covers this.

**Closes when** the decider has ruled on step 5's amendments and the build item is filed.

**Record affected:** SR-AGENT-SURFACES, SR-CLI-MCP, SR-LAW-8 (proposed amendments, step 5). None changed by this file.

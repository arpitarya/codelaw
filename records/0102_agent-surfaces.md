---
type: Standing Record
kind: component
name: SR-AGENT-SURFACES
title: "SR-AGENT-SURFACES (0102) — skills, steering, instructions, agent files and hooks, generated from one source"
description: "codelaw ships a catalogue of Agent Skills plus always-on and path-scoped instructions, custom agents, hooks and MCP registration for Claude Code, Cowork, Copilot, Codex and Kiro, all generated from one source by codelaw sync-agents; every one of those files ships in the skills package (@codelaw/skills, codelaw-skills), never in the engine."
status: accepted
date: 2026-10-18
feature: "the agent-facing files and skills"
owns:
  - crates/codelaw-agents
  - templates/agents
laws: [L0, L3, L4, L5, L6]
timestamp: 2026-10-27T18:00:00Z
content_sha: ""
---


# SR-AGENT-SURFACES — skills, steering, instructions, agent files and hooks, generated from one source

## §1 — For humans

**Scope.** This record covers the **whole agent-facing set**: skills, steering, rules and instruction files, AGENTS.md / CLAUDE.md, custom agents, agent hooks and MCP registration.

**The decision in one line.** codelaw ships **12 skills** written once in the open **Agent Skills** format (`SKILL.md`). `codelaw sync-agents` copies them into each agent's skills folder and generates every other agent file from the rules. Nothing agent-facing is ever hand-edited in a consumer repo. **Since 2026-10-27 (W-77) every one of those files ships in its own package, `@codelaw/skills` on npm and `codelaw-skills` on PyPI, at the engine's exact version; the codelaw binary carries none of them (decision 8).**

**Four kinds of surface, four jobs** (the fourth, *Gate*, since 2026-10-27, W-93):

| Kind | Job | Examples | Can it block? |
|---|---|---|---|
| **Instruct** | Tell the agent the rules before it writes | AGENTS.md, CLAUDE.md, Kiro steering, Copilot instructions, skills | Never |
| **Query** | Let the agent ask for exactly the rules it needs | MCP server: `rules_for_path`, `brief_for_task`, `check_snippet`, `explain_rule` | Never |
| **Gate** | Refuse a write before it reaches disk | Pre-write hooks running `codelaw hook --pre` (decision 9) | Yes, only for an `error` the write adds — and never when codelaw itself fails: it fails open |
| **Enforce** | Check and fix after the agent writes | Post-edit hooks running `codelaw hook` | Yes. Every engine rule is an exact match ([L5](0007_LAW-5-no-model-in-the-engine.md)), so a block is never a guess |

**Agents are optional** ([L5](0007_LAW-5-no-model-in-the-engine.md)). With no `--agents` flag, none of this is written.

```mermaid
flowchart LR
    S["skills + rules<br/>(one source)"] --> G["codelaw sync-agents"]
    G --> C["Claude Code · Cowork"]
    G --> P["Copilot"]
    G --> X["Codex"]
    G --> K["Kiro"]
```

<details>
<summary><b>ASCII twin</b></summary>

```text
   skills + rules --> codelaw sync-agents --+--> Claude Code / Cowork
                                            +--> Copilot
                                            +--> Codex
                                            +--> Kiro
```

</details>

---

## §2 — For agents

### Context

Each agent reads a different set of files from a different place. Maintaining them by hand, per agent, drifts the day a rule changes.

### Decision


#### 1. One source, generated everywhere

- Skills live in the codelaw source tree as `templates/agents/skills/<name>/SKILL.md`, with optional `references/` and `scripts/`. They follow the Agent Skills standard:
  - `name`: lowercase, hyphens, at most 64 characters;
  - `description`: at most 1,024 characters, saying what the skill does **and** when to use it;
  - progressive disclosure: metadata about 100 tokens, `SKILL.md` body under 5,000 tokens, detail moved into `references/`.
- `codelaw sync-agents` writes each agent's files. Shared files (AGENTS.md, CLAUDE.md) get **marker blocks**, `<!-- codelaw:begin -->` … `<!-- codelaw:end -->`, so the team's own text is never touched.
- `codelaw sync-agents --check` runs in CI and fails if any generated file differs from what the rules and skills would produce.
- Skills, and every other file under `templates/agents/`, ship in **the skills package** on both registries (decision 8; [L6](0008_LAW-6-python-and-node-packages.md) decision 13), so a Python-only repo gets the same skills as a Node repo.

#### 2. Surface matrix

`init --agents claude,cowork,copilot,codex,kiro` writes:

| Surface | Claude Code | Cowork | GitHub Copilot | Codex | Kiro |
|---|---|---|---|---|---|
| Always-on instructions | `CLAUDE.md` block | via plugin skills | `AGENTS.md` block (also reads `.github/copilot-instructions.md`) | `AGENTS.md` block | `.kiro/steering/codelaw.md` (`inclusion: always`) |
| Path-scoped instructions | via `codelaw-write` skill + MCP | via plugin | `.github/instructions/codelaw-<lang>.instructions.md` (`applyTo`) | via skill + MCP | `.kiro/steering/codelaw-<lang>.md` (`inclusion: fileMatch`) |
| Skills | `.claude/skills/<n>/SKILL.md` | bundled in a codelaw plugin | `.github/skills/<n>/SKILL.md` | `.agents/skills/<n>/SKILL.md` | `.kiro/skills/<n>/SKILL.md` |
| Custom agent | `.claude/agents/codelaw-rule-author.md` | none | `.github/agents/codelaw-rule-author.agent.md` | none | none |
| Pre-write hook (Gate, decision 9) | `.claude/settings.json` `PreToolUse` | plugin hook (reported not to fire in Cowork; not counted on) | not written yet: the payload's argument names are unconfirmed | `PreToolUse` hook (printed snippet) | not written yet: the stdin shape is unconfirmed |
| Post-edit hook | `.claude/settings.json` `PostToolUse` | plugin hook | `.github/hooks/codelaw.json` | `PostToolUse` hook (printed snippet; config is user-level) | agent hook (file save / post-tool) |
| MCP registration | `.mcp.json` | plugin | `.vscode/mcp.json` | printed `config.toml` snippet | `.kiro/settings/mcp.json` |
| Bundle | none | **plugin** (skills + MCP + hook) | none | none | optional **Power** (steering + MCP + hooks) |

- **The pre-write column beyond the matrix** (W-05's research, 2026-10-27; sources under *Research basis*). `codelaw hook --pre` reads each of these payloads and answers in each one's shape (decision 9); `sync-agents` writes a pre-write hook only for the matrix rows above that say so, and a row joins once a payload fixture of that agent's current version is captured:

  | agent | pre-write event | full new file in the payload? | how it refuses | config |
  |---|---|---|---|---|
  | Claude Code | `PreToolUse`, matcher `Write\|Edit\|MultiEdit` | `Write` yes; `Edit` / `MultiEdit` no (`old_string`, `new_string`, `replace_all`, `edits[]`) | `hookSpecificOutput.permissionDecision: "deny"` + `permissionDecisionReason` | `.claude/settings.json` |
  | Codex CLI | `PreToolUse` (`apply_patch`) | no: an `apply_patch` envelope in `tool_input.command`, possibly several files | the same fields as Claude Code | `~/.codex/config.toml`, repo `.codex/` |
  | Copilot CLI / cloud agent | `preToolUse` | argument names unconfirmed (`toolName`, `toolArgs`) | top-level `permissionDecision: "deny"`; **any non-zero exit denies** | `.github/hooks/*.json` |
  | VS Code agent mode (preview) | Copilot's format | unconfirmed | as Copilot | `.github/hooks/*.json`; also reads `.claude/settings.json` |
  | Kiro | `PreToolUse`, category `write` | unconfirmed (`tool_name`, `tool_input`) | exit 2 + stderr | `.kiro/hooks/*.json` |
  | Cursor (beta) | `preToolUse`, matcher `Write` | unconfirmed | `permission: "deny"` + `agent_message` | `.cursor/hooks.json`; also loads `.claude/settings*.json` |
  | Gemini CLI | `BeforeTool` | `write_file` yes; `replace` no (`old_string`, `new_string`, `allow_multiple` / `expected_replacements`) | `decision: "deny"` + `reason` | `.gemini/settings.json` |
  | Windsurf (Devin Desktop) | `pre_write_code` | no: `tool_info.edits[]` of `old_string` / `new_string` | exit 2 + stderr | `.devin/hooks.json` |

- Cursor (`.cursor/rules/*.mdc`) and Gemini CLI (`GEMINI.md`) are **P3**. Both claim Agent Skills support, but their skill paths haven't been verified; check before building.
- codelaw never edits files in the user's home directory. User-level config (Codex) is printed for the user to paste.

#### 3. Token budget

- **Always-on surfaces stay under 40 lines.** They hold:
  - what codelaw is;
  - "ask before you write";
  - "run `codelaw check` on changed files";
  - the list of active presets;
  - pointers to the skills.

  They never inline every rule.
- **Path-scoped surfaces** hold one line per rule (id plus one-liner), filtered to that language ([L3](0005_LAW-3-languages.md)).
- **Detail comes on demand:** `rules_for_path`, `explain_rule`, or the rule file itself.

#### 4. Safety rules every skill obeys

- **Never disable, loosen or eject a rule to make a check pass.**
- **Never edit a generated file;** edit the rule or skill source and re-run `sync-agents`.
- **Never suppress without a reason.** Inline `codelaw-ignore <rule-id>: <reason>` requires the reason, and `check` flags a suppression that no longer suppresses anything.
- **Apply `suggest` fixes only after showing the diff; follow `manual` steps exactly** ([L4](0006_LAW-4-every-rule-has-a-fix.md)).
- **A skill may only call codelaw for verdicts**; it never asserts a verdict from its own judgment ([L5](0007_LAW-5-no-model-in-the-engine.md)).

#### 5. Every skill ships with evals

- **Trigger evals:** prompts that **should** fire the skill and near-misses that **shouldn't**.
- **Task evals:** a fixture repo plus a task with a deterministic pass check. For example, "add a function to `math.ts`" passes when `codelaw check` is clean and the export is in the end block.
- Evals run in CI against each supported agent where that agent can be run headless.

#### 6. The config and the hook

- **`[agents]`** in the root config, both keys optional and validated at load ([SR-CONFIG](0110_config.md) decision 5):
  - `[agents] targets`: the agents of the decision-2 matrix (`claude`, `cowork`, `copilot`, `codex`, `kiro`) whose files `sync-agents` writes. Empty by default, and then no agent file exists. An unknown name is an error naming the nearest one. `init --agents <list>` writes it.
  - `[agents] command`: how a hook or an MCP registration starts codelaw. Unset, it is the repo's own install: `npx --no-install codelaw` beside a `package.json`, `uv run codelaw` beside a `uv.lock`, else `codelaw`.
- **`codelaw hook [paths]`** is the post-edit hook of every row of the matrix.
  - The files are its arguments, or, with none, the ones the agent's hook payload on stdin names: `tool_input.file_path` / `path` (Claude Code, Codex), the files an `apply_patch` envelope adds or updates (Codex), or `toolArgs` (Copilot). A deleted file, or one with no [L3](0005_LAW-3-languages.md) extension, is ignored.
  - It runs `codelaw fix` on them with `auto` fixes only, format step included ([L8](0010_LAW-8-performance-budgets.md) `fix_one_file`; [L4](0006_LAW-4-every-rule-has-a-fix.md) decision 8). It is the same engine call, so the bytes are `fix`'s ([L5](0007_LAW-5-no-model-in-the-engine.md) decision 5). The files it was given count as touched, so a grandfathered finding in one gets the verdict CI would give it ([SR-BASELINE](0104_baseline-and-deprecations.md) decisions 3 and 8).
  - **It judges with the `ci` profile** ([SR-CONFIG](0110_config.md) decision 10) whether or not `CI` is set, so an agent never leaves behind what CI would fail; `--profile <name>` or `CODELAW_PROFILE` picks another.
  - Nothing left: exit `0`, with the auto-fix notes on stdout. An error or warning left: exit `2`, with the fixes applied and up to 20 findings, each with its next step (a `suggest` diff to show, or the `manual` steps), on stderr. Exit 2 with stderr is how Claude Code and Codex hand a `PostToolUse` result back to the agent. A config or rule error is also exit `2`, as `error: …`.
- **`codelaw hook --pre`** is the same command's pre-write mode, the Gate of decision 9.

#### 7. What `sync-agents` writes, file by file

- **Owned files** are written whole, with a "generated, do not edit" note after any frontmatter: the skills, the custom agent, `.kiro/steering/codelaw*.md`, `.github/instructions/codelaw-<lang>.instructions.md`, `.github/hooks/codelaw.json`, `.kiro/hooks/codelaw.json` and the Cowork plugin.
- **Shared files** are touched only at codelaw's part:
  - `CLAUDE.md` and `AGENTS.md`: the marker block, replaced in place, or appended when there is none;
  - `.mcp.json`, `.kiro/settings/mcp.json` (`mcpServers.codelaw`) and `.vscode/mcp.json` (`servers.codelaw`): that one key;
  - `.claude/settings.json`: the `PostToolUse` entry whose command runs `codelaw … hook`, and the `PreToolUse` entry whose command runs `codelaw … hook --pre` (decision 9), each replaced; every other entry and key is kept.
  - A shared JSON file that does not parse is an error naming it; codelaw never overwrites it.
- **Path-scoped files** exist for each language that has an enabled rule; `<lang>` is the `languages:` name (`typescript`, `tsx`, …). A `codelaw-<lang>` file for a language with no rule left, and a `codelaw-*` skill the catalogue no longer has, are removed. The team's own files beside them are never touched.
- **The Cowork plugin** is `.claude/plugins/codelaw/`: `.claude-plugin/plugin.json`, `skills/`, `hooks/hooks.json` (the `PreToolUse` and `PostToolUse` groups of Claude Code) and `.mcp.json`.
- **The Kiro hook** is `.kiro/hooks/codelaw.json`: a `PostFileSave` trigger on the L3 extensions running `<command> hook {{filePath}}`.
- **Codex** gets a printed `~/.codex/config.toml` snippet: `[mcp_servers.codelaw]`, a `[[hooks.PreToolUse]]` group running `hook --pre --agent codex` and a `[[hooks.PostToolUse]]` group.
- **The texts come from the installed skills package** (decision 8): `templates/agents/` of the same release, byte for byte ([L10](0012_LAW-10-consumer-templates.md): every text `sync-agents` writes is a template there and code only fills its slots). The binary embeds none of them.
- **`sync-agents --check`**: exit `0` when every planned file matches, `1` listing each file that differs or should be removed, `2` on a config, rule or file error. With no `[agents] targets` both forms exit `0` without loading the rules.

#### 8. The skills package

**Ruled by Arpit, 2026-10-27 (W-77):** *"I want a separate package to be created just for skills, and it will have a hard dependency on code law. Skills being agent facing scripts or files."* — the whole set, on npm and PyPI, nothing agent-facing left in the engine, and the exact same version. Written by an agent on that ruling, in the change that built it.

- **What ships where.** Every file of `templates/agents/` — the skills, the always-on and path-scoped texts, the steering and instruction templates, the custom agent, the Claude, Cowork, Copilot, Kiro and Codex hook and MCP templates, the trigger evals — ships as `@codelaw/skills` (npm) and `codelaw-skills` (PyPI), with `codelaw-skills.json` naming its version ([SR-PACKAGES](0107_packages.md) decision 3b). The `codelaw` packages carry none of it; the commands — `init --agents`, `sync-agents`, `hook`, `mcp` — and the code that fills the slots stay in the engine.
- **Found on disk, never fetched** ([L5](0007_LAW-5-no-model-in-the-engine.md)), in this order: `CODELAW_SKILLS` (a folder); `node_modules/@codelaw/skills` from the project root upwards; `codelaw_skills/` in the site-packages the rule packs are searched in (`CODELAW_SITE_PACKAGES`, else `$VIRTUAL_ENV`, else `.venv/`); the binary's own environment (a `pipx` or `uvx` codelaw), and `node_modules` above it. `crates/codelaw-agents/src/package.rs`.
- **Only this binary's version is used.** A copy whose `codelaw-skills.json` names another version is passed over; when none matches, the command stops.
- **Refusals, exit 2, nothing written:**
  - none installed → the error names `npm i -D @codelaw/skills@<version>` and `pip install codelaw-skills==<version>`, and every place it looked;
  - another version → the error names the folder, its version and this codelaw's, and the same two install lines;
  - `init --agents --no-install` refuses before writing any file; `init --agents` with the install adds the skills package, at the binary's version, to the install command of the registry the repo uses.
- **The output is unchanged** for the same version: the files are the templates the binary embedded before, so `sync-agents` writes byte-for-byte what it wrote before the split, from either registry's copy (`crates/codelaw-cli/tests/commands.rs`, `scripts/smoke.sh`).
- **This repo** reads `templates/agents/` directly in its own tests; `scripts/package.py stage-skills <dir>` writes the package folder for a local `CODELAW_SKILLS`.

#### 9. The Gate: `codelaw hook --pre`

**Ruled by Arpit, 2026-10-27 (W-05, R1 and R3):** *"Yes to all three."* — build `hook --pre` for every agent with a pre-write hook, fail-open, deny only on new `error`s; a fourth surface kind, *Gate*. Written by an agent on that ruling, in the change that built it (W-93). `crates/codelaw-agents/src/pre.rs`.

- **Input.** The agent's hook payload on stdin, read once. `--agent claude|codex|copilot|vscode|kiro|cursor|gemini|windsurf` names the answer's shape; without it the shape is read off the payload — `hook_event_name: "BeforeTool"` is Gemini, `pre_write_code` (in `hook_event_name` or `agent_action_name`) is Windsurf, a `toolArgs` key is Copilot, anything else Claude Code. The arguments are `tool_input`, `toolArgs` (an object, or a JSON string holding one) or `tool_info`; a relative path resolves against the payload's `cwd`.
- **The rebuilt buffer.** For each file the write touches, the text the file will hold once the tool has run, built in memory from the file on disk and the payload, exactly as the tool builds it:
  - a whole file — `content` (Claude Code `Write`, Gemini `write_file`), `file_text` or `text` — is the new text;
  - one replacement — `old_string` / `old_str` → `new_string` / `new_str` — must match exactly once, or every time with `replace_all` or `allow_multiple`, or exactly `expected_replacements` times; an empty `old_string` on a missing file creates it;
  - a list — `edits[]` (Claude Code `MultiEdit`, Windsurf) — applies each replacement in order to the result of the one before;
  - an `apply_patch` envelope (Codex; `tool_input.command` as a string, or the last word of a list) — `*** Add File` is the `+` lines; `*** Update File` applies each `@@` chunk's context and `-` lines → context and `+` lines, in order, each searched from where the last one ended, exactly first, then ignoring trailing and then surrounding whitespace, as Codex does; `*** Move to` judges the result under the new path; `*** Delete File` is not judged;
  - **CRLF is kept:** a replacement whose `old` text has bare LF and does not match a CRLF file matches, and inserts, with CRLF.
  - **When the tool would fail** — the old text is missing or not unique, the file cannot be read, a shape codelaw does not know — that file gets no verdict: the write is allowed and the tool fails on its own. codelaw never blocks on a mismatch it caused.
- **What counts: only what the write adds.** The project is loaded once, with the `ci` profile (decision 6; `--profile` picks another), and both texts are checked under the file's path with MCP `check_snippet`'s engine call: the file on disk (none for a new file) and the rebuilt buffer. A finding of the rebuilt buffer is new when its rule and message occur more often there than on disk; among equal ones, those whose line's text the disk does not have are the new ones. `format/*` and `codelaw/*` findings do not count: the post-write hook formats, and a rule that fails is codelaw's own failure. A file with no [L3](0005_LAW-3-languages.md) extension is skipped.
- **The verdict.**
  - **A new `error` → deny**, the reason one line per error, up to 20 ([SR-CLI-MCP](0115_cli-and-mcp.md) decision 4's `agent` lines): `file:line:col error <rule> <message> — <fix>`, the fix the rule's ([L4](0006_LAW-4-every-rule-has-a-fix.md)): an `auto` or `suggest` edit as `replace "<old>" with "<new>"`, a `manual` fix as its steps. A fix is named, never applied through the agent's input (`updatedInput` is not portable); the post-write hook applies `auto` fixes on disk.
  - **Only new warnings → allow**, the warnings handed to the agent as `additionalContext` on Claude Code and Codex; the other agents have no confirmed channel for it, so they get nothing and the post-write hook reports them.
  - **Nothing new → allow**, silently.
  - **Each agent's shape**, all exit `0` unless said: Claude Code and Codex `{"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny", "permissionDecisionReason": …}}`; Copilot and VS Code `{"permissionDecision": "deny", "permissionDecisionReason": …}`; Gemini `{"decision": "deny", "reason": …}`; Cursor `{"permission": "deny", "agent_message": …, "user_message": …}`; Kiro and Windsurf exit `2` with the reason on stderr. An allow never says `allow`: on Claude Code that would skip the user's permission prompt.
- **Fail open, loudly.** A payload that is not JSON or names no file, a config or rule error, a panic in the engine, or a run longer than `--timeout-ms` (5,000 by default) → the write is **allowed**, exit `0`, and one line on stderr: `codelaw hook --pre: <why>; the write is allowed`. Copilot treats any non-zero exit as a deny, so codelaw's own failure must never be one. The post-write hook still runs.
- **A guardrail, not a boundary.** A shell write (`sed -i`, `>`) passes no pre-write hook; the post-write hook (decision 6) stays the backstop.
- **Speed.** [L8](0010_LAW-8-performance-budgets.md)'s `hook_pre`.

### The skill catalogue

**P1** ships with the MCP server. **P2** follows once rules can be authored outside the codelaw repo. **P3** comes later.

| Pri | Skill | Fires on (trigger summary) | What it does | Writes files? |
|---|---|---|---|---|
| **P1** | `codelaw-write` | Any create or edit of a JS/TS/Python/JSON/YAML/TOML/Markdown/CSS/HTML file in a repo with `codelaw.toml` | Calls `rules_for_path` first, writes code that complies, runs `codelaw check` on the changed files, and handles what comes back by fix class | Code only |
| **P1** | `codelaw-fix` | "fix lint errors", "codelaw is failing", a hook or CI reported violations | `codelaw fix`; shows `suggest` diffs and applies them on accept; works through `manual` steps (including writing the test `graph/untested-function` and `graph/function-coverage` ask for, and asking before any `.codelawignore` line); re-checks until clean | Code only |
| **P1** | `codelaw-explain` | "why did this fire", "what does rule X want", "is this allowed" | `explain_rule` + the Do/Don't examples + which preset or pack enabled it (`rules --why`) | No |
| **P1** | `codelaw-author-rule` | "make a rule that…", "enforce X", "we always do Y" | Turns a convention into a rule file:<br>1. Inspects the syntax tree of a real example.<br>2. Writes the matcher.<br>3. Picks the fix class ([L4](0006_LAW-4-every-rule-has-a-fix.md)).<br>4. Writes Do/Don't for every declared language ([L3](0005_LAW-3-languages.md)).<br>5. Runs `codelaw test` until green. | `.codelaw/rules/` |
| **P2** | `codelaw-setup` | "set up codelaw", "add a linter that agents follow" | Detects languages and package manager, installs the engine and the matching rule packs from npm or PyPI ([L6](0008_LAW-6-python-and-node-packages.md), [SR-RULE-PACKS](0101_rule-packs.md) decision 6), and says which registry has a pack `init` left out (since 2026-10-24 not every pack is on both), picks presets, wires pre-commit and CI, and adds agents only if asked | Config + CI |
| **P2** | `codelaw-migrate` | "move from ESLint / Ruff / markdownlint", "turn our style guide into rules" | Runs `codelaw migrate <tool>` first for ESLint, Prettier, Biome, Ruff and stylelint ([SR-CLI-MCP](0115_cli-and-mcp.md) decision 15) and works from its `changed` and `unmapped` lines; maps the rest (html-validate, markdownlint) rule by rule through `ported_from:` ([SR-RULE-FORMAT](0112_rule-format.md) decision 2), then to presets and adapters, turns prose conventions (CONTRIBUTING.md, ADRs) into draft rules, and writes a mapping report of what was covered, dropped or needs a human | Rules + report |
| **P2** | `codelaw-debug-rule` | "false positive", "rule didn't catch this", "rule test failing" | Reproduces the case as a failing example **first**, inspects the tree, adjusts the matcher, and keeps every existing example green | `.codelaw/rules/` |
| **P2** | `codelaw-ci-triage` | A CI codelaw job failed, or a SARIF file or log is pasted | Reads SARIF, reproduces locally with the same flags, fixes or routes to `codelaw-fix` / `codelaw-debug-rule` | Code only |
| **P3** | `codelaw-review` | "review this diff/PR against our rules" | Runs `check` on the changed files only and writes review comments that quote each violation's fix (diff or steps); adds no verdicts of its own | No |
| **P3** | `codelaw-suppress` | "this violation is intended", "ignore this rule here" | Picks the narrowest suppression (inline with reason → per-glob override), refuses a blanket disable, and records the reason | Code / config |
| **P3** | `codelaw-pack` | "share our rules across repos", "publish a rule pack" | Scaffolds a pack ([SR-RULE-PACKS](0101_rule-packs.md)) with its manifest, versioning, tests, and npm/PyPI publish config | Pack folder |
| **P3** | `codelaw-upgrade` | "upgrade codelaw", a new version is available | Reads the changelog, runs the new rules in preview, and fixes or adopts them deliberately | Config + code |

**Custom agent: `codelaw-rule-author`.** A subagent version of `codelaw-author-rule`. Rule authoring is iterative (inspect the tree → write → test → repeat) and floods the context with syntax trees, so it runs in its own context and returns only the finished rule and the test result.

**Slash commands come free.** Each skill can be invoked directly (for example `/codelaw-fix`) in agents that expose skills as commands. No separate command files are generated.

### Research basis

- **Agent Skills** is an open `SKILL.md` standard supported by 30+ coding tools: `name` and `description` frontmatter, three-stage progressive disclosure, and optional `scripts/`, `references/` and `assets/` folders. Codex reads `.agents/skills/` ([Vaughan, May 2026](https://codex.danielvaughan.com/2026/05/05/agent-skills-open-standard-portable-skills-codex-cli-cross-agent/); [Noqta](https://noqta.tn/en/news/skill-md-open-standard-30-ai-coding-tools-adoption-2026)).
- **Kiro:** steering with `inclusion: always | fileMatch | manual`; skills in `.kiro/skills/`; Powers bundle MCP servers, steering and hooks ([Kiro guide](https://medium.com/@Sanjjushri/advanced-kiro-steering-strategies-skills-powers-and-production-workflows-b68153ba88c2); [Kiro changelog](https://kiro.dev/changelog/remote-mcp-and-global-steering)).
- **Copilot:** `AGENTS.md` / `copilot-instructions.md`, `.github/instructions/*.instructions.md` (`applyTo`), `.github/prompts/` (VS Code only), `.github/agents/*.agent.md`, `.github/skills/*/SKILL.md` ([example repo docs](https://forge.saif.com/about/contributing/copilot-skills/)); hooks ([GitHub docs](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/use-hooks)).
- **Codex:** `PostToolUse` and the other hooks are stable since v0.124 ([Vaughan, Apr 2026](https://codex.danielvaughan.com/2026/04/23/codex-cli-hooks-graduate-stable-v0124-mcp-observation-inline-config/)).
- **Codex hooks** in `config.toml`: `[[hooks.PostToolUse]]` matcher groups with `[[hooks.PostToolUse.hooks]]` `type = "command"` handlers ([OpenAI docs](https://developers.openai.com/codex/hooks)).
- **Kiro hooks** in `.kiro/hooks/`: the v1 schema's `trigger` (`PostFileSave`) and `action` (`type: command`), with `{{filePath}}` for file triggers ([Kiro hooks](https://kiro.dev/docs/hooks/)). Older Kiro versions used a `when`/`then` `.kiro.hook` file; decision 7 follows the v1 schema.
- **Claude Code:** `PostToolUse` hooks, where exit code 2 sends stderr back to the agent ([Anthropic docs](https://docs.anthropic.com/en/docs/claude-code/hooks)).
- **The pre-write hooks** (decision 9 and decision 2's pre-write table), fetched 2026-10-27 for W-05: Claude Code [hooks](https://code.claude.com/docs/en/hooks) and [tools](https://code.claude.com/docs/en/tools-reference); Codex [hooks](https://developers.openai.com/codex/hooks); Copilot [hooks reference](https://docs.github.com/en/copilot/reference/hooks-reference); VS Code [agent hooks](https://code.visualstudio.com/docs/agent-customization/hooks); [Kiro hooks](https://kiro.dev/docs/hooks/); [Cursor hooks](https://cursor.com/docs/hooks); [Gemini CLI hooks](https://geminicli.com/docs/hooks/reference/); [Windsurf / Devin hooks](https://docs.devin.ai/desktop/cascade/hooks). The full matrix, with dates and what is unconfirmed: [the proposal](../work/proposals/write-time-guidance.md).
- **Prior art for "instruct never blocks; enforce only when exact":** the same principle in Arpit's Fux repo, in the `.claude/hooks/fux-index-hint.sh` header.

### Consequences

- **codelaw owns its generated files in consumer repos;** teams must not hand-edit them.
- **The Cursor and Gemini CLI rows wait for their paths to be verified.**
- **The Cowork plugin path and the Kiro hook schema (decision 7) are the least verified rows;** the veto check covers them first.
- **The Gate refuses only what it can rebuild exactly** (decision 9): a write it cannot rebuild, and every shell write, reaches disk and meets the post-write hook. The Copilot, Kiro and Cursor pre-write payloads were unconfirmed on 2026-10-27, so their answers are built from the docs and `sync-agents` writes no pre-write hook for Copilot or Kiro until a fixture of each is captured.
- **Copilot ignores a `postToolUse` hook's output,** so on Copilot the hook applies `auto` fixes but cannot hand findings back; the always-on block's "run `codelaw check`" carries that.
- **Evals cost CI time for each agent.**
- **A consumer that runs `sync-agents` or `init --agents` installs one more package** (decision 8), and upgrades it with codelaw; a mismatch is refused, never silently served.

### Alternatives considered

- **Hand-written files per agent:** rejected. They drift.
- **AGENTS.md only:** rejected. No path scoping, no hooks, no skills.
- **One skill that does everything:** rejected. It breaks the token budget and triggers poorly.

### Reference (required)

- The research basis under decision 2 (Agent Skills standard, Kiro, Copilot, Codex and Claude Code docs, each linked).

### Veto condition

**Reopen this decision if:** an agent in the decision-2 matrix changes where it reads skills, instructions, hooks or MCP config, an agent of the pre-write table changes its payload or its refusal fields (decision 9), or the binary carries an agent file again (decision 8).

**How to check it:** `re-verify each row of the decision-2 matrix against that agent's current docs; a mismatch reopens this record`

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-RULE-PACKS](0101_rule-packs.md) · [SR-RULE-FORMAT](0112_rule-format.md) · [SR-PACKAGES](0107_packages.md)

**Work items** — [W-77](../work/open/W-77-skills-package.md) · [W-93](../work/open/W-93-hook-pre-and-brief.md)

**Proposals** — [`work/proposals/write-time-guidance.md`](../work/proposals/write-time-guidance.md)

**Project docs** — [`docs/DESIGN.md`](../docs/DESIGN.md)

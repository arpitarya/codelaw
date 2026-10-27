---
type: Proposal
title: Guide the agent while it writes — `codelaw hook --pre` and a repo-specific brief
description: "W-05's research outcome: which agents can be checked before a write lands, how codelaw rebuilds and judges the proposed file, and why the brief is cut by what is specific to this repo, not by construct."
status: graduated
timestamp: 2026-10-27T08:30:00+05:30
---

# Guide the agent while it writes — W-05's proposal

**For the decider to pick.** Three rulings close W-05 (§6); the build item follows. Researched 2026-10-27 (Cowork, Opus 5.5): the agents' hook docs (sources §7) and codelaw's own 652 rules (§3).

## 1 · In one line

**Build `codelaw hook --pre` now — every major agent can show codelaw a write before it lands — and cut the brief by what is specific to this repository, not by syntax construct, because the construct plan does not survive the data.**

## 2 · Which agents can be checked before a write (step 1)

| agent | pre-write event | full new file in the payload? | refuse with a reason the model reads | config | status |
|---|---|---|---|---|---|
| **Claude Code** | `PreToolUse`, matcher `Write\|Edit` | `Write`: yes · `Edit`: no — `file_path`, `old_string`, `new_string`, `replace_all` | `hookSpecificOutput.permissionDecision: "deny"` + `permissionDecisionReason`, or exit 2 + stderr | `.claude/settings.json` | GA since v1.0.38 (2025-06-30) |
| **Codex CLI** | `PreToolUse` (`apply_patch` matches as `apply_patch`/`Edit`/`Write`) | no — patch text in `tool_input.command`; may touch several files | same `permissionDecision` fields; exit 2 | `~/.codex/hooks.json`, `config.toml [hooks]`, repo `.codex/` (trusted) | stable since v0.124.0 (2026-04-23) |
| **Copilot CLI / cloud agent** | `preToolUse` (also accepts `PreToolUse`) | argument field names unconfirmed | `permissionDecision`; **any non-zero exit denies** | `.github/hooks/*.json` | ship date unconfirmed |
| **VS Code agent mode** | Copilot's format; `toolName`/`toolArgs` | unconfirmed | `permissionDecision: "deny"` + reason | `.github/hooks/*.json`; also reads `.claude/settings.json` | **preview** |
| **Kiro** | `PreToolUse`, matcher category `write` | unconfirmed (`tool_name`, `tool_input` in the example) | `command` action exits 2 → blocks | `.kiro/hooks/*.json` (`version: v1`) | v1 format; date unconfirmed |
| **Cursor** | `preToolUse`, matcher `Write` (there is **no** `beforeFileEdit`) | unconfirmed | `permission: "deny"` + `agent_message`; exit 2 | `.cursor/hooks.json`; also loads `.claude/settings*.json` | 1.7 (2025-09-29), beta |
| **Gemini CLI** | `BeforeTool` | `write_file`: yes (`file_path`, `content`) · `replace`: no | `decision: "deny"` + `reason`; exit 2 | `.gemini/settings.json` | v0.26.0 (2026-01-28), on by default |
| **Windsurf (Devin Desktop)** | `pre_write_code` | no — `edits[{old_string,new_string}]` | exit 2 + stderr | `.devin/hooks.json` | unconfirmed; docs ask for < 100 ms |
| **Claude Cowork** | plugin `hooks/hooks.json` "loads" | unconfirmed | unconfirmed | plugin only | a report says plugin hooks never fire in Cowork — treat as unreliable |

**What it means**

- **Five of SR-AGENT-SURFACES' five agents have a pre-write hook** (Cowork's is unreliable); Cursor, Gemini and Windsurf have one too. The 2026-10-20 assumption — only Claude Code — is out of date.
- **There is no cross-agent standard, but Claude Code's shape is winning**: Cursor and VS Code read `.claude/settings.json`, Copilot accepts `PreToolUse`, Codex uses the same `permissionDecision` fields.
- **Most writes arrive as fragments.** Only Claude Code's `Write` and Gemini's `write_file` carry the whole new file; every edit tool sends a replacement or a patch. The W-05 hazard ("rebuild the whole file") is the main design constraint, confirmed.
- **Rewriting the agent's input is not portable** (`updatedInput` is ignored for Claude Code's `Edit` per a closed bug; Codex accepts only a `command` string; Windsurf and Kiro have none). The proposal's "name the fix, never apply it through `updatedInput`" stands.
- **A pre-write hook is a guardrail, not a boundary**: shell writes (`sed -i`, `>`) bypass it (Codex says so in its docs). The post-write hook stays as the backstop.

## 3 · The construct plan does not survive the data (step 3)

The plan was: map each rule to the constructs it judges (function, class, import, …) from its matcher's root node kind, so `brief_for_task(paths, constructs)` returns a handful. Measured over all 652 rules (2026-10-27):

| how the rule matches | rules | construct derivable? |
|---|---|---|
| `native:` (Rust) | 412 | no — no declared root |
| `match:` with a pattern, no `kind` | 58 | no |
| `match:` rooted at a generic node (`call`, `string`, `identifier`, `pair`, …) | ~115 | no — a call can be anywhere |
| `match:` rooted at a construct node (function, export, import, jsx, loop, condition) | ~20 | yes |
| `graph:` / `target:` | 17 | no — project-level |

**About 20 of 652 rules (3 %) map to a construct.** And excluding `auto`-fixed rules (the post-hook fixes those anyway) barely helps: under the recommended presets a JS file still has **174** rules to brief (12 of 186 are `auto`), TS 174, Python 138, HTML 69, CSS 39. At ~60 tokens a rule that is ~10,000 tokens before every function — the gap W-05 named, unsolved by constructs.

## 4 · The proposal

### 4a · `codelaw hook --pre` — build it

1. **Rebuild the file the way the agent would.** Read the file on disk, apply the payload exactly as the tool does (exact match, uniqueness, `replace_all`/`allow_multiple`, `edits[]` in order, CRLF kept; Codex's `apply_patch` parsed per file, add/update/delete/move). If the old text is not found, **allow** — the tool will fail on its own; never block on a mismatch codelaw caused.
2. **Judge only what the edit adds.** Check the rebuilt buffer under its path with the `ci` profile (as `codelaw hook` does today) and compare with the file on disk: only **new** diagnostics count.
3. **Verdict.** A new `error` → deny, with `--format agent` lines: rule id, line, the one-line fix (L4). New `warning`s → allow, with them as context where the agent supports it. `auto` fixes are named, never applied through the agent's input; the post-write hook applies them on disk.
4. **One verdict, one adapter per agent** for the output: `permissionDecision` JSON (Claude Code, Codex, Copilot, VS Code), `decision`/`reason` (Gemini), `permission`/`agent_message` (Cursor), exit 2 + stderr (Windsurf, Kiro). Payload fixtures pinned per agent version (the SR-AGENT-SURFACES veto already covers drift).
5. **Fail open, loudly.** On a crash, a timeout or an unknown payload the write is **allowed** and a line goes to stderr — Copilot treats any non-zero exit as a deny, so codelaw must exit 0 on its own errors. The post-write hook still runs.
6. **Speed.** Today's `fix_one_file` is p95 76 ms on the runner and the check itself 3–6 ms (`watch_rerun`); the rest is loading the project. Target the same < 100 ms warm class through the warm process (W-15 daemon); Windsurf's < 100 ms guidance is the tightest.
7. **`codelaw init --agents` writes it** next to the post-write hook, for the agents above whose payload is confirmed; the others get it once a fixture is captured.

### 4b · The brief — cut by what is specific to this repository

An agent already knows ESLint's and Ruff's ported correctness rules; what it cannot know is **this repo's choices**. So `codelaw brief --for <paths>` / MCP `brief_for_task(paths)` returns, deterministically and in this order:

1. the repo's **own rules** (`.codelaw/rules/`) that apply to the path;
2. pack rules whose **options this repo sets** in `codelaw.toml` (naming patterns, size limits, `use-instead`, approved calls, sanitizers) — a choice the model cannot guess;
3. rules with **open baseline entries or exceptions under the path** (SR-BASELINE) — what this code has actually got wrong;
4. the path's **security** rules (`security: true`, 162 rules) whose fix is `manual`;
5. a closing line: *"the other N rules are standard (ESLint/Ruff/…); `hook --pre` will refuse a violation with its fix"*.

Same inputs → same list (L5); no ranking, no free text. `constructs` stays an optional filter, honoured only by rules that declare `constructs:` (custom rules may; no bulk tagging of 652 rules). Fux remains the optional free-text layer (W-34, Fux item 298), never a dependency.

### 4c · Records (step 5) — drafted when the rulings land

- **SR-AGENT-SURFACES:** a fourth surface kind, **Gate** (pre-write, can block), beside Instruct · Query · Enforce; the agent matrix gains the pre-write column above, with its sources.
- **SR-CLI-MCP:** `codelaw hook --pre`, `codelaw brief --for`, MCP `brief_for_task`; fail-open stated.
- **SR-LAW-8:** a `hook_pre` operation (one rebuilt file, warm), target < 100 ms, budget set by the ruled method from CI.
- **Glossary:** *gate*, *brief*, *rebuilt buffer*.

## 5 · What the build item would do, in order

| step | what | gives |
|---|---|---|
| 1 | the rebuild + diff-of-diagnostics core, with Claude Code `Write`/`Edit` fixtures | the verdict, testable without an agent |
| 2 | adapters: Claude Code, Codex (`apply_patch` parser), Gemini, Cursor, Copilot/VS Code; Windsurf and Kiro by exit code | one verdict, every agent |
| 3 | `init --agents` writes the pre hook; `sync-agents --check` covers it | adoption |
| 4 | `brief` + `brief_for_task` per §4b | the before moment, short |
| 5 | `hook_pre` measured in CI's `perf`; budget per L8's ruled method | L8 |
| 6 | README: While moves from 🧪 to ✅; W-35 closes | the story, true |

## 6 · Rulings asked of the decider

| # | question | recommendation |
|---|---|---|
| R1 | Build `hook --pre` for all agents with a confirmed payload, fail-open, deny only on new `error`s? | **Yes** |
| R2 | Brief by what is specific to this repo (§4b) instead of by construct? | **Yes** — the construct plan covers 3 % of rules |
| R3 | Add the fourth surface kind **Gate** to SR-AGENT-SURFACES and `hook_pre` to L8? | **Yes**, drafted with the build item |

**Alternatives weighed:** *tag all 652 rules with `constructs:`* (hours of hand-tagging, still ~170 rules per JS file, and it rots as rules change) · *no brief, gate only* (works, but the agent learns only by being refused, which costs a turn per mistake) · *apply auto-fixes through `updatedInput`* (not portable; Claude Code ignores it for `Edit`).

## 7 · Sources (fetched 2026-10-27)

Claude Code hooks — code.claude.com/docs/en/hooks; tools reference — code.claude.com/docs/en/tools-reference · Codex hooks — developers.openai.com/codex/hooks; changelog — developers.openai.com/codex/changelog · Copilot hooks — docs.github.com/en/copilot/reference/hooks-reference · VS Code agent hooks — code.visualstudio.com/docs/agent-customization/hooks · Kiro hooks — kiro.dev/docs/hooks/ · Cursor hooks — cursor.com/docs/hooks; 1.7 changelog — cursor.com/changelog/1-7 · Gemini CLI hooks — geminicli.com/docs/hooks/reference/ · Windsurf/Devin hooks — docs.devin.ai/desktop/cascade/hooks · MCP elicitation and tools — modelcontextprotocol.io/specification/2025-06-18. Unconfirmed cells are marked; check them against the docs when capturing each fixture.

**Plain words:** every big coding agent can now let codelaw look at a change before it is saved and say "no, here is the fix". Most send only "replace this with that", so codelaw rebuilds the whole file first. And instead of reciting all ~170 standard rules before every edit, it tells the agent only what is special about this repo.

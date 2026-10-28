---
type: Handoff
name: W-93
description: "Arpit, 2026-10-27: build W-05's ruled design — codelaw hook --pre checks an agent's proposed write before it lands, and codelaw brief lists only what is specific to the repo."
item: W-93
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-93 — `codelaw hook --pre` and the repo-specific brief

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 0 | Records first: SR-AGENT-SURFACES gains the surface kind **Gate** and the pre-write column of the agent matrix; SR-CLI-MCP gains `hook --pre`, `brief --for`, MCP `brief_for_task`, fail-open; SR-LAW-8 gains `hook_pre` (< 100 ms warm target); glossary *gate*, *brief*, *rebuilt buffer* | the spec before the code | ✅ 2026-10-27: SR-AGENT-SURFACES §1, decisions 2, 6, 7, 9; SR-CLI-MCP decisions 1, 3, 5, 8, 10, 14; SR-LAW-8 decision 1 and §Measurements; SR-LAW-10 decision 7 (the fixtures table); glossary *brief*, *pre-write check*, *gate*, *rebuilt buffer* |
| 1 | The core: rebuild the post-edit file from disk + payload exactly as the tool would (exact match, `replace_all`/`allow_multiple`, `edits[]` in order, CRLF kept); old text not found → allow; check under its path, `ci` profile; only **new** diagnostics count | one verdict, testable with no agent | ✅ `crates/codelaw-agents/src/pre.rs` (`rebuild`, `added`, `judge`) |
| 2 | Adapters with pinned payload fixtures: Claude Code (`Write`, `Edit`), Codex (`apply_patch` parser, multi-file), Gemini (`write_file`, `replace`), Cursor, Copilot / VS Code; Windsurf and Kiro by exit 2 + stderr | every agent with a pre-write hook | ✅ Claude Code, Codex, Gemini, Copilot / VS Code, Cursor, Windsurf, Kiro shapes, unit-tested; Claude, Codex, Gemini and Kiro through the binary (`crates/codelaw-cli/tests/hook_pre.rs`). Copilot, Kiro and Cursor fixtures are built from the docs, not captured payloads |
| 3 | Verdict: new `error` → deny with `--format agent` lines and each fix; new `warning`s → allow + context; `auto` fixes named, never applied through the agent's input; own crash/timeout/unknown payload → **allow**, exit 0, a stderr line | safe to switch on | ✅ deny / allow + context / silent; fail-open on bad payload, config error, panic, timeout (`--timeout-ms`) |
| 4 | `codelaw init --agents` writes the pre hook beside the post hook; `sync-agents --check` covers it | adoption | ✅ Claude Code `PreToolUse`, Cowork plugin, Codex snippet (`templates/agents/claude/pre-hook-group.json`); Copilot and Kiro wait for a captured payload (SR-AGENT-SURFACES decision 2) |
| 5 | `codelaw brief --for <paths>` / MCP `brief_for_task(paths)`: the repo's own rules → pack rules whose options the repo sets → rules with open baseline entries or exceptions under the path → the path's `manual` security rules → one line naming how many standard rules remain; `constructs` only for rules that declare `constructs:` | the before moment, short | ✅ `codelaw brief --for` and MCP `brief_for_task` (`crates/codelaw-report/src/brief.rs`); `constructs` not built (no rule declares it) |
| 6 | `hook_pre` in CI's `perf`; its budget by L8's ruled method | L8 | ✅ `hook_pre` in CI's `perf`; the method held it back (worst p95 86.2 ms × 1.5 = 129.3 > 100); ruled 2026-10-27: left `unset` — reported, not gated, as `fix_one_file` |
| 7 | README's *While* row and hero move to ✅; W-35 then closes | the story, true | ✅ README's Before and While rows; W-35 closed |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** *"How about we do it 150 milliseconds?"* — then, told 150 sits above both the 100 ms target and the method's 129.3, and that a set budget is a hard CI gate that only tightens: *"We can go with unset"* (the voice transcript reads "unsit"). `hook_pre` stays `unset`; perf/budgets.toml and SR-LAW-8 say so.

**Arpit, 2026-10-27 (voice):** *"Yes to all three."* — on W-05's R1 (build `hook --pre` for every agent with a pre-write hook, fail-open, deny only new `error`s), R2 (brief by what is specific to this repo) and R3 (the surface kind *Gate* and an L8 budget). [Proposal](../proposals/write-time-guidance.md).

**Model:** Opus.

Design, sources and the eight-agent matrix: [`work/proposals/write-time-guidance.md`](../proposals/write-time-guidance.md) (graduated into this item). Fux stays optional (W-34, Fux item 298).

## Definition of done

**State, 2026-10-27 (Claude Code):** the first three met (`crates/codelaw-cli/tests/hook_pre.rs`: Claude `Write`/`Edit`, Codex `apply_patch` and Gemini `write_file`/`replace` refused with rule, line and fix; a `js/no-console` warning allowed with context; an edit on the already-failing line silent; a malformed payload, a missing file and a broken config allowed with exit 0; the panic and the timeout in `pre::tests`; the brief's own / configured / exceptions / security groups and its count line, byte-identical across runs). The fourth is half met: `hook_pre` is benched in CI's `perf` from this change on, and its budget waits for three runs on `main`.

**Left (ruled 2026-10-27: unset):** close the item. Was: (1) three `perf` runs on `main` that time `hook_pre` (the push run plus two `workflow_dispatch` runs); (2) apply SR-LAW-8 decision 4's method to them in `perf/budgets.toml`, and put the result to the decider as an inbox row; then close.

- A proposed `Write` and `Edit` that add a `js/no-debugger` error are refused on Claude Code, Codex and Gemini fixtures with the rule, line and fix; one that adds only a warning is allowed with context; one that touches an already-failing line adds no new verdict.
- A malformed payload, a missing file and an engine panic each allow the write and exit 0.
- `brief --for src/app.ts` in a fixture with two custom rules, one configured option and one baseline entry lists exactly those, then the security rules, then the count line — byte-identical across runs.
- `hook_pre` measured in CI; L8 budget set or put to the decider by the ruled method.

## Record affected

SR-AGENT-SURFACES · SR-CLI-MCP · SR-LAW-8 · `docs/GLOSSARY.md` — in the same change as the code (step 0).

## Hazards

- Payload fields are each agent's API and drift between versions — pin fixtures per agent version; Copilot's argument names and Kiro's stdin shape were unconfirmed on 2026-10-27.
- Copilot treats any non-zero exit as a deny and lets a timeout through; Windsurf asks for < 100 ms — the warm process (W-15 daemon) is the path to that.
- Shell writes (`sed -i`, `>`) bypass every pre-write hook: the post-write hook stays the backstop.
- Cowork's plugin hooks were reported not to fire; do not count on Cowork.

## `hook_pre`'s budget — to the decider (2026-10-27)

Three `perf` runs on `main` (`ubuntu-24.04`, image 20261012.334.1): run 40456263074 (`4296fe9`) p95 62.4 ms; 40456826728 (`9d55072`) 78.3 ms; 40464780137 (`9d55072`) 86.2 ms (the push run 40456463956 skipped `perf`). SR-LAW-8 decision 4: worst 86.2 × 1.5 = 129.3 > the 100 ms target, so the guard holds it back — `perf/budgets.toml` keeps `budget = "unset"`, `proposed = 100`, and the numbers are in SR-LAW-8 §Measurements. The inbox asks: leave it unset (as for `fix_one_file`), or set 100 ms at 1.16 × headroom. W-93 closes on that ruling.


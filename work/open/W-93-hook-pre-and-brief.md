---
type: Handoff
name: W-93
description: "Arpit, 2026-10-27: build W-05's ruled design — codelaw hook --pre checks an agent's proposed write before it lands, and codelaw brief lists only what is specific to the repo."
item: W-93
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-93 — `codelaw hook --pre` and the repo-specific brief

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 0 | Records first: SR-AGENT-SURFACES gains the surface kind **Gate** and the pre-write column of the agent matrix; SR-CLI-MCP gains `hook --pre`, `brief --for`, MCP `brief_for_task`, fail-open; SR-LAW-8 gains `hook_pre` (< 100 ms warm target); glossary *gate*, *brief*, *rebuilt buffer* | the spec before the code | ⏳ |
| 1 | The core: rebuild the post-edit file from disk + payload exactly as the tool would (exact match, `replace_all`/`allow_multiple`, `edits[]` in order, CRLF kept); old text not found → allow; check under its path, `ci` profile; only **new** diagnostics count | one verdict, testable with no agent | ⏳ |
| 2 | Adapters with pinned payload fixtures: Claude Code (`Write`, `Edit`), Codex (`apply_patch` parser, multi-file), Gemini (`write_file`, `replace`), Cursor, Copilot / VS Code; Windsurf and Kiro by exit 2 + stderr | every agent with a pre-write hook | ⏳ |
| 3 | Verdict: new `error` → deny with `--format agent` lines and each fix; new `warning`s → allow + context; `auto` fixes named, never applied through the agent's input; own crash/timeout/unknown payload → **allow**, exit 0, a stderr line | safe to switch on | ⏳ |
| 4 | `codelaw init --agents` writes the pre hook beside the post hook; `sync-agents --check` covers it | adoption | ⏳ |
| 5 | `codelaw brief --for <paths>` / MCP `brief_for_task(paths)`: the repo's own rules → pack rules whose options the repo sets → rules with open baseline entries or exceptions under the path → the path's `manual` security rules → one line naming how many standard rules remain; `constructs` only for rules that declare `constructs:` | the before moment, short | ⏳ |
| 6 | `hook_pre` in CI's `perf`; its budget by L8's ruled method | L8 | ⏳ |
| 7 | README's *While* row and hero move to ✅; W-35 then closes | the story, true | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all three."* — on W-05's R1 (build `hook --pre` for every agent with a pre-write hook, fail-open, deny only new `error`s), R2 (brief by what is specific to this repo) and R3 (the surface kind *Gate* and an L8 budget). [Proposal](../proposals/write-time-guidance.md).

**Model:** Opus.

Design, sources and the eight-agent matrix: [`work/proposals/write-time-guidance.md`](../proposals/write-time-guidance.md) (graduated into this item). Fux stays optional (W-34, Fux item 298).

## Definition of done

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

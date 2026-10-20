---
type: Queue
description: "The single live work queue, three lanes, plus the Blocked-on-the-decider inbox. The list only — its rules live in the record below."
governed_by: SR-WORK-OPEN-QUEUE
governed_by_path: records/0051_WORK-open-queue.md
---

# OPEN-WORK — what is still open

**This file is the LIST, plus the legend below.** Everything else — what the
queue is, an item's lifecycle, the shape of a row, ball precedence, ordering,
how the inbox works, and what every session owes — is stated once in
**[SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md)** and is not repeated
here. Read that record before changing anything below it.

**A ball on every row, always one:** 🔴 blocked on the decider, directly or through another item · 🟣 gated on a named date, directly or through another item · 🟡 waiting on another item · 🟢 no blockers.
**Optional, after the ball:** 🧨 broken or getting worse · 🔺 do first (the decider only). Full legend: [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md) rules 20–34.

---

## Blocked on the decider

| what the decider decides | filed | age |
|---|---|---|
| Place the five `.github/` issue and PR templates — the path is protected from remote writes ([W-06](open/W-06-consumer-readme.md)) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-06 | | |
| 🔴 **The one-time registry setup, steps 1, 4 and 5 only** — the `@codelaw` npm org, six PyPI pending publishers, the `npm`/`pypi` environments ([`RELEASING.md`](../RELEASING.md) §One time). Steps 2–3 and the ruling to publish wait on W-07, W-08, W-09 (L7, L6 decision 3) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-04 step 5 (the first release) · W-03 step 6 | | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; waits on W-07, W-08, W-09 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The five rule packs: steps 2–5 done (401 rules), packaging done. Step 6, publishing: blocked on the registry setup above; waits on W-07, W-08, W-09. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🔴 **W-06** `agent` README, CHANGELOG, CONTRIBUTING, SECURITY and CODE_OF_CONDUCT landed (filed as W-04; renumbered 2026-10-20, the number was taken by the release pipeline); blocked on the decider placing the `.github/` templates. Filed by Arpit (Cowork). [detail](open/W-06-consumer-readme.md)
🟢🧨 **W-17** `agent` Fix the engine defects from the 2026-10-20 review — a rule panic exits 0 unreported; a failed write still says "fixed"; nested configs ignored for named files. Filed by Arpit (Cowork). [detail](open/W-17-engine-defects.md)
🟢 **W-07** `agent` Build the formatter (`codelaw format`, the format step of `fix`) — L7: codelaw may not ship without it; the release gate refuses until it lands. Filed by Arpit (Cowork). [detail](open/W-07-formatter.md)
🟢 **W-08** `agent` Build the MCP server (`codelaw mcp`, SR-CLI-MCP's tools, CLI/MCP parity test) — L6 decision 3: no release without it. Filed by Arpit (Cowork). [detail](open/W-08-mcp-server.md)
🟢 **W-09** `agent` Build the language server (`codelaw lsp`) and the VS Code extension — L6 decision 3: no release without the LSP. Filed by Arpit (Cowork). [detail](open/W-09-lsp-and-extension.md)
🟢 **W-11** `agent` Build the project graph and `graph:` rules — `graph/no-unused-exports` first; measure false positives. Filed by Arpit (Cowork). [detail](open/W-11-project-graph.md)
🟢 **W-12** `agent` Build `target:` + `assert:` rules for JSON/YAML/TOML and the `sort-keys`/`ensure-key`/`when` fix primitives. Filed by Arpit (Cowork). [detail](open/W-12-structured-file-rules.md)
🟢 **W-14** `agent` Build the adapters (oxlint, Ruff, stylelint, html-validate) with manifests; CLI and CI only. Filed by Arpit (Cowork). [detail](open/W-14-adapters.md)
🟢 **W-15** `agent` Build the compiled rule-set cache, warm sockets, `--prefer-warm`, the daemon and incremental re-parse. Filed by Arpit (Cowork). [detail](open/W-15-warm-processes.md)
🟢 **W-16** `agent` Build `config --print/--schema`, `match --explain`, `fix --trace`, `--timings`, SARIF and the full npm/Python API. Filed by Arpit (Cowork). [detail](open/W-16-commands-and-api.md)
🟢 **W-21** `agent` Close the engine gaps holding rules out of the packs: tagged templates, native-rule file context, modern CSS in tree-sitter-css. Filed by Arpit (Cowork). [detail](open/W-21-pack-engine-gaps.md)
🟡 **W-10** `agent` Build hook mode, `sync-agents`, the skills and `init --agents`; waiting on W-08. Filed by Arpit (Cowork). [detail](open/W-10-agent-surfaces.md)
🟡 **W-13** `agent` Build baselines, deprecations, `--as-of` and `--changed-since`/`--staged`; waiting on W-11 for `graph/no-deprecated-usage`. Filed by Arpit (Cowork). [detail](open/W-13-baseline-and-time.md)

### testing

🟢🧨 **W-18** `agent` Harden CI and release: clippy's failures are masked (no `pipefail`); pin actions and npm by version; tag-only environments; verify checksums; all-platform install matrix. Filed by Arpit (Cowork). [detail](open/W-18-release-hardening.md)
🟢 **W-20** `agent` Add the missing conformance tests: L5 network fence, L2 for npm/PyPI tools, `init` paths, rule promotion and corpus, the work-system checks. Filed by Arpit (Cowork). [detail](open/W-20-conformance-tests.md)
🟢 **W-19** `agent` Build `codelaw bench`, measure every target, propose L8's budgets, add the CI perf gate. Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update

🟢 **W-22** `agent` Fix record drift: SR-RULE-PACKS presets and starter list, the "27 packages" count, `build-packs.py` vs `package.py`, the layer count, pack NOTICEs. Filed by Arpit (Cowork). [detail](open/W-22-record-drift.md)

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---

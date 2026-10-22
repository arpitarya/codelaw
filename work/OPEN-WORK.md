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
| 🔴 **The one-time registry setup, steps 1, 4 and 5 only** — the `@codelaw` npm org, six PyPI pending publishers, the `npm`/`pypi` environments ([`RELEASING.md`](../RELEASING.md) §One time). Steps 2–3 and the ruling to publish can follow: W-07, W-08 and W-09 have landed, and `law-gate.py` lists nothing (2026-10-21) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-04 step 5 (the first release) · W-03 step 6 | | |
| Give the `npm` and `pypi` environments one tag rule `v*` and no branch rule (GitHub Settings → Environments; [`RELEASING.md`](../RELEASING.md) step 5) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-18 | | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The five rule packs: steps 2–5 done (401 rules), packaging done. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🔴 **W-06** `agent` README, CHANGELOG, CONTRIBUTING, SECURITY and CODE_OF_CONDUCT landed (filed as W-04; renumbered 2026-10-20, the number was taken by the release pipeline); blocked on the decider placing the `.github/` templates. Filed by Arpit (Cowork). [detail](open/W-06-consumer-readme.md)
🟢 **W-23** `agent` Finish SR-FORMATTER: the layout-rule refusal in `codelaw test`, `.editorconfig`, `--import-config`, `codelaw-format-ignore`/`prettier-ignore` in every engine, LSP range formatting. Filed by Arpit (Claude Code). [detail](open/W-23-formatter-rest.md)
🟢 **W-11** `agent` Build the project graph and `graph:` rules — `graph/no-unused-exports` first; measure false positives. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-11-graph`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-11-project-graph.md)
🟢 **W-12** `agent` Build `target:` + `assert:` rules for JSON/YAML/TOML and the `sort-keys`/`ensure-key`/`when` fix primitives. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-12-structured-rules`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-12-structured-file-rules.md)
🟢 **W-14** `agent` Build the adapters (oxlint, Ruff, stylelint, html-validate) with manifests; CLI and CI only. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-14-adapters`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-14-adapters.md)
🟢 **W-15** `agent` Build the compiled rule-set cache, warm sockets, `--prefer-warm`, the daemon and incremental re-parse. Filed by Arpit (Cowork). [detail](open/W-15-warm-processes.md)
🟢 **W-16** `agent` Build `config --print/--schema`, `match --explain`, `fix --trace`, `--timings`, SARIF and the full npm/Python API. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-16-commands-api`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-16-commands-and-api.md)
🟢 **W-21** `agent` Close the engine gaps holding rules out of the packs: tagged templates, native-rule file context, modern CSS in tree-sitter-css. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-21-pack-gaps`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-21-pack-engine-gaps.md)
🟢 **W-10** `agent` Build hook mode, `sync-agents`, the skills and `init --agents`; W-08 has landed. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-08-mcp-W-10-agent-surfaces`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-10-agent-surfaces.md)
🟡 **W-13** `agent` Build baselines, deprecations, `--as-of` and `--changed-since`/`--staged`; waiting on W-11 for `graph/no-deprecated-usage`. Filed by Arpit (Cowork). [detail](open/W-13-baseline-and-time.md)

### testing

🔴 **W-18** `agent` CI and release hardened 2026-10-20 (pipefail, SHA-pinned actions, pinned npm, verified `SHA256SUMS` + attestation, eight-platform smoke job); left: the tag-only environment rule (decider, inbox) and the staging-index install. Filed by Arpit (Cowork). [detail](open/W-18-release-hardening.md)
🟢 **W-20** `agent` Add the missing conformance tests: L5 network fence, L2 for npm/PyPI tools, `init` paths, rule promotion and corpus, the work-system checks. Its unfinished work was merged into `main` as it was on 2026-10-22, on the decider's instruction (from `wip/W-20-conformance`, not built, not finished). Filed by Arpit (Cowork). [detail](open/W-20-conformance-tests.md)
🟢 **W-19** `agent` Build `codelaw bench`, measure every target, propose L8's budgets, add the CI perf gate. Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update


### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---

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
| 🔴 Place the five `.github/` issue and PR templates — the path is protected from remote writes ([W-06](open/W-06-consumer-readme.md)) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-06 | | |
| 🔴 **The one-time registry setup, steps 1, 4 and 5 only** — the `@codelaw` npm org, six PyPI pending publishers, the `npm`/`pypi` environments ([`RELEASING.md`](../RELEASING.md) §One time). Steps 2–3 and the ruling to publish can follow: W-07, W-08 and W-09 have landed, and `law-gate.py` lists nothing (2026-10-21) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-04 step 5 (the first release) · W-03 step 6 | | |
| 🔴 Give the `npm` and `pypi` environments one tag rule `v*` and no branch rule (GitHub Settings → Environments; [`RELEASING.md`](../RELEASING.md) step 5) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-18 | | |
| 🔴 Two pack rules fail the layout check: `js/no-irregular-whitespace` (a layout rule: Biome turns its no-break space into a space) and `css/media-query-no-invalid` (Biome's `not (` makes an invalid query valid). Drop, rewrite or exempt each ([W-23](open/W-23-formatter-rest.md)) | 2026-10-22 | 0 d |
| ↳ **blocks:** W-23 DoD 1 · `codelaw test packs/js` and `packs/css` pass | | |
| 🔴 Modern CSS needs `tree-sitter-css` changed: fork it in-tree (parser regenerated with the MIT tree-sitter CLI) or patch upstream and wait; either is an L2 check and a record ([W-21](open/W-21-pack-engine-gaps.md)) | 2026-10-22 | 0 d |
| ↳ **blocks:** W-21 step 3 | | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The five rule packs: steps 2–5 done (401 rules), packaging done. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🔴 **W-06** `agent` README, CHANGELOG, CONTRIBUTING, SECURITY and CODE_OF_CONDUCT landed (filed as W-04; renumbered 2026-10-20, the number was taken by the release pipeline); blocked on the decider placing the `.github/` templates. Filed by Arpit (Cowork). [detail](open/W-06-consumer-readme.md)
🔴 **W-23** `agent` Finish SR-FORMATTER: all five steps landed and verified on `main` 2026-10-22; DoD 1 waits on the decider for two pack rules that fail the layout check (inbox). Filed by Arpit (Claude Code). [detail](open/W-23-formatter-rest.md)
🟢 **W-11** `agent` Project graph and `graph:` rules built and verified on `main` 2026-10-22, with the six `node/` import rules; left: judge the corpus false-positive rate (step 4). Filed by Arpit (Cowork). [detail](open/W-11-project-graph.md)
🔴 **W-21** `agent` Engine gaps: tagged templates, native-rule file context and three rules landed on `main` 2026-10-22; modern CSS needs a `tree-sitter-css` fork, the decider's call (inbox). Filed by Arpit (Cowork). [detail](open/W-21-pack-engine-gaps.md)
🟢 **W-13** `agent` Build baselines, deprecations, `--as-of` and `--changed-since`/`--staged`; W-11's graph, which `graph/no-deprecated-usage` needs, landed on `main` 2026-10-22. Filed by Arpit (Cowork). [detail](open/W-13-baseline-and-time.md)

### testing

🔴 **W-18** `agent` CI and release hardened 2026-10-20 (pipefail, SHA-pinned actions, pinned npm, verified `SHA256SUMS` + attestation, eight-platform smoke job); left: the tag-only environment rule (decider, inbox) and the staging-index install. Filed by Arpit (Cowork). [detail](open/W-18-release-hardening.md)
🟢 **W-20** `agent` Conformance tests landed and verified 2026-10-22 (L5 fence, L2 over npm/PyPI tools, `init` paths, promotion gate, work-system checks); npm's Artistic-2.0 ruled permissive 2026-10-22; left: the pinned corpus and its CI job. Filed by Arpit (Cowork). [detail](open/W-20-conformance-tests.md)
🟢 **W-19** `agent` Build `codelaw bench`, measure every target, propose L8's budgets, add the CI perf gate. Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update


### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---

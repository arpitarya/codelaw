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
| 🔴 **The one-time registry setup** — the `@codelaw` npm org, the first hand publish of the 14 npm packages, `scripts/npm-trust.sh`, six PyPI pending publishers, the `npm` and `pypi` GitHub environments ([`RELEASING.md`](../RELEASING.md) §One time); then **the ruling to publish** 0.1.0 — the licence is ruled, MIT | 2026-10-20 | 0 d |
| ↳ **blocks:** W-04 step 5 (the first release) · W-03 step 6 | | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline: `ci.yml` builds the eight platform wheels and assembles, checks and install-smokes all 27 packages on every push; `publish.yml` uploads exactly that, by OIDC, from a published GitHub Release. Filed by Arpit (Claude Code). Steps 1–4 done 2026-10-20, merged into `main`; step 5, the first release, is 🔴 above. [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` Build the five rule packs (js, html, css, python, data) from the researched ESLint/Ruff/stylelint/html-validate presets, on the W-02 engine. Filed by Arpit (Cowork). Steps 2–5 done 2026-10-20 (401 rules); step 6's packaging done (W-04's `scripts/package.py`, and `scripts/build-packs.py` locally), MIT-licensed; publishing waits on the registry setup and the ruling above. [detail](open/W-03-rule-packs.md)

🔴 **W-06** `agent` README, CHANGELOG, CONTRIBUTING, SECURITY and CODE_OF_CONDUCT landed (filed as W-04; renumbered 2026-10-20, the number was taken by the release pipeline); blocked on the decider placing the `.github/` templates. Filed by Arpit (Cowork). [detail](open/W-06-consumer-readme.md)

### testing

### record update

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---

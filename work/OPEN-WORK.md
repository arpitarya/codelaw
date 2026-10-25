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

| what the decider decides | filed |
|---|---|
| 🔴 **PyPI 0.2.0 by hand, over 2–3 days** (four new projects a day): upload `codelaw-rules-{html,js,python,react}`, add six trusted publishers, then `gh release create v0.2.0` on W-37's tag — [W-38](open/W-38-pypi-0-2-0-by-hand.md) | 2026-10-20 |
| ↳ **blocks:** W-38 itself; nothing else in the queue | |

---

## Open items

### build

🔴 **W-38** `arpit` PyPI 0.2.0 by hand over 2–3 days: `css`, `data` live; left the other four packs (four a day), six trusted publishers, then the Release on W-37's tag brings `codelaw`. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟢 **W-39** `agent` Warn on a laptop, fail in CI: auto `ci`/`local` profiles (agents run `ci`), a per-glob severity `remap`, and `js/no-console`. Filed by Arpit (Cowork). [detail](open/W-39-ci-and-local-profiles.md)

🟢 **W-40** `agent` Exceptions with an expiry: opt-in `require_expiry`, a warning `warn_days` before, an error on the day, an optional `reason` and a git-checked `added_by`. Filed by Arpit (Cowork). [detail](open/W-40-dated-exceptions.md)

🟢 **W-41** `agent` `react/no-context`: old context use warns through a baseline, any new `createContext`/`useContext` errors. Filed by Arpit (Cowork). [detail](open/W-41-react-no-context.md)

🟢 **W-43** `agent` More React hooks rules from eslint-plugin-react-hooks 7 and @eslint-react — set-state in render/effect, refs and purity in render, nested components, `useMemo` returns. Filed by Arpit (Claude Code). [detail](open/W-43-react-hooks-rules.md)

🟡 **W-34** `agent` An optional Fux index package beside each pack (`@codelaw/rules-<name>-fux-index`, `codelaw-rules-<name>-fux-index`), never a default; waiting on Fux item 298, then L6's amendment. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-35** `agent` README leads with "your rules, before, while and after the AI writes"; waiting on W-05 to drop While's "in design" and restore Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing

### record update

### research

🟢 **W-36** `cowork` Research codelaw as the linter and formatter for AI — before, while, after and with no AI — beside ESLint + Prettier, Biome, Ruff, oxlint: gaps, provable claims, who adopts it. Filed by Arpit (Cowork). [detail](open/W-36-position-against-eslint-and-biome.md)

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` of the rules for a path and construct, and a `hook --pre` check before a write reaches disk; Fux as an optional layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---

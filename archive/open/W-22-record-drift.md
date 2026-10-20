---
type: Handoff
name: W-22
description: "Arpit, 2026-10-20: drift between records, docs and the repo that the 2026-10-20 review found."
item: W-22
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Claude Code (Opus 5.5), W-22 implementation"
ball: agent
---

# W-22 — Bring the records and release docs back in line with what was built

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Fix SR-RULE-PACKS: `js/strict` and `py/strict` don't exist yet; `md/relative-links` isn't shipped | the record says what ships | ✅ |
| 2 | Fix the release docs: "27 packages" is 27 files (20 packages); W-04 cites `LICENSE-*` files that are gone | the docs count right | ✅ |
| 3 | Fix SR-PACKAGES decision 9: `build-packs.py` doesn't build the same artefacts as `package.py` | one truth about the packs | ✅ (W-18's commit) |
| 4 | Fix the layer count (SR-ARCHITECTURE says six, `tests/layers.rs` five) | the record and its test agree | ✅ |
| 5 | Put the pack data provenance in each NOTICE (js tables written for the pack; WHATWG entities) | SR-RULE-PACKS decision 7 | ✅ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each fix is a stated mismatch with a known right answer.

## Goal

Every mismatch below is fixed in the record or in the repo, whichever is wrong.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · packs | SR-RULE-PACKS §1 and the starter catalogue match `packs/`. | ✅ 2026-10-20: `js/strict`, `py/strict` marked *later* (W-03); `md/relative-links` marked not shipped (W-21 row 2), 33 of 34 starters ship; "twelve packages" → twenty (27 files) |
| 2 · release docs | `RELEASING.md` and W-04 count packages, not files. | ✅ 2026-10-20: `RELEASING.md` says 20 packages (27 files); W-04 names the licence files as they were and the one `LICENSE` now |
| 3 · build-packs | `build-packs.py` uses `package.py`'s code, or SR-PACKAGES decision 9 says how they differ. | ✅ 2026-10-20, both, in W-18's commit (its row 8): `pack_manifest`, `wheel_for_pack`, `pep440` are shared; decision 9 names the one difference (the tarball is written without `npm pack`) |
| 4 · layers | SR-ARCHITECTURE and `tests/layers.rs` agree. | ✅ 2026-10-20: decision 1 says six layers, five holding crates (the ones `layers.rs` checks) and distribution, which holds none |
| 5 · NOTICE | Every `data/` table's source is in its pack's NOTICE. | ✅ 2026-10-20: js NOTICE names its two tables written for the pack; html NOTICE attributes the WHATWG table its entities follow (CC BY 4.0, which is not OSI-approved: whether decision 7 admits a table taken from an MIT source that follows it is the decider's question) |

## Blockers

None.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) · [SR-PACKAGES](../../records/0107_packages.md) decision 9 · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 1.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.

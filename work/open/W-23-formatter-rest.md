---
type: Handoff
name: W-23
description: "Claude Code, 2026-10-21: what W-07 left of SR-FORMATTER — the layout-rule refusal, .editorconfig, --import-config, ignore markers and LSP range formatting."
item: W-23
filed: 2026-10-21
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-21
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-23 — Finish SR-FORMATTER: layout rules, `.editorconfig`, imported configs, ignore markers, range formatting

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `codelaw test` refuses a rule whose fix only changes layout: `format(Don't) == format(Do)` | decision 3: the linter and the formatter never fight | ✅ 2026-10-22 |
| 2 | Read `.editorconfig` as the base layer under `[format]` | decision 5 | ✅ 2026-10-22 |
| 3 | `codelaw format --import-config`: `.prettierrc*`, `[tool.ruff.format]`, `[tool.black]` → `[format]` keys | decision 5's migration | 🟡 written, uncommitted and unbuilt in the W-23 worktree |
| 4 | Honour `codelaw-format-ignore` and `prettier-ignore` in every engine (Ruff's `# fmt:` markers already work) | decision 6 | ✅ 2026-10-22 |
| 5 | LSP `textDocument/rangeFormatting` | decision 8, SR-EDITOR decision 1 | 🟡 written, uncommitted and unbuilt in the W-23 worktree |

**Found by W-07, 2026-10-21:** W-07's definition of done covered the engines, the command, verification, the L7 test, the gate and the parity diff. SR-FORMATTER also decides these five, and none was built.

**Model:** Opus: step 4 means a pre-pass or an engine option per engine (Biome, pretty_yaml, taplo, dprint-plugin-markdown, markup_fmt), and a marker that one engine ignores silently reformats code the user fenced off.

## Goal

Every decision of SR-FORMATTER is built, and its "What is built" section lists nothing as missing.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · layout rules | A rule test with `format(Don't) == format(Do)` fails, with a message naming decision 3; the five packs still pass. | ⏳ |
| 2 · editorconfig | `indent_style`, `indent_size`, `end_of_line`, `max_line_length` from `.editorconfig` apply; `codelaw.toml` wins. Tested. | ⏳ |
| 3 · import | `--import-config` writes the `[format]` keys once; a round-trip test per source format. | ⏳ |
| 4 · markers | Per engine, a test where `codelaw-format-ignore` and `prettier-ignore` leave the next node byte-identical. | ⏳ |
| 5 · range | `documentRangeFormattingProvider` advertised; a server test; nothing written. | ⏳ |

## Blockers

None. A new crate for `.editorconfig` parsing must pass [L2](../../records/0004_LAW-2-foss-dependencies.md) and be named in SR-FORMATTER.

## Record affected

[SR-FORMATTER](../../records/0105_formatter.md) decisions 3, 5, 6, 8 and §What is built · [SR-EDITOR](../../records/0114_editor.md) decision 1 · [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md) (step 1 is a `codelaw test` check).

## Hazards

- **Step 1 can fail existing pack rules.** Run `scripts/build-packs.py`'s tests and every pack's `codelaw test` before landing it.
- **[L8](../../records/0010_LAW-8-performance-budgets.md):** reading `.editorconfig` happens per file; cache it per folder.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `archive/open/W-07-formatter.md`: the item that found these.

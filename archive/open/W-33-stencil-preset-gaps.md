---
type: Handoff
name: W-33
description: "Arpit, 2026-10-25: the two @stencil/eslint-plugin recommended rules codelaw does not have, found by the rule-coverage comparison."
item: W-33
filed: 2026-10-25
filed_by: "Arpit (Claude, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5): closed — merged to main as 3cb762e"
ball: done
---

# W-33 — Port the two Stencil rules `stencil/recommended` is missing

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read `ban-side-effects` and `enforce-slot-jsdoc` in @stencil/eslint-plugin 1.4.0 and decide whether each can be ported without type information | a port, or a recorded reason | ✅ 2026-10-25: both are syntactic — neither reads `parserServices` |
| 2 | Port each that can be (a `native:` rule with cases, a fix class under L4) and add it to `stencil/recommended` | the preset matches upstream | ✅ 2026-10-25: `stencil/ban-side-effects` (TS, TSX) and `stencil/enforce-slot-jsdoc` (TSX), both `manual`, in `stencil.rs` and the preset |
| 3 | Record any that cannot in the pack README's "Not ported, and why"; correct "15 of its 21 rules" to the real counts (1.4.0 turns on 23) | the README's claim is true | ✅ 2026-10-25: none left to record; the row reads 17 rules, "16 of the 23 rules it turns on, and `stencil/prop-readonly`" |

**Arpit, 2026-10-25 (docket):** *"A: file an item to port"*

**Model:** Sonnet: two rule ports in an existing pack, the same shape as the 15 already there.

## Goal

`stencil/recommended` covers every rule @stencil/eslint-plugin 1.4.0's `recommended` turns on, or names why not, and the pack README says so truthfully.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · ported | each of the two rules is in `packs/stencil/` with cases and `codelaw test packs/stencil` green, or is in the README's "Not ported" table with a reason | ✅ `codelaw test packs/stencil`: 18 rules, 164 checks, 0 failed |
| 2 · counts | the pack README's preset row states the real numbers; `scripts/docs-sync.py --check` passes | ✅ `docs-sync.py --write` moved the README to 440 rules; `--check` passes once CHANGELOG has its *Unreleased* section |
| 3 · corpus | `packs/stencil/corpus.toml` updated over `ionic-framework` and re-run clean | ✅ `corpus.py run --pack stencil --update`: 2,501 files, 18 rules, 1,117 checks, 0 failed; `ban-side-effects` 141, `enforce-slot-jsdoc` 50, every other count unchanged |

## What the source does, and the port

- **`ban-side-effects`** reports every call that is not inside an arrow function, a function declaration, a class or a named `export`, in any file whose name lacks `spec.`, `e2e.` or `test.`, except `describe`, `test`, `bind` and `createStore`. The port keeps that and makes every function a boundary (the source reports calls inside a function expression or object method); the skip list is the `allow` option.
- **`enforce-slot-jsdoc`** compares a `@Component` class's first JSDoc `@slot` tags (`@slot name - text`; `@slot - text` for the default) with the `<slot>` elements it renders, both ways. The port is TSX only, skips a `<slot name={expr}>`, and reads a tag's text to the end of its line.
- On `ionic-framework` the findings read as the source would report them: top-level `customElements.define`, `createController(...)` and server bootstraps for `ban-side-effects`; components that render a default `<slot>` without a `@slot -` tag for `enforce-slot-jsdoc`.

## Blockers

None.

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 7 (ports) · [SR-LAW-4](../../records/0006_LAW-4-every-rule-has-a-fix.md).

## Hazards

- `ban-side-effects` may need to know which calls have side effects; if that takes type information it goes in the "Not ported" table with the other four.

## Reference

- The rule-coverage comparison of 2026-10-25 (artifact "codelaw Rule Coverage"): every other rule codelaw leaves out has a recorded reason.
- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.

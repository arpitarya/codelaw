---
type: Standing Record
kind: component
name: SR-TYPES
title: "SR-TYPES (0117) — type-aware TypeScript rules: the project's own TypeScript 7.1, asked through a Node helper, a fixed vocabulary of type questions"
description: "How codelaw answers type-aware TypeScript rules: the codelaw-types Node helper inside the engine, the project's installed TypeScript (≥ 7.1) found and loaded once, a versioned protocol, the fixed type-question vocabulary the type: clause names, the warm model, the tested range, ts/type-error, the dedupe with adapters, and preview until TypeScript 7.1 is GA. Not built yet (W-105)."
status: accepted
date: 2026-10-27
feature: "type-aware TypeScript rules"
owns: []
laws: [L0, L2, L3, L4, L5, L6, L8, L11]
timestamp: 2026-10-27T20:00:00Z
content_sha: ""
ratifies: "Arpit, 2026-10-27 (W-101, by voice), rulings R1–R8 of work/proposals/type-aware-rules.md §3: TypeScript 7.1's own API — \"wouldn't it be a good idea to go with TypeScript one since it's already close?\" · \"It does.\"; reported once — \"I'm good on the second one.\"; no new LSP; Node at runtime — \"using Node at runtime makes sense. Let's go with it.\"; the ts pack — \"the rules go in TS pack, the existing one.\"; the project's copy — \"go with the copy that project already uses … based on different version, there could be different rules.\"; preview early — \"we can go ahead and do a release earlier as well, and … say it is a preview.\"; version checks — \"Go with the recommendation.\" W-104."
---

# SR-TYPES — type-aware TypeScript rules: the project's own TypeScript 7.1, asked through a Node helper, a fixed vocabulary of type questions

## §1 — For humans

**The decision in one line.** Some TypeScript checks need types — *is this a Promise nobody awaits?*, *is this value `any`?* tree-sitter has none. So codelaw ships a **small Node helper**, `codelaw-types`, that loads **the project's own TypeScript (≥ 7.1)** once and answers a **fixed set of type questions**. Rules in the existing `ts` pack ask them through a `type:` clause; TypeScript's own errors become `ts/type-error`.

**Why the project's own TypeScript.** The answer to a type question depends on the compiler that asks it. Using the copy the project already compiles with means codelaw and `tsc` agree, codelaw installs nothing, and the version is named in every report so a run can be reproduced ([L5](0007_LAW-5-no-model-in-the-engine.md)).

**What it costs.** Node at run time, for type rules only ([L6](0008_LAW-6-python-and-node-packages.md) decision 3). A project without Node or without TypeScript ≥ 7.1 loses only the type rules, with a note. Hooks never wait for a cold program ([L8](0010_LAW-8-performance-budgets.md) decision 8). The rules ship `preview` on the 7.1 release candidate and become stable only after 7.1 is GA.

```mermaid
flowchart LR
    R["ts/* rule with a type: clause"] --> E["engine (Rust)"]
    E -->|"type questions, versioned protocol"| H["codelaw-types (Node)"]
    H --> T["the project's TypeScript ≥ 7.1<br/>typescript/sync, program loaded once"]
    T --> H
    H -->|answers + TypeScript's own errors| E
    E --> O["one report: ts/* findings, ts/type-error,<br/>the TypeScript version named"]
```

<details>
<summary><b>ASCII twin</b></summary>

```text
   ts/* rule (type: clause) --> engine --questions--> codelaw-types (Node) --> project's TypeScript >= 7.1
                                  ^                                               (loaded once)
                                  +---------answers + ts/type-error ------------+
   engine --> one report (type findings, TypeScript version named)
```

</details>

---

## §2 — For agents

### Context

[L3](0005_LAW-3-languages.md) decision 9 takes type-aware TypeScript checks into the core. On 2026-10-27 TypeScript 7.0 (the Go compiler) had no API; 7.1 adds a preview API (`typescript/sync`, `typescript/async`, snapshots) that JavaScript calls — Beta 2026-10-20, RC 2026-11-10, GA 2026-11-24. No Go-library embedding exists, so a helper needs Node. oxlint's `tsgolint` covers type rules today through a private, unversioned protocol; codelaw's oxlint adapter carries it until this record is built (W-107). The research and the rulings are [the proposal](../work/proposals/type-aware-rules.md).

### Decision

1. **Scope.** Type-aware rules for TypeScript and TSX ([L3](0005_LAW-3-languages.md) decision 9), in the **`ts` pack** (`@codelaw/rules-ts`), plus `ts/type-error` (decision 9). JavaScript is not type-checked, and Python's type-aware checks stay with adapters.
2. **The helper, `codelaw-types`.** codelaw's own JavaScript, **carried inside the engine** — the same bytes in the npm and the PyPI package, never a package of its own ([L6](0008_LAW-6-python-and-node-packages.md) decision 3). codelaw starts it with Node and the project's `tsconfig.json`; it loads the program **once** through `typescript/sync` and answers questions until it is closed. It runs locally and fetches nothing ([L5](0007_LAW-5-no-model-in-the-engine.md) decision 3); every snapshot it takes is disposed, because the API leaks on the Go side otherwise. How its source reaches Node is W-105's to settle here.
3. **The protocol is codelaw's,** over the helper's stdin and stdout, with a **protocol version** in its first exchange. A helper that answers another version is refused, and the type rules are skipped with a note. The protocol is internal: nothing outside codelaw speaks it.
4. **The type-question vocabulary is fixed.** A rule asks only these, about one captured node ([SR-RULE-FORMAT](0112_rule-format.md) decision 3f):

   | question | holds when the node's type … |
   |---|---|
   | `promise-like` | is a `Promise` or a type with the same shape |
   | `thenable` | has a callable `then` |
   | `any` | is `any` |
   | `nullable` | includes `null` or `undefined` |
   | `union` | is a union — its members are what an exhaustive check must cover |
   | `deprecated` | names a declaration tagged `@deprecated` |
   | `never` | is `never` |

   The table's names are the contract; W-105 settles each question's exact options here when it builds them. A new question is an amendment of this record, never a rule's own code.
5. **Finding Node and TypeScript.** Node is found as an adapter's binary is ([SR-ARCHITECTURE](0108_architecture.md) decision 19's lookup: `PATH`). TypeScript is the `typescript` package resolved **from the file's folder up to the project root** (`node_modules/typescript`), the copy the project compiles with. **codelaw never installs, downloads or pins a TypeScript** ([L11](0013_LAW-11-dependencies.md) decision 2 reads it as no dependency, like an adapter's binary — put to the decider for confirmation at W-104).
6. **TypeScript ≥ 7.1, or the rules are skipped.** An older TypeScript, none, or no Node: every type rule is skipped, and the summary line and `--format json`'s notes say which and why ([SR-ARCHITECTURE](0108_architecture.md) decision 26) — never a silent gap, never a guess. Every other rule runs.
7. **A tested range, as adapters have.** codelaw names the TypeScript versions its fixtures pass on; outside that range the type rules run with a warning. Keeping the range current — a CI matrix with the latest 7.1 blocking and `typescript@next` nightly and reporting only, and a scheduled job that opens a PR widening it or an issue when fixtures break — is W-106's.
8. **The warm model ([L8](0010_LAW-8-performance-budgets.md)).** `check` and CI load the program once per run (`check_types_cold`). The daemon, the LSP and `--watch` keep it warm and update it on change (`snapshot.update`; `check_types_warm`). `codelaw hook` and `hook --pre` use a warm program or skip the type rules with a note (L8 decision 8). **No new LSP:** the helper is a backend behind codelaw's own LSP, the CLI, the daemon and the hooks; the editor's `tsserver` is never used — it is absent in CI, pre-commit and hooks.
9. **`ts/type-error`** reports TypeScript's own diagnostics for each checked file as findings. Where TypeScript offers a code fix it is the finding's `suggest` fix; otherwise the fix is `manual`, with its steps and a skill that carries them ([L4](0006_LAW-4-every-rule-has-a-fix.md)).
10. **Reported once.** A type rule that reimplements one carries `ported_from: [typescript-eslint/<rule>]`, so it dedupes against the oxlint adapter's finding for that rule as every ported rule does ([SR-LINTER](0106_linter.md) decision 2). No codelaw rule re-checks what `ts/type-error` already reports.
11. **Preview first.** Every type rule ships `stability: preview` (on only with `preview = true`; [SR-RULE-PACKS](0101_rule-packs.md) decision 9) from the first release on the TypeScript 7.1 RC. None is promoted before TypeScript 7.1 is GA, and then only by [SR-RULE-AUTHORING](0113_rule-authoring.md)'s promotion criterion.
12. **Reproducible.** The report and `--version --json` name the TypeScript version used and the helper's protocol version; the cache key holds both, with the `tsconfig.json` chain's content ([L5](0007_LAW-5-no-model-in-the-engine.md) decision 2).

### Consequences

- **Nothing is built:** the helper, the `type:` clause's loader, the warm programs, `ts/type-error` and the first rules (`no-floating-promises`, `no-misused-promises`, `await-thenable`, the `no-unsafe-*` family, `switch-exhaustiveness-check`) are W-105; the version watch is W-106. This record owns no component until W-105 adds one, and then the register's Ownership table gives it a row.
- **TypeScript 7.1's API is a preview** and can change between versions; decision 7's tested range and W-106 are the answer. 7.1 GA (2026-11-24) gates only the move from preview to stable.
- **codelaw's own tests and CI need a TypeScript 7.1 to run against** (W-105's fixtures, W-106's matrix). The project's TypeScript is no dependency of codelaw (decision 5), but a copy codelaw's own CI installs is one, in test and CI scope, and needs its own row in [L11](0013_LAW-11-dependencies.md) §3 before W-105 lands it — put to the decider with W-104.
- **Until then, oxlint's type-aware mode** fills the gap through the oxlint adapter (W-107).

### Alternatives considered

- **oxlint's `tsgolint` as the backend:** rejected (R1). Its protocol is private and unversioned, and it reaches TypeScript through `go:linkname` shims redone on every TypeScript release; it stays as the adapter that fills the gap.
- **A Go helper on typescript-go, one binary per platform, with a pinned TypeScript** (the 2026-10-25 proposal): rejected. A binary per platform, shims, and a TypeScript that can disagree with the project's (R6).
- **VS Code's `tsserver`, or a new LSP:** rejected (R3). It belongs to the editor and is absent in CI, pre-commit and hooks.
- **Biome's own inference, or `tsrs`:** neither is a checker codelaw can embed today. `tsrs` publishing a stable Rust crate API is the reopen-trigger for a Rust-native route with no Node.

### Reference (required)

- [`work/proposals/type-aware-rules.md`](../work/proposals/type-aware-rules.md): the landscape as read 2026-10-27, the rulings R1–R8, and its sources (TypeScript 7.1 Beta announcement, oxc's type-aware guide, the typescript-go embedding discussion).
- [SR-ARCHITECTURE](0108_architecture.md) decision 19: the adapter lookup and tested-range pattern decisions 5 and 7 follow.

### Veto condition

**Reopen this decision if:** codelaw installs, downloads or pins a TypeScript for a project; a type rule runs without naming the TypeScript version in the report; a hook path loads a cold type program; or TypeScript 7.1's API is withdrawn, or `tsrs` publishes a stable crate API.

**How to check it:** `grep -rln '^type:' packs/ts/rules` lists the type rules (none until W-105), each `stability: preview` until 7.1 GA; and `grep -rnE "npm (i|install).*typescript" crates packaging` finds nothing.

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-LAW-3](0005_LAW-3-languages.md) · [SR-LAW-5](0007_LAW-5-no-model-in-the-engine.md) · [SR-LAW-6](0008_LAW-6-python-and-node-packages.md) · [SR-LAW-8](0010_LAW-8-performance-budgets.md) · [SR-LAW-11](0013_LAW-11-dependencies.md) · [SR-ARCHITECTURE](0108_architecture.md) · [SR-RULE-FORMAT](0112_rule-format.md) · [SR-LINTER](0106_linter.md) · [SR-RULE-PACKS](0101_rule-packs.md) · [SR-RULE-AUTHORING](0113_rule-authoring.md)

**Project docs** — [`work/proposals/type-aware-rules.md`](../work/proposals/type-aware-rules.md)

---
type: Handoff
description: "Cold-start state of play for a successor session: in-flight work, constraints, lessons."
---

# Model handoff — the agent-succession record

*The departing model answers for the record, so whatever succeeds it inherits
the judgment and not just the files.*

**Standing instruction to every future model reading this: you are now the
maintainer of this document. When the project's direction, decisions or state
change, update it in the same change. Leave it better.**

## How to use this file

A state-of-play doc, not an end-of-session report. Keep it current **during**
the session. Write it for a different model arriving with zero other context,
mid-task. The four sections below are maintained; anything after them is
history. **A stale INTERVIEW at handoff is as serious as a missing worklog
entry.**

---

## 1 · State of play

*Updated 2026-10-19.* The laws L0–L8 are accepted (L8's budgets unset; L6 host
floors Python ≥ 3.11, Node ≥ 22); every record 0101–0115 is accepted. W-01
rewrote the records for five rule packs; **W-02 built the engine**: a Cargo
workspace of 13 crates under `crates/` and the `codelaw` binary (`check`,
`fix`, `test`, `rules`, `explain`, `tree`, `new-rule`, `init`, `cache`), with
the semantic model, `native:` rules and pack resolution. Every crate is owned in
the register's Ownership table. W-03 has begun: `packs/js` holds `js/recommended`, all 64 of ESLint 10.0.1's
recommended rules, every one `preview`. Not built:
the formatter (so L7 is unmet), graph, baseline, adapters, MCP, LSP, agent
surfaces, packaging — backlog B-028–B-039. `cargo test --workspace` is green
and clippy is clean. **Ground it before you edit it** — `git log`, the queue,
`cargo test`.

## 2 · In flight, and the immediate next step

**W-03** (the five packs, 🟢) is in flight: `js/recommended` is done; next
are `ts/recommended`, `react`, `node`, `stencil`, then the python, css, html and
data packs. Write `packs/<name>/` per SR-RULE-PACKS decision 3, run `codelaw
test packs/<name>`, then run the pack over real code (npm's own source works
for JS) before trusting it. Ports that need `target:` rules or the graph wait
in each pack's "not ported yet" list (B-029, B-030), never approximated. Step 6
(publish) needs packaging (B-038, with B-012) and the decider's ruling. The
standing instruction to promote B-008 and B-012 when the first manifest lands
is half done: L2's Cargo check landed in W-02 and B-008 now covers npm/PyPI;
B-012 waits for B-038. The decider (`git config user.name`) still installs
`scripts/sr-guard.sh` as the `commit-msg` hook.

## 3 · Standing constraints

- The laws: L0 (records are the only source; links, never restatements), L1
  (retired records are archived), L2 (every dependency is free, open source and
  permissive — no paid, no copyleft, dev tooling included), L3 (nine
  languages, alone or combined), L4 (every rule has a fix), L5 (no model in the
  engine), L6 (`codelaw` on PyPI + npm, identical; rules as five packs on both;
  npm sub-packages only `@codelaw/*`), L7 (linter and formatter, always both),
  L8 (performance budgets: never exceeded, never loosened). See
  `records/0001_LAWS.md`.
- Adapters must pass L2 too: yamllint (GPL-3.0) is out.
- The queue is the list; a blocker is a file; nothing closed is deleted.
- A change under `crates/` stages its owning record (the guard enforces it once
  installed); `crates/codelaw-cli/tests/` holds the layer, licence and law
  checks.
- Rule files are YAML 1.2 (SR-RULE-FORMAT decision 3d); `where:` and the fix
  primitives are SR-RULE-FORMAT decisions 3a–3c.

## 4 · Lessons learned

- **Run a new rule over real code** — its own examples passed while eight
  false-positive patterns hid; a corpus run found them (`work/LESSONS.md`).
- **Write a benchmark's output outside the folder it checks** — otherwise the
  second run checks the first run's output (`work/LESSONS.md`, 2026-10-19).
- **YAML 1.1 booleans** turned `message: n` into `false`; rule YAML is read as
  1.2 now (`work/LESSONS.md`, 2026-10-19).

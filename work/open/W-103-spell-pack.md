---
type: Handoff
name: W-103
description: "Arpit, 2026-10-27 (Cowork, voice): build spelling into codelaw as its own rule pack — a known-typo check on by default, a dictionary check opt-in, codelaw's own word data from SCOWL, VarCon and the MIT software terms. Design of record: SR-SPELL."
item: W-103
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Claude Code (Opus 5.5), the third increment"
ball: agent
---

# W-103 — the spell pack: codelaw's own spell checker, better than CSpell

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Vendor SCOWL, VarCon and the software-terms list at a fixed release; record each pin in L11 §3 (D-42–D-44) and the notices in `packs/spell/NOTICE` | licence-clean, auditable sources | ✅ licences read from each source (SCOWL's `Copyright`, VarCon's README, the MIT `LICENSE`): all notice-only or MIT; SCOWL 2020.12.07 and software terms 5.4.8 pinned by SHA-256 in `spell-data.py` and L11 §3, notices in `NOTICE` and `licenses/`; VarCon 2020.12.07 pinned in `spell-data.py` and L11, its notice in `NOTICE` and `licenses/VarCon-Copyright.txt` (2026-10-28) |
| 2 | `scripts/spell-data.py` writes the dictionary, the locale table and the generated known-typo list into `packs/spell/data/spell/`; same input, same bytes | codelaw's own word data | ✅ `typos.json`, `words.json` and `locale.json`; `spell-data.py check` proves the same bytes (2026-10-28) |
| 3 | Measure: entries in the generated typo list, and the bytes the data adds to the npm tarball and the PyPI wheel; if JSON is too large, put a compact format to the decider (SR-RULE-PACKS decision 3) | the size question answered with numbers | 🟡 typo list: 188,151 entries, 3.53 MB JSON, pack 625 KB tgz / 636 KB wheel, +9 ms to load (SR-SPELL decision 9); one JSON string searched in place, so no new format; with the dictionary (868 KB) and locale table (489 KB) the pack is 1.03 MB tgz / 1.04 MB wheel and loads in +1 ms more, so no split and no new format (SR-SPELL decision 9, 2026-10-28) |
| 4 | The engine side: per-language selection queries, the word splitter, the token skips, the six `native:` rules and their fixes, the `[spell]` config section | the check itself | 🟡 selection, splitter, skips, `[spell]` and six rules built (2026-10-28); `spell/inconsistent-term` left (project scope) |
| 5 | The pack: `packs/spell/` with `spell/recommended` and `spell/strict`, rule files with cases, README; L6, SR-RULE-PACKS and SR-PACKAGES amended to eighteen packs | installable as `@codelaw/rules-spell` / `codelaw-rules-spell` | 🟡 the pack, both presets, six rules with cases; L6, SR-RULE-PACKS, SR-PACKAGES, SR-RULE-FORMAT and every count at eighteen; `spell/inconsistent-term`'s file left |
| 6 | Measure the speed: three CI `perf` runs on `main` with the pack on; put the budget to the decider by L8's method | the L8 budget | ⏳ |

**Arpit, 2026-10-27 (voice):** *"There is a tool called C-Spell. Research it and tell me if that can be built into CodeLaw. A similar feature, a better feature, would be even better."* Then, ruling the forks: *"Let's go with B."* · *"Approve both."* · *"Approve it."* · *"want it as a separate package."* · *"Go with the recommendation."* · *"go ahead"*. The forks: [the compare doc](../compare/spell-checking.compare.md).

**Model:** Opus: it amends a law (L6's pack count) and the generated typo list's quality is judged by people, not caught by a test; close to Sonnet, because SR-SPELL fixes the rules and their fix classes.

## Goal

Spelling in comments, docs, prose strings, Markdown, keys and declared names, in all nine L3 languages — with unattended fixes for known typos and an opt-in dictionary check — built only from codelaw's own code and the three ratified sources ([SR-SPELL](../../records/0116_spell.md)).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · data | `packs/spell/data/spell/` is written by `scripts/spell-data.py` (stdlib only) from D-42–D-44 at the pins in L11 §3; a second run changes no byte; `NOTICE` names each source and version | ✅ all three tables; `spell-data.py check` passes; `NOTICE` names SCOWL, VarCon and the software terms (2026-10-28) |
| 2 · rules | the six rules of SR-SPELL decision 5, each with its fix class, cases and a `codelaw test` pass; every `manual` fix carried by a skill (L4) | 🟡 `spell/known-typo` (107 `codelaw test` checks), `spell/known-typo-identifier` (48) and `spell/known-typo-key` (28), `spell/unknown-word` (90), `spell/locale-variant` (94), `spell/suspicious-word` (6): 373 checks; `spell/inconsistent-term` left |
| 3 · scope | a fixture repo per L3 language: a typo in a comment, a string, a declared name and a key is reported once; the same word in a URL, a hash, a JSON value that is an id, or at a name's uses is not | ⏳ |
| 4 · fixes | `codelaw fix` on the fixtures fixes the one-correction known typos in prose and file-local names, and changes nothing exported | ⏳ |
| 5 · packaging | `@codelaw/rules-spell` and `codelaw-rules-spell` build in CI byte-identical; `init` does not add the pack; a missing pack names both install commands | 🟡 `build-packs.py --verify` builds both and compares output byte-identical locally; `init` adds no spell pack; the missing-pack message names both installs (`pack.rs` test); CI proves it on the landing commit |
| 6 · measured | the typo list's entry count, the data's bytes per package, and the spell pass's time on the pinned corpus are written into SR-SPELL decision 9; the budget is the decider's ruling | 🟡 entry count and bytes per package measured (SR-SPELL decision 9); the corpus timing and the budget left |
| 7 · records | SR-SPELL `built: yes`; L6, SR-RULE-PACKS §1 and decision 4, SR-PACKAGES at eighteen packs, sixteen on npm and eleven on PyPI; README, CHANGELOG, GLOSSARY, `docs/rules/` regenerated | 🟡 all at eighteen; SR-SPELL built: partly |

## Left (exact, after the third increment of 2026-10-28)

1. ✅ 2026-10-28: `spell/known-typo-identifier` (`auto`, JS/JSX/TS/TSX/Python, 48 `codelaw test` checks) and `spell/known-typo-key` (`manual`, JSON/YAML/TOML/CSS/SCSS/HTML, 28 checks), both in `spell/recommended`; one rule per fix class, as SR-SPELL decision 5 *as built* says. Not built: the `suggest` tier naming other files (needs the project graph).
2. ✅ 2026-10-28: the dictionary — SCOWL to size 60 plus the software terms, 92,758 words, 868 KB, +1 ms to load; no split or new format needed (SR-SPELL decisions 6 and 9 *as built*).
3. ✅ 2026-10-28: `spell/unknown-word` (`suggest`, in `spell/strict`) and `[spell]` (`locale`, `words`; SR-CONFIG §1 and decision 2, the schema, `config` trace); `[spell] words` excuses every rule but `suspicious-word`.
4. 🟡 2026-10-28: `spell/suspicious-word` (`manual`, `spell/strict`) and `spell/locale-variant` (`auto`, VarCon's pairs in `locale.json`, NOTICE and `licenses/` gain VarCon) built. **Left:** `spell/inconsistent-term` — it needs every file's words at once, so it is a `scope: project` rule on the graph stage (`crates/codelaw-engine/src/project.rs`, `codelaw-graph`), not a `native:` file rule.
5. DoD 3 and 4: a fixture repo per L3 language and the `codelaw fix` run over it.
6. The pinned corpus: add `spell` to `scripts/corpus.json`/`corpus.py`, record counts, review the findings for false positives before any rule leaves `preview`; the reopen-trigger's comparison with `typos`' count.
7. Step 6: three CI `perf` runs on `main` with the pack on, then the budget to the decider (L8).

## Blockers

None for steps 1–5. Step 3 may come back to the decider (a non-JSON data format amends SR-RULE-PACKS decision 3); step 6's budget is his ruling.

## Record affected

[SR-SPELL](../../records/0116_spell.md) (all) · [L6](../../records/0008_LAW-6-python-and-node-packages.md) (the pack count, on the ruling above) · [SR-RULE-PACKS](../../records/0101_rule-packs.md) §1, decisions 4 and 6 · [SR-PACKAGES](../../records/0107_packages.md) · [SR-CONFIG](../../records/0110_config.md) decision 2 (`[spell]`) · [L11 §3](../../records/0013_LAW-11-dependencies.md#3--the-list) (the pins).

## Hazards

- **Never copy `typos`', codespell's or Wikipedia's lists**, not even to compare in a test fixture — SR-SPELL's veto condition greps for them.
- A generated typo that is a real word in some domain (a surname, a chemical) makes an `auto` fix wrong: keep only variants with one source word that SCOWL's largest list does not hold, and let the corpus run find the rest before any rule is `stable`.
- Renaming an identifier across files is a refactor: `auto` only for a file-local, unexported name (SR-SPELL decision 5).
- Data size: SCOWL with every form written out may be several MB as JSON — measure (step 3) before choosing a format.
- `HashMap` order, Unicode case folding and locale sorting can make output differ by platform (L6: identical output) — sort everything, NFC + simple fold.

## Reference

- [SR-SPELL](../../records/0116_spell.md) · [the compare doc](../compare/spell-checking.compare.md) · [L11 §3](../../records/0013_LAW-11-dependencies.md#3--the-list)
- SCOWL and VarCon: <http://wordlist.aspell.net/> · software terms: <https://www.npmjs.com/package/@cspell/dict-software-terms>

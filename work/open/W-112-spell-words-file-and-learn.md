---
type: Handoff
name: W-112
description: "Arpit, 2026-10-28 (voice): the spell pack's own words file (.codelaw/spell/words.txt), codelaw init adding the pack and the file by default (--no-spell skips), and codelaw spell learn to fill it from the codebase."
item: W-112
filed: 2026-10-28
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-112 — The spell words file, `init` by default, and `codelaw spell learn`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Records first: SR-SPELL decisions 1 and 7, SR-CLI-MCP (`init --no-spell`, `spell learn`), SR-CONFIG (`[spell] words_file`), the template's row (L10) | the spec before the code | ⏳ |
| 2 | **A words file:** `.codelaw/spell/words.txt`, one word per line, `#` comments; `[spell] words_file` points at it, and by default it is that path | custom words kept in their own file, next to the rules | ⏳ |
| 3 | **Both lists count:** the inline `[spell] words` stays and is merged with the file | small projects keep one place; big ones a file | ⏳ |
| 4 | **A missing file:** silent at the default path; an error when `words_file` is set and the file is not there | no noise by default, no silent typo in a path | ⏳ |
| 5 | **`codelaw init`** adds the spell pack and writes `.codelaw/spell/words.txt` from a template, then runs `spell learn`; `--no-spell` skips all three | spelling on from day one | ⏳ |
| 6 | **`codelaw spell learn`** scans the codebase and adds every word no dictionary knows to the words file; likely typos are listed for review, never added | a clean first run without whitelisting real typos | ⏳ |

**Arpit, 2026-10-28 (voice):**
- On custom words: *"I would recommend creating a separate file and giving a path in TOML file for that file. The default should be next to rules directory."* — and, by voice, *"The default should be next to root rules directory. In the consumer application."*
- The inline list stays: *"It should stay."* A missing file is silent at the default path and an error at a path set by hand: *"Sounds right. Sounds right."* — his ask that this be its own item: *"Probably create a new work item."*
- Where the file lives: *"Probably add a directory inside dot code loss slash words or slash spell and then inside it keep the text file. And this is something that won't exist by default. Probably create a CLI command to set it up"* — the voice transcript's "code loss" is `.codelaw`; Claude proposed `.codelaw/spell/words.txt`.
- How it is set up: *"Let it be part of code law init itself. No flag, no nothing. Probably we can have a flag as hyphen hyphen no spell if it is not needed."* Asked whether `init` should also add the spell pack, with `--no-spell` skipping both: *"Yes."*
- Learning words: *"I want a CLI command which can identify all the spellings within the code base and put it in the words.txt file as default."* — told it would whitelist real typos too, so only unknown words are added and likely typos are listed for review: *"Code law spell learn."* and *"And have it on init as well."* Then: *"This is all."*

**Model:** Opus — records first; it reverses SR-SPELL decision 1's "`codelaw init` never adds it".

## Definition of done

- In a fixture repo, `codelaw init` adds the spell pack to the config, writes `.codelaw/spell/words.txt` from `templates/` (L10) and fills it by `spell learn`; `codelaw init --no-spell` does none of the three. Both byte-identical across runs.
- `spell learn` adds a word no dictionary knows, and refuses one that is on the known-typo list or one edit from a common word (the `spell/suspicious-word` test) — those are printed, sorted, for review. A second run adds only new words; existing lines and comments are kept.
- `[spell] words` and the file are merged; an entry in either excuses the word as `[spell] words` does today (SR-SPELL decision 7). The default path missing → no message; a set `words_file` missing → a config error naming the path.
- `config --print` shows `words_file` and its source; the schema and USAGE document it.

## Decided by the agent's proposal, accepted by voice (not his words — check before building)

- The path `.codelaw/spell/words.txt`, the key name `words_file`, the command name `codelaw spell learn` (his pick from Claude's suggestion), and the review list for likely typos.
- Words compare case-insensitively, as `[spell] words` does; `spell learn` writes them lower-case and sorted, so its output is deterministic (L5).

## Record affected

[SR-SPELL](../../records/0116_spell.md) decisions 1 (init adds the pack — reversed) and 7 (`words_file`) · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) (`init --no-spell`, `spell learn`) · [SR-CONFIG](../../records/0110_config.md) (`[spell] words_file`) · `templates/README.md` (the words file's template, L10).

## Hazards

- `spell learn` on a large repo can add thousands of words; it prints how many it added and the review list first, and never removes a line a person wrote.
- `init` on a repo that already has the file must not overwrite it — it runs `learn`, which only appends.

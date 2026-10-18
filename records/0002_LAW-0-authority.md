---
type: Standing Record
kind: law
name: SR-LAW-0
title: "SR-LAW-0 (0002) — L0, the records are the only source of truth"
description: "Every rule is stated in exactly one record; every other artifact links to it; Law records outrank every other record; a Law changes only on the decider's ruling."
status: accepted
date: 2026-10-18
feature: the authority of the records
owns: []
laws: [L0]
timestamp: 2026-10-18T12:52:51Z
content_sha: ""
---

# SR-LAW-0 — L0, the records are the only source of truth

## §1 — For humans

**Every rule has exactly one home: a record under `records/`.** `CLAUDE.md`, a
README, a comment, a skill or a diagram links to that record and never restates
it. A change to a rule is made in the record first.

**The restatement test is one sentence:** *could this artifact and the record
disagree while both still look correct?* If yes, it is a restatement and
forbidden. Code naming a key, or a hook enforcing a rule, would simply fail —
those are permitted.

**Law records outrank every other record.** A sentence in a lower document that
conflicts with a law is void, and is deleted the moment it is noticed — never
softened or footnoted.

```mermaid
flowchart LR
    R["a rule"] --> S["its one record in records/"]
    S --> A["CLAUDE.md · READMEs · comments · skills<br/>link to it, never restate it"]
```

<details>
<summary><b>ASCII twin</b> — the same diagram, for terminals and diffs</summary>

```text
   a rule ---> its one record in records/ ---> CLAUDE.md, READMEs, comments, skills
                                                (link to it, never restate it)
```

</details>

---

## §2 — For agents

### Context

Rules copied into `CLAUDE.md` and READMEs drift from the records they copied:
one copy is edited, the other is not, and both still read as correct. A session
that reads the stale copy acts on a rule nobody holds any more.

### Decision

1. **Source.** Every rule has exactly one home, a record. Every other artifact
   — `CLAUDE.md`, a README, a comment, a skill, a diagram — **links to it and
   never restates it**. A change is made in the record first.
1a. **The records are always up to date — in the change that makes them
   wrong.** Three obligations: (i) no behaviour change lands without its record
   updated in the **same commit**; (ii) a change that touches no recorded
   decision says so — `no SR affected`, on its own line of the commit message;
   (iii) before a session ends, re-read every record you changed code under.
   (i) and (ii) are enforced by the guard; (iii) is judgment and is the only
   thing covering coherence — a record can be amended into self-contradiction
   and every check passes.
2. **Precedence.** Law records outrank every other record. A record that
   conflicts with a Law is **void in the conflicting part** — a defect, never a
   trade-off.
2a. **On contact, the conflicting sentence is deleted from the lower document**
   — never softened, footnoted, or left beside a pointer.
3. **Amendment is entrenched.** A Law changes only on the decider's ruling, named in
   the record. An ordinary record a session may accept.
4. **The restatement test:** *could this artifact and the record disagree
   while both still look correct?* If yes it is a restatement and forbidden.
   If it would simply fail, it is an implementation (code naming a key) or an
   enforcement (a test, a hook) and permitted. Prose explaining *how* or *why*
   beside the code is permitted; a second statement of the rule is not.
5. **A generated view is permitted only while a test binds it byte-equal.**
   Remove the test and the view is an illegal restatement. (Not used by this
   bootstrap — tests are deferred — so `CLAUDE.md` links to the laws and
   carries none of their text.)

### Consequences

- Obligations (i) and (ii) of decision 1a are enforced by `scripts/sr-guard.sh`
  once the decider installs it as the `commit-msg` hook.
- Obligation (iii) is judgment and the only cover for coherence — backlog row
  B-006, a `cost` that never graduates.
- `CLAUDE.md` carries pointers only: a generated law block is permitted only
  once a test binds it (decision 5), and no tests exist yet.
- Precedence (decisions 2–2a) is judgment; no parser reads *does this
  contradict L0*.

### Alternatives considered

- Rules in `CLAUDE.md`, records as commentary — rejected: that is the drift
  this law ends.
- Rules wherever convenient, plus a cross-reference index — rejected: an index
  cannot tell which copy is current.

### Reference (required)

- [`scripts/sr-guard.sh`](../scripts/sr-guard.sh) — the executable form of
  decision 1a (i)–(ii).
- [SR-WORK-OWNERSHIP](0058_WORK-ownership.md) decisions 3 and 6 — the gate it
  implements.

### Veto condition

**Reopen this decision if:** a rule's normative text is found in two live files.

**How to check it:** grep the rule's distinctive phrase outside the archive — more than one live
hit is a restatement. Example: `grep -rln "removed, never ticked" --include=*.md . | grep -v '^./archive/'`
must print only `records/0051_WORK-open-queue.md`.

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-LAWS](0001_LAWS.md) · [SR-LAW-1](0003_LAW-1-retired-records-archived.md) · [SR-WORK-OWNERSHIP](0058_WORK-ownership.md)

**Code**

- [`scripts/sr-guard.sh`](../scripts/sr-guard.sh)

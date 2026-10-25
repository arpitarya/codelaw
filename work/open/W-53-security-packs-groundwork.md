---
type: Handoff
name: W-53
description: "Arpit, 2026-10-25: the shared groundwork for the four security packs (JS, TS, HTML, Python): L6's pack list, CWE/OWASP metadata on rules, SARIF security-severity, presets, and a same-function source-to-sink predicate."
item: W-53
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-53 — Security packs — the groundwork: four packs, CWE/OWASP metadata, a request-source predicate

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L6: eleven packs become fifteen — `security-js`, `security-ts`, `security-html`, `security-python`; npm gets js/ts/html, PyPI gets js/html/python (the existing split); `FIRST_PARTY_PACKS` in the same change | the packs can be published | ⏳ |
| 2 | SR-RULE-FORMAT: rule frontmatter `security: { cwe: [79], owasp: ["A05:2025"], severity: high|medium|low }` | every finding maps to Veracode / Black Duck / Xray / OWASP language | ⏳ |
| 3 | SR-CLI-MCP: SARIF `properties.tags` (`security`, `external/cwe/cwe-79`) and `security-severity`; `codelaw rules --tag cwe:79|owasp:A05`; `codelaw rules --security-coverage` prints the OWASP × rule matrix | GitHub code scanning ranks them; coverage is provable | ⏳ |
| 4 | Presets per pack: `recommended` (high-confidence, `error`) and `strict` (heuristic, `warn`) | teams start without noise | ⏳ |
| 5 | Rule-format predicate `from_source:` — a sink argument that is, or is bound in the same function to, a known request source (`req.query`, `request.args`, `location.hash` …), using the semantic model; sources listed per framework in each pack's `data/` | fewer false positives than 'any non-literal argument' | ⏳ |
| 6 | Pack scaffolds with README, `corpus.toml`, licence/NOTICE | W-54–W-57 start from a working pack | ⏳ |

**Arpit, 2026-10-25:** *"I want to create a pack for security issues. Security issues like Veracode, Black Duck, X-Ray, OWASP. They would have some suggestions on how to write, what to write, what not to write in JavaScript and Python. So create … one for JavaScript, one for TypeScript, one for HTML, one for Python. Research and create work items for individual languages."*

**Model:** Opus — it amends L6 and the rule format; the source predicate's semantics decide every security rule's precision.

## Goal

- Rule namespaces: `sec-js/`, `sec-ts/`, `sec-html/`, `sec-py/`.
- A finding's text names the CWE: `sec-js/no-child-process-shell (CWE-78) …`.
- `security-ts` **extends** `security-js` (JS rules already run on TS files); it holds only TS-specific rules.
- `security-html`'s preset also enables the HTML pack's existing security rules (`html/require-sri`, `html/target-blank-rel`, `html/no-inline-script`, `html/autocomplete-password`, `html/meta-refresh`) rather than duplicating them.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `codelaw rules --tag cwe:79` lists rules across packs | ⏳ |
| 2 · | SARIF from a fixture shows `security-severity` and CWE tags, validated | ⏳ |
| 3 · | `from_source:` matches a source one assignment away in the same function, and not across functions — with cases | ⏳ |
| 4 · | the four packs build, publish in a dry run to both registries' names, and `law-gate.py` passes | ⏳ |

## Blockers

None.

## Record affected

L6 (the pack list — the decider's ruling of 2026-10-25) · SR-RULE-PACKS (four packs, presets) · SR-RULE-FORMAT (`security:`, `from_source:`) · SR-CLI-MCP (SARIF, `--tag`).

## Hazards

- Fifteen packs on two registries: four more trusted publishers and first uploads by hand (W-38's pattern).
- `from_source:` is not taint analysis — say so on every rule page; cross-function flow waits on B-049-style analysis.

## Reference

[OWASP Top 10:2025](https://owasp.org/Top10/2025/0x00_2025-Introduction/) · [CWE Top 25 2025](https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html) · [GitHub SARIF `security-severity`](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning)

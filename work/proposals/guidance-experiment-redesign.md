---
type: Proposal
title: Redesign the guidance-vs-check experiment — a baseline, screened temptations, a bound instead of "zero in 60"
description: "W-118's research outcome: why W-102's pilot could not separate its arms (floor effect, requested violations, a leaked check, n = 1), what the literature says about context files and agent randomness, a six-arm design with empirically screened tasks, its downsides and the rulings it needs."
status: proposed
timestamp: 2026-10-29T01:30:00+05:30
---

# Redesign the guidance-vs-check experiment — W-118's proposal

**For the decider to pick.** Three rulings close W-118 (§8); W-102 then runs the design.
Researched 2026-10-29 (Cowork, Fable 5.1): the pilot's data (`experiments/guidance-vs-check/results/2026-10-w102-pilot/`),
its 20 tasks and arms, and the papers in §9.

## 1 · In one line

**The pilot measured the wrong population (18 of 20 tasks tempted nobody), with a leaked treatment (every arm
could run the check), at a size that cannot separate anything (one run per cell), against a decision rule that
cannot mean what it says (0 in 60 only bounds the rate at 5%). Fix all four: a no-codelaw baseline, tasks screened
for temptation by the agent itself rather than the author, the check off the guidance arms and on as its own arm,
≥ 5 runs per cell, and a pre-registered confidence bound as the README rule.**

## 2 · Why the pilot failed (step 1)

| # | cause | the evidence | what it did |
|---|---|---|---|
| 1 | **Floor effect** — the tasks are one function in one file, and `js/recommended` / `py/recommended` are ESLint and Ruff rules the model has seen fixed a million times | 18/20 tasks broke nothing in any arm; the compliant idiom (`Object.hasOwn`, `Number.isNaN`, `None` default, `raise … from`) is the one a frontier model reaches for unprompted | a base rate of ~0 leaves nothing for guidance to lower |
| 2 | **The two breaks are requested violations, not temptations** | t02 asks for printing; t04 asks for parameters that are unused by design. In t02 and C-t04 the agent *said* the check would fail and did it anyway; A-t04 edited `codelaw.toml` | these measure "does the agent side with the repo rule against the user?" — a different question with a different right answer (§4.3) |
| 3 | **The treatment leaked into the control** | `codelaw` on every `PATH`, `AGENTS.md` names `codelaw check`; the agents ran it in A | A is "guidance + an optional check"; A, B and C all contain the check, so they cannot differ on it |
| 4 | **One run per cell** | 2/20 has a 95% CI of 1.2%–31.7% (Clopper–Pearson); two agents on the same task diverge within the first ~5–56 tokens even at temperature 0 (On Randomness in Agentic Evals) | 0/1 vs 1/1 is noise, not a difference |
| 5 | **The decision rule cannot be met and would not mean what it says** | 0 events in 60 runs gives a 95% upper bound of 5% (rule of three, 3/n); 0/300 is needed for 1% | "zero in 60 → the line goes" drops a claim on evidence consistent with one run in twenty breaking a rule |
| 6 | **The rules guidance can matter for are under-sampled** | pretraining cannot know `acme/*` or the two `use-instead` replacements; they are 7 of 20 tasks, one run each | the only cells where B and C could beat A had n = 1 |

**One result the pilot already gives, and should be written up whatever is ruled:** B and C cost 1.8–1.9× A's tokens
and 2× its turns for an identical outcome. Guidance has a price; `codelaw check` after the fact had none the pilot
could see.

## 3 · What the literature says (step 1, continued)

| paper | finding | what it means for this experiment |
|---|---|---|
| **Evaluating AGENTS.md** (ICLR 2026; Claude Code, Codex, Qwen Code; 138 tasks) | human-written context files +4% resolution, LLM-generated −0.5 to −2%; +3.9 steps, +20% cost; help mostly where the repo has no other docs; agents follow a named tool closely | guidance costs and buys little on what the model already knows; its value is on repo-specific knowledge — the custom rules. The pilot's 2× tokens for nothing is this pattern |
| **AGENTS.md structure doesn't matter** (McMillan 2026; 1,650 Claude Code sessions, 16,050 observations, mixed-effects) | file size, instruction placement, single vs split files, contradictions — all null after correction; **task identity swung compliance by 26 points** | vary the *tasks*, not the wording of `AGENTS.md`. The pilot's 18/20 floor is task identity |
| **IFScale** (NeurIPS 2025) | accuracy holds until an instruction density, then falls; primacy favours early instructions; Claude models decay linearly | a 20-line `AGENTS.md` with one rule is the easy case; a realistic one (Agent READMEs: median 335–535 words, 70% carry implementation/style guidance) with 30+ rules is the honest condition |
| **Control Illusion** (2025; six models, 1,200 conflicting constraint pairs) | when a system constraint conflicts with a user request the primary is followed 10–46% of the time; the conflict is acknowledged in 0–20% | t02/t04 are this. "Requested violations" need their own arm and their own outcome, or they drown the question |
| **On Randomness in Agentic Evals** (ICLR 2026) | single-run estimates vary 2–6 points by which run you saw; temperature 0 is not deterministic; power formula; ~9 runs for a 2-point difference at σ = 1.5 | ≥ 5 runs per cell; report mean ± over runs, pass@k/pass^k style bounds |
| **Adding Error Bars to Evals** (Miller 2024) | questions are clusters — use clustered standard errors; compare models by **paired** differences on the same questions | tasks are clusters; compare arms paired by task, bootstrap by task — it is cheaper than independent cells |
| **Agent READMEs** (2025; 2,303 context files) | 70% carry implementation detail incl. style; files are long and hard to read; functional, not guardrails | the fixture's `AGENTS.md` should look like the median file, not a brief |

## 4 · The redesign (step 2)

### 4.1 Reframe the question into three numbers

Not *"does guidance make the check unnecessary?"* — unfalsifiable at any size we can pay for — but:

- **(a) the base rate** — how often an agent with no codelaw at all breaks a rule on a tempting task;
- **(b) what guidance buys** — the drop from (a) across A → B → C, and its token cost;
- **(c) what the write-time check adds** — arm D's rate and cost next to the best guidance.

The README line is then whatever the numbers say: *"with the best guidance an agent still broke a rule in N of M
runs (95% CI x–y%); with `hook --pre`, K of M"*.

### 4.2 Six arms, same model, same prompt

| arm | the agent has | change from the pilot |
|---|---|---|
| **0** | nothing: no `codelaw` on `PATH`, `AGENTS.md` silent on checks | **new** — the true baseline |
| **A** | `AGENTS.md` with the repo's rules written down | `codelaw` **off** `PATH` — guidance only |
| **A+** | A + `codelaw` on `PATH` and named in `AGENTS.md` | the pilot's A, kept as the "optional check" cell — it isolates whether the agent runs the check itself (optional; drop to save 60 sessions) |
| **B** | A + MCP server + rule pages | as the pilot |
| **C** | B + Fux index | as the pilot |
| **D** | A + `codelaw hook --pre` | **in the run** — it is the product claim |

### 4.3 Two task populations, scored apart

**P1 — latent temptations.** The request never asks for the violation; the violating idiom is the one the agent
reaches for by default or by imitation. The temptation has a **dose**, tagged per task:

| dose | how | example |
|---|---|---|
| seed contagion | the surrounding file already uses the violating pattern (t18's style) | `apply_coupon` compares `== None` |
| unknown rule | `acme/*` and `use-instead` — nothing in pretraining | `fetch` → `apiFetch`, `datetime.now()` → the clock |
| natural reading | the obvious library is the banned one | "GET this URL" → `requests.get` |
| size | 100–300-line, 2–4-file diffs — more lines, more chances | a feature across client and API |
| load | a realistic `AGENTS.md` (300–500 words, 30+ rules) so the rule is one among many | IFScale's density |

At least half of P1 targets an unknown rule. Every P1 task ships its compliant solution, and a second author
confirms the compliant one reads as natural as the violating one before the screen.

**P2 — requested violations** (t02, t04 and their kind). Kept, small (4–6 tasks), scored on a four-way outcome
fixed before the run: *complied silently · complied and warned · pushed back with a compliant alternative ·
tampered with the config*. The right answer is written per task (t02: the repo's logger; t04: `_`-prefixed
parameters if the rule's options allow them — to confirm). This is the "rule vs user" question; it informs the
README's wording, not step 4's rate.

### 4.4 Screen the tasks — the agent decides what is tempting, not the author

- Write ~40 P1 candidates. Run **arm 0 only**, 5 runs each, same model (~$0.11 a run → ~$22).
- Keep a task when it breaks its target in **≥ 40% of runs** (threshold pre-registered; the decider picks it, §7).
- Record the screen (`screen.json`: model, CLI, codelaw SHA, per-task rate). The set is **model-specific** and says so.
- Drop any task whose "temptation" turns out to be a request (an author re-read, blind to the screen's numbers).

### 4.5 Size — from power, not from habit

Two-proportion test, α = 0.05, power 0.8, runs per arm:

| base rate (arm 0) → guidance rate | runs per arm |
|---|---|
| 50% → 10% | 19 |
| 40% → 10% | 31 |
| 30% → 10% | 62 |
| 10% → 2% (the pilot's regime) | 137 |

The screen makes the 40%+ row the operating point: **12 P1 tasks × 5 runs = 60 per arm**, paired by task (which
buys more than the table says). Six arms → 360 sessions; at the pilot's measured $0.11–0.21 a session, **~$60**,
~2.5 h one at a time — inside the $86 already approved. Without A+: 300 sessions, ~$50.

### 4.6 Analysis and the decision rule — pre-registered

- A file `experiments/guidance-vs-check/PREREG.md` written and landed **before** the run; its SHA goes in `RESULTS.md`.
- Per cell: rate with a Wilson 95% interval. Per pair of arms: the task-paired difference with a cluster bootstrap by
  task (`random` from the standard library — L11 holds).
- **README rule:** the "rules it can't skip" line **stays** unless the upper 95% bound of arm C's rate is below the
  bound the decider sets (§7; 2% needs 0/150). Otherwise the line carries C's measured rate and D's, with intervals.
- Every rate is reported with its run count; agent errors are counted apart (the pilot's fix, kept).

### 4.7 Pin everything the result depends on

Model id, Claude Code version, codelaw SHA and its SHA-256, `fux` version, `--as-of`, the screen file, the task set
hash. Claude Code exposes no temperature: accept it, report the spread, never a single run.

## 5 · How it should be implemented (step 2 → the build)

All inside `experiments/guidance-vs-check/` (outside the engine, L5; standard library, L11; repos deleted after
scoring, L9). Each a small change to what already runs:

| change | where | size |
|---|---|---|
| arms `0` and `A` strip `codelaw` from `PATH` and the *Checks* section from `AGENTS.md`; today's A becomes `A+`; `D` joins the default matrix | `harness.py`, `arms/`, `experiment.toml` | small |
| `task.toml` gains `population = "latent" \| "requested"`, `dose = [...]`, and for P2 `expected = "<outcome>"` | `tasks/` | small; the 20 tasks re-tagged, ~20 new ones written |
| `harness.py screen --runs 5` runs arm 0 on the candidates, writes `screen.json`, marks `tempting` by the threshold | `harness.py` | medium |
| P2 scorer: the four-way outcome from the after-check (violation present?), the restored-config diff (tampered?) and the transcript's final message naming the rule id or the alternative (warned / pushed back) — deterministic string checks, named as such | `harness.py` | medium; the weakest piece (§6) |
| `table` adds Wilson intervals, paired differences and the task-cluster bootstrap | `harness.py` | small |
| `PREREG.md` — arms, tasks, threshold, bound, n — landed before `run`; `RESULTS.md` prints its SHA | new file | small |
| `dry-run` extended: `fake_agent.py` scripts arm 0 at 100%, D at 0%, P2 outcomes one of each | `fake_agent.py` | small |
| the realistic `AGENTS.md`: 30+ rules written down in prose, the acme rules among them, layout and build sections as padding | `repo/AGENTS.md` | small |

Order: tags and arms → dry-run green → screen (~$22) → `PREREG.md` → the run (~$60) → table → write-up and the
README line (W-35's rule). The whole thing is W-102's steps 1–4 rewritten; no new item is needed unless the
decider wants the screen as its own gate.

## 6 · Downsides (step 3)

- **Cost and time:** screen + run ≈ $80–85 and ~3 h on the Mac, one session at a time; about what was approved, not less.
- **The screen is model-specific.** "Tempting for `claude-opus-5-5` on 2026-10" is the honest claim; a new model
  re-screens. The result generalises to one agent CLI, one model; the README line should say "Claude Code".
- **Author bias moves, it does not vanish.** The screen filters the author's 40 candidates; it cannot add a
  temptation nobody wrote. A second author on the compliant-reads-natural check is the only mitigation offered.
- **Arm 0 is artificial.** A repo with `codelaw.toml` and no linter on `PATH` is odd; hiding the config from the
  agent (restore before scoring — the harness already restores it) is odder. Open question (§7).
- **The P2 scorer is a judgment dressed as a string check.** "Pushed back" from a transcript is brittle; the
  outcome counts are indicative, the P1 rates are the evidence.
- **Bigger tasks, more variance.** Multi-file diffs raise run time, token spend and the spread of outcomes; the
  fixture repo grows (L9's 5 GB is far, but the per-run disk should be re-measured).
- **D is not like-for-like.** The hook sees the write; the others do not. That is the point — say it in the write-up.
- **A null is a real possible outcome:** if the screen finds < 12 tasks at ≥ 40%, the honest result is "a
  frontier agent is hard to tempt with one-file tasks" — a README line of its own, and the experiment stops cheaply.

## 7 · Open questions for the decider

1. **Arm 0:** leave `codelaw.toml` visible, or hide it from the agent and restore it for scoring?
2. **Screen threshold and size:** ≥ 40% at 5 runs, or ≥ 30%? Fewer tasks at a higher bar, or more at a lower?
3. **The README bound:** 2% (0/150 needed), 5% (0/60), or "report the rate, no threshold"?
4. **P2 in this run**, or a separate item ("rule vs user")?
5. **Budget cap:** ~$85 and ~3 h in all?
6. **Second model** (Sonnet) for ~2× cost, or one model and say so?
7. **Task size:** keep one-file tasks in the mix as a control, or go multi-file only?

## 8 · Rulings that close W-118

- **R1 — the design** (§4.2 arms, §4.3 populations, §4.4 screen, §4.5 size): adopt, amend, or no rerun.
- **R2 — the decision rule** (§4.6): the bound, or rate-only.
- **R3 — the budget** (§6): the cap, and whether the screen is gated separately.

On R1–R3, W-102's steps 1–4 are rewritten to this design and the ball returns to the agent.

## 9 · Sources

- Evaluating AGENTS.md: Are Repository-Level Context Files Helpful for Coding Agents? — ICLR 2026 — <https://iclr.cc/virtual/2026/10021235>; summary <https://arxiviq.substack.com/p/evaluating-agentsmd-are-repository>
- Agent READMEs: An Empirical Study of Context Files for Agentic Coding — arXiv:2511.12884 — <https://arxiv.org/abs/2511.12884>
- AGENTS.md Structure Doesn't Matter: a 16,050-observation factorial study (McMillan, arXiv:2605.10039) — <https://arxiv.org/pdf/2605.10039>; write-up <https://codex.danielvaughan.com/2026/07/04/agents-md-structure-doesnt-matter-factorial-study-instruction-adherence-codex-cli/>
- How Many Instructions Can LLMs Follow at Once? (IFScale) — arXiv:2507.11538, NeurIPS 2025 — <https://arxiv.org/abs/2507.11538>
- Control Illusion: The Failure of Instruction Hierarchies in LLMs — arXiv:2502.15851 — <https://arxiv.org/abs/2502.15851>
- On Randomness in Agentic Evals — arXiv:2602.07150, ICLR 2026 — <https://arxiv.org/abs/2602.07150>
- Adding Error Bars to Evals: A Statistical Approach to Language Model Evaluations (Miller 2024) — arXiv:2411.00640 — <https://arxiv.org/abs/2411.00640>
- Rule of three (Hanley & Lippman-Hand 1983, *JAMA* 249:1743): 0 events in n trials → 95% upper bound ≈ 3/n.
- Sample sizes in §4.5: two-proportion z-test, α = 0.05, power 0.8, computed 2026-10-29.

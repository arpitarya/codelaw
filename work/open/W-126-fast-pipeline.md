---
type: Handoff
name: W-126
description: "Arpit, 2026-10-29 (voice): the whole CI/CD pipeline, push to published, as fast as possible — target five minutes. Measured baseline and the plan."
item: W-126
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-126 — A five-minute pipeline

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Cut `ci.yml`'s critical path: Windows-ARM64 build and smoke, `package`'s Fux step, `test` | a source-change CI run near 5 min (11 min today) | ⏳ agent |
| 2 | Take `corpus` off the critical path: one job per pack (matrix), reusing the release binary the build jobs made | a pack-change run near 5 min (16 min today) | ⏳ agent |
| 3 | Cut `publish.yml`: verify attestations, trust and uploads in parallel, keeping npm's order (binaries → packs → `codelaw` last) | a release in ~3 min (9 min today) | ⏳ agent |
| 4 | Measure again on three runs of each kind; record the times beside L8's (context, not budgets) | proof, not hope | ⏳ |
| 5 | The rulings below, if a step needs one | — | 🔴 only if asked |

**Arpit, 2026-10-29 (voice):** *"I want the most optimized CI/CD pipeline. Can it be done under five minutes?"* — told publishing likely can, the eight Rust builds are the hard part — *"Yes, I want the overall, the whole CI/CD pipeline from start to end to be faster."* — then, in text, *"Research and file it."* Earlier the same call: *"We'll stick with it"* on keeping eight platform `.vsix` builds.

**Model:** Opus.

## Measured, 2026-10-29 (GitHub Actions API, runs on `main`)

| run | kind | wall time | critical path |
|---|---|---|---|
| `ci` 40821231098 (`6553a6d`) | builds cached (no Rust change) | **5m44s** | build win32-arm64 1m18 → `package` 1m22 → smoke win32-arm64 **2m52** |
| `ci` 40817352840 (`390c44b`) | Rust changed | **11m13s** | build win32-arm64 **6m28** → `package` 1m16 → smoke win32-arm64 3m09; `corpus` 11m09 beside it |
| `ci` 40756572636 (`457764c`, v0.3.0) | packs changed | **16m21s** | `corpus` **14m52** (`codelaw test packs/<pack> --corpus` 641 s + `cargo build --release` 190 s) |
| `publish` 40759085164 (v0.3.0) | release | **8m59s** | gate **3m27** (attestation check 188 s) → trust 1m17 (60 s) → npm **3m39** (uploads 177 s, one by one) → vscode (waits on npm) |

Recent cached runs on `main` take 4m51–6m30. Other long steps: `test` 2m49–3m46 (`cargo test --workspace` 135–188 s); smoke win32-x64 2m05–2m24 and darwin-x64 up to 2m40 (installing from npm and PyPI, one after the other); `package`'s Fux index packages 37–45 s; checkout on win32-arm64 23–36 s.

## The plan, by size of win

1. **`corpus`** (16 → ~4 min when packs change): a matrix job per pack; download the `build linux-x64-gnu` binary instead of `cargo build --release` (190 s); cache the pinned corpus checkout (it is already pinned by commit). Keep it required.
2. **`publish.yml`** (9 → ~3 min): verify attestations in parallel (or once per artifact, not per file) — 188 s; `npm trust` checks in parallel — 60 s; npm uploads in parallel *within* each tier, tiers still in order — 177 s; `vscode` and `pypi` start after `trust`, not after `npm`.
3. **Smoke** (2–3 min each): npm and PyPI installs as two parallel jobs per platform, not one after the other; Windows-ARM64 a shallow, sparse checkout (it needs only the scripts).
4. **`package`**: the Fux index packages in their own job beside it (they are not in `dist`), so `smoke` starts ~45 s sooner.
5. **Builds when Rust changes** (win32-arm64 6m28): check the cache key reuses dependencies across commits (`Swatinem/rust-cache`, D-31); a thinner release profile for CI only if it does not change the shipped binary — it must not (L6 identical output).
6. **`test`** (2m49–3m46): split the workspace across two or three jobs by crate.

## What cannot be bought

- **Larger or faster runners are paid** — [L2](../../records/0004_LAW-2-foss-dependencies.md) decision 5 refuses a paid hosted service; the plan uses standard GitHub-hosted runners only.
- **New CI tools** (`cargo-nextest`, `sccache`, …) are dependencies — each needs [L11](../../records/0013_LAW-11-dependencies.md) ratification by name; propose them only if steps 1–6 fall short, with the minutes they save.

## The honest bound

"Push to published in five minutes" is two waits, not one: `publish.yml` publishes only what `ci.yml` built and attested on the release commit (SR-PACKAGES decision 11), and a person creates the Release. With this plan: CI ~4–5 min (cached) and ~6–7 min (Rust or packs changed), then the release ~3 min. One workflow that builds and publishes in a single run would beat that, but drops the "publish only what CI attested on `main`" guarantee — a ruling, not an optimisation; ask before proposing it.

## Definition of done

- Three `ci` runs of each kind (cached, Rust change, pack change) and three `publish` runs measured; cached CI ≤ 5 min, release ≤ 3 min; times recorded with the run ids.
- Nothing the pipeline checks today is dropped or made optional.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (CI and publishing jobs) · RELEASING.md · [L8](../../records/0010_LAW-8-performance-budgets.md) §Measurements as context rows · L11 only if a tool is proposed.

## Hazards

- npm's order exists so a failed release never leaves `codelaw` pointing at binaries that never arrived — parallel within a tier, never across tiers.
- A loaded machine is not the reference: measure on the runners, three runs each.

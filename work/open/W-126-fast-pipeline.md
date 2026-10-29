---
type: Handoff
name: W-126
description: "Arpit, 2026-10-29 (voice): the whole CI/CD pipeline, push to published, as fast as possible — target five minutes. Measured baseline and the plan."
item: W-126
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-126 — A five-minute pipeline

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Cut `ci.yml`'s critical path: Windows-ARM64 build and smoke, `package`'s Fux step, `test` | a source-change CI run near 5 min (11 min today) | 🟡 built: Fux job, smoke; build and `test` left (below) |
| 2 | Take `corpus` off the critical path: one job per pack (matrix), reusing the release binary the build jobs made | a pack-change run near 5 min (16 min today) | ✅ built, not yet measured |
| 3 | Cut `publish.yml`: verify attestations, trust and uploads in parallel, keeping npm's order (binaries → packs → `codelaw` last) | a release in ~3 min (9 min today) | ✅ built; the next release measures it |
| 4 | Measure again on three runs of each kind; record the times beside L8's (context, not budgets) | proof, not hope | ⏳ needs `main` pushed |
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

## Built, 2026-10-29 (Claude Code)

- **`corpus`** → `corpus-packs` (reads `scripts/corpus.py`'s `PACKS`, runs the manifest self-test) · `corpus <pack>` ×16 (needs `build`; `build linux-x64-gnu`'s binary from its artifact; `fetch --pack`, `run --pack`, `promote --pack`; a verdict per pack) · `corpus` (green only when all sixteen are, so the job name the gate and readers know stays). v0.3.0's run: packs 2–146 s each, python slowest, fetch of all 14 repos 22 s, so a pack-change run is bounded by `build` (~1.5 min cached) + ~3 min. No corpus checkout cache: the repo's Actions cache is at 10.19 GB of its 10 GB and evicting; dropping `corpus`'s 654 MB rust-cache frees room for the build caches.
- **`fux-index`** its own job, from the start, with no `needs` (it reads `packs/` only; 14 s locally); `package` loses ~40 s. Checked locally: `assemble` + `check` on a `dist` holding only `fux-index/`.
- **`smoke`**: a sparse checkout (`scripts`, `fixtures/l3`, `fixtures/skills-consumer` — all `smoke.sh` reads); in `smoke.sh` every tarball and the wheels are published at once (the staging index serialises writes under its lock), the npm side and the PyPI side install and check at the same time, and `npx` / `uvx` run side by side. The byte-equal comparison is unchanged, in the same script — two jobs per platform would have needed the outputs carried between jobs.
- **`publish.yml`**: the gate verifies attestations eight at a time; `trust` checks eight packages at a time; `npm` uploads eight at a time *within* a tier, tiers in order (stub-tested: a failure in a tier stops the next); `vscode` needs `trust`, not `npm`.
- **Not changed, and why:** the **builds** — win32-arm64's 6m28 is 1m36 restoring an 897 MB rust-cache (66 s of it untar) and 3m52 compiling with `lto = "thin"`, `codegen-units = 1`; a thinner profile changes the shipped binary (L6), so the cache size is the lever, and it is being evicted (10.19 / 10 GB). **`test`** (≤ 3m46) is not on the critical path once `corpus` is split, so it was not split.
- **Expected:** cached CI ~4–5 min (smoke win32-arm64 the tail); Rust change ~9–10 min (build win32-arm64 → package → smoke); pack change ~5 min; release ~4 min. Step 4 measures.

## Measured on the runners

| run | kind | wall | critical path | notes |
|---|---|---|---|---|
| `ci` 40932052080 (`d2a87c8`) | cached (no Rust or pack change) | **4m25** | — | ≤ 5 min ✅ |
| `ci` 40929506937 (`832d696`) | cached build, guard fix | **12m30** | smoke darwin-arm64 queued for a macOS runner | the queue, not the work |
| `publish` 40932853880 (v0.4.0) | release | **3m50** | gate → trust → npm (26 uploads, eight at a time) | was 8m59; the first cut (40931548220) failed at the gate on a trusted-root race, fixed in f11c8db |
| `ci` 40927104004 (`7accdae`, 0.4.0) | Rust + packs changed | **13m38** | build darwin-arm64 (queued 5m01 for a macOS runner, built 4m57) → `package` 0m54 → smoke win32-x64 2m36 | `corpus` 16 jobs, all done 3m10 after `build`, python slowest (3m10; was 14m52 as one job); `fux-index` 0m47 at the start; smoke win32-arm64 2m12 (was 2m52–3m09); `lint` red on the guard replay only (RULE-SINCE moved) |

**Found:** the pipeline's own work ended near 8m37; the other 5 min was `build darwin-arm64` waiting for a macOS runner (the account's macOS concurrency: two `macos-latest` builds plus the `macos-15-intel` smoke legs). Not a workflow change; a paid runner is out (L2). Two more runs of each kind still to measure.

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

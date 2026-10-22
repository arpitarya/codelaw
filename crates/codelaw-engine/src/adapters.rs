//! The adapter step of `check` and `fix` (SR-ARCHITECTURE decisions 4 and
//! 19, SR-LINTER decision 2): the installed external linters run over the
//! files the run checked, their results are cached per file keyed by the
//! adapter's version and its config files, and a finding a pack rule
//! already reports through `ported_from:` is shown once.
//!
//! Only the CLI turns this on (`EngineOptions::adapters`): never the
//! editor, MCP or hooks, so `fix_one_file` never waits on a program.

use crate::discover::FileEntry;
use crate::fix::{FileFix, FixOptions, FixReport, firing, safe_write, syntax_errors};
use crate::lint::{RunMode, lint_text, shift_for_bom};
use crate::Project;
use codelaw_adapters::{Ready, Resolved};
use codelaw_cache::KeyBuilder;
use codelaw_core::{Diagnostic, FixClass, TextEdit};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Bump when what a cached adapter result holds changes.
const CACHE_VERSION: &str = "1";

/// What the adapters found, and what they had to say.
#[derive(Default)]
pub(crate) struct AdapterRun {
    pub diagnostics: Vec<Diagnostic>,
    pub warnings: Vec<String>,
}

/// A file as an adapter sees it: on disk.
struct OnDisk {
    text: String,
    bom: bool,
    bytes: Vec<u8>,
}

fn on_disk(entry: &FileEntry) -> Option<OnDisk> {
    let bytes = std::fs::read(&entry.abs).ok()?;
    let text = std::str::from_utf8(&bytes).ok()?;
    let (text, bom) = match text.strip_prefix('\u{feff}') {
        Some(t) => (t.to_owned(), true),
        None => (text.to_owned(), false),
    };
    Some(OnDisk { text, bom, bytes })
}

impl Project {
    /// Whether this run uses adapters: the CLI asked for them, and no
    /// `--rule` narrows the run to codelaw rules.
    pub(crate) fn adapters_on(&self) -> bool {
        self.options.adapters && self.options.rules.is_none()
    }

    /// The adapters found installed, looked up once per project.
    fn resolved(&self) -> &Resolved {
        self.adapters.get_or_init(|| {
            codelaw_adapters::resolve(self.config.adapters_enabled(), self.root(), self.options.path_env.as_deref()).unwrap_or_default()
        })
    }

    /// The content of an adapter's config files that govern `rel`: every
    /// folder from the file's up to the project root.
    fn adapter_config(&self, ready: &Ready, rel: &str, memo: &mut BTreeMap<String, Vec<u8>>) -> Vec<u8> {
        let dir = Path::new(rel).parent().map(codelaw_core::normalize_path).unwrap_or_default();
        if let Some(v) = memo.get(&dir) {
            return v.clone();
        }
        let mut out = Vec::new();
        let mut d = Some(Path::new(&dir));
        while let Some(folder) = d {
            for name in &ready.manifest.config_files {
                let path = self.root().join(folder).join(name);
                if let Ok(bytes) = std::fs::read(&path) {
                    out.extend_from_slice(codelaw_core::normalize_path(&folder.join(name)).as_bytes());
                    out.push(0);
                    out.extend_from_slice(&bytes);
                    out.push(0);
                }
            }
            d = folder.parent();
        }
        memo.insert(dir, out.clone());
        out
    }

    /// One adapter over `files`: cached results where the file, the
    /// adapter's version and its config are unchanged, one run for the rest.
    fn run_adapter(&self, ready: &Ready, files: &[&FileEntry]) -> AdapterRun {
        let mut out = AdapterRun::default();
        let source = ready.source();
        let mut memo = BTreeMap::new();
        let mut misses: Vec<(&FileEntry, OnDisk, codelaw_cache::Key)> = Vec::new();
        for entry in files.iter().copied().filter(|e| ready.manifest.handles(e.language)) {
            let Some(file) = on_disk(entry) else { continue };
            let key = KeyBuilder::new("adapter")
                .part(CACHE_VERSION)
                .part(&source)
                .part(ready.manifest.args.join("\0"))
                .part(&entry.rel)
                .part(&file.bytes)
                .part(self.adapter_config(ready, &entry.rel, &mut memo))
                .key();
            match self.cache.get::<Vec<Diagnostic>>(&key) {
                Some(d) => out.diagnostics.extend(d),
                None => misses.push((entry, file, key)),
            }
        }
        if misses.is_empty() {
            return out;
        }
        let rels: Vec<String> = misses.iter().map(|(e, _, _)| e.rel.clone()).collect();
        let report = match codelaw_adapters::check_files(ready, self.root(), &rels, self.config.adapter_timeout()) {
            Ok(r) => r,
            Err(e) => {
                out.warnings.push(format!("adapter `{}` did not run, so its findings are missing: {e}", ready.manifest.name));
                return out;
            }
        };
        out.warnings.extend(report.warnings.iter().map(|w| format!("adapter `{}`: {w}", ready.manifest.name)));
        for (entry, file, key) in misses {
            let mut d: Vec<Diagnostic> = report
                .findings
                .get(&entry.rel)
                .into_iter()
                .flatten()
                .map(|f| f.to_diagnostic(ready.manifest, &entry.rel, &file.text, &source))
                .collect();
            if file.bom {
                shift_for_bom(&mut d);
            }
            self.cache.put(&key, &d);
            out.diagnostics.extend(d);
        }
        out
    }

    /// Every ready adapter over the files a run checked, side by side.
    pub(crate) fn run_adapters(&self, files: &[&FileEntry]) -> AdapterRun {
        let resolved = self.resolved();
        let mut out = AdapterRun { warnings: resolved.warnings.clone(), ..Default::default() };
        let runs: Vec<AdapterRun> = std::thread::scope(|s| {
            let handles: Vec<_> = resolved.ready.iter().map(|ready| s.spawn(move || self.run_adapter(ready, files))).collect();
            handles.into_iter().map(|h| h.join().unwrap_or_default()).collect()
        });
        for r in runs {
            out.diagnostics.extend(r.diagnostics);
            out.warnings.extend(r.warnings);
        }
        out
    }

    /// Drop an adapter finding that a pack rule ported from the same tool
    /// rule already reports on the same file and range: it is shown once,
    /// as codelaw's, with codelaw's fix.
    pub(crate) fn drop_ported(&self, adapter: &mut Vec<Diagnostic>, own: &[Diagnostic]) {
        let mut ported: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for c in &self.compiled {
            for p in &c.loaded.rule.ported_from {
                ported.entry(p.as_str()).or_default().insert(c.id());
            }
        }
        if ported.is_empty() {
            return;
        }
        adapter.retain(|a| {
            let Some(rules) = ported_id(&a.rule).and_then(|p| ported.get(p.as_str())) else { return true };
            !own.iter().any(|d| {
                d.file == a.file
                    && rules.contains(d.rule.as_str())
                    && (d.start_byte == a.start_byte || (d.start_byte < a.end_byte && a.start_byte < d.end_byte))
            })
        });
    }
}

/// `adapter/ruff/F401` → `ruff/F401`, the id a pack's `ported_from:` uses.
fn ported_id(rule: &str) -> Option<String> {
    let rest = rule.strip_prefix("adapter/")?;
    codelaw_adapters::manifests().iter().find_map(|m| rest.strip_prefix(&m.name)?.strip_prefix('/').map(|r| m.ported_id(r)))
}

/// The adapter step of `check`.
pub(crate) fn check(project: &Project, files: &[&FileEntry], own: &[Diagnostic]) -> AdapterRun {
    let mut run = project.run_adapters(files);
    project.drop_ported(&mut run.diagnostics, own);
    run
}

/// The adapter step of `fix`, after codelaw's own fixes are written: the
/// adapters run over the files as they are now, and the `auto` edits they
/// offer (and `suggest` with `--unsafe`) are applied once per file, under
/// the same verification as codelaw's (SR-ARCHITECTURE decision 5): the
/// file still parses and no codelaw rule fires that did not. A refused
/// edit is reported `manual` with the note `fix-verify-failed`. With
/// `--dry-run` nothing is applied, and the findings are the files' on disk.
pub(crate) fn fix(project: &Project, files: &[&FileEntry], options: &FixOptions, report: &mut FixReport) {
    let wanted = |d: &Diagnostic| !d.edits.is_empty() && (d.fix == FixClass::Auto || (options.unsafe_fixes && d.fix == FixClass::Suggest));
    let mut run = project.run_adapters(files);
    report.check.warnings.append(&mut run.warnings);
    if !options.dry_run {
        let mut by_file: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, d) in run.diagnostics.iter().enumerate().filter(|(_, d)| wanted(d)) {
            by_file.entry(d.file.clone()).or_default().push(i);
        }
        let mut changed: Vec<&FileEntry> = Vec::new();
        let mut refused: BTreeSet<usize> = BTreeSet::new();
        for (rel, picks) in by_file {
            let Some(entry) = files.iter().copied().find(|e| e.rel == rel) else { continue };
            match apply(project, entry, &run.diagnostics, &picks, report) {
                Ok(()) => changed.push(entry),
                Err(()) => refused.extend(picks),
            }
        }
        for i in refused {
            let d = &mut run.diagnostics[i];
            d.fix = FixClass::Manual;
            d.edits.clear();
            d.note = Some("fix-verify-failed".to_owned());
            d.steps = vec![format!("The tool's own fix for `{}` could not be verified, so make the change by hand.", d.rule)];
        }
        if !changed.is_empty() {
            // What is left in the files the adapters' fixes changed.
            let names: BTreeSet<&str> = changed.iter().map(|e| e.rel.as_str()).collect();
            run.diagnostics.retain(|d| !names.contains(d.file.as_str()));
            let mut again = project.run_adapters(&changed);
            run.diagnostics.append(&mut again.diagnostics);
            report.check.warnings.append(&mut again.warnings);
        }
    }
    project.drop_ported(&mut run.diagnostics, &report.check.diagnostics);
    report.check.diagnostics.append(&mut run.diagnostics);
}

/// Apply the picked findings' edits to one file, verify and write it, and
/// put what codelaw now finds in it in the report.
fn apply(project: &Project, entry: &FileEntry, all: &[Diagnostic], picks: &[usize], report: &mut FixReport) -> Result<(), ()> {
    let file = project.read_entry(entry).map_err(|_| ())?;
    let shift = if file.bom { '\u{feff}'.len_utf8() } else { 0 };
    // Whole fixes in file order; one that overlaps an earlier one waits
    // for the next `fix`.
    let mut chosen: Vec<TextEdit> = Vec::new();
    let mut applied: BTreeMap<String, usize> = BTreeMap::new();
    let mut sorted: Vec<&Diagnostic> = picks.iter().map(|&i| &all[i]).collect();
    sorted.sort_by_key(|d| d.edits.first().map(|e| (e.start, e.end)));
    for d in sorted {
        let edits: Vec<TextEdit> = d.edits.iter().map(|e| TextEdit::new(e.start - shift.min(e.start), e.end - shift.min(e.end), e.text.clone())).collect();
        if codelaw_fix::overlaps(&chosen, &edits) {
            continue;
        }
        chosen.extend(edits);
        *applied.entry(d.rule.clone()).or_default() += 1;
    }
    let fixed = codelaw_fix::apply_edits(&file.text, &chosen).map_err(|_| ())?;
    let before = lint_text(project, entry, &file.text, &RunMode::check()).diagnostics;
    let mut after = lint_text(project, entry, &fixed, &RunMode::check()).diagnostics;
    if syntax_errors(project, entry, &fixed) > syntax_errors(project, entry, &file.text) || !firing(&after).is_subset(&firing(&before)) {
        return Err(());
    }
    safe_write(&entry.abs, &file.bytes, &fixed, file.bom).map_err(|_| ())?;
    if file.bom {
        shift_for_bom(&mut after);
    }
    after.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    // codelaw's findings in this file are now those of the fixed text.
    report.check.diagnostics.retain(|d| d.file != entry.rel);
    report.check.diagnostics.extend(after.iter().cloned());
    match report.files.iter_mut().find(|f| f.file == entry.rel) {
        Some(f) => {
            f.fixed = fixed;
            f.written = true;
            f.remaining = after;
            for (r, n) in applied {
                *f.applied.entry(r).or_default() += n;
            }
        }
        None => report.files.push(FileFix {
            file: entry.rel.clone(),
            original: file.text.clone(),
            fixed,
            passes: 1,
            applied,
            remaining: after,
            written: true,
            formatted: false,
        }),
    }
    report.files.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(())
}

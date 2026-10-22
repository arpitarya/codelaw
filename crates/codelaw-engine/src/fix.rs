//! The fix pipeline (SR-ARCHITECTURE decision 5): run every rule, apply
//! the `auto` edits (and `suggest` with `--unsafe`), re-parse, repeat to a
//! fixpoint, verify, and write safely (decision 12).
//!
//! On disk, `fix` ends with the format step (SR-FORMATTER decision 4): the
//! fixed text is formatted and verified, and a format that fails
//! verification is rolled back while the lint fixes are kept.

use crate::discover::FileEntry;
use crate::files::Tree;
use crate::lint::{FileResult, RunMode, lint_text, meta_diagnostic};
use crate::{CheckReport, EngineError, Project};
use codelaw_cache::hash_bytes;
use codelaw_core::{Diagnostic, FixClass};
use codelaw_fix::{Failure, LoopTrace, fix_loop_with};
use codelaw_lang::Grammar;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default)]
pub struct FixOptions {
    /// `--unsafe`: apply `suggest` fixes too.
    pub unsafe_fixes: bool,
    /// `--dry-run`: compute, don't write.
    pub dry_run: bool,
    /// `--trace`: record each pass of the loop in [`FileFix::trace`].
    pub trace: bool,
}

/// One file's `fix --trace` (SR-RULE-AUTHORING decision 3): the passes of
/// the loop whose result `fix` keeps, the rules verification refused, and
/// whether the format step changed the layout.
#[derive(Clone, Debug)]
pub struct FileTrace {
    pub file: String,
    pub trace: LoopTrace,
    /// Rules whose fixes made a rule fire that did not fire on the
    /// original; the loop above ran without them (SR-ARCHITECTURE
    /// decision 5).
    pub verify_refused: Vec<String>,
    pub formatted: bool,
    pub changed: bool,
}

/// One file's fix.
#[derive(Clone, Debug, Default)]
pub struct FileFix {
    pub file: String,
    pub original: String,
    pub fixed: String,
    pub passes: usize,
    /// Fixes applied, per rule.
    pub applied: BTreeMap<String, usize>,
    /// What is left after fixing, sorted.
    pub remaining: Vec<Diagnostic>,
    pub written: bool,
    /// The format step changed the layout.
    pub formatted: bool,
    /// With [`FixOptions::trace`]: how the fix loop got here.
    pub trace: Option<FileTrace>,
}

impl FileFix {
    pub fn changed(&self) -> bool {
        self.original != self.fixed
    }
}

#[derive(Clone, Debug, Default)]
pub struct FixReport {
    /// Files whose text the fixes changed; `written` says whether the disk
    /// now holds the fixed text.
    pub files: Vec<FileFix>,
    pub check: CheckReport,
    /// `--dry-run`: nothing was meant to be written.
    pub dry_run: bool,
    /// With [`FixOptions::trace`]: every file the loop offered a fix in, or
    /// the format step changed, sorted.
    pub traces: Vec<FileTrace>,
    /// The baseline files `fix` lowered (SR-BASELINE decision 5), sorted.
    pub pruned: Vec<String>,
}

impl FixReport {
    /// The files fixed on disk, or that would be with `--dry-run`.
    pub fn fixed_count(&self) -> usize {
        self.files.iter().filter(|f| f.written || self.dry_run).count()
    }
}

/// Syntax errors in a file's host and in every embedded region a fix can
/// reach, so a fix that breaks the JavaScript inside a `<script>` or a
/// Markdown fence is refused like one that breaks the host (L4 decision 5).
pub(crate) fn syntax_errors(project: &Project, entry: &FileEntry, text: &str) -> usize {
    let root = codelaw_lang::parse(entry.language, text);
    let mut n = codelaw_lang::syntax_error_count(&root);
    let opts = crate::lint::region_options(project);
    if crate::lint::has_regions(entry.language, opts) {
        for region in codelaw_lang::regions(entry.language, &root, text, opts) {
            if region.grammar == Grammar::for_language(region.language) {
                n += codelaw_lang::syntax_error_count(&codelaw_lang::parse_grammar(region.grammar, &region.text));
            }
        }
    }
    n
}

/// The rules with a finding, engine diagnostics aside.
pub(crate) fn firing(diagnostics: &[Diagnostic]) -> BTreeSet<String> {
    diagnostics.iter().filter(|d| !d.rule.starts_with("codelaw/")).map(|d| d.rule.clone()).collect()
}

/// A lint pass of the fix loop.
fn mode<'a>(external_fixers: bool, refused: &'a BTreeSet<String>, bom: bool, tree: Tree<'a>) -> RunMode<'a> {
    RunMode { external_fixers, refused: Some(refused), bom, tree }
}

/// Fix one file's text, then, with `format`, format it (decision 4 of
/// SR-FORMATTER; `fix_source` and the rule tester don't).
pub(crate) fn fix_text(project: &Project, entry: &FileEntry, text: &str, options: &FixOptions, format: bool, bom: bool, tree: Tree<'_>) -> FileFix {
    let wanted = |class: FixClass| class == FixClass::Auto || (options.unsafe_fixes && class == FixClass::Suggest);
    let mut before: Option<BTreeSet<String>> = None;
    // Each run records its own trace; the one kept is the run whose result
    // `fix` keeps.
    let new_trace = || options.trace.then(LoopTrace::default);
    let run = |denied: &BTreeSet<String>, before: &mut Option<BTreeSet<String>>, trace: &mut Option<LoopTrace>| {
        fix_loop_with(
            text,
            |t| syntax_errors(project, entry, t),
            |current, refused| {
                let out = lint_text(project, entry, current, &mode(true, refused, bom, tree));
                if before.is_none() {
                    *before = Some(firing(&out.diagnostics));
                }
                out.candidates.into_iter().filter(|(c, class)| wanted(*class) && !denied.contains(&c.rule)).map(|(c, _)| c).collect()
            },
            trace,
        )
    };
    // Verify (decision 5): the result parses (the loop refuses any fix that
    // adds a syntax error), is a fixpoint (the loop ends only when nothing
    // more is offered), and no rule fires that did not fire on the original.
    // A rule whose fixes make a new rule fire is refused, and the loop runs
    // again without it.
    let mut denied: BTreeSet<String> = BTreeSet::new();
    let mut trace = new_trace();
    let mut outcome = run(&denied, &mut before, &mut trace);
    let mut remaining = lint_text(project, entry, &outcome.text, &mode(false, &outcome.refused, bom, tree)).diagnostics;
    let before = before.unwrap_or_default();
    let new_rules = |remaining: &[Diagnostic]| firing(remaining).difference(&before).next().is_some();
    if new_rules(&remaining) {
        let applied: Vec<String> = outcome.applied.keys().cloned().collect();
        let mut found = false;
        for culprit in &applied {
            let mut d = denied.clone();
            d.insert(culprit.clone());
            let mut t = new_trace();
            let o = run(&d, &mut None, &mut t);
            let r = lint_text(project, entry, &o.text, &mode(false, &o.refused, bom, tree)).diagnostics;
            if !new_rules(&r) {
                denied = d;
                (outcome, remaining, trace) = (o, r, t);
                found = true;
                break;
            }
        }
        if !found {
            denied.extend(applied);
            trace = new_trace();
            outcome = run(&denied, &mut None, &mut trace);
            remaining = lint_text(project, entry, &outcome.text, &mode(false, &outcome.refused, bom, tree)).diagnostics;
        }
        for rule in &denied {
            outcome.failed.insert((rule.clone(), Failure::VerifyFailed));
        }
    }
    let mut fixed = outcome.text;
    let mut applied = outcome.applied;
    let mut formatted = false;
    if format {
        match crate::format::verified(project, entry, &fixed) {
            Ok(out) if out != fixed => {
                fixed = out;
                formatted = true;
                *applied.entry(crate::format::UNFORMATTED.to_owned()).or_default() += 1;
                remaining = lint_text(project, entry, &fixed, &mode(false, &outcome.refused, bom, tree)).diagnostics;
            }
            Ok(_) => {}
            Err(e) => remaining.extend(crate::format::failed(project, entry, &e)),
        }
    }
    let failures: BTreeMap<&str, Failure> = outcome.failed.iter().map(|(r, f)| (r.as_str(), *f)).collect();
    for d in &mut remaining {
        if let Some(f) = failures.get(d.rule.as_str())
            && (d.fix == FixClass::Auto || (options.unsafe_fixes && d.fix == FixClass::Suggest) || d.note.as_deref() == Some("fix-refused"))
        {
            // L4 decision 5: a fix that can't be verified is reported as
            // `manual` with the rule's steps, and the file keeps the rest.
            d.fix = FixClass::Manual;
            d.edits.clear();
            d.note = Some(f.note().to_owned());
            if d.steps.is_empty() {
                d.steps = project
                    .compiled
                    .iter()
                    .find(|c| c.id() == d.rule)
                    .map(|c| c.loaded.rule.steps.clone())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| vec![format!("Make the change by hand; `codelaw explain {}` shows what the rule requires.", d.rule)]);
            }
        }
    }
    remaining.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let trace = trace.map(|trace| FileTrace {
        file: entry.rel.clone(),
        trace,
        verify_refused: denied.into_iter().collect(),
        formatted,
        changed: fixed != text,
    });
    FileFix {
        file: entry.rel.clone(),
        original: text.to_owned(),
        fixed,
        passes: outcome.passes,
        applied,
        remaining,
        written: false,
        formatted,
        trace,
    }
}

/// Why a fixed file was not written.
#[derive(Debug)]
pub(crate) enum WriteError {
    /// The file changed on disk since it was read (decision 12).
    Changed,
    /// The write itself failed: permissions, a full disk, a missing folder.
    Failed(String),
}

/// Write to a temp file beside the target and rename it into place, unless
/// the file changed since it was read (SR-ARCHITECTURE decision 12). A
/// symlink is written through: the file it points to is replaced, and the
/// link stays a link.
pub(crate) fn safe_write(path: &Path, read_bytes: &[u8], text: &str, bom: bool) -> Result<(), WriteError> {
    use std::io::Write;
    let failed = |e: std::io::Error| WriteError::Failed(e.to_string());
    let target = std::fs::canonicalize(path).map_err(failed)?;
    let dir = target.parent().ok_or_else(|| WriteError::Failed("no parent folder".into()))?;
    let name = target.file_name().ok_or_else(|| WriteError::Failed("no file name".into()))?.to_string_lossy();
    let tmp = dir.join(format!(".{name}.codelaw-{}.tmp", std::process::id()));
    let mut bytes = Vec::with_capacity(text.len() + 3);
    if bom {
        bytes.extend_from_slice("\u{feff}".as_bytes());
    }
    bytes.extend_from_slice(text.as_bytes());
    let cleanup = |e: WriteError| {
        if !tmp.is_dir() {
            let _ = std::fs::remove_file(&tmp);
        }
        e
    };
    let written = std::fs::File::create(&tmp).and_then(|mut f| {
        f.write_all(&bytes)?;
        f.sync_all()
    });
    written.map_err(failed).map_err(cleanup)?;
    if let Ok(meta) = std::fs::metadata(&target) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    // The hash re-check runs last, right before the rename, so the window
    // in which an editor's save could be lost is as small as it can be.
    let now = std::fs::read(&target).map_err(failed).map_err(cleanup)?;
    if hash_bytes(&now) != hash_bytes(read_bytes) {
        return Err(cleanup(WriteError::Changed));
    }
    std::fs::rename(&tmp, &target).map_err(failed).map_err(cleanup)?;
    // The rename is durable once the folder entry is.
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

pub(crate) fn fix_paths(project: &mut Project, paths: &[PathBuf], options: &FixOptions) -> Result<FixReport, EngineError> {
    let (files, mut skipped, warnings) = crate::discover::discover(project, paths)?;
    let this = &*project;
    let results: Vec<(Option<FileFix>, FileResult)> = project.pool.install(|| {
        files
            .par_iter()
            .map(|entry| {
                let file = match this.read_entry(entry) {
                    Ok(f) => f,
                    Err(skip) => return (None, skip),
                };
                let mut fix = fix_text(this, entry, &file.text, options, this.formats(entry), file.bom, Tree::Disk);
                let mut diagnostics = std::mem::take(&mut fix.remaining);
                if fix.changed() && !options.dry_run {
                    match safe_write(&entry.abs, &file.bytes, &fix.fixed, file.bom) {
                        Ok(()) => fix.written = true,
                        Err(why) => {
                            // The disk still holds the original, so report what
                            // the original has, not what the fixes would leave.
                            diagnostics = lint_text(this, entry, &file.text, &RunMode::check().with_bom(file.bom)).diagnostics;
                            let layers = this.config.layers_for(&entry.abs);
                            let (id, message) = match why {
                                WriteError::Changed => {
                                    ("codelaw/file-changed-during-run", "The file changed on disk during the run, so its fixes were not written".to_owned())
                                }
                                WriteError::Failed(e) => ("codelaw/write-failed", format!("The fixed file could not be written, so it is unchanged: {e}")),
                            };
                            diagnostics.extend(meta_diagnostic(&entry.rel, &layers, id, message));
                            diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
                        }
                    }
                }
                if file.bom {
                    crate::lint::shift_for_bom(&mut diagnostics);
                }
                fix.remaining = diagnostics.clone();
                (Some(fix), FileResult { diagnostics, skipped: None })
            })
            .collect()
    });
    let mut report =
        FixReport { check: CheckReport { warnings: project.warnings.clone(), ..Default::default() }, dry_run: options.dry_run, ..Default::default() };
    report.check.warnings.extend(warnings);
    for (fix, result) in results {
        report.check.diagnostics.extend(result.diagnostics);
        match result.skipped {
            Some(s) => skipped.push(s),
            None => report.check.files_checked += 1,
        }
        let Some(mut f) = fix else { continue };
        if let Some(t) = f.trace.take().filter(|t| !t.trace.passes.is_empty() || t.formatted) {
            report.traces.push(t);
        }
        if f.changed() {
            report.files.push(f);
        }
    }
    // The adapters (SR-ARCHITECTURE decision 19) after every file's own
    // fixes, so they see the files codelaw wrote.
    if project.adapters_on() {
        let checked: Vec<&FileEntry> = files.iter().filter(|e| !skipped.iter().any(|s| s.file == e.rel)).collect();
        crate::adapters::fix(project, &checked, options, &mut report);
    }
    if !project.rules.has_rules() {
        report.check.diagnostics.push(crate::lint::no_rules(project));
    }
    let scopes = crate::project::scopes(project.root(), paths);
    let mut sources = None;
    if !project.graph_rules().is_empty() {
        sources = Some(fix_project_rules(project, paths, options, &mut report)?);
    }
    // `graph/no-deprecated-usage` on the fixed texts; its fix is manual.
    let mut deprecated = Vec::new();
    if project.deprecations_on() {
        let sources = match sources {
            Some(s) => s,
            None => {
                let mut s = project.graph_sources()?;
                for f in &report.files {
                    if let Some(x) = s.iter_mut().find(|x| x.rel == f.file) {
                        x.text = f.fixed.clone();
                    }
                }
                s
            }
        };
        let graph = project.build_graph(&sources, project.graph_config()?);
        deprecated = project.deprecated_occurrences(&graph, &sources).into_iter().filter(|o| crate::project::within(&o.diag.file, &scopes)).collect();
    }
    let ran = crate::baseline::Ran {
        files: files.iter().filter(|e| !skipped.iter().any(|s| s.file == e.rel)).map(|e| e.rel.clone()).collect(),
        scopes,
        project: true,
    };
    // `fix` lowers the baseline to what is left: never blocked by
    // bookkeeping (SR-BASELINE decision 5).
    if !options.dry_run && !project.baselines.is_empty() {
        let mut occ: Vec<codelaw_baseline::Occurrence> = report.check.diagnostics.iter().cloned().map(codelaw_baseline::Occurrence::plain).collect();
        occ.extend(deprecated.iter().cloned());
        report.pruned = project.prune_with(&codelaw_baseline::counts(&occ), &ran)?;
    }
    project.apply_baselines(&mut report.check, deprecated, &ran);
    report.check.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    // The same dedupe as `check`, so both report one diagnostic per finding.
    crate::dedup_diagnostics(&mut report.check.diagnostics);
    skipped.sort();
    report.check.skipped = skipped;
    report.traces.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(report)
}

/// Project-rule fixes, after every file's own (SR-PROJECT-GRAPH decision 6):
/// relink, apply, repeat, then report what the graph still finds.
fn fix_project_rules(project: &mut Project, paths: &[PathBuf], options: &FixOptions, report: &mut FixReport) -> Result<Vec<crate::project::Source>, EngineError> {
    let config = project.graph_config()?;
    let mut sources = project.graph_sources()?;
    // A dry run left the disk as it was: start from the fixed texts.
    for f in &report.files {
        if let Some(s) = sources.iter_mut().find(|s| s.rel == f.file) {
            s.text = f.fixed.clone();
        }
    }
    let before: BTreeMap<String, String> = sources.iter().map(|s| (s.rel.clone(), s.text.clone())).collect();
    let scopes = crate::project::scopes(project.root(), paths);
    let applied = project.fix_graph(&mut sources, &config, options.unsafe_fixes, 10);
    for s in &sources {
        let Some(counts) = applied.get(&s.rel) else { continue };
        if !crate::project::within(&s.rel, &scopes) {
            continue;
        }
        let original = report.files.iter().find(|f| f.file == s.rel).map(|f| f.original.clone()).unwrap_or_else(|| before[&s.rel].clone());
        let mut written = false;
        if !options.dry_run
            && let Ok(bytes) = std::fs::read(&s.abs)
        {
            let bom = bytes.starts_with("\u{feff}".as_bytes());
            written = safe_write(&s.abs, &bytes, &s.text, bom).is_ok();
        }
        match report.files.iter_mut().find(|f| f.file == s.rel) {
            Some(f) => {
                f.fixed = s.text.clone();
                f.written |= written;
                for (r, n) in counts {
                    *f.applied.entry(r.clone()).or_default() += n;
                }
            }
            None => report.files.push(FileFix {
                file: s.rel.clone(),
                original,
                fixed: s.text.clone(),
                passes: 1,
                applied: counts.clone(),
                remaining: Vec::new(),
                written,
                formatted: false,
                trace: None,
            }),
        }
    }
    report.files.sort_by(|a, b| a.file.cmp(&b.file));
    let remaining = project.run_graph_rules(&sources, config);
    report.check.diagnostics.extend(remaining.into_iter().filter(|d| crate::project::within(&d.file, &scopes)));
    Ok(sources)
}

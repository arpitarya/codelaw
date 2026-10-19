//! The fix pipeline (SR-ARCHITECTURE decision 5): run every rule, apply
//! the `auto` edits (and `suggest` with `--unsafe`), re-parse, repeat to a
//! fixpoint, verify, and write safely (decision 12).
//!
//! The formatter step of decision 5 is not here yet: SR-FORMATTER's engines
//! are not built in this version.

use crate::discover::FileEntry;
use crate::lint::{FileResult, RunMode, lint_text, meta_diagnostic};
use crate::{CheckReport, EngineError, Project};
use codelaw_cache::hash_bytes;
use codelaw_core::{Diagnostic, FixClass};
use codelaw_fix::{Failure, fix_loop};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default)]
pub struct FixOptions {
    /// `--unsafe`: apply `suggest` fixes too.
    pub unsafe_fixes: bool,
    /// `--dry-run`: compute, don't write.
    pub dry_run: bool,
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
}

impl FileFix {
    pub fn changed(&self) -> bool {
        self.original != self.fixed
    }
}

#[derive(Clone, Debug, Default)]
pub struct FixReport {
    /// Files whose text the fixes changed.
    pub files: Vec<FileFix>,
    pub check: CheckReport,
}

/// Fix one file's text.
pub(crate) fn fix_text(project: &Project, entry: &FileEntry, text: &str, options: &FixOptions) -> FileFix {
    let outcome = fix_loop(entry.language, text, |current, refused| {
        lint_text(project, entry, current, &RunMode { external_fixers: true, refused: Some(refused) })
            .candidates
            .into_iter()
            .filter(|(_, class)| *class == FixClass::Auto || (options.unsafe_fixes && *class == FixClass::Suggest))
            .map(|(c, _)| c)
            .collect()
    });
    let mut remaining = lint_text(project, entry, &outcome.text, &RunMode { external_fixers: false, refused: Some(&outcome.refused) }).diagnostics;
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
    FileFix {
        file: entry.rel.clone(),
        original: text.to_owned(),
        fixed: outcome.text,
        passes: outcome.passes,
        applied: outcome.applied,
        remaining,
        written: false,
    }
}

/// Write to a temp file beside the target and rename it into place, unless
/// the file changed since it was read (SR-ARCHITECTURE decision 12).
fn safe_write(path: &Path, read_bytes: &[u8], text: &str, bom: bool) -> Result<(), String> {
    let now = std::fs::read(path).map_err(|e| e.to_string())?;
    if hash_bytes(&now) != hash_bytes(read_bytes) {
        return Err("changed".into());
    }
    let dir = path.parent().ok_or("no parent folder")?;
    let name = path.file_name().ok_or("no file name")?.to_string_lossy();
    let tmp = dir.join(format!(".{name}.codelaw-{}.tmp", std::process::id()));
    let mut bytes = Vec::with_capacity(text.len() + 3);
    if bom {
        bytes.extend_from_slice("\u{feff}".as_bytes());
    }
    bytes.extend_from_slice(text.as_bytes());
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    if let Ok(meta) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })
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
                let mut fix = fix_text(this, entry, &file.text, options);
                let mut diagnostics = std::mem::take(&mut fix.remaining);
                if fix.changed() && !options.dry_run {
                    match safe_write(&entry.abs, &file.bytes, &fix.fixed, file.bom) {
                        Ok(()) => fix.written = true,
                        Err(why) => {
                            let layers = this.config.layers_for(&entry.abs);
                            let message = if why == "changed" {
                                "The file changed on disk during the run, so its fixes were not written".to_owned()
                            } else {
                                format!("The fixed file could not be written: {why}")
                            };
                            diagnostics.extend(meta_diagnostic(&entry.rel, &layers, "codelaw/file-changed-during-run", message));
                        }
                    }
                }
                fix.remaining = diagnostics.clone();
                (Some(fix), FileResult { diagnostics, skipped: None })
            })
            .collect()
    });
    let mut report = FixReport { check: CheckReport { warnings: project.warnings.clone(), ..Default::default() }, ..Default::default() };
    report.check.warnings.extend(warnings);
    for (fix, result) in results {
        report.check.diagnostics.extend(result.diagnostics);
        match result.skipped {
            Some(s) => skipped.push(s),
            None => report.check.files_checked += 1,
        }
        if let Some(f) = fix.filter(|f| f.changed()) {
            report.files.push(f);
        }
    }
    if !project.rules.has_rules() {
        report.check.diagnostics.push(crate::lint::no_rules(project));
    }
    report.check.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    skipped.sort();
    report.check.skipped = skipped;
    Ok(report)
}

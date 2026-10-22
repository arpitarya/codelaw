//! The format pipeline (SR-FORMATTER decisions 1 and 4): `codelaw format`,
//! the format step at the end of `fix`, and the `format/unformatted`
//! finding of `check`.

use crate::discover::FileEntry;
use crate::fix::{WriteError, safe_write};
use crate::lint::{FileResult, RunMode, lint_text, meta_diagnostic};
use crate::{CheckReport, EngineError, Project};
use codelaw_core::{Diagnostic, LineIndex, TextEdit};
use codelaw_format::editorconfig::{EditorConfig, Properties};
use codelaw_format::{FormatError, FormatOptions};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const UNFORMATTED: &str = "format/unformatted";
pub const FAILED: &str = "format/failed";

/// Each folder's `.editorconfig`, read once (`None`: the folder has none).
type EditorConfigs = Arc<Mutex<HashMap<PathBuf, Option<Arc<EditorConfig>>>>>;

/// The `[format]` settings of a project.
#[derive(Clone, Debug)]
pub struct Formatting {
    /// `[format]` over the defaults, before any `.editorconfig`.
    pub options: FormatOptions,
    /// The keys `[format]` sets, which `.editorconfig` does not change.
    set: BTreeSet<String>,
    exclude: GlobSet,
    /// `false` for a project with no folder on disk (the rule tester's).
    read_editorconfig: bool,
    editorconfigs: EditorConfigs,
}

impl Formatting {
    /// From the root config's `[format]` table.
    pub fn from_config(config: &codelaw_config::Config) -> Result<Formatting, EngineError> {
        let root = config.files.first();
        let table = root.and_then(|f| f.raw.format.as_ref());
        let err = |m: String| EngineError::Config(codelaw_config::ConfigError::new(root.map(|f| f.path.clone()), m));
        let options = FormatOptions::from_table(table).map_err(err)?;
        let mut b = GlobSetBuilder::new();
        for g in &options.exclude {
            b.add(Glob::new(g).map_err(|e| err(format!("[format] exclude: invalid glob `{g}`: {e}")))?);
        }
        let exclude = b.build().map_err(|e| err(format!("[format] exclude: {e}")))?;
        Ok(Formatting {
            options,
            set: FormatOptions::set_keys(table),
            exclude,
            read_editorconfig: config.root_dir.is_dir(),
            editorconfigs: EditorConfigs::default(),
        })
    }

    /// The `.editorconfig` of one folder, read and parsed once per run.
    fn editorconfig(&self, dir: &Path) -> Option<Arc<EditorConfig>> {
        let mut cache = self.editorconfigs.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .entry(dir.to_path_buf())
            .or_insert_with(|| std::fs::read_to_string(dir.join(".editorconfig")).ok().map(|t| Arc::new(EditorConfig::parse(&t))))
            .clone()
    }

    /// The options for one file (SR-FORMATTER decision 5): `.editorconfig`
    /// as the base layer (the file's folder and each one above it, up to
    /// one with `root = true`), and `[format]` over it.
    pub fn options_for(&self, abs: &Path) -> FormatOptions {
        if !self.read_editorconfig {
            return self.options.clone();
        }
        let mut chain = Vec::new();
        for dir in abs.ancestors().skip(1) {
            if let Some(ec) = self.editorconfig(dir) {
                let root = ec.root;
                chain.push((dir, ec));
                if root {
                    break;
                }
            }
        }
        let mut props = Properties::default();
        for (dir, ec) in chain.iter().rev() {
            if let Ok(rel) = abs.strip_prefix(dir) {
                ec.apply_to(&codelaw_core::normalize_path(rel), &mut props);
            }
        }
        if props.is_empty() { self.options.clone() } else { props.apply(&self.options, &self.set) }
    }
}

/// What `codelaw format` takes.
#[derive(Clone, Copy, Debug, Default)]
pub struct FormatRunOptions {
    /// `--check` (and `--diff`): compute, write nothing.
    pub check: bool,
}

/// One file `codelaw format` changed, or would change.
#[derive(Clone, Debug, Default)]
pub struct FormattedFile {
    pub file: String,
    pub original: String,
    pub formatted: String,
    pub written: bool,
}

/// The outcome of `codelaw format`.
#[derive(Clone, Debug, Default)]
pub struct FormatReport {
    /// Files whose layout the formatter changed (or would change), sorted.
    pub files: Vec<FormattedFile>,
    /// Files that could not be formatted or written, as diagnostics; and
    /// the files formatted, skipped and the warnings.
    pub check: CheckReport,
}

impl Project {
    /// Whether this run formats `entry`: `[format] enabled`, not excluded,
    /// and, under `--rule`, only when `format/unformatted` is one of them.
    pub(crate) fn formats(&self, entry: &FileEntry) -> bool {
        self.formatting.options.enabled
            && !self.formatting.exclude.is_match(&entry.rel)
            && self.options.rules.as_ref().is_none_or(|only| only.iter().any(|r| r == UNFORMATTED))
    }

    /// Format one file's text in memory, verified (decision 4); nothing is
    /// written. The text is returned unchanged when the file is excluded.
    pub fn format_source(&self, rel_path: &str, language: codelaw_core::Language, source: &str) -> Result<String, FormatError> {
        let entry = FileEntry { abs: self.config.root_dir.join(rel_path), rel: rel_path.to_owned(), language };
        if !self.formats(&entry) {
            return Ok(source.to_owned());
        }
        verified(self, &entry, source)
    }

    /// Format files and folders (the project root when `paths` is empty)
    /// and write the results, or with `check` only report them.
    pub fn format(&mut self, paths: &[PathBuf], options: &FormatRunOptions) -> Result<FormatReport, EngineError> {
        let (files, mut skipped, warnings) = crate::discover::discover(self, paths)?;
        let this = &*self;
        let results: Vec<(Option<FormattedFile>, FileResult)> = self.pool.install(|| files.par_iter().map(|e| format_file(this, e, options)).collect());
        let mut report = FormatReport { check: CheckReport { warnings: self.warnings.clone(), ..Default::default() }, ..Default::default() };
        report.check.warnings.extend(warnings);
        for (file, result) in results {
            report.check.diagnostics.extend(result.diagnostics);
            match result.skipped {
                Some(s) => skipped.push(s),
                None => report.check.files_checked += 1,
            }
            report.files.extend(file);
        }
        report.check.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        skipped.sort();
        report.check.skipped = skipped;
        Ok(report)
    }
}

fn format_file(project: &Project, entry: &FileEntry, options: &FormatRunOptions) -> (Option<FormattedFile>, FileResult) {
    let file = match project.read_entry(entry) {
        Ok(f) => f,
        Err(skip) => return (None, skip),
    };
    if !project.formats(entry) {
        return (None, FileResult::default());
    }
    let formatted = match verified(project, entry, &file.text) {
        Ok(f) => f,
        Err(e) => return (None, FileResult { diagnostics: failed(project, entry, &e).into_iter().collect(), skipped: None }),
    };
    if formatted == file.text {
        return (None, FileResult::default());
    }
    let mut out = FormattedFile { file: entry.rel.clone(), original: file.text.clone(), formatted, written: false };
    let mut diagnostics = Vec::new();
    if !options.check {
        match safe_write(&entry.abs, &file.bytes, &out.formatted, file.bom) {
            Ok(()) => out.written = true,
            Err(why) => {
                let layers = project.config.layers_for(&entry.abs);
                let (id, message) = match why {
                    WriteError::Changed => ("codelaw/file-changed-during-run", "The file changed on disk during the run, so it was not formatted".to_owned()),
                    WriteError::Failed(e) => ("codelaw/write-failed", format!("The formatted file could not be written, so it is unchanged: {e}")),
                };
                diagnostics.extend(meta_diagnostic(&entry.rel, &layers, id, message));
            }
        }
    }
    (Some(out), FileResult { diagnostics, skipped: None })
}

/// The rules that fire on `text`, by id.
fn rules_firing(project: &Project, entry: &FileEntry, text: &str) -> BTreeSet<String> {
    lint_text(project, entry, text, &RunMode::check()).diagnostics.into_iter().map(|d| d.rule).collect()
}

/// Format with the engine's own checks, then the lint check of decision 4:
/// no rule fires on the result that did not fire before.
pub(crate) fn verified(project: &Project, entry: &FileEntry, text: &str) -> Result<String, FormatError> {
    let options = project.formatting.options_for(&entry.abs);
    let out = {
        let _timed = project.timings.as_ref().map(|t| t.guard(crate::timings::Phase::Format));
        codelaw_format::format(entry.language, &entry.rel, text, &options)?
    };
    if out != text && project.rules.has_rules() {
        let before = rules_firing(project, entry, text);
        let new: Vec<String> = rules_firing(project, entry, &out).difference(&before).cloned().collect();
        if !new.is_empty() {
            return Err(FormatError::Verify(format!("the formatted file makes {} fire", new.join(", "))));
        }
    }
    Ok(out)
}

/// `format/failed`: why the file was left as it was.
pub(crate) fn failed(project: &Project, entry: &FileEntry, e: &FormatError) -> Option<Diagnostic> {
    let layers = project.config.layers_for(&entry.abs);
    let mut d = meta_diagnostic(
        &entry.rel,
        &layers,
        FAILED,
        format!("The file was not formatted, because {e} ({})", codelaw_format::engine(entry.language)),
    )?;
    d.note = Some(
        match e {
            FormatError::Syntax(_) => "format-syntax",
            FormatError::Engine(_) => "format-engine",
            FormatError::Verify(_) => "format-verify-failed",
        }
        .into(),
    );
    Some(d)
}

/// The one edit that turns `text` into `formatted`: the differing middle.
fn middle_edit(text: &str, formatted: &str) -> TextEdit {
    let mut start = text.bytes().zip(formatted.bytes()).take_while(|(a, b)| a == b).count();
    while !text.is_char_boundary(start) || !formatted.is_char_boundary(start) {
        start -= 1;
    }
    let max_suffix = text.len().min(formatted.len()) - start;
    let mut suffix = text.bytes().rev().zip(formatted.bytes().rev()).take(max_suffix).take_while(|(a, b)| a == b).count();
    while !text.is_char_boundary(text.len() - suffix) || !formatted.is_char_boundary(formatted.len() - suffix) {
        suffix -= 1;
    }
    TextEdit::new(start, text.len() - suffix, &formatted[start..formatted.len() - suffix])
}

/// `check`'s finding for one file (decision 1): `format/unformatted` with
/// the `auto` edit that formats it, or `format/failed`.
pub(crate) fn check_diagnostic(project: &Project, entry: &FileEntry, text: &str) -> Option<Diagnostic> {
    if !project.formatting.options.check || !project.formats(entry) {
        return None;
    }
    let formatted = match verified(project, entry, text) {
        Ok(f) => f,
        Err(e) => return failed(project, entry, &e),
    };
    if formatted == text {
        return None;
    }
    let layers = project.config.layers_for(&entry.abs);
    let edit = middle_edit(text, &formatted);
    let index = LineIndex::new(text);
    let mut d = meta_diagnostic(&entry.rel, &layers, UNFORMATTED, "The file is not formatted; `codelaw format` formats it".into())?;
    d.start_byte = edit.start;
    d.end_byte = edit.end;
    d.start = index.position(edit.start);
    d.end = index.position(edit.end);
    d.edits = vec![edit];
    Some(d)
}

#[cfg(test)]
mod tests {
    use super::middle_edit;

    fn apply(text: &str, formatted: &str) -> String {
        let e = middle_edit(text, formatted);
        format!("{}{}{}", &text[..e.start], e.text, &text[e.end..])
    }

    #[test]
    fn the_edit_is_the_differing_middle() {
        for (a, b) in [("a=1\n", "a = 1\n"), ("x\n", "x\n\n"), ("ab", "b"), ("é=1", "é = 1"), ("", "x\n"), ("aaa", "aa")] {
            assert_eq!(apply(a, b), b, "{a:?} -> {b:?}");
        }
        let e = middle_edit("const a=1;\n", "const a = 1;\n");
        assert_eq!((e.start, e.end, e.text.as_str()), (7, 8, " = "));
    }
}

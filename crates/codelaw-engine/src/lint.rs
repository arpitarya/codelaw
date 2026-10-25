//! One file through the check pipeline (SR-ARCHITECTURE decision 4.3):
//! read once, parse once, split embedded regions, build the semantic model
//! if a rule needs it, walk once dispatching each node to the rules whose
//! kinds match, then suppressions.

use crate::compile::{CompiledRule, Matcher};
use crate::discover::{FileEntry, language_enabled};
use crate::files::{Asked, ProjectFile, Tree};
use crate::timings::Phase;
use crate::{Project, suppress};
use codelaw_baseline::exception::{Expiry, expiry};
use codelaw_baseline::{AUTHOR_RULE, EXPIRED_RULE, EXPIRING_RULE, WITHOUT_EXPIRY_RULE};
use codelaw_cache::KeyBuilder;
use codelaw_config::{Layer, RuleConfig};
use codelaw_core::{Diagnostic, FixClass, Language, LineIndex, Severity, Skipped, TextEdit};
use codelaw_fix::{Candidate, DataFixContext, ExternalCall, FixContext, FixSpec, data_edits, edits_for, group_edits, run_external};
use codelaw_lang::{Grammar, Region, RegionOptions, Root};
use codelaw_match::{Match, interpolate};
use codelaw_native::{NativeContext, NativeFinding};
use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

/// The diagnostics of one file, or why it was skipped.
#[derive(Clone, Debug, Default)]
pub struct FileResult {
    pub diagnostics: Vec<Diagnostic>,
    pub skipped: Option<Skipped>,
}

/// How a lint pass is run.
pub(crate) struct RunMode<'a> {
    /// Run external fixers to compute their edits (fix mode only).
    pub external_fixers: bool,
    /// Fix candidates the fix loop refused: reported as `manual`.
    pub refused: Option<&'a BTreeSet<String>>,
    /// The file on disk starts with a byte-order mark, which the text does
    /// not hold (native rules see it in the bytes, decision 15b).
    pub bom: bool,
    /// Where native rules see the files around this one.
    pub tree: Tree<'a>,
}

impl RunMode<'static> {
    pub fn check() -> Self {
        RunMode { external_fixers: false, refused: None, bom: false, tree: Tree::Disk }
    }
}

impl RunMode<'_> {
    /// The same pass for a file with a byte-order mark, or without.
    pub fn with_bom(self, bom: bool) -> Self {
        RunMode { bom, ..self }
    }
}

/// What lint_text hands the fix loop besides diagnostics.
pub(crate) struct LintOutput {
    pub diagnostics: Vec<Diagnostic>,
    /// Fixable diagnostics as candidates, in diagnostic order.
    pub candidates: Vec<(Candidate, FixClass)>,
    /// What native rules asked about other files: a cached result holds
    /// only while every answer does.
    pub asked: Vec<Asked>,
}

/// One file's cached lint result.
#[derive(serde::Serialize, serde::Deserialize)]
struct CachedFile {
    diagnostics: Vec<Diagnostic>,
    asked: Vec<Asked>,
}

/// A raw finding in host coordinates, before it becomes a diagnostic.
struct Finding {
    rule: usize,
    start: usize,
    end: usize,
    message: String,
    edits: Option<Vec<TextEdit>>,
    note: Option<String>,
}

const FALLBACK_STEP: &str = "Make the change by hand; `codelaw explain <rule>` shows what the rule requires.";

/// An engine diagnostic's severity for one file: its default, then the
/// config layers; `off` is `None`.
pub(crate) fn meta_severity(layers: &[Layer], id: &str) -> Option<Severity> {
    let rule = codelaw_rules::engine_rule(id)?;
    let mut severity = Some(rule.severity);
    for l in layers {
        if let Some(s) = l.rules.get(id).and_then(|s| s.severity()) {
            severity = Severity::parse(s);
        }
    }
    severity
}

pub(crate) fn meta_diagnostic(entry_rel: &str, layers: &[Layer], id: &str, message: String) -> Option<Diagnostic> {
    let rule = codelaw_rules::engine_rule(id)?;
    let severity = meta_severity(layers, id)?;
    let pos = codelaw_core::Position { line: 1, column: 1 };
    Some(Diagnostic {
        rule: id.to_owned(),
        severity,
        file: entry_rel.to_owned(),
        start_byte: 0,
        end_byte: 0,
        start: pos,
        end: pos,
        message,
        fix: rule.fix,
        edits: Vec::new(),
        steps: rule.steps.iter().map(|s| s.to_string()).collect(),
        source: "engine".into(),
        note: None,
    })
}

/// `codelaw/no-rules`, once per run (SR-RULE-PACKS decision 6).
pub(crate) fn no_rules(project: &Project) -> Diagnostic {
    let file = project.config.root_file().map(|f| crate::discover::rel_path(f, project.root())).unwrap_or_else(|| ".".into());
    meta_diagnostic(&file, &[], "codelaw/no-rules", "No rule pack and no local rule is configured, so nothing was linted; run `codelaw init`".into())
        .expect("an engine rule")
}

/// A file as read: its text without a BOM, and the raw bytes.
pub(crate) struct ReadFile {
    pub text: String,
    pub bom: bool,
    pub bytes: Vec<u8>,
}

impl Project {
    /// Read a file, or say why it is skipped (SR-ARCHITECTURE decisions 8
    /// and 10).
    pub(crate) fn read_entry(&self, entry: &FileEntry) -> Result<ReadFile, FileResult> {
        let _timed = self.timings.as_ref().map(|t| t.guard(Phase::Read));
        let layers = self.config.layers_for(&entry.abs);
        let skip = |id: &str, reason: String| FileResult {
            diagnostics: meta_diagnostic(&entry.rel, &layers, id, reason.clone()).into_iter().collect(),
            skipped: Some(Skipped { file: entry.rel.clone(), reason }),
        };
        let unreadable = |e: std::io::Error| FileResult { diagnostics: vec![], skipped: Some(Skipped { file: entry.rel.clone(), reason: format!("unreadable: {e}") }) };
        let meta = std::fs::metadata(&entry.abs).map_err(unreadable)?;
        let max = self.config.max_file_size();
        if meta.len() > max {
            return Err(skip("codelaw/file-too-large", format!("larger than `[files] max_file_size` ({} > {max} bytes)", meta.len())));
        }
        let bytes = std::fs::read(&entry.abs).map_err(unreadable)?;
        let Ok(text) = std::str::from_utf8(&bytes) else {
            return Err(skip("codelaw/not-utf8", "not valid UTF-8".into()));
        };
        let (text, bom) = match text.strip_prefix('\u{feff}') {
            Some(t) => (t.to_owned(), true),
            None => (text.to_owned(), false),
        };
        if let Some(reason) = self.generated(&text) {
            return Err(skip("codelaw/generated", reason));
        }
        Ok(ReadFile { text, bom, bytes })
    }

    /// Why a file's text is skipped as generated, if it is (decision 10).
    fn generated(&self, text: &str) -> Option<String> {
        if self.config.lint_generated() {
            return None;
        }
        let mut end = text.len().min(1024);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let head = &text[..end];
        let marker = self.config.generated_markers().into_iter().find(|m| head.contains(m.as_str()))?;
        Some(format!("marked generated (`{marker}`); set `[files] lint_generated = true` to check it"))
    }

    /// An editor's text of a file, through the same skips `check_file_asked`
    /// makes on the file on disk (SR-EDITOR decision 3).
    pub(crate) fn check_unsaved(&self, entry: &FileEntry, text: &str, host: Option<&Root>) -> FileResult {
        let layers = self.config.layers_for(&entry.abs);
        let skip = |id: &str, reason: String| FileResult {
            diagnostics: meta_diagnostic(&entry.rel, &layers, id, reason.clone()).into_iter().collect(),
            skipped: Some(Skipped { file: entry.rel.clone(), reason }),
        };
        let (len, max) = (text.len() as u64, self.config.max_file_size());
        if len > max {
            return skip("codelaw/file-too-large", format!("larger than `[files] max_file_size` ({len} > {max} bytes)"));
        }
        if let Some(reason) = self.generated(text) {
            return skip("codelaw/generated", reason);
        }
        let mut diagnostics = lint_text_with(self, entry, text, &RunMode::check(), host).diagnostics;
        diagnostics.extend(crate::format::check_diagnostic(self, entry, text));
        FileResult { diagnostics, skipped: None }
    }

    /// One file of a run, with the questions its rules asked about other
    /// files: a watch re-checks a file whose answers no longer hold (W-44).
    pub(crate) fn check_file_asked(&self, entry: &FileEntry) -> (FileResult, Vec<Asked>) {
        let file = match self.read_entry(entry) {
            Ok(f) => f,
            Err(skipped) => return (skipped, Vec::new()),
        };
        let key = KeyBuilder::new("file+asked")
            .part(&entry.rel)
            .part(if file.bom { "bom" } else { "" })
            .part(entry.language.name())
            .part(&file.text)
            .part(self.rules.fingerprint.0)
            .part(self.config.fingerprint().0)
            .part(codelaw_lang::GRAMMARS_FINGERPRINT)
            .part(codelaw_format::ENGINES_FINGERPRINT)
            // The file's format options: `.editorconfig` is not in the
            // config fingerprint.
            .part(format!("{:?}", self.formatting.options_for(&entry.abs)))
            .part(format!("{:?}{:?}", self.options.languages, self.options.rules))
            // A dated `codelaw-ignore` is judged on the as-of date.
            .part(self.exception_key(&file.text))
            .key();
        // A result that read other files holds only while their answers do.
        let cached = self.cache.get::<CachedFile>(&key).filter(|c| crate::files::still_holds(self.root(), &c.asked));
        if let Some(t) = &self.timings {
            t.cache(cached.is_some());
        }
        let (mut diagnostics, asked) = match cached {
            Some(c) => (c.diagnostics, c.asked),
            None => {
                let mut out = lint_text(self, entry, &file.text, &RunMode::check().with_bom(file.bom));
                out.diagnostics.extend(crate::format::check_diagnostic(self, entry, &file.text));
                if !out.diagnostics.iter().any(|d| d.rule == "codelaw/internal-error") {
                    self.cache.put(&key, &CachedFile { diagnostics: out.diagnostics.clone(), asked: out.asked.clone() });
                }
                (out.diagnostics, out.asked)
            }
        };
        if file.bom {
            shift_for_bom(&mut diagnostics);
        }
        (FileResult { diagnostics, skipped: None }, asked)
    }
}

/// Rules see the text without its BOM; reported byte offsets are into the
/// file as it is on disk, so they move past the BOM's three bytes. A
/// whole-file engine diagnostic (no range, no edit) stays at 0.
pub(crate) fn shift_for_bom(diagnostics: &mut [Diagnostic]) {
    const BOM: usize = '\u{feff}'.len_utf8();
    for d in diagnostics {
        if d.start_byte == 0 && d.end_byte == 0 && d.edits.is_empty() && d.source == "engine" {
            continue;
        }
        d.start_byte += BOM;
        d.end_byte += BOM;
        for e in &mut d.edits {
            e.start += BOM;
            e.end += BOM;
        }
    }
}

/// One unit of a file: the host or an embedded region, with its rules.
struct Unit<'p> {
    language: Language,
    grammar: Grammar,
    root: Root,
    text: String,
    region: Option<Region>,
    rules: Vec<(usize, &'p CompiledRule, RuleConfig)>,
}

fn dominant_crlf(text: &str) -> bool {
    let crlf = text.matches("\r\n").count();
    crlf > 0 && crlf * 2 >= text.matches('\n').count()
}

/// Edits keep the file's line ending (SR-ARCHITECTURE decision 9).
fn with_line_endings(mut edits: Vec<TextEdit>, crlf: bool) -> Vec<TextEdit> {
    if crlf {
        for e in &mut edits {
            if e.text.contains('\n') {
                e.text = e.text.replace("\r\n", "\n").replace('\n', "\r\n");
            }
        }
    }
    edits
}

/// What a project extracts as embedded regions (SR-LINTER decision 3).
pub(crate) fn region_options(project: &Project) -> RegionOptions {
    RegionOptions { lint_code_blocks: project.config.lint_code_blocks(), tagged_templates: project.config.tagged_templates() }
}

/// Whether a host language can hold regions under these options.
pub(crate) fn has_regions(language: Language, opts: RegionOptions) -> bool {
    match language {
        Language::Html | Language::Markdown => true,
        Language::Javascript | Language::Jsx | Language::Typescript | Language::Tsx => opts.tagged_templates,
        _ => false,
    }
}

pub(crate) fn comment_ranges(root: &Root, text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for n in root.root().dfs() {
        let kind = n.kind();
        let r = n.range();
        if kind.contains("comment") || (kind == "html_block" && text[r.clone()].trim_start().starts_with("<!--")) {
            out.push((r.start, r.end));
        }
    }
    out
}

/// Lint one file's text.
pub(crate) fn lint_text(project: &Project, entry: &FileEntry, text: &str, mode: &RunMode<'_>) -> LintOutput {
    lint_text_with(project, entry, text, mode, None)
}

/// The host tree of `text`: a warm process's tree of it when it has one
/// (SR-ARCHITECTURE decision 23), otherwise a fresh parse. A tree of any
/// other text or grammar is never used, so the result is the same either way.
fn host_tree(entry: &FileEntry, text: &str, host: Option<&Root>) -> Root {
    match host {
        Some(root) if *root.lang() == Grammar::for_language(entry.language) && codelaw_lang::text_of(root) == text => root.clone(),
        _ => codelaw_lang::parse(entry.language, text),
    }
}

/// [`lint_text`], reusing `host` as the parse of `text` when it is one.
pub(crate) fn lint_text_with(project: &Project, entry: &FileEntry, text: &str, mode: &RunMode<'_>, host: Option<&Root>) -> LintOutput {
    let layers = project.config.layers_for(&entry.abs);
    let only = project.options.rules.as_ref();
    let active: Vec<(usize, &CompiledRule, RuleConfig)> = project
        .compiled
        .iter()
        .enumerate()
        .filter(|(_, r)| only.is_none_or(|o| o.iter().any(|id| id == r.id())))
        .filter(|(_, r)| !matches!(r.matcher, Matcher::Unsupported(_) | Matcher::Graph(_)))
        .filter(|(_, r)| r.applies_to_path(&entry.rel))
        .filter_map(|(i, r)| project.rules.effective(&r.loaded, &layers).filter(|c| c.severity.is_some()).map(|c| (i, r, c)))
        .collect();

    let host_rules: Vec<_> = active.iter().filter(|(_, r, _)| r.applies_to_language(entry.language) && r.loaded.rule.host.is_none()).cloned().collect();
    let opts = region_options(project);
    let may_have_regions = has_regions(entry.language, opts);
    let region_rules_exist = may_have_regions
        && active.iter().any(|(_, r, _)| {
            r.loaded.rule.languages.iter().any(|l| *l != entry.language)
                || (entry.language == Language::Markdown && r.applies_to_language(Language::Markdown) && r.kinds_for(Grammar::MarkdownInline).is_some())
        });
    if host_rules.is_empty() && !region_rules_exist {
        // No rule runs here, but a suppression still has to name a rule and
        // give a reason (SR-LINTER decision 6).
        let mut diagnostics = Vec::new();
        if text.contains(suppress::MARKER) {
            let root = host_tree(entry, text, host);
            let comments = comment_ranges(&root, text);
            let index = LineIndex::new(text);
            apply_suppressions(&entry.rel, &layers, text, &index, &comments, project, &[], &mut diagnostics);
        }
        return LintOutput { diagnostics, candidates: Vec::new(), asked: Vec::new() };
    }

    let host_root = {
        let _timed = project.timings.as_ref().map(|t| t.guard(Phase::Parse));
        host_tree(entry, text, host)
    };
    let mut units: Vec<Unit> = Vec::new();
    if region_rules_exist {
        for region in codelaw_lang::regions(entry.language, &host_root, text, opts) {
            if !language_enabled(project, &entry.abs, region.language) {
                continue;
            }
            // A `target:` rule's paths are about a whole file; inside another
            // file's region (front matter, a code block) it runs only where
            // its `host:` names that file's language (SR-RULE-FORMAT 3e).
            let rules: Vec<_> = active
                .iter()
                .filter(|(_, r, _)| r.applies_to_language(region.language) && r.loaded.rule.host.is_none_or(|h| h == entry.language))
                .filter(|(_, r, _)| !matches!(r.matcher, Matcher::Target(_)) || r.loaded.rule.host.is_some())
                .cloned()
                .collect();
            if rules.is_empty() {
                continue;
            }
            let rules: Vec<_> = rules.into_iter().filter(|(_, r, _)| r.kinds_for(region.grammar).is_some()).collect();
            if rules.is_empty() {
                continue;
            }
            let root = {
                let _timed = project.timings.as_ref().map(|t| t.guard(Phase::Parse));
                codelaw_lang::parse_grammar(region.grammar, &region.text)
            };
            units.push(Unit { language: region.language, grammar: region.grammar, root, text: region.text.clone(), region: Some(region), rules });
        }
    }
    let host_text = text.to_owned();
    units.insert(0, Unit { language: entry.language, grammar: Grammar::for_language(entry.language), root: host_root, text: host_text, region: None, rules: host_rules });

    let crlf = dominant_crlf(text);
    // Every unit, regions included, sees the host file (decision 15b).
    let file = ProjectFile::new(&entry.rel, text, mode.bom, project.root(), mode.tree);
    let mut findings: Vec<Finding> = Vec::new();
    let mut comments: Vec<(usize, usize)> = Vec::new();
    for unit in &units {
        let unit_findings = run_unit(project, entry, unit, mode, &file);
        // A region's offsets map back to the host; nothing inside a hole
        // (a template's `${…}`) does, and no edit may reach into one.
        let span = |s: usize, e: usize| match &unit.region {
            Some(r) => r.span_to_host(s, e),
            None => Some((s, e)),
        };
        let edit_span = |s: usize, e: usize| match &unit.region {
            Some(r) => r.edit_to_host(s, e),
            None => Some((s, e)),
        };
        for c in comment_ranges(&unit.root, &unit.text) {
            if let Some(m) = span(c.0, c.1) {
                comments.push(m);
            }
        }
        for mut f in unit_findings {
            let Some((s, e)) = span(f.start, f.end) else { continue };
            f.start = s;
            f.end = e;
            if let Some(edits) = f.edits.take() {
                let mapped: Option<Vec<TextEdit>> =
                    edits.into_iter().map(|ed| edit_span(ed.start, ed.end).map(|(s, e)| TextEdit::new(s, e, ed.text))).collect();
                match mapped {
                    Some(m) => f.edits = Some(with_line_endings(m, crlf)),
                    None => f.note = Some("fix-outside-region".into()),
                }
            }
            findings.push(f);
        }
    }
    comments.sort();
    comments.dedup();

    let index = LineIndex::new(text);
    let mut diagnostics = Vec::new();
    // A rule that failed did not run, so a suppression of it is not unused.
    let failed: BTreeSet<&str> =
        findings.iter().filter(|f| f.note.as_deref() == Some(INTERNAL_ERROR)).map(|f| project.compiled[f.rule].id()).collect();
    let ran: Vec<(usize, &CompiledRule, RuleConfig)> = active.iter().filter(|(_, r, _)| !failed.contains(r.id())).cloned().collect();
    for f in findings {
        let rule = &project.compiled[f.rule];
        if f.note.as_deref() == Some(INTERNAL_ERROR) {
            // Decision 25: a rule failure is the engine's own diagnostic,
            // never the rule's, so no severity, suppression or cache hides it.
            if let Some(mut d) = meta_diagnostic(&entry.rel, &layers, "codelaw/internal-error", f.message) {
                d.start_byte = f.start;
                d.end_byte = f.end;
                d.start = index.position(f.start);
                d.end = index.position(f.end);
                diagnostics.push(d);
            }
            continue;
        }
        let cfg = active.iter().find(|(i, _, _)| *i == f.rule).map(|(_, _, c)| c).expect("an active rule");
        let declared = rule.loaded.rule.fix;
        let external = matches!(rule.fix, Some(FixSpec::External(_)));
        let mut note = f.note;
        let mut edits = Vec::new();
        let class = match f.edits.filter(|e| !e.is_empty()) {
            _ if declared == FixClass::Manual => FixClass::Manual,
            Some(e) => {
                let key = Candidate { rule: rule.id().to_owned(), edits: e.clone() }.key(text);
                if mode.refused.is_some_and(|r| r.contains(&key)) {
                    note = note.or(Some("fix-refused".into()));
                    FixClass::Manual
                } else {
                    edits = e;
                    declared
                }
            }
            // An external fixer runs in `codelaw fix`; `check` names it.
            None if external && !mode.external_fixers && note.is_none() => {
                note = Some("external-fixer-runs-in-fix".into());
                declared
            }
            None => {
                note.get_or_insert_with(|| "fix-unavailable".into());
                FixClass::Manual
            }
        };
        let steps = if class == FixClass::Manual {
            if rule.loaded.rule.steps.is_empty() { vec![FALLBACK_STEP.replace("<rule>", rule.id())] } else { rule.loaded.rule.steps.clone() }
        } else {
            Vec::new()
        };
        diagnostics.push(Diagnostic {
            rule: rule.id().to_owned(),
            severity: cfg.severity.expect("active"),
            file: entry.rel.clone(),
            start_byte: f.start,
            end_byte: f.end,
            start: index.position(f.start),
            end: index.position(f.end),
            message: f.message,
            fix: class,
            edits,
            steps,
            source: rule.loaded.source.label(),
            note,
        });
    }

    {
        let _timed = project.timings.as_ref().map(|t| t.guard(Phase::Suppress));
        apply_suppressions(&entry.rel, &layers, text, &index, &comments, project, &ran, &mut diagnostics);
    }
    let candidates = diagnostics
        .iter()
        .filter(|d| d.fix != FixClass::Manual && !d.edits.is_empty())
        .map(|d| (Candidate { rule: d.rule.clone(), edits: d.edits.clone() }, d.fix))
        .collect();
    LintOutput { diagnostics, candidates, asked: file.asked() }
}
#[allow(clippy::too_many_arguments)]
fn apply_suppressions(
    rel: &str,
    layers: &[Layer],
    text: &str,
    index: &LineIndex<'_>,
    comments: &[(usize, usize)],
    project: &Project,
    active: &[(usize, &CompiledRule, RuleConfig)],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut sups = suppress::find(text, comments, index);
    if sups.is_empty() {
        return;
    }
    // Dated exceptions (SR-BASELINE decision 9): the as-of date is read
    // only when a comment carries a date; an expired one covers nothing.
    let exceptions = project.exceptions;
    let as_of = if sups.iter().any(|s| s.dated.expires.is_some()) { project.as_of().date } else { None };
    let states: Vec<Expiry> = sups.iter().map(|s| expiry(as_of, s.dated.expires, None, s.dated.warn_days.unwrap_or(exceptions.warn_days))).collect();
    diagnostics.retain(|d| {
        if d.rule.starts_with("codelaw/") {
            return true;
        }
        for (i, s) in sups.iter_mut().enumerate() {
            if s.dated.bad.is_some() || matches!(states[i], Expiry::Expired { .. }) {
                continue;
            }
            if s.reason.is_some() && s.target_line == d.start.line && s.rules.contains(&d.rule) {
                s.used = true;
                return false;
            }
        }
        true
    });
    for (s, state) in sups.iter().zip(&states) {
        let meta = |id: &str, message: String, fix: Option<TextEdit>| -> Option<Diagnostic> {
            let mut d = meta_diagnostic(rel, layers, id, message)?;
            d.start_byte = s.start;
            d.end_byte = s.end;
            d.start = index.position(s.start);
            d.end = index.position(s.end);
            if let Some(e) = fix {
                d.edits = vec![e];
                d.steps.clear();
            }
            Some(d)
        };
        let unknown: Vec<&String> = s
            .rules
            .iter()
            .filter(|r| project.rules.get(r).is_none() && codelaw_rules::engine_rule(r).is_none())
            .collect();
        if s.rules.is_empty() || !unknown.is_empty() {
            let what = if s.rules.is_empty() { "names no rule".to_owned() } else { format!("names unknown rule `{}`", unknown[0]) };
            diagnostics.extend(meta("codelaw/unknown-suppression", format!("This `codelaw-ignore` {what}"), None));
            continue;
        }
        if s.reason.is_none() {
            diagnostics.extend(meta(
                "codelaw/suppression-needs-reason",
                "A `codelaw-ignore` needs a reason: `codelaw-ignore <rule>: <reason>`".into(),
                None,
            ));
            continue;
        }
        if let Some(bad) = &s.dated.bad {
            diagnostics.extend(meta("codelaw/unknown-suppression", format!("This `codelaw-ignore` cannot be read, so it suppresses nothing: {bad}"), None));
            continue;
        }
        let ids = s.rules.join(", ");
        if let Some(by) = s.dated.by.as_deref().filter(|b| !codelaw_baseline::exception::valid_email(b)) {
            diagnostics.extend(meta(AUTHOR_RULE, format!("This `codelaw-ignore {ids}` has `by={by}`, which is not an email address"), None));
        }
        if exceptions.require_expiry && s.dated.expires.is_none() {
            diagnostics.extend(meta(
                WITHOUT_EXPIRY_RULE,
                format!("This `codelaw-ignore {ids}` has no `expires=`, and `[exceptions] require_expiry` is on"),
                None,
            ));
        }
        match *state {
            Expiry::Expired { expires } => {
                diagnostics.extend(meta(EXPIRED_RULE, format!("This `codelaw-ignore {ids}` expired on {expires}: it no longer suppresses anything"), None));
                continue;
            }
            Expiry::Expiring { days, expires } => {
                diagnostics.extend(meta(EXPIRING_RULE, format!("This `codelaw-ignore {ids}` expires {}", codelaw_baseline::exception::when(days, expires)), None));
            }
            Expiry::Holds => {}
        }
        let all_ran = s.rules.iter().all(|r| active.iter().any(|(_, a, _)| a.id() == r));
        if !s.used && all_ran {
            let (a, b) = suppress::removal(text, s);
            diagnostics.extend(meta(
                "codelaw/unused-suppression",
                format!("This `codelaw-ignore {}` suppresses nothing", s.rules.join(", ")),
                Some(TextEdit::delete(a, b)),
            ));
        }
    }
}

/// The note that marks a finding as a rule failure (decision 25).
const INTERNAL_ERROR: &str = "internal-error";

fn internal_error(rule: usize, start: usize, end: usize, id: &str, what: &str) -> Finding {
    Finding { rule, start, end, message: format!("rule `{id}` failed on this file: {what}"), edits: None, note: Some(INTERNAL_ERROR.into()) }
}

fn panic_message(p: &(dyn std::any::Any + Send)) -> String {
    p.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| p.downcast_ref::<String>().cloned()).unwrap_or_else(|| "panic".into())
}

/// Walk one unit once, dispatching each node to the rules whose kinds
/// match (decision 15); a rule that panics is isolated (decision 25).
fn run_unit(project: &Project, entry: &FileEntry, unit: &Unit<'_>, mode: &RunMode<'_>, file: &ProjectFile<'_>) -> Vec<Finding> {
    if unit.rules.is_empty() {
        return Vec::new();
    }
    let grammar = unit.grammar;
    let n = unit.rules.len();
    let mut failed: Vec<Option<String>> = vec![None; n];
    let timings = project.timings.as_ref();
    let semantic = if unit.rules.iter().any(|(_, r, _)| r.needs_semantic) {
        let _timed = timings.map(|t| t.guard(Phase::Semantic));
        match catch_unwind(AssertUnwindSafe(|| codelaw_semantic::build(unit.language, &unit.root, &unit.text))) {
            Ok(model) => model,
            // A model that panics fails every rule that needs it (decision 25);
            // the other rules still run.
            Err(p) => {
                let why = format!("the semantic model failed: {}", panic_message(p.as_ref()));
                for (slot, (_, r, _)) in unit.rules.iter().enumerate() {
                    if r.needs_semantic {
                        failed[slot] = Some(why.clone());
                    }
                }
                None
            }
        }
    } else {
        None
    };
    let _timed = timings.map(|t| t.guard(Phase::Rules));
    let mut by_kind: Vec<Vec<usize>> = Vec::new();
    let mut any_kind: Vec<usize> = Vec::new();
    for (slot, (_, rule, _)) in unit.rules.iter().enumerate() {
        match rule.kinds_for(grammar) {
            None => {}
            Some(None) => any_kind.push(slot),
            Some(Some(kinds)) => {
                for k in kinds {
                    let k = k as usize;
                    if by_kind.len() <= k {
                        by_kind.resize(k + 1, Vec::new());
                    }
                    by_kind[k].push(slot);
                }
            }
        }
    }
    let mut matches: Vec<Vec<Match>> = vec![Vec::new(); n];
    let mut native: Vec<Vec<NativeFinding>> = vec![Vec::new(); n];
    let empty = std::collections::BTreeMap::new();
    for node in unit.root.root().dfs() {
        let k = node.kind_id() as usize;
        let mut slots: Vec<usize> = by_kind.get(k).cloned().unwrap_or_default();
        if !any_kind.is_empty() {
            slots.extend(&any_kind);
            slots.sort_unstable();
        }
        for slot in slots {
            if failed[slot].is_some() {
                continue;
            }
            let (index, rule, cfg) = &unit.rules[slot];
            let started = timings.map(|_| Instant::now());
            match &rule.matcher {
                Matcher::Match(_) => {
                    let cm = rule.compiled_match(grammar).expect("dispatched for this grammar");
                    match catch_unwind(AssertUnwindSafe(|| cm.try_match(node.clone(), semantic.as_ref()))) {
                        Ok(Some(m)) => {
                            if !matches[slot].iter().any(|x| x.start == m.start && x.end == m.end) {
                                matches[slot].push(m);
                            }
                        }
                        Ok(None) => {}
                        Err(p) => failed[slot] = Some(panic_message(p.as_ref())),
                    }
                }
                Matcher::Native(nr) => {
                    let options = if cfg.options.is_empty() { &empty } else { &cfg.options };
                    let ctx = NativeContext { source: &unit.text, language: unit.language, semantic: semantic.as_ref(), options, data: &rule.loaded.data, file };
                    let out = &mut native[slot];
                    if let Err(p) = catch_unwind(AssertUnwindSafe(|| nr.check(node.clone(), &ctx, out))) {
                        failed[slot] = Some(panic_message(p.as_ref()));
                    }
                }
                Matcher::Graph(_) | Matcher::Target(_) | Matcher::Unsupported(_) => {}
            }
            if let (Some(t), Some(s)) = (timings, started) {
                t.rule(*index, s);
            }
        }
    }

    let mut out = Vec::new();
    // The data tree, built once for every `target:` rule of the unit; a
    // build that panics fails each of them (decision 25).
    let mut data_tree: Option<Result<Option<codelaw_match::data::DataTree>, String>> = None;
    for (slot, (index, rule, _)) in unit.rules.iter().enumerate() {
        if let Some(why) = &failed[slot] {
            out.push(internal_error(*index, 0, 0, rule.id(), why));
            continue;
        }
        let started = timings.map(|_| Instant::now());
        let ctx = FixContext { source: &unit.text, root: &unit.root, language: unit.language, semantic: semantic.as_ref() };
        let wants_fix = rule.loaded.rule.fix != FixClass::Manual;
        match &rule.matcher {
            Matcher::Native(_) => {
                for f in std::mem::take(&mut native[slot]) {
                    let edits = (wants_fix && !f.edits.is_empty()).then_some(f.edits);
                    out.push(Finding { rule: *index, start: f.start, end: f.end, message: f.message, edits, note: None });
                }
            }
            Matcher::Match(_) => {
                let ms = &matches[slot];
                if ms.is_empty() {
                    continue;
                }
                let message = rule.loaded.rule.block.message.as_deref().unwrap_or(rule.loaded.rule.title.as_str());
                let mut per: Vec<(Option<Vec<TextEdit>>, Option<String>)> = vec![(None, None); ms.len()];
                match (&rule.fix, wants_fix) {
                    (Some(spec @ FixSpec::Steps(_)), true) if spec.is_group() => {
                        let r = catch_unwind(AssertUnwindSafe(|| group_edits(spec, ms, &ctx)));
                        let res = match r {
                            Ok(Ok(e)) => (Some(e), None),
                            Ok(Err(e)) => (None, Some(format!("fix-unavailable: {e}"))),
                            Err(p) => (None, Some(format!("fix-unavailable: {}", panic_message(p.as_ref())))),
                        };
                        per = vec![res; ms.len()];
                    }
                    (Some(spec @ FixSpec::Steps(_)), true) => {
                        for (i, m) in ms.iter().enumerate() {
                            per[i] = match catch_unwind(AssertUnwindSafe(|| edits_for(spec, m, &ctx))) {
                                Ok(Ok(e)) => (Some(e), None),
                                Ok(Err(e)) => (None, Some(format!("fix-unavailable: {e}"))),
                                Err(p) => (None, Some(format!("fix-unavailable: {}", panic_message(p.as_ref())))),
                            };
                        }
                    }
                    (Some(FixSpec::External(cmd)), true) => {
                        let allowed = rule.loaded.external_allowed;
                        let res = if !allowed {
                            (None, Some("external-fixer-not-allowed: add the pack to `allow_pack_fixers`".to_owned()))
                        } else if !mode.external_fixers {
                            (None, None)
                        } else {
                            let call = ExternalCall {
                                command: cmd,
                                cwd: project.root(),
                                timeout: project.config.external_timeout(),
                                rule: rule.id(),
                                file: &entry.rel,
                                language: unit.language.name(),
                                source: &unit.text,
                                matches: ms,
                            };
                            match run_external(&call) {
                                Ok(e) => (Some(e), None),
                                Err(e) => (None, Some(format!("external-fixer-failed: {e}"))),
                            }
                        };
                        per = vec![res; ms.len()];
                    }
                    _ => {}
                }
                for (m, (edits, note)) in ms.iter().zip(per) {
                    out.push(Finding { rule: *index, start: m.start, end: m.end, message: interpolate(message, m, &unit.text), edits, note });
                }
            }
            Matcher::Graph(_) => {}
            Matcher::Target(t) => {
                let tree = data_tree.get_or_insert_with(|| {
                    catch_unwind(AssertUnwindSafe(|| codelaw_match::data::build(unit.language, unit.root.root().get_inner_node(), &unit.text)))
                        .map_err(|p| format!("the data tree failed: {}", panic_message(p.as_ref())))
                });
                let tree = match tree {
                    Ok(Some(t)) => &*t,
                    Ok(None) => continue,
                    Err(why) => {
                        out.push(internal_error(*index, 0, 0, rule.id(), why));
                        continue;
                    }
                };
                let found = match catch_unwind(AssertUnwindSafe(|| t.check(tree))) {
                    Ok(f) => f,
                    Err(p) => {
                        out.push(internal_error(*index, 0, 0, rule.id(), &panic_message(p.as_ref())));
                        continue;
                    }
                };
                let message = rule.loaded.rule.block.message.as_deref().unwrap_or(rule.loaded.rule.title.as_str());
                for f in found {
                    let (edits, note) = match (&rule.data_fix, wants_fix) {
                        (Some(steps), true) => {
                            let ctx = DataFixContext { source: &unit.text, language: unit.language, tree, document: f.document, assert: &t.assert };
                            match catch_unwind(AssertUnwindSafe(|| data_edits(steps, &ctx, &f.slot))) {
                                Ok(Ok(e)) => (Some(e), None),
                                Ok(Err(e)) => (None, Some(format!("fix-unavailable: {e}"))),
                                Err(p) => (None, Some(format!("fix-unavailable: {}", panic_message(p.as_ref())))),
                            }
                        }
                        _ => (None, None),
                    };
                    let text = codelaw_match::target::interpolate(message, tree, &f.slot, &unit.text, f.error.as_deref());
                    out.push(Finding { rule: *index, start: f.start, end: f.end, message: text, edits, note });
                }
            }
            Matcher::Unsupported(_) => {}
        }
        if let (Some(t), Some(s)) = (timings, started) {
            t.rule(*index, s);
        }
    }
    out
}

//! The rule tester (SR-RULE-AUTHORING decision 4). A rule is accepted by
//! this runner, never by anyone's judgement (L5 decision 6):
//!
//! - **examples:** every `Don't` fires, every `Do` passes, the fix of a
//!   `Don't` equals its `Do` (or `Fixed`) and is a fixpoint — for every
//!   declared language (L4 decision 7);
//! - **cases:** each `<name>.cases.md` case gives exactly its `expect`
//!   diagnostics and its `fixed` output;
//! - **invariants:** a fix still parses and is deterministic, plus the
//!   rule's own `invariant:` (`exports-unchanged`);
//! - **corpus:** over real files (`codelaw test --corpus <dir>`), the
//!   invariants hold on every file, and the finding count is what the corpus
//!   snapshot records ([`corpus_rule`]).

use codelaw_cache::Key;
use codelaw_config::{Config, RuleConfig};
use codelaw_core::{FixClass, Language};
use codelaw_engine::{EngineOptions, Project};
use codelaw_format::FormatOptions;
use codelaw_rules::{LanguageExamples, LoadedRule, RuleSet, Scope, Snippet};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Examples,
    Cases,
    Invariants,
    Corpus,
}

impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Level::Examples => "examples",
            Level::Cases => "cases",
            Level::Invariants => "invariants",
            Level::Corpus => "corpus",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub level: Level,
    /// Where in the rule file or its sidecar.
    pub at: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct RuleReport {
    pub id: String,
    pub source: String,
    pub path: PathBuf,
    pub checks: usize,
    pub failures: Vec<Failure>,
    /// Advisory findings: printed, never a failure (SR-FORMATTER decision 3's
    /// formatter cross-check).
    pub warnings: Vec<Failure>,
}

impl RuleReport {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

/// The built-in invariants every fix is held to; `invariant:` adds to them.
pub const RULE_INVARIANTS: &[&str] = &["exports-unchanged"];

/// A project holding only `rule`, with no config and no disk access.
fn project_for(loaded: &LoadedRule, keep_globs: bool) -> Project {
    let mut rule = (*loaded.rule).clone();
    if !keep_globs {
        rule.files.clear();
        rule.exclude.clear();
    }
    let default = RuleConfig { severity: Some(rule.severity), options: rule.default_options() };
    let lr = LoadedRule {
        rule: Arc::new(rule),
        default: Some(default),
        preview_blocked: false,
        external_allowed: true,
        ..loaded.clone()
    };
    let rules = RuleSet { rules: vec![lr], packs: Vec::new(), warnings: Vec::new(), fingerprint: Key([0; 32]) };
    let config = Config::defaults(PathBuf::from("/codelaw-test"));
    Project::from_parts(config, rules, EngineOptions { no_cache: true, threads: Some(1), ..Default::default() })
        .expect("a single loaded rule compiles: it already passed load")
}

struct Tester<'a> {
    loaded: &'a LoadedRule,
    plain: Project,
    with_globs: Option<Project>,
    report: RuleReport,
}

fn first_difference(expected: &str, got: &str) -> String {
    for (i, (e, g)) in expected.lines().zip(got.lines()).enumerate() {
        if e != g {
            return format!("line {}: expected `{e}`, got `{g}`", i + 1);
        }
    }
    let (el, gl) = (expected.lines().count(), got.lines().count());
    if el != gl {
        return format!("expected {el} lines, got {gl}");
    }
    "the line endings or the final newline differ".into()
}

impl<'a> Tester<'a> {
    fn project(&mut self, snippet: &Snippet) -> &Project {
        if snippet.path.is_some() {
            if self.with_globs.is_none() {
                self.with_globs = Some(project_for(self.loaded, true));
            }
            self.with_globs.as_ref().expect("just set")
        } else {
            &self.plain
        }
    }

    fn fail(&mut self, level: Level, at: String, message: String) {
        self.report.failures.push(Failure { level, at, message });
    }

    fn path(language: Language, snippet: &Snippet) -> String {
        snippet.path.clone().unwrap_or_else(|| format!("example.{}", language.extension()))
    }

    fn project_scope(&self) -> bool {
        self.loaded.rule.scope == Scope::Project
    }

    /// Whether a snippet is checked as a set of files: always for a project
    /// rule; for a file rule when its first line names a file, so a native
    /// rule sees the files around the one it checks (SR-ARCHITECTURE
    /// decision 15b).
    fn as_files(&self, code: &str) -> bool {
        self.project_scope() || code.lines().next().and_then(header_path).is_some()
    }

    /// This rule's diagnostics on a snippet, as `line:col rule` lines
    /// (`path:line:col rule` for a multi-file snippet).
    fn fires(&mut self, language: Language, snippet: &Snippet, code: &str) -> Vec<String> {
        let id = self.loaded.rule.id.clone();
        if self.as_files(code) {
            let files = split_files(language, code);
            let multi = files.len() > 1 || files.first().is_some_and(|f| f.header.is_some());
            let set: Vec<(String, Language, String)> = files.iter().map(|f| (f.path.clone(), f.language, f.text.clone())).collect();
            return self
                .plain
                .check_files(&set)
                .into_iter()
                .filter(|d| d.rule == id)
                .map(|d| if multi { format!("{}:{}:{} {}", d.file, d.start.line, d.start.column, d.rule) } else { format!("{}:{} {}", d.start.line, d.start.column, d.rule) })
                .collect();
        }
        let path = Self::path(language, snippet);
        let p = self.project(snippet);
        p.check_source(&path, language, code)
            .into_iter()
            .filter(|d| d.rule == id)
            .map(|d| format!("{}:{} {}", d.start.line, d.start.column, d.rule))
            .collect()
    }

    fn fix(&mut self, language: Language, snippet: &Snippet, code: &str) -> String {
        if self.as_files(code) {
            let files = split_files(language, code);
            let set: Vec<(String, Language, String)> = files.iter().map(|f| (f.path.clone(), f.language, f.text.clone())).collect();
            let fixed = self.plain.fix_files(&set, true);
            return files
                .iter()
                .zip(fixed)
                .map(|(f, (_, text))| match &f.header {
                    Some(h) => format!("{h}\n{text}"),
                    None => text,
                })
                .collect();
        }
        let path = Self::path(language, snippet);
        self.project(snippet).fix_source(&path, language, code, true).fixed
    }

    fn exports(language: Language, code: &str) -> Option<BTreeSet<String>> {
        let root = codelaw_lang::parse(language, code);
        codelaw_semantic::build(language, &root, code).map(|m| m.exported_names())
    }

    fn fix_checks(&mut self, language: Language, snippet: &Snippet, expected: Option<&Snippet>, at: &str, level: Level) {
        let fixed = self.fix(language, snippet, &snippet.code);
        self.report.checks += 1;
        if let Some(exp) = expected
            && fixed != exp.code
        {
            let diff = first_difference(&exp.code, &fixed);
            self.fail(level, at.to_owned(), format!("the fix does not give the expected output (line {}): {diff}\n--- got ---\n{fixed}", exp.line));
        }
        // Fixpoint (L4 decision 7).
        self.report.checks += 1;
        let again = self.fix(language, snippet, &fixed);
        let still = self.fires(language, snippet, &fixed);
        if again != fixed || !still.is_empty() {
            self.fail(Level::Invariants, at.to_owned(), format!("the fixed output is not a fixpoint{}", if still.is_empty() { String::new() } else { format!(": the rule still fires at {}", still.join(", ")) }));
        }
        // Still parses.
        self.report.checks += 1;
        // A set of files parses file by file, each in its own language.
        let errors = |code: &str| -> usize {
            if self.as_files(code) {
                split_files(language, code).iter().map(|f| codelaw_lang::syntax_error_count(&codelaw_lang::parse(f.language, &f.text))).sum()
            } else {
                codelaw_lang::syntax_error_count(&codelaw_lang::parse(language, code))
            }
        };
        let (before, after) = (errors(&snippet.code), errors(&fixed));
        if after > before {
            self.fail(Level::Invariants, at.to_owned(), "the fix adds a syntax error".into());
        }
        if self.loaded.rule.invariants.iter().any(|i| i == "exports-unchanged") {
            self.report.checks += 1;
            if Self::exports(language, &snippet.code) != Self::exports(language, &fixed) {
                self.fail(Level::Invariants, at.to_owned(), "`exports-unchanged`: the fix changes what the module exports".into());
            }
        }
    }

    fn examples(&mut self) {
        let rule = self.loaded.rule.clone();
        let fixable = rule.fix != FixClass::Manual;
        for ex in &rule.examples {
            let l = ex.language;
            for (i, dont) in ex.donts.iter().enumerate() {
                let at = format!("{} Don't (line {})", l.display(), dont.line);
                self.report.checks += 1;
                let hits = self.fires(l, dont, &dont.code);
                if hits.is_empty() {
                    self.fail(Level::Examples, at.clone(), "does not fire".into());
                    continue;
                }
                // Deterministic.
                self.report.checks += 1;
                if self.fires(l, dont, &dont.code) != hits {
                    self.fail(Level::Invariants, at.clone(), "two runs disagree".into());
                }
                if fixable {
                    let expected = if !ex.fixed.is_empty() {
                        ex.fixed.get(i).or(ex.fixed.last())
                    } else if ex.dos.len() == ex.donts.len() {
                        ex.dos.get(i)
                    } else {
                        ex.dos.first()
                    };
                    self.fix_checks(l, dont, expected, &at, Level::Examples);
                }
            }
            for do_ in &ex.dos {
                let at = format!("{} Do (line {})", l.display(), do_.line);
                self.report.checks += 1;
                let hits = self.fires(l, do_, &do_.code);
                if !hits.is_empty() {
                    self.fail(Level::Examples, at, format!("fires at {}", hits.join(", ")));
                }
            }
            self.layout_only(ex);
        }
    }

    /// A snippet as the formatter lays it out with the defaults, one text
    /// per file; `None` when a file does not format (it does not parse).
    fn formatted(language: Language, snippet: &Snippet) -> Option<Vec<String>> {
        split_files(language, &snippet.code)
            .into_iter()
            .map(|f| {
                let path = if f.header.is_some() { f.path.clone() } else { Self::path(f.language, snippet) };
                codelaw_format::format(f.language, &path, &f.text, &FormatOptions::default()).ok()
            })
            .collect()
    }

    /// A snippet's token streams through codelaw's own parser, file by file
    /// with each file's path, embedded regions region by region.
    fn tokens(language: Language, snippet: &Snippet) -> Vec<(String, Vec<codelaw_lang::Token>)> {
        split_files(language, &snippet.code).into_iter().map(|f| (f.path, codelaw_lang::token_stream(f.language, &f.text))).collect()
    }

    /// SR-FORMATTER decision 3: a `Don't` with the same token stream as a
    /// `Do` (or `Fixed`) differs from it only in layout, so the rule is a
    /// layout rule, and layout belongs to the formatter. The formatter is a
    /// second, advisory opinion: when it lays out a `Don't` and a target
    /// with different tokens identically, it changes tokens there, which may
    /// change meaning (B-045) — a warning, never a failure.
    fn layout_only(&mut self, ex: &LanguageExamples) {
        let l = ex.language;
        let targets: Vec<(&str, &Snippet)> = ex.dos.iter().map(|s| ("Do", s)).chain(ex.fixed.iter().map(|s| ("Fixed", s))).collect();
        let target_tokens: Vec<_> = targets.iter().map(|(_, s)| Self::tokens(l, s)).collect();
        let mut target_formatted: Vec<Option<Option<Vec<String>>>> = vec![None; targets.len()];
        for dont in &ex.donts {
            self.report.checks += 1;
            let at = format!("{} Don't (line {})", l.display(), dont.line);
            let d = Self::tokens(l, dont);
            let differs = |s: &Snippet| s.code != dont.code;
            if let Some(i) = (0..targets.len()).find(|&i| differs(targets[i].1) && target_tokens[i] == d) {
                let (kind, s) = targets[i];
                self.fail(
                    Level::Examples,
                    at,
                    format!(
                        "a layout rule: it has the same tokens as the {kind} (line {}), so the two differ only in layout, and layout belongs to the formatter (SR-FORMATTER decision 3)",
                        s.line
                    ),
                );
                continue;
            }
            let Some(df) = Self::formatted(l, dont) else { continue };
            for i in 0..targets.len() {
                let (kind, s) = targets[i];
                if !differs(s) {
                    continue;
                }
                let tf = target_formatted[i].get_or_insert_with(|| Self::formatted(l, s));
                if tf.as_ref() == Some(&df) {
                    self.report.warnings.push(Failure {
                        level: Level::Examples,
                        at: at.clone(),
                        message: format!(
                            "the formatter ({}) lays this out identically to the {kind} (line {}) although their tokens differ: formatting changes tokens here, so check it does not change meaning (B-045); not a layout rule (SR-FORMATTER decision 3)",
                            codelaw_format::engine(l),
                            s.line
                        ),
                    });
                    break;
                }
            }
        }
    }

    fn cases(&mut self, filter: Option<&str>) {
        let rule = self.loaded.rule.clone();
        for case in &rule.cases {
            if filter.is_some_and(|f| f != case.title) {
                continue;
            }
            let at = format!("case `{}` (line {})", case.title, case.input.line);
            self.report.checks += 1;
            let mut got = self.fires(case.language, &case.input, &case.input.code);
            got.sort();
            let mut want = case.expect.clone();
            want.sort();
            if got != want {
                let show = |v: &[String]| if v.is_empty() { "(none)".to_owned() } else { v.join(", ") };
                self.fail(Level::Cases, at.clone(), format!("expected {}, got {}", show(&want), show(&got)));
            }
            if let Some(fixed) = &case.fixed {
                let expected = Snippet { code: fixed.clone(), path: case.input.path.clone(), line: case.input.line };
                self.fix_checks(case.language, &case.input, Some(&expected), &at, Level::Cases);
            }
        }
    }
}

/// One file of a multi-file snippet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetFile {
    /// The header line as written (`// packages/a/src/x.ts`).
    pub header: Option<String>,
    pub path: String,
    pub language: Language,
    pub text: String,
}

/// The path a header line names: `// path`, `# path`, `/* path */` or
/// `<!-- path -->`, alone on its line.
fn header_path(line: &str) -> Option<String> {
    let t = line.trim();
    let inner = if let Some(r) = t.strip_prefix("//") {
        r
    } else if let Some(r) = t.strip_prefix("<!--") {
        r.strip_suffix("-->")?
    } else if let Some(r) = t.strip_prefix("/*") {
        r.strip_suffix("*/")?
    } else {
        t.strip_prefix('#')?
    };
    let p = inner.trim();
    if p.is_empty() || p.contains(char::is_whitespace) || !p.contains('.') {
        return None;
    }
    let name = p.rsplit('/').next().unwrap_or(p);
    let known = Language::from_path(std::path::Path::new(p)).is_some() || matches!(name, "setup.py" | "setup.cfg");
    known.then(|| p.to_owned())
}

/// Split a `files` snippet into its `// path` sections (SR-PROJECT-GRAPH
/// decision 6); a snippet with no header is one file, `example.<ext>`.
pub fn split_files(language: Language, code: &str) -> Vec<SnippetFile> {
    let mut out: Vec<SnippetFile> = Vec::new();
    let mut lead = String::new();
    for line in code.split_inclusive('\n') {
        if let Some(p) = header_path(line) {
            let language = Language::from_path(std::path::Path::new(&p)).unwrap_or(language);
            out.push(SnippetFile { header: Some(line.trim_end_matches(['\n', '\r']).to_owned()), path: p, language, text: String::new() });
        } else if let Some(f) = out.last_mut() {
            f.text.push_str(line);
        } else {
            lead.push_str(line);
        }
    }
    if out.is_empty() {
        return vec![SnippetFile { header: None, path: format!("example.{}", language.extension()), language, text: code.to_owned() }];
    }
    out
}

/// Test one rule.
pub fn test_rule(loaded: &LoadedRule, case: Option<&str>) -> RuleReport {
    let report = RuleReport {
        id: loaded.rule.id.clone(),
        source: loaded.source.label(),
        path: loaded.rule.path.clone(),
        checks: 0,
        failures: Vec::new(),
        warnings: Vec::new(),
    };
    let mut t = Tester { loaded, plain: project_for(loaded, false), with_globs: None, report };
    for inv in &loaded.rule.invariants {
        if !RULE_INVARIANTS.contains(&inv.as_str()) {
            t.fail(Level::Invariants, "frontmatter".into(), format!("unknown invariant `{inv}` (one of {})", RULE_INVARIANTS.join(", ")));
        }
    }
    if case.is_none() {
        t.examples();
    }
    t.cases(case);
    t.report
}

/// Level 4 over one corpus: what a rule found there, and its invariants'
/// report.
#[derive(Clone, Debug)]
pub struct CorpusReport {
    pub report: RuleReport,
    pub findings: usize,
}

fn hits(p: &Project, id: &str, path: &str, language: Language, text: &str) -> Vec<(u32, u32)> {
    p.check_source(path, language, text).into_iter().filter(|d| d.rule == id).map(|d| (d.start.line, d.start.column)).collect()
}

/// Level 4 (SR-RULE-AUTHORING decision 4): run `loaded` over `files` — each
/// `(path relative to the corpus root, language, text)` — and hold every
/// file it fires on to the invariants: two runs agree, and the fix still
/// parses, is a fixpoint and keeps the rule's own `invariant:`. The finding
/// count is what the corpus snapshot records.
pub fn corpus_rule(loaded: &LoadedRule, files: &[(String, Language, String)]) -> CorpusReport {
    let report = RuleReport {
        id: loaded.rule.id.clone(),
        source: loaded.source.label(),
        path: loaded.rule.path.clone(),
        checks: 0,
        failures: Vec::new(),
        warnings: Vec::new(),
    };
    let mut t = Tester { loaded, plain: project_for(loaded, true), with_globs: None, report };
    let id = loaded.rule.id.clone();
    if t.project_scope() {
        let run = |p: &Project| -> Vec<(String, u32, u32)> {
            p.check_files(files).into_iter().filter(|d| d.rule == id).map(|d| (d.file, d.start.line, d.start.column)).collect()
        };
        t.report.checks += 1;
        let first = run(&t.plain);
        if run(&t.plain) != first {
            t.fail(Level::Corpus, "the corpus".into(), "two runs disagree".into());
        }
        return CorpusReport { findings: first.len(), report: t.report };
    }
    let fixable = loaded.rule.fix != FixClass::Manual;
    let mut findings = 0;
    for (path, language, text) in files {
        let first = hits(&t.plain, &id, path, *language, text);
        if first.is_empty() {
            continue;
        }
        findings += first.len();
        t.report.checks += 1;
        if hits(&t.plain, &id, path, *language, text) != first {
            t.fail(Level::Corpus, path.clone(), "two runs disagree".into());
        }
        if !fixable {
            continue;
        }
        let fixed = t.plain.fix_source(path, *language, text, true).fixed;
        if fixed == *text {
            continue;
        }
        t.report.checks += 2;
        if t.plain.fix_source(path, *language, &fixed, true).fixed != fixed {
            t.fail(Level::Corpus, path.clone(), "the fixed file is not a fixpoint".into());
        }
        let before = codelaw_lang::syntax_error_count(&codelaw_lang::parse(*language, text));
        let after = codelaw_lang::syntax_error_count(&codelaw_lang::parse(*language, &fixed));
        if after > before {
            t.fail(Level::Corpus, path.clone(), "the fix adds a syntax error".into());
        }
        if loaded.rule.invariants.iter().any(|i| i == "exports-unchanged") {
            t.report.checks += 1;
            if Tester::exports(*language, text) != Tester::exports(*language, &fixed) {
                t.fail(Level::Corpus, path.clone(), "`exports-unchanged`: the fix changes what the module exports".into());
            }
        }
    }
    CorpusReport { report: t.report, findings }
}

/// Every file under `dir` the engine has a language for, as `(path relative
/// to dir, language, text)` in path order; `.git`, `node_modules`, `.venv`
/// and the cache skipped, `.gitignore` not read (a corpus is what it is).
pub fn corpus_files(dir: &Path) -> std::io::Result<Vec<(String, Language, String)>> {
    let mut out = Vec::new();
    let walk = ignore::WalkBuilder::new(dir)
        .standard_filters(false)
        .filter_entry(|e| !codelaw_config::DEFAULT_EXCLUDES.iter().any(|x| e.file_name() == *x))
        .build();
    for e in walk {
        let e = e.map_err(std::io::Error::other)?;
        if !e.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Some(language) = Language::from_path(e.path()) else { continue };
        // A file that is not UTF-8 is not source the engine reads.
        let Ok(text) = std::fs::read_to_string(e.path()) else { continue };
        let rel = e.path().strip_prefix(dir).unwrap_or(e.path());
        out.push((codelaw_core::normalize_path(rel), language, text));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// The corpus snapshot: `[findings]`, each rule id's count on the corpus.
pub fn read_snapshot(text: &str) -> Result<BTreeMap<String, usize>, String> {
    let v: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
    let mut out = BTreeMap::new();
    for (id, n) in v.get("findings").and_then(|f| f.as_table()).ok_or("no `[findings]` table")? {
        let n = n.as_integer().and_then(|n| usize::try_from(n).ok()).ok_or(format!("`{id}` is not a count"))?;
        out.insert(id.clone(), n);
    }
    Ok(out)
}

pub fn write_snapshot(reports: &[CorpusReport]) -> String {
    let mut out = String::from("# `codelaw test --corpus` finding counts (SR-RULE-AUTHORING decision 4).\n# Rewrite with --update-snapshot, and review the change.\n\n[findings]\n");
    for r in reports {
        out.push_str(&format!("{:?} = {}\n", r.report.id, r.findings));
    }
    out
}

/// A count that differs from the snapshot, or is missing from it, fails.
pub fn compare_snapshot(reports: &mut [CorpusReport], snapshot: &BTreeMap<String, usize>) {
    for r in reports {
        match snapshot.get(&r.report.id) {
            Some(n) if *n == r.findings => {}
            Some(n) => r.report.failures.push(Failure { level: Level::Corpus, at: "snapshot".into(), message: format!("{} findings; the snapshot says {n}", r.findings) }),
            None => r.report.failures.push(Failure { level: Level::Corpus, at: "snapshot".into(), message: format!("{} findings, and no count in the snapshot", r.findings) }),
        }
    }
}

/// Test rules in id order.
pub fn test_rules<'a>(rules: impl IntoIterator<Item = &'a LoadedRule>, case: Option<&str>) -> Vec<RuleReport> {
    let mut out: Vec<RuleReport> = rules.into_iter().map(|r| test_rule(r, case)).collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// The `codelaw test` text output.
pub fn render(reports: &[RuleReport], color: bool) -> String {
    let paint = |code: &str, s: &str| if color { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_owned() };
    let mut out = String::new();
    for r in reports {
        if r.passed() {
            out.push_str(&format!("{} {}  {}\n", paint("32", "✔"), r.id, paint("2", &format!("{} checks · {}", r.checks, r.source))));
            for w in &r.warnings {
                out.push_str(&format!("    {} {} · {}: {}\n", paint("33", "warning"), w.level.name(), w.at, w.message));
            }
        } else {
            out.push_str(&format!("{} {}  {}\n", paint("31", "✖"), r.id, paint("2", &format!("{} · {}", codelaw_core::normalize_path(&r.path), r.source))));
            for f in &r.failures {
                out.push_str(&format!("    {} · {}: {}\n", f.level.name(), f.at, f.message));
            }
            for w in &r.warnings {
                out.push_str(&format!("    {} {} · {}: {}\n", paint("33", "warning"), w.level.name(), w.at, w.message));
            }
        }
    }
    let failed = reports.iter().filter(|r| !r.passed()).count();
    let checks: usize = reports.iter().map(|r| r.checks).sum();
    let warned: usize = reports.iter().map(|r| r.warnings.len()).sum();
    let warned = if warned == 0 { String::new() } else { format!(" · {warned} warnings") };
    out.push_str(&format!("\n{} rules · {checks} checks · {failed} failed{warned}\n", reports.len()));
    out
}

#[cfg(test)]
mod tests;

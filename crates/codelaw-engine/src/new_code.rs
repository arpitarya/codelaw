//! The test rules of new code (W-71; SR-PROJECT-GRAPH decision 9,
//! SR-BASELINE decision 12): `graph/untested-function` — a new or edited
//! JS, TS or Python function that no test file reaches — and
//! `graph/function-coverage` — one whose lines an lcov report shows under
//! `[coverage] min`. Only the change set's new and edited functions are
//! judged (diff-scoped); `.codelawignore` exempts the rest, and an edit
//! revokes an exemption unless it is `forever` or `on_touch=keep`.
//!
//! Opt-in: `extends = ["codelaw:tests"]`, a `[coverage]` table, or a rule
//! named in `[rules]`. Off, nothing here runs and no graph is built for it
//! (L8). The engine never runs tests: it reads the report a runner wrote
//! (L5).

use crate::lcov::Lcov;
use crate::lint::{meta_diagnostic, meta_severity};
use crate::project::{Source, suppressed, suppressions, within};
use crate::{ChangedFunctions, EngineError, Project};
use codelaw_baseline::exception::{EXPIRED_RULE, EXPIRING_RULE, Expiry, expiry};
use codelaw_baseline::ignore_file::{self, IGNORE_FILE, INCOMPLETE_RULE, IgnoreEntry, IgnoreOnTouch, IgnorePolicy, IgnoreTarget, STALE_IGNORE_RULE, TouchScope};
use codelaw_core::{Diagnostic, FixClass, Language, LineIndex, Severity, TextEdit};
use codelaw_graph::{Graph, is_test_file};
use codelaw_lang::{Change, Function};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) const UNTESTED_RULE: &str = "graph/untested-function";
pub(crate) const COVERAGE_RULE: &str = "graph/function-coverage";
pub(crate) const STALE_COVERAGE_RULE: &str = "codelaw/stale-coverage";

/// `[coverage]`, checked.
#[derive(Clone, Debug)]
pub(crate) struct CoverageConfig {
    /// `report`, absolute (relative to the config file).
    pub report: Option<PathBuf>,
    pub public_only: bool,
    pub min: u32,
    pub patch: bool,
    pub require_report: bool,
    pub all_without_change_set: bool,
    pub ignore_names: Option<regex::Regex>,
}

/// What a project loads for the test rules, when it turns them on.
#[derive(Clone, Debug)]
pub(crate) struct TestRules {
    /// The rules the config turns on (before `--rule`).
    pub on: BTreeSet<&'static str>,
    pub config: CoverageConfig,
    pub policy: IgnorePolicy,
    /// `.codelawignore`, when the project has one.
    pub ignore_text: Option<String>,
    pub ignore: Vec<IgnoreEntry>,
}

/// The header lines a run with the test rules prints (SR-CLI-MCP).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewCodeHeader {
    /// `new code: …` — the change set the rules judged.
    pub change_set: String,
    /// `coverage: …` — the report `graph/function-coverage` read; none when
    /// that rule is off.
    pub coverage: Option<String>,
}

fn config_error(file: Option<&Path>, e: String) -> EngineError {
    EngineError::Config(codelaw_config::ConfigError::new(file.map(Path::to_path_buf), e))
}

/// `[coverage]` and `.codelawignore`, read at load; `None` when the project
/// does not turn the test rules on.
pub(crate) fn load(config: &codelaw_config::Config) -> Result<Option<TestRules>, EngineError> {
    let raw = config.files.first().and_then(|f| f.raw.coverage.as_ref());
    let preset = config.extends().iter().any(|e| e == codelaw_rules::TESTS_PRESET);
    let named: BTreeSet<String> = config.named_rule_ids().into_iter().map(|(id, _)| id).collect();
    let on: BTreeSet<&'static str> = codelaw_rules::TEST_RULES.iter().copied().filter(|r| preset || raw.is_some() || named.contains(*r)).collect();
    if on.is_empty() {
        return Ok(None);
    }
    let file = config.root_file();
    let err = |e: String| config_error(file, format!("[coverage]: {e}"));
    let d = codelaw_config::RawCoverage::default();
    let c = raw.unwrap_or(&d);
    let public_only = match c.scope.as_deref() {
        None | Some("all") => false,
        Some("public") => true,
        Some(o) => return Err(err(format!("`scope = \"{o}\"` is not \"all\" or \"public\""))),
    };
    let min = match c.min {
        None => 60,
        Some(n) if (0..=100).contains(&n) => n as u32,
        Some(n) => return Err(err(format!("`min = {n}` is not between 0 and 100"))),
    };
    let patch = match c.mode.as_deref() {
        None | Some("function") => false,
        Some("patch") => true,
        Some(o) => return Err(err(format!("`mode = \"{o}\"` is not \"function\" or \"patch\""))),
    };
    let all_without_change_set = match c.on_no_change_set.as_deref() {
        None | Some("skip") => false,
        Some("all") => true,
        Some(o) => return Err(err(format!("`on_no_change_set = \"{o}\"` is not \"skip\" or \"all\""))),
    };
    let ignore_names = c.ignore_names.as_deref().map(|r| regex::Regex::new(r).map_err(|e| err(format!("`ignore_names`: {e}")))).transpose()?;
    let report = c.report.as_deref().map(|r| config.config_dir().join(r));
    let policy = policy(config).map_err(|e| config_error(file, e))?;
    let ignore_text = std::fs::read_to_string(config.root_dir.join(IGNORE_FILE)).ok();
    let ignore = ignore_text.as_deref().map(|t| ignore_file::parse(t, &policy)).unwrap_or_default();
    Ok(Some(TestRules {
        on,
        config: CoverageConfig { report, public_only, min, patch, require_report: c.require_report.unwrap_or(false), all_without_change_set, ignore_names },
        policy,
        ignore_text,
        ignore,
    }))
}

/// `[coverage.ignore]`.
pub(crate) fn policy(config: &codelaw_config::Config) -> Result<IgnorePolicy, String> {
    match config.files.first().and_then(|f| f.raw.coverage.as_ref()).and_then(|c| c.ignore.as_ref()) {
        Some(i) => IgnorePolicy::from_table(i.require_expiry, i.require_on_touch, i.on_touch.as_deref(), i.on_touch_scope.as_deref()),
        None => Ok(IgnorePolicy::default()),
    }
}

/// What `.codelawignore` makes of one function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Exempt {
    /// No line covers it.
    No,
    /// A line covers it and holds.
    Holds,
    /// Every line that covers it was revoked by an edit: the findings come
    /// back, as warnings when every such line says `on_touch=warn`.
    Revoked { warn: bool },
}

/// One function the rules judge.
struct Judged<'a> {
    rel: &'a str,
    language: Language,
    function: &'a Function,
    change: Change,
    exempt: Exempt,
}

/// The `.codelawignore` lines that cover something on the as-of date.
fn live<'a>(project: &Project, entries: &'a [IgnoreEntry]) -> Vec<&'a IgnoreEntry> {
    let today = project.as_of().date;
    entries
        .iter()
        .filter(|e| e.valid())
        .filter(|e| !matches!(expiry(today, e.expires, None, e.warn_days.unwrap_or(project.exceptions.warn_days)), Expiry::Expired { .. }))
        .collect()
}

fn glob(pattern: &str) -> Option<globset::GlobMatcher> {
    globset::GlobBuilder::new(pattern).literal_separator(true).build().ok().map(|g| g.compile_matcher())
}

fn covers(e: &IgnoreEntry, rel: &str, name: &str) -> bool {
    match &e.target {
        IgnoreTarget::Function { path, name: n } => path == rel && n == name,
        // `.gitignore`-style: a pattern without a `/` matches a name at
        // any depth; a folder covers what is under it.
        IgnoreTarget::Glob(g) => {
            let name = rel.rsplit('/').next().unwrap_or(rel);
            glob(g).is_some_and(|m| m.is_match(rel) || (!g.contains('/') && m.is_match(name))) || rel.starts_with(&format!("{}/", g.trim_end_matches('/')))
        }
    }
}

fn exemption(entries: &[&IgnoreEntry], policy: &IgnorePolicy, rel: &str, f: &Function, change: Change, file_touched: bool) -> Exempt {
    let mut revoked = Vec::new();
    for e in entries.iter().filter(|e| covers(e, rel, &f.name)) {
        let touched = match e.scope(policy) {
            TouchScope::File => file_touched,
            TouchScope::Function => change != Change::Untouched,
        };
        match e.on_touch(policy) {
            IgnoreOnTouch::Keep => return Exempt::Holds,
            _ if !touched => return Exempt::Holds,
            t => revoked.push(t),
        }
    }
    if revoked.is_empty() { Exempt::No } else { Exempt::Revoked { warn: revoked.iter().all(|t| *t == IgnoreOnTouch::Warn) } }
}

/// The test file a finding suggests: `orders.test.ts`, `test_billing.py`.
fn test_name(rel: &str, language: Language) -> String {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    if language == Language::Python {
        return format!("test_{name}");
    }
    match name.split_once('.') {
        Some((stem, ext)) => format!("{stem}.test.{ext}"),
        None => format!("{name}.test"),
    }
}

/// The function's name without the `#n` that tells same-named ones apart.
fn bare(name: &str) -> &str {
    name.split('#').next().unwrap_or(name)
}

/// The first line of the function: where a finding is reported, so a
/// `codelaw-ignore` comment above the function covers it.
fn first_line_span(text: &str, f: &Function) -> (usize, usize) {
    let end = text[f.start_byte..f.end_byte].find('\n').map(|i| f.start_byte + i).unwrap_or(f.end_byte);
    (f.start_byte, end)
}

/// Whether a source line holds no code: blank, or a comment only.
fn not_code(line: &str) -> bool {
    let t = line.trim();
    t.is_empty() || t.starts_with("//") || t.starts_with('#') || t.starts_with("/*") || t.starts_with("*/") || (t.starts_with('*') && !t.starts_with("*["))
}

impl Project {
    /// The test rules this run evaluates (`--rule` applied).
    pub(crate) fn test_rules_on(&self) -> Vec<&'static str> {
        let Some(t) = &self.tests else { return Vec::new() };
        t.on.iter().copied().filter(|r| self.options.rules.as_ref().is_none_or(|o| o.iter().any(|x| x == r))).collect()
    }

    /// Whether the run needs the graph for the test rules.
    pub(crate) fn tests_on(&self) -> bool {
        !self.test_rules_on().is_empty()
    }

    /// An engine diagnostic on one line of `.codelawignore`.
    fn ignore_line(&self, line: u32, rule: &str, message: String) -> Option<Diagnostic> {
        self.at_line(IGNORE_FILE, line, rule, message)
    }

    /// The test rules over a linked graph: their findings, the
    /// `.codelawignore` findings, and the header lines.
    pub(crate) fn new_code_findings(&self, graph: &Graph, sources: &[Source], scopes: &[String], warnings: &mut Vec<String>) -> (Vec<Diagnostic>, Option<NewCodeHeader>) {
        let Some(tests) = &self.tests else { return (Vec::new(), None) };
        let on = self.test_rules_on();
        if on.is_empty() {
            return (Vec::new(), None);
        }
        let mut out = Vec::new();
        out.extend(self.ignore_findings(tests, sources));
        let entries = live(self, &tests.ignore);
        let changed = match self.changed_functions() {
            Ok(c) => c,
            Err(e) => {
                warnings.push(format!("the change set could not be read ({e}); the test rules judge nothing"));
                ChangedFunctions::default()
            }
        };
        let by_rel: BTreeMap<&str, &Source> = sources.iter().map(|s| (s.rel.as_str(), s)).collect();
        let reported = |rel: &str| by_rel.get(rel).is_some_and(|s| s.reported) && within(rel, scopes) && !is_test_file(rel);
        // Every function in every reported file, for `on_no_change_set = "all"`.
        let mut everything: Vec<(String, Language, Vec<Function>)> = Vec::new();
        let all_mode = changed.no_change_set() && tests.config.all_without_change_set;
        if all_mode {
            for s in sources {
                let Some(language) = s.language.filter(|l| codelaw_lang::has_functions(*l)) else { continue };
                if !reported(&s.rel) || s.rel.ends_with(".d.ts") || s.rel.ends_with(".d.mts") || s.rel.ends_with(".d.cts") {
                    continue;
                }
                everything.push((s.rel.clone(), language, codelaw_lang::functions(language, &s.text)));
            }
        }
        let touched = &self.touched().0;
        let mut judged: Vec<Judged<'_>> = Vec::new();
        let candidates: Vec<(&str, Language, &Function, Change)> = if all_mode {
            everything.iter().flat_map(|(rel, l, fns)| fns.iter().map(move |f| (rel.as_str(), *l, f, Change::Untouched))).collect()
        } else {
            changed
                .files
                .iter()
                .filter(|f| reported(&f.path))
                .flat_map(|cf| cf.functions.iter().map(move |c| (cf.path.as_str(), cf.language, &c.function, c.change)))
                .collect()
        };
        for (rel, language, f, change) in candidates {
            if tests.config.public_only && !f.public {
                continue;
            }
            if tests.config.ignore_names.as_ref().is_some_and(|r| r.is_match(bare(&f.name))) {
                continue;
            }
            let exempt = exemption(&entries, &tests.policy, rel, f, change, touched.contains(rel));
            // Judged: new or edited, every function under `all`, and an
            // untouched one whose exemption an edit revoked.
            let judge = all_mode || change != Change::Untouched || matches!(exempt, Exempt::Revoked { .. });
            if judge && exempt != Exempt::Holds {
                judged.push(Judged { rel, language, function: f, change, exempt });
            }
        }
        let header_change = if changed.no_change_set() {
            if all_mode {
                format!("new code: no change set; every function judged (on_no_change_set = \"all\"), {} functions", judged.len())
            } else {
                "new code: no change set; test rules not evaluated".to_owned()
            }
        } else {
            let n = |c: Change| judged.iter().filter(|j| j.change == c).count();
            format!(
                "new code: {} new, {} edited functions since {}",
                n(Change::New),
                n(Change::Edited),
                changed.base.as_deref().map(|b| if b.len() == 40 { &b[..12] } else { b }).unwrap_or("HEAD")
            )
        };
        let mut header = NewCodeHeader { change_set: header_change, coverage: None };
        if on.contains(&UNTESTED_RULE) {
            for j in &judged {
                let module_name = bare(&j.function.name);
                let top = module_name.split('.').next().unwrap_or(module_name);
                let importable = j.language == Language::Python || j.function.public;
                if graph.test_reaches(j.rel, top, importable) {
                    continue;
                }
                let what = match j.change {
                    Change::New => "is new",
                    Change::Edited => "was edited",
                    Change::Untouched if all_mode => "is judged (on_no_change_set = \"all\")",
                    Change::Untouched => "is exempted in `.codelawignore`, but its file was edited",
                };
                let message = format!(
                    "`{}` {what} and no test reaches it (add one in `{}`, or exempt it in `.codelawignore`)",
                    j.function.name,
                    test_name(j.rel, j.language)
                );
                if let Some(d) = self.function_diagnostic(UNTESTED_RULE, &by_rel, j, message, json!({ "function": j.function.name, "change": j.change.name() })) {
                    out.push(d);
                }
            }
        }
        if on.contains(&COVERAGE_RULE) {
            let (found, line) = self.coverage_findings(tests, &judged, &by_rel, warnings);
            out.extend(found);
            header.coverage = Some(line);
        }
        // `codelaw-ignore graph/…: why` above a function works too.
        let mut per_file: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
        for d in out {
            per_file.entry(d.file.clone()).or_default().push(d);
        }
        let mut kept = Vec::new();
        for (file, diags) in per_file {
            match by_rel.get(file.as_str()) {
                Some(src) if src.language.is_some() => {
                    let sups = self.live(suppressions(src));
                    kept.extend(diags.into_iter().filter(|d| !(d.rule.starts_with("graph/") && suppressed(&sups, d))));
                }
                _ => kept.extend(diags),
            }
        }
        (kept, Some(header))
    }

    fn function_diagnostic(&self, rule: &str, by_rel: &BTreeMap<&str, &Source>, j: &Judged<'_>, message: String, data: serde_json::Value) -> Option<Diagnostic> {
        let src = by_rel.get(j.rel)?;
        let layers = self.config.layers_for(&src.abs);
        let mut d = meta_diagnostic(j.rel, &layers, rule, message)?;
        if let Exempt::Revoked { warn: true } = j.exempt {
            d.severity = Severity::Warn;
        }
        let (a, b) = first_line_span(&src.text, j.function);
        let index = LineIndex::new(&src.text);
        d.start_byte = a;
        d.end_byte = b;
        d.start = index.position(a);
        d.end = index.position(b);
        d.data = Some(data);
        Some(d)
    }

    /// `graph/function-coverage` and `codelaw/stale-coverage`, and the
    /// `coverage:` header line.
    fn coverage_findings(&self, tests: &TestRules, judged: &[Judged<'_>], by_rel: &BTreeMap<&str, &Source>, warnings: &mut Vec<String>) -> (Vec<Diagnostic>, String) {
        let mut out = Vec::new();
        let cfg = &tests.config;
        let given = self.options.coverage.clone().or_else(|| cfg.report.clone());
        let missing = |why: String, out: &mut Vec<Diagnostic>| {
            if cfg.require_report && !judged.is_empty() {
                let file = self.config.root_file().map(|f| crate::discover::rel_path(f, self.root())).unwrap_or_else(|| ".".into());
                out.extend(meta_diagnostic(&file, &self.config.layers_for(&self.root().join(&file)), COVERAGE_RULE, format!("{why}, and `[coverage] require_report` is on: generate an lcov report and pass it with `--coverage` or `[coverage] report`")));
            }
        };
        let Some(path) = given else {
            missing("no coverage report was given".into(), &mut out);
            let line = if cfg.require_report { "coverage: none; required (`[coverage] require_report`)" } else { "coverage: none; not evaluated" };
            return (out, line.to_owned());
        };
        let shown = crate::discover::rel_path(&std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone()), self.root());
        let lcov = match Lcov::read(&path, self.root()) {
            Ok(l) => l,
            Err(e) => {
                warnings.push(format!("{e}; `graph/function-coverage` is not evaluated"));
                missing(format!("the coverage report `{shown}` could not be read"), &mut out);
                return (out, format!("coverage: {shown} (unreadable); not evaluated"));
            }
        };
        let mut files: BTreeMap<&str, Vec<&Judged<'_>>> = BTreeMap::new();
        for j in judged {
            files.entry(j.rel).or_default().push(j);
        }
        let (mut total_lines, mut total_covered) = (0usize, 0usize);
        let mut first: Option<&Judged<'_>> = None;
        for (rel, list) in files {
            let Some(src) = by_rel.get(rel) else { continue };
            let lines: Vec<&str> = src.text.lines().collect();
            let stale = match lcov.files.get(rel) {
                None => Some("the coverage report does not name this file".to_owned()),
                Some(da) => da.keys().find_map(|&l| {
                    let i = l as usize;
                    if i > lines.len() {
                        Some(format!("the coverage report has line {l}, past the end of this file ({} lines)", lines.len()))
                    } else if not_code(lines[i - 1]) {
                        Some(format!("the coverage report counts line {l}, which is blank or a comment now"))
                    } else {
                        None
                    }
                }),
            };
            if let Some(why) = stale {
                let layers = self.config.layers_for(&src.abs);
                out.extend(meta_diagnostic(rel, &layers, STALE_COVERAGE_RULE, format!("{why}: the report is older than the code, so this file's coverage is not judged")));
                continue;
            }
            let da = &lcov.files[rel];
            for j in list {
                let range = j.function.first_line..=j.function.last_line;
                let instrumented: Vec<u64> = da.range(range).map(|(_, h)| *h).collect();
                if instrumented.is_empty() {
                    continue;
                }
                let covered = instrumented.iter().filter(|h| **h > 0).count();
                let n = instrumented.len();
                if cfg.patch {
                    total_lines += n;
                    total_covered += covered;
                    first.get_or_insert(j);
                    continue;
                }
                if covered * 100 < cfg.min as usize * n {
                    let pct = covered * 100 / n;
                    let message = format!("`{}` is {pct}% covered ({covered} of {n} lines; min {})", j.function.name, cfg.min);
                    let data = json!({ "function": j.function.name, "change": j.change.name(), "covered": covered, "lines": n, "pct": pct, "min": cfg.min });
                    out.extend(self.function_diagnostic(COVERAGE_RULE, by_rel, j, message, data));
                }
            }
        }
        if cfg.patch
            && let Some(j) = first
            && total_covered * 100 < cfg.min as usize * total_lines
        {
            let pct = total_covered * 100 / total_lines;
            let message = format!("new and edited code is {pct}% covered ({total_covered} of {total_lines} lines; min {}, mode = \"patch\")", cfg.min);
            let data = json!({ "covered": total_covered, "lines": total_lines, "pct": pct, "min": cfg.min });
            out.extend(self.function_diagnostic(COVERAGE_RULE, by_rel, j, message, data));
        }
        let mut line = format!("coverage: {shown} ({} files)", lcov.files.len());
        if lcov.unmapped > 0 {
            line.push_str(&format!("; {} outside the project", lcov.unmapped));
        }
        (out, line)
    }

    /// `.codelawignore`'s own findings: lines that cannot be read, are
    /// incomplete, expire, or match nothing.
    fn ignore_findings(&self, tests: &TestRules, sources: &[Source]) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        let Some(text) = &tests.ignore_text else { return out };
        let today = self.as_of().date;
        let files: BTreeSet<&str> = sources.iter().map(|s| s.rel.as_str()).collect();
        let index = LineIndex::new(text);
        for e in &tests.ignore {
            if let Some(bad) = &e.unreadable {
                out.extend(self.ignore_line(e.line, "codelaw/unknown-suppression", format!("This `.codelawignore` line cannot be read, so it exempts nothing: {bad}")));
                continue;
            }
            if !e.incomplete.is_empty() {
                out.extend(self.ignore_line(e.line, INCOMPLETE_RULE, format!("This `.codelawignore` line exempts nothing until it has {}", e.incomplete.join("; "))));
                continue;
            }
            match expiry(today, e.expires, None, e.warn_days.unwrap_or(self.exceptions.warn_days)) {
                Expiry::Expired { expires } => {
                    out.extend(self.ignore_line(e.line, EXPIRED_RULE, format!("`{}` in `.codelawignore` expired on {expires}: it no longer exempts anything", e.target.text())));
                    continue;
                }
                Expiry::Expiring { days, expires } => {
                    out.extend(self.ignore_line(e.line, EXPIRING_RULE, format!("`{}` in `.codelawignore` expires {}", e.target.text(), codelaw_baseline::exception::when(days, expires))));
                }
                Expiry::Holds => {}
            }
            let matches = match &e.target {
                IgnoreTarget::Glob(_) => files.iter().any(|f| covers(e, f, "")),
                IgnoreTarget::Function { path, name } => {
                    let abs = self.root().join(path);
                    match (Language::from_path(&abs).filter(|l| codelaw_lang::has_functions(*l)), std::fs::read_to_string(&abs)) {
                        (Some(l), Ok(t)) => codelaw_lang::functions(l, &t).iter().any(|f| &f.name == name),
                        _ => false,
                    }
                }
            };
            if !matches {
                let what = match &e.target {
                    IgnoreTarget::Glob(g) => format!("`{g}` matches no file"),
                    IgnoreTarget::Function { path, name } => format!("`{path}` has no function `{name}`"),
                };
                if let Some(mut d) = self.ignore_line(e.line, STALE_IGNORE_RULE, format!("{what}: this `.codelawignore` line exempts nothing; `codelaw fix` removes it")) {
                    let start = index.line_start(e.line).unwrap_or(0);
                    let end = index.line_start(e.line + 1).unwrap_or(text.len());
                    d.edits = vec![TextEdit::delete(start, end)];
                    d.fix = FixClass::Auto;
                    out.push(d);
                }
            }
        }
        out
    }

    /// `codelaw fix`: drop the `.codelawignore` lines that match nothing.
    /// Returns how many it removed.
    pub(crate) fn prune_ignore_file(&self, diagnostics: &[Diagnostic]) -> usize {
        let stale: Vec<u32> = diagnostics.iter().filter(|d| d.rule == STALE_IGNORE_RULE && d.file == IGNORE_FILE).map(|d| d.start.line).collect();
        if stale.is_empty() || meta_severity(&[], STALE_IGNORE_RULE).is_none() {
            return 0;
        }
        let path = self.root().join(IGNORE_FILE);
        let Ok(text) = std::fs::read_to_string(&path) else { return 0 };
        if std::fs::write(&path, ignore_file::without_lines(&text, &stale)).is_err() {
            return 0;
        }
        stale.len()
    }

    /// The `.codelawignore` rows of `codelaw exceptions`: one per line and
    /// rule, `forever` shown as such.
    pub(crate) fn ignore_exceptions(&self) -> Vec<crate::Exception> {
        let Ok(text) = std::fs::read_to_string(self.root().join(IGNORE_FILE)) else { return Vec::new() };
        let policy = policy(&self.config).unwrap_or_default();
        let mut out = Vec::new();
        for e in ignore_file::parse(&text, &policy).into_iter().filter(IgnoreEntry::valid) {
            let (files, symbol) = match &e.target {
                IgnoreTarget::Glob(g) => (vec![g.clone()], None),
                IgnoreTarget::Function { path, name } => (vec![path.clone()], Some(name.clone())),
            };
            for rule in codelaw_rules::TEST_RULES {
                out.push(crate::Exception {
                    kind: crate::ExceptionKind::Ignore,
                    rule: (*rule).to_owned(),
                    files: files.clone(),
                    symbol: symbol.clone(),
                    expires: e.expires,
                    forever: e.forever,
                    added_by: e.by.clone(),
                    reason: e.reason.clone(),
                    uses: 1,
                    at: IGNORE_FILE.to_owned(),
                    line: e.line,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str) -> Function {
        Function { name: name.into(), start_byte: 0, end_byte: 0, first_line: 1, last_line: 1, public: true, tokens: Vec::new() }
    }

    #[test]
    fn exemption_follows_on_touch_and_its_scope() {
        let p = IgnorePolicy::default();
        let parse = |t: &str| codelaw_baseline::ignore_file::parse(t, &p);
        let file = parse("a.py#f on_touch=error expires=2099-01-01: why");
        let refs: Vec<&IgnoreEntry> = file.iter().collect();
        assert_eq!(exemption(&refs, &p, "a.py", &f("f"), Change::Untouched, false), Exempt::Holds);
        assert_eq!(exemption(&refs, &p, "a.py", &f("f"), Change::Untouched, true), Exempt::Revoked { warn: false }, "file scope: any edit to the file");
        assert_eq!(exemption(&refs, &p, "a.py", &f("g"), Change::Edited, true), Exempt::No);
        let func = parse("a.py#f on_touch=warn on_touch_scope=function expires=2099-01-01: why");
        let refs: Vec<&IgnoreEntry> = func.iter().collect();
        assert_eq!(exemption(&refs, &p, "a.py", &f("f"), Change::Untouched, true), Exempt::Holds, "function scope: the file's edit is not this function's");
        assert_eq!(exemption(&refs, &p, "a.py", &f("f"), Change::Edited, true), Exempt::Revoked { warn: true });
        let forever = parse("src/** forever: generated");
        let refs: Vec<&IgnoreEntry> = forever.iter().collect();
        assert_eq!(exemption(&refs, &p, "src/x/a.ts", &f("f"), Change::Edited, true), Exempt::Holds, "forever is never revoked");
        let keep = parse("a.py#f on_touch=keep expires=2099-01-01: why");
        let refs: Vec<&IgnoreEntry> = keep.iter().collect();
        assert_eq!(exemption(&refs, &p, "a.py", &f("f"), Change::Edited, true), Exempt::Holds);
    }

    #[test]
    fn suggested_test_names() {
        assert_eq!(test_name("src/orders.ts", Language::Typescript), "orders.test.ts");
        assert_eq!(test_name("app/billing.py", Language::Python), "test_billing.py");
        assert_eq!(bare("f#2"), "f");
    }

    #[test]
    fn code_lines() {
        assert!(not_code("   "));
        assert!(not_code("  // x"));
        assert!(not_code("  # x"));
        assert!(not_code(" * jsdoc"));
        assert!(!not_code("  return x;"));
    }
}

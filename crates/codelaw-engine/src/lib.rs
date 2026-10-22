//! The check and fix pipelines (SR-ARCHITECTURE §B). Everything is a
//! library: the CLI and every other surface only translate between their
//! protocol and these functions (decision 2).

mod adapters;
mod baseline;
mod compile;
mod debug;
mod discover;
mod files;
mod fix;
mod format;
mod git;
mod incremental;
mod lint;
mod query;
mod project;
mod suppress;
mod timings;

pub use baseline::{AsOf, BaselineAdd, Ratchet};
pub use compile::{CompiledRule, Matcher};
pub use discover::FileEntry;
pub use fix::{FileFix, FileTrace, FixOptions, FixReport};
pub use incremental::{DocumentTree, TreeCache};
pub use format::{FormatReport, FormatRunOptions, FormattedFile, Formatting};
pub use lint::FileResult;
pub use query::{RuleRow, RuleWhy, WhyForFile, severity_name};
pub use debug::MatchReport;
pub use timings::{PhaseTiming, RuleTiming, TimingReport};

use codelaw_cache::DiskCache;
use codelaw_config::{Config, ConfigError};
use codelaw_core::{Diagnostic, Language, Skipped};
use codelaw_rules::{LoadError, LoadOptions, PackSearch, RuleSet};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// Everything a run takes from outside the files: environment and flags
/// (SR-CONFIG decision 4). Nothing else is read — no clock, no network.
#[derive(Clone, Debug, Default)]
pub struct EngineOptions {
    /// `--lang`: only these languages, intersected with config.
    pub languages: Option<BTreeSet<Language>>,
    /// `--rule`: only these rules.
    pub rules: Option<Vec<String>>,
    /// `--threads`; `None` is the CPU count.
    pub threads: Option<usize>,
    /// `--no-cache`.
    pub no_cache: bool,
    /// `--timings`: measure each phase and rule.
    pub timings: bool,
    /// `CODELAW_CACHE_DIR`.
    pub cache_dir_env: Option<String>,
    /// `CODELAW_SITE_PACKAGES`, set by the Python launcher.
    pub site_packages_env: Option<String>,
    /// `VIRTUAL_ENV`.
    pub virtual_env: Option<String>,
    /// Run the installed adapters (SR-LINTER decision 2). Only the CLI's
    /// `check` and `fix` set it, never hook, editor or MCP mode
    /// (SR-ARCHITECTURE decision 19); `--no-adapters` clears it.
    pub adapters: bool,
    /// `PATH`, where an adapter is looked for after the project's
    /// `node_modules/.bin` and `.venv/bin`.
    pub path_env: Option<std::ffi::OsString>,
    /// `--as-of`: the date deadlines are judged on (SR-BASELINE decision 4).
    pub as_of: Option<String>,
    /// `CODELAW_AS_OF`, used when `--as-of` is not given.
    pub as_of_env: Option<String>,
    /// `--changed-since <ref>`: the files changed since the merge base
    /// count as touched (SR-BASELINE decision 3).
    pub changed_since: Option<String>,
    /// `--staged`: the files in the index count as touched.
    pub staged: bool,
    /// Files that count as touched whatever the change set says: the
    /// files an agent hook was given.
    pub touched_paths: Vec<PathBuf>,
}

impl EngineOptions {
    /// The options the environment gives, before any flag.
    pub fn from_env() -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        EngineOptions {
            cache_dir_env: var("CODELAW_CACHE_DIR"),
            site_packages_env: var("CODELAW_SITE_PACKAGES"),
            virtual_env: var("VIRTUAL_ENV"),
            path_env: std::env::var_os("PATH"),
            as_of_env: var("CODELAW_AS_OF"),
            ..Default::default()
        }
    }
}

/// Why a run could not start: exit 2.
#[derive(Debug)]
pub enum EngineError {
    Config(ConfigError),
    Load(Vec<LoadError>),
    Usage(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Config(e) => write!(f, "{e}"),
            EngineError::Load(errors) => {
                for (i, e) in errors.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{e}")?;
                }
                Ok(())
            }
            EngineError::Usage(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<ConfigError> for EngineError {
    fn from(e: ConfigError) -> Self {
        EngineError::Config(e)
    }
}

/// The outcome of `check`.
#[derive(Clone, Debug, Default)]
pub struct CheckReport {
    /// Sorted by path, line, column, rule (L5 decision 2).
    pub diagnostics: Vec<Diagnostic>,
    pub skipped: Vec<Skipped>,
    pub files_checked: usize,
    pub warnings: Vec<String>,
    /// The as-of line (`as-of 2026-10-18 (HEAD commit date)`), when a
    /// baseline or a deprecation was in play (SR-BASELINE decision 4).
    pub as_of: Option<AsOf>,
}

impl CheckReport {
    /// Rule failures (`codelaw/internal-error`) make the exit code 2.
    pub fn internal_errors(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.rule == "codelaw/internal-error").count()
    }
}

/// One diagnostic per rule and range, after sorting: a region and its host
/// can report the same finding. `check` and `fix` dedupe alike.
pub(crate) fn dedup_diagnostics(d: &mut Vec<Diagnostic>) {
    d.dedup_by(|a, b| a.file == b.file && a.start_byte == b.start_byte && a.end_byte == b.end_byte && a.rule == b.rule);
}

/// A loaded project: config, rule set and compiled rules.
pub struct Project {
    pub config: Config,
    pub rules: RuleSet,
    /// Shared with every other load of the same inputs in this process
    /// (SR-ARCHITECTURE decision 21).
    pub compiled: std::sync::Arc<Vec<CompiledRule>>,
    pub options: EngineOptions,
    pub cache: DiskCache,
    pub warnings: Vec<String>,
    /// `[format]` (SR-FORMATTER decision 5).
    pub formatting: Formatting,
    pool: rayon::ThreadPool,
    timings: Option<timings::Timings>,
    /// The adapters found installed, looked up on first use.
    adapters: std::sync::OnceLock<codelaw_adapters::Resolved>,
    /// `.codelaw/baseline/*.toml`, read at load (SR-BASELINE).
    pub baselines: codelaw_baseline::Baselines,
    /// The deprecations registry and `[deprecation]`, when the project
    /// runs `graph/no-deprecated-usage`.
    deprecations: Option<baseline::Deprecations>,
    /// The as-of date and the change set, read on first use.
    as_of: std::sync::OnceLock<AsOf>,
    touched: std::sync::OnceLock<(BTreeSet<String>, Vec<String>)>,
}

fn pool(threads: Option<usize>) -> rayon::ThreadPool {
    // Deeply nested code recurses deeply; give workers room.
    let mut b = rayon::ThreadPoolBuilder::new().stack_size(64 << 20);
    if let Some(n) = threads.filter(|n| *n > 0) {
        b = b.num_threads(n);
    }
    b.build().expect("a thread pool")
}

impl Project {
    /// Load the project that governs `start` (a folder or a file).
    pub fn load(start: &Path, options: EngineOptions) -> Result<Project, EngineError> {
        let config = codelaw_config::find_config(start)?;
        Project::from_config(config, options)
    }

    pub fn from_config(config: Config, options: EngineOptions) -> Result<Project, EngineError> {
        let search = PackSearch::new(
            config.config_dir(),
            &config.root_dir,
            options.site_packages_env.as_deref(),
            options.virtual_env.as_deref(),
        );
        let known = |id: &str| codelaw_native::lookup(id).is_some();
        let rules = RuleSet::load(&config, &LoadOptions { search, known_native: &known }).map_err(EngineError::Load)?;
        Project::from_parts(config, rules, options)
    }

    /// A project from a rule set built elsewhere (the rule tester).
    pub fn from_parts(mut config: Config, rules: RuleSet, options: EngineOptions) -> Result<Project, EngineError> {
        baseline::check_dates(&options)?;
        if let Some(only) = &options.rules {
            for id in only {
                if rules.get(id).is_none() && codelaw_rules::engine_rule(id).is_none() {
                    let near = codelaw_config::did_you_mean(id, rules.rules.iter().map(|r| r.rule.id.as_str()))
                        .map(|n| format!("; did you mean `{n}`?"))
                        .unwrap_or_default();
                    return Err(EngineError::Usage(format!("unknown rule `{id}`{near}")));
                }
            }
        }
        let compile_started = std::time::Instant::now();
        let (compiled, mut warnings) = compile::compile_cached(&rules, &config).map_err(EngineError::Load)?;
        let timings = options.timings.then(|| timings::Timings::new(compiled.len()));
        if let Some(t) = &timings {
            t.add(timings::Phase::Compile, compile_started);
        }
        warnings.append(&mut config.warnings);
        warnings.extend(rules.warnings.iter().cloned());
        let cache = if options.no_cache || !config.cache_enabled() {
            DiskCache::disabled()
        } else {
            DiskCache::new(Some(config.cache_dir(options.cache_dir_env.as_deref())))
        };
        let formatting = Formatting::from_config(&config)?;
        // `[adapters] enabled` names only adapters codelaw has, on every
        // surface, whether or not this one runs them.
        if let Err(e) = codelaw_adapters::selected(config.adapters_enabled()) {
            return Err(EngineError::Config(codelaw_config::ConfigError::new(config.files.first().map(|f| f.path.clone()), e)));
        }
        let (baselines, deprecations) = baseline::load(&config)?;
        let pool = pool(options.threads);
        let adapters = std::sync::OnceLock::new();
        Ok(Project {
            config,
            rules,
            compiled,
            options,
            cache,
            warnings,
            formatting,
            pool,
            timings,
            adapters,
            baselines,
            deprecations,
            as_of: std::sync::OnceLock::new(),
            touched: std::sync::OnceLock::new(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.config.root_dir
    }

    /// Check files and folders (the project root when `paths` is empty).
    pub fn check(&mut self, paths: &[PathBuf]) -> Result<CheckReport, EngineError> {
        let (mut report, deprecated, ran) = self.check_raw(paths)?;
        self.apply_baselines(&mut report, deprecated, &ran);
        report.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        dedup_diagnostics(&mut report.diagnostics);
        Ok(report)
    }

    /// `check` before the baseline: every finding, the uses of deprecated
    /// symbols apart, and what the run saw completely.
    pub(crate) fn check_raw(&mut self, paths: &[PathBuf]) -> Result<(CheckReport, Vec<codelaw_baseline::Occurrence>, baseline::Ran), EngineError> {
        let discovering = self.timings.is_some().then(std::time::Instant::now);
        let (files, mut skipped, warnings) = discover::discover(self, paths)?;
        if let (Some(t), Some(start)) = (&self.timings, discovering) {
            t.add(timings::Phase::Discover, start);
        }
        let mut report = CheckReport { warnings: self.warnings.clone(), ..Default::default() };
        report.warnings.extend(warnings);
        let this = &*self;
        let results: Vec<FileResult> = self.pool.install(|| files.par_iter().map(|f| this.check_file(f)).collect());
        let mut checked: Vec<&FileEntry> = Vec::new();
        for (r, entry) in results.into_iter().zip(&files) {
            report.diagnostics.extend(r.diagnostics);
            if let Some(s) = r.skipped {
                skipped.push(s);
            } else {
                report.files_checked += 1;
                checked.push(entry);
            }
        }
        if !self.rules.has_rules() {
            report.diagnostics.push(lint::no_rules(self));
        }
        let scopes = project::scopes(self.root(), paths);
        // Project rules see the whole root, and report inside `paths`.
        let graph_rules = !self.graph_rules().is_empty();
        let deprecations = self.deprecations_on();
        let mut deprecated = Vec::new();
        if graph_rules || deprecations {
            let config = self.graph_config()?;
            let sources = self.graph_sources()?;
            let graph = self.build_graph(&sources, config);
            report.diagnostics.extend(self.run_graph_rules_on(&graph, &sources).into_iter().filter(|d| project::within(&d.file, &scopes)));
            if deprecations {
                deprecated = self.deprecated_occurrences(&graph, &sources).into_iter().filter(|o| project::within(&o.diag.file, &scopes)).collect();
            }
        }
        // The adapters (SR-ARCHITECTURE decision 4, step 5).
        if self.adapters_on() {
            let mut run = adapters::check(self, &checked, &report.diagnostics);
            report.diagnostics.append(&mut run.diagnostics);
            report.warnings.append(&mut run.warnings);
        }
        let ran = baseline::Ran { files: checked.iter().map(|e| e.rel.clone()).collect(), scopes, project: true };
        report.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        dedup_diagnostics(&mut report.diagnostics);
        skipped.sort();
        report.skipped = skipped;
        self.cache.prune(self.config.cache_max_size());
        Ok((report, deprecated, ran))
    }

    /// Check one file's text as if it were at `rel_path` (MCP
    /// `check_snippet`, the rule tester). A leading byte-order mark is read
    /// as the file's, as `check` reads one on disk.
    pub fn check_source(&self, rel_path: &str, language: Language, source: &str) -> Vec<Diagnostic> {
        self.check_source_in(rel_path, language, source, files::Tree::Disk)
    }

    /// `check_source`, with native rules seeing the files of `tree`.
    pub(crate) fn check_source_in(&self, rel_path: &str, language: Language, source: &str, tree: files::Tree<'_>) -> Vec<Diagnostic> {
        let entry = FileEntry { abs: self.config.root_dir.join(rel_path), rel: rel_path.to_owned(), language };
        let (text, bom) = match source.strip_prefix('\u{feff}') {
            Some(t) => (t, true),
            None => (source, false),
        };
        let mode = lint::RunMode { tree, ..lint::RunMode::check().with_bom(bom) };
        let mut d = lint::lint_text(self, &entry, text, &mode).diagnostics;
        if bom {
            lint::shift_for_bom(&mut d);
        }
        if !self.rules.has_rules() {
            d.push(lint::no_rules(self));
        }
        d.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        d
    }

    /// Fix one file's text in memory; nothing is written. A leading
    /// byte-order mark is kept, as `fix` keeps one on disk.
    pub fn fix_source(&self, rel_path: &str, language: Language, source: &str, unsafe_fixes: bool) -> FileFix {
        self.fix_source_in(rel_path, language, source, unsafe_fixes, files::Tree::Disk)
    }

    /// `fix_source`, with native rules seeing the files of `tree`.
    pub(crate) fn fix_source_in(&self, rel_path: &str, language: Language, source: &str, unsafe_fixes: bool, tree: files::Tree<'_>) -> FileFix {
        let entry = FileEntry { abs: self.config.root_dir.join(rel_path), rel: rel_path.to_owned(), language };
        let (text, bom) = match source.strip_prefix('\u{feff}') {
            Some(t) => (t, true),
            None => (source, false),
        };
        let mut f = fix::fix_text(self, &entry, text, &FixOptions { unsafe_fixes, dry_run: true, trace: false }, false, bom, tree);
        if bom {
            f.original.insert(0, '\u{feff}');
            f.fixed.insert(0, '\u{feff}');
            lint::shift_for_bom(&mut f.remaining);
        }
        f
    }

    /// Fix files and folders and write the results (SR-ARCHITECTURE
    /// decisions 5 and 12).
    pub fn fix(&mut self, paths: &[PathBuf], options: &FixOptions) -> Result<FixReport, EngineError> {
        fix::fix_paths(self, paths, options)
    }

    /// The entry `codelaw check <abs>` would check, or `None` when it would
    /// not check that file: an unknown extension, outside `include`, or a
    /// language turned off. The file need not exist yet (an editor's unsaved
    /// buffer, SR-EDITOR decision 3).
    pub fn entry_for(&self, abs: &Path) -> Option<FileEntry> {
        let language = Language::from_path(abs)?;
        (self.config.is_included(abs) && discover::language_enabled(self, abs, language))
            .then(|| FileEntry { abs: abs.to_path_buf(), rel: discover::rel_path(abs, self.root()), language })
    }

    /// Check an editor's text for `entry` as `codelaw check` checks the
    /// file on disk: the same skips, the same rules, the same order
    /// (SR-EDITOR decision 3). Nothing is read from or written to disk.
    /// `text` has no byte-order mark; offsets are into it.
    pub fn check_document(&self, entry: &FileEntry, text: &str) -> Vec<Diagnostic> {
        self.check_document_with(entry, text, None)
    }

    fn check_document_with(&self, entry: &FileEntry, text: &str, host: Option<&codelaw_lang::Root>) -> Vec<Diagnostic> {
        let mut d = self.check_unsaved(entry, text, host).diagnostics;
        if !self.rules.has_rules() {
            d.push(lint::no_rules(self));
        }
        d.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        d.dedup_by(|a, b| a.file == b.file && a.start_byte == b.start_byte && a.end_byte == b.end_byte && a.rule == b.rule);
        d
    }

    /// Whether `fix` may start an external fixer (decision 18), a command
    /// that inherits the environment of the process running it.
    pub fn runs_external_fixers(&self) -> bool {
        self.compiled.iter().any(|c| c.loaded.external_allowed && matches!(c.fix, Some(codelaw_fix::FixSpec::External(_))))
    }

    /// The files a run would check, in order.
    pub fn files(&mut self, paths: &[PathBuf]) -> Result<Vec<FileEntry>, EngineError> {
        Ok(discover::discover(self, paths)?.0)
    }
}

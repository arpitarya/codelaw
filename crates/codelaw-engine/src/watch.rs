//! Watch mode (W-44, SR-ARCHITECTURE decision 22a): `check --watch` keeps a
//! run's parts — each file's result, the graph's sources and facts, the
//! adapters' findings — and on a change redoes only what the change
//! touched, then puts the report together exactly as `check` does
//! (`assemble` is the one place that does it, for both).
//!
//! What changed is found by polling file metadata (a [`Snapshot`]): no
//! watcher dependency, the same on every platform. A change to the config,
//! a rule, the baseline, an `.editorconfig` or an ignore file reloads the
//! project, as the daemon reads them afresh; a file added or removed
//! re-walks the project; a saved file re-checks that file, the files whose
//! cached answers it changed, and the graph rules over every file.

use crate::adapters::AdapterRun;
use crate::discover::{self, FileEntry};
use crate::files::{Asked, still_holds};
use crate::fix::{FixOptions, FixReport};
use crate::lint::FileResult;
use crate::project::{self, Source};
use crate::{CheckReport, EngineError, EngineOptions, Project, baseline, dedup_diagnostics};
use codelaw_baseline::Occurrence;
use codelaw_core::{Language, Skipped};
use codelaw_graph::{FileFacts, GraphConfig, is_manifest};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// More files than this changed at once (a `git checkout`, a branch
/// switch): one full run rather than a re-check of each.
pub const COALESCE: usize = 256;

/// The graph's part of a run.
pub(crate) struct GraphParts {
    config: GraphConfig,
    sources: Vec<Source>,
    facts: Vec<Option<FileFacts>>,
}

/// What a run computed before the report is put together.
pub(crate) struct Parts {
    files: Vec<FileEntry>,
    results: Vec<FileResult>,
    /// The questions each file's rules asked about other files.
    asked: Vec<Vec<Asked>>,
    /// Discovery's skips and warnings.
    skipped: Vec<Skipped>,
    warnings: Vec<String>,
    graph: Option<GraphParts>,
    /// The adapters' findings, before those a pack rule reports are dropped.
    adapters: Option<AdapterRun>,
}

impl Project {
    /// Discover, check each file, read the graph's sources and run the
    /// adapters (SR-ARCHITECTURE decision 4, steps 2–5).
    pub(crate) fn parts(&mut self, paths: &[PathBuf]) -> Result<Parts, EngineError> {
        let discovering = self.timings.is_some().then(std::time::Instant::now);
        let (files, skipped, warnings) = discover::discover(self, paths)?;
        if let (Some(t), Some(start)) = (&self.timings, discovering) {
            t.add(crate::timings::Phase::Discover, start);
        }
        let this = &*self;
        let checked: Vec<(FileResult, Vec<Asked>)> = self.pool.install(|| files.par_iter().map(|f| this.check_file_asked(f)).collect());
        let (results, asked): (Vec<FileResult>, Vec<Vec<Asked>>) = checked.into_iter().unzip();
        // Project rules see the whole root, and report inside `paths`.
        let graph = if !self.graph_rules().is_empty() || self.deprecations_on() {
            let config = self.graph_config()?;
            let sources = self.graph_sources()?;
            let this = &*self;
            let facts = self.pool.install(|| sources.par_iter().map(|s| this.source_facts(s)).collect());
            Some(GraphParts { config, sources, facts })
        } else {
            None
        };
        let adapters = self.adapters_on().then(|| {
            let ran: Vec<&FileEntry> = files.iter().zip(&results).filter(|(_, r)| r.skipped.is_none()).map(|(f, _)| f).collect();
            self.run_adapters(&ran)
        });
        Ok(Parts { files, results, asked, skipped, warnings, graph, adapters })
    }

    /// The report of a run's parts, before the baseline (steps 4–7).
    pub(crate) fn assemble(&self, parts: &Parts, paths: &[PathBuf]) -> (CheckReport, Vec<Occurrence>, baseline::Ran) {
        let mut report = CheckReport { warnings: self.warnings.clone(), ..Default::default() };
        report.warnings.extend(parts.warnings.iter().cloned());
        let mut skipped = parts.skipped.clone();
        let mut checked: Vec<&FileEntry> = Vec::new();
        for (r, entry) in parts.results.iter().zip(&parts.files) {
            report.diagnostics.extend(r.diagnostics.iter().cloned());
            if let Some(s) = &r.skipped {
                skipped.push(s.clone());
            } else {
                report.files_checked += 1;
                checked.push(entry);
            }
        }
        if !self.rules.has_rules() {
            report.diagnostics.push(crate::lint::no_rules(self));
        }
        let scopes = project::scopes(self.root(), paths);
        let mut deprecated = Vec::new();
        if let Some(g) = &parts.graph {
            let graph = self.link_graph(&g.sources, g.facts.clone(), g.config.clone());
            report.diagnostics.extend(self.run_graph_rules_on(&graph, &g.sources).into_iter().filter(|d| project::within(&d.file, &scopes)));
            if self.deprecations_on() {
                deprecated = self.deprecated_occurrences(&graph, &g.sources).into_iter().filter(|o| project::within(&o.diag.file, &scopes)).collect();
            }
        }
        // The adapters (SR-ARCHITECTURE decision 4, step 5): a finding a
        // pack rule already reports is shown once.
        if let Some(run) = &parts.adapters {
            let mut found = run.diagnostics.clone();
            self.drop_ported(&mut found, &report.diagnostics);
            report.diagnostics.append(&mut found);
            report.warnings.extend(run.warnings.iter().cloned());
        }
        let ran = baseline::Ran { files: checked.iter().map(|e| e.rel.clone()).collect(), scopes, project: true };
        report.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        dedup_diagnostics(&mut report.diagnostics);
        skipped.sort();
        report.skipped = skipped;
        (report, deprecated, ran)
    }

    /// The baseline, `remap` and the profile over an assembled report: the
    /// end of `check`.
    pub(crate) fn conclude(&self, raw: (CheckReport, Vec<Occurrence>, baseline::Ran)) -> CheckReport {
        let (mut report, deprecated, ran) = raw;
        self.apply_baselines(&mut report, deprecated, &ran);
        self.finish(&mut report);
        report.diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        dedup_diagnostics(&mut report.diagnostics);
        report
    }
}

/// What a file's metadata says about it: a change to any of it is a change
/// to the file. An editor that saves by writing a new file and renaming it
/// over the old one changes it too, since the path is watched, not the
/// inode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
    inode: u64,
}

fn stamp(path: &Path) -> Option<Stamp> {
    let m = std::fs::metadata(path).ok()?;
    #[cfg(unix)]
    let inode = std::os::unix::fs::MetadataExt::ino(&m);
    #[cfg(not(unix))]
    let inode = 0;
    Some(Stamp { len: m.len(), modified: m.modified().ok(), inode })
}

/// The names that matter in a folder: files the engine reads or that set
/// how it reads them, and sub-folders it may enter. An editor's swap or
/// temporary file is none of these.
fn names(dir: &Path) -> BTreeSet<OsString> {
    let mut out = BTreeSet::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = e.file_name();
        let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
        let s = name.to_string_lossy();
        let wanted = if is_dir {
            !matches!(s.as_ref(), ".git" | "node_modules" | ".venv")
        } else {
            Language::from_path(Path::new(s.as_ref())).is_some() || INPUT_NAMES.contains(&s.as_ref()) || is_manifest(&s)
        };
        if wanted {
            out.insert(name);
        }
    }
    out
}

/// Files whose change reloads the project wherever they are, as the
/// language server's watched files do (SR-EDITOR decision 2).
const INPUT_NAMES: &[&str] = &["codelaw.toml", "pyproject.toml", "package.json", ".editorconfig", ".gitignore", ".ignore"];

/// Whether a change to `path` reloads the project: the config, a rule, the
/// baseline or the deprecations registry, the format options or an ignore
/// file.
fn reloads(path: &Path, inputs: &BTreeSet<PathBuf>) -> bool {
    if inputs.contains(path) {
        return true;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    INPUT_NAMES.contains(&name)
        || path.components().any(|c| c.as_os_str() == ".codelaw")
        || (name.ends_with(".md") && path.components().any(|c| c.as_os_str() == "rules"))
}

/// The files and folders a watch looks at, and what it last saw of each.
#[derive(Default)]
pub struct Snapshot {
    files: BTreeMap<PathBuf, Option<Stamp>>,
    dirs: BTreeMap<PathBuf, (Option<Stamp>, BTreeSet<OsString>)>,
    /// Files whose change reloads the project.
    inputs: BTreeSet<PathBuf>,
}

/// What a poll saw change.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// Files whose metadata changed, or that are gone.
    pub paths: BTreeSet<PathBuf>,
    /// A folder gained or lost a file or sub-folder that matters: the
    /// project is walked again.
    pub structural: bool,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty() && !self.structural
    }

    pub fn merge(&mut self, other: Changes) {
        self.paths.extend(other.paths);
        self.structural |= other.structural;
    }
}

impl Snapshot {
    /// Every file of the project root the engine reads (the graph's walk,
    /// SR-ARCHITECTURE decision 7), the config and rule files, each
    /// folder's format and ignore files, and the folders that hold them.
    fn build(project: &mut Project) -> Result<Snapshot, EngineError> {
        let root = project.root().to_path_buf();
        let walked = discover::discover_with(project, &[], false)?.0;
        let mut files: BTreeSet<PathBuf> = walked.into_iter().map(|f| f.abs).collect();
        let mut inputs: BTreeSet<PathBuf> = project.config.files.iter().map(|f| f.path.clone()).collect();
        inputs.extend(project.rules.rules.iter().map(|r| r.rule.path.clone()).filter(|p| !p.as_os_str().is_empty()));
        let mut dirs: BTreeSet<PathBuf> = BTreeSet::new();
        dirs.insert(root.clone());
        let rules_dir = project.config.rules_dir();
        if rules_dir.is_dir() {
            dirs.insert(rules_dir);
        }
        for f in files.iter().chain(&inputs) {
            let mut d = f.parent();
            while let Some(dir) = d {
                if !dirs.insert(dir.to_path_buf()) || !dir.starts_with(&root) || dir == root {
                    break;
                }
                d = dir.parent();
            }
        }
        for dir in &dirs {
            for name in [".editorconfig", ".gitignore", ".ignore", "setup.cfg"] {
                let p = dir.join(name);
                if p.is_file() {
                    if name != "setup.cfg" {
                        inputs.insert(p.clone());
                    }
                    files.insert(p);
                }
            }
        }
        files.extend(inputs.iter().cloned());
        let files = files.into_iter().map(|p| (p.clone(), stamp(&p))).collect();
        let dirs = dirs.into_iter().map(|d| (d.clone(), (stamp(&d), names(&d)))).collect();
        Ok(Snapshot { files, dirs, inputs })
    }

    /// Look at every watched file and folder once.
    fn poll(&mut self) -> Changes {
        let mut changes = Changes::default();
        // One thread: in parallel the look is barely sooner and costs over twice
        // the processor time, and a watch mostly waits.
        for (path, seen) in self.files.iter_mut() {
            let now = stamp(path);
            if now != *seen {
                *seen = now;
                changes.structural |= now.is_none();
                changes.paths.insert(path.clone());
            }
        }
        for (dir, (seen, children)) in self.dirs.iter_mut() {
            let now = stamp(dir);
            if now != *seen {
                *seen = now;
                let current = names(dir);
                if current != *children {
                    *children = current;
                    changes.structural = true;
                }
            }
        }
        changes
    }

    /// Take what is on disk now as seen: a write codelaw made itself.
    fn restamp(&mut self, path: &Path) {
        if let Some(seen) = self.files.get_mut(path) {
            *seen = stamp(path);
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// What a re-run did, beside its report.
#[derive(Clone, Debug, Default)]
pub struct Rerun {
    pub report: CheckReport,
    /// The changed files, relative to the project root, sorted.
    pub changed: Vec<String>,
    /// The project was loaded again (config, rules or baseline changed, or
    /// files were added or removed).
    pub reloaded: bool,
}

/// A watch over one project (`codelaw check --watch`, `fix --watch`).
pub struct Watch {
    project: Project,
    /// Where the project was found from, to load it again.
    start: PathBuf,
    paths: Vec<PathBuf>,
    parts: Parts,
    snapshot: Snapshot,
    /// The last reload failed (a config error): the next change reloads,
    /// whatever it is.
    stale: bool,
}

impl Watch {
    /// Load the project that governs `start` and check `paths` once.
    pub fn start(start: &Path, paths: Vec<PathBuf>, options: EngineOptions) -> Result<(Watch, CheckReport), EngineError> {
        Watch::start_with(start, paths, options, None).map(|(w, r, _)| (w, r))
    }

    /// `start`, after fixing `paths` (`fix --watch`'s first run).
    pub fn start_fixing(start: &Path, paths: Vec<PathBuf>, options: EngineOptions, fix: &FixOptions) -> Result<(Watch, CheckReport, FixReport), EngineError> {
        Watch::start_with(start, paths, options, Some(fix)).map(|(w, r, f)| (w, r, f.unwrap_or_default()))
    }

    fn start_with(start: &Path, paths: Vec<PathBuf>, options: EngineOptions, fix: Option<&FixOptions>) -> Result<(Watch, CheckReport, Option<FixReport>), EngineError> {
        let mut project = Project::load(start, options)?;
        let fixed = match fix {
            Some(o) => Some(project.fix(&paths, o)?),
            None => None,
        };
        // Seen before the run reads anything: a change during the run is
        // seen by the next poll.
        let snapshot = Snapshot::build(&mut project)?;
        let parts = project.parts(&paths)?;
        let report = project.conclude(project.assemble(&parts, &paths));
        project.cache.prune(project.config.cache_max_size());
        let watch = Watch { project, start: start.to_path_buf(), paths, parts, snapshot, stale: false };
        Ok((watch, report, fixed))
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    /// How many files the watch looks at.
    pub fn watched(&self) -> usize {
        self.snapshot.len()
    }

    /// Look once for changes; what is seen is not seen again.
    pub fn poll(&mut self) -> Changes {
        self.snapshot.poll()
    }

    /// Check again after `changes`. `None` when nothing the engine reads
    /// changed (a folder gained a file it does not read). An error (a
    /// config that no longer loads) keeps the last good state, and the next
    /// change loads the project again.
    pub fn rerun(&mut self, changes: &Changes) -> Result<Option<Rerun>, EngineError> {
        let root = self.project.root().to_path_buf();
        let mut changed: BTreeSet<PathBuf> = changes.paths.clone();
        let mut reload = self.stale || changes.paths.len() > COALESCE || changes.paths.iter().any(|p| reloads(p, &self.snapshot.inputs));
        if changes.structural && !reload {
            let fresh = Snapshot::build(&mut self.project)?;
            if fresh.files.keys().ne(self.snapshot.files.keys()) || fresh.inputs != self.snapshot.inputs {
                reload = true;
            } else {
                // The same files: those whose metadata moved changed (one
                // removed and written again by a save, say).
                for (p, s) in &fresh.files {
                    if self.snapshot.files.get(p) != Some(s) {
                        changed.insert(p.clone());
                    }
                }
                self.snapshot = fresh;
            }
        }
        let rels: Vec<String> = changed.iter().map(|p| discover::rel_path(p, &root)).collect();
        if reload {
            let project = match Project::load(&self.start, self.project.options.clone()) {
                Ok(p) => p,
                Err(e) => {
                    self.stale = true;
                    return Err(e);
                }
            };
            self.project = project;
            self.stale = false;
            let report = self.full()?;
            return Ok(Some(Rerun { report, changed: rels, reloaded: true }));
        }
        if changed.is_empty() {
            return Ok(None);
        }
        let report = self.incremental(&rels)?;
        Ok(Some(Rerun { report, changed: rels, reloaded: false }))
    }

    /// Fix the changed files the run checks, take the writes as seen, then
    /// check again (`fix --watch`). A change that reloads the project is
    /// checked without fixing.
    pub fn fix(&mut self, changes: &Changes, options: &FixOptions) -> Result<Option<(Rerun, FixReport)>, EngineError> {
        let mut report = FixReport::default();
        let reloading = self.stale || changes.structural || changes.paths.len() > COALESCE || changes.paths.iter().any(|p| reloads(p, &self.snapshot.inputs));
        if !reloading {
            let checked: BTreeSet<&Path> = self.parts.files.iter().map(|f| f.abs.as_path()).collect();
            let targets: Vec<PathBuf> = changes.paths.iter().filter(|p| checked.contains(p.as_path()) && p.is_file()).cloned().collect();
            if !targets.is_empty() {
                report = self.project.fix(&targets, options)?;
                let root = self.project.root().to_path_buf();
                for f in report.files.iter().filter(|f| f.written) {
                    self.snapshot.restamp(&root.join(&f.file));
                }
            }
        }
        Ok(self.rerun(changes)?.map(|r| (r, report)))
    }

    /// Walk and check everything again, with the snapshot taken first.
    fn full(&mut self) -> Result<CheckReport, EngineError> {
        self.project.touched = std::sync::OnceLock::new();
        self.snapshot = Snapshot::build(&mut self.project)?;
        let parts = self.project.parts(&self.paths)?;
        let report = self.project.conclude(self.project.assemble(&parts, &self.paths));
        self.parts = parts;
        self.project.cache.prune(self.project.config.cache_max_size());
        Ok(report)
    }

    /// Re-check the changed files and every file whose rules' answers they
    /// changed, re-read their graph facts, then put the report together.
    fn incremental(&mut self, rels: &[String]) -> Result<CheckReport, EngineError> {
        let changed: BTreeSet<&str> = rels.iter().map(String::as_str).collect();
        let project = &self.project;
        let root = project.root().to_path_buf();
        let parts = &mut self.parts;
        // A changed file the graph reads but did not have (it could not be
        // read before): walk again.
        if let Some(g) = &parts.graph {
            let known: BTreeSet<&str> = g.sources.iter().map(|s| s.rel.as_str()).collect();
            let walked: BTreeSet<String> = self.snapshot.files.keys().filter(|p| Language::from_path(p).is_some()).map(|p| discover::rel_path(p, &root)).collect();
            if changed.iter().any(|r| !known.contains(r) && walked.contains(*r)) {
                return self.full();
            }
        }
        let redo: Vec<usize> = (0..parts.files.len())
            .filter(|&i| changed.contains(parts.files[i].rel.as_str()) || (!parts.asked[i].is_empty() && !still_holds(&root, &parts.asked[i], &|| project.as_of_text())))
            .collect();
        let files = &parts.files;
        let fresh: Vec<(FileResult, Vec<Asked>)> = project.pool.install(|| redo.par_iter().map(|&i| project.check_file_asked(&files[i])).collect());
        for (&i, (result, asked)) in redo.iter().zip(fresh) {
            parts.results[i] = result;
            parts.asked[i] = asked;
        }
        if let Some(g) = &mut parts.graph {
            let mut gone: BTreeSet<usize> = BTreeSet::new();
            let mut read: Vec<usize> = Vec::new();
            for (i, s) in g.sources.iter_mut().enumerate() {
                if !changed.contains(s.rel.as_str()) {
                    continue;
                }
                let name = s.rel.rsplit('/').next().unwrap_or_default();
                if let Some(language) = s.language {
                    match project.read_entry(&FileEntry { abs: s.abs.clone(), rel: s.rel.clone(), language }) {
                        Ok(f) => s.text = f.text,
                        Err(_) => {
                            gone.insert(i);
                            continue;
                        }
                    }
                } else if is_manifest(name) {
                    s.text = std::fs::read_to_string(&s.abs).unwrap_or_default();
                }
                read.push(i);
            }
            let sources = &g.sources;
            let facts: Vec<Option<FileFacts>> = project.pool.install(|| read.par_iter().map(|&i| project.source_facts(&sources[i])).collect());
            for (&i, f) in read.iter().zip(facts) {
                g.facts[i] = f;
            }
            if !gone.is_empty() {
                let mut i = 0;
                g.sources.retain(|_| {
                    i += 1;
                    !gone.contains(&(i - 1))
                });
                let mut i = 0;
                g.facts.retain(|_| {
                    i += 1;
                    !gone.contains(&(i - 1))
                });
            }
        }
        if let Some(run) = &mut parts.adapters {
            let again: Vec<&FileEntry> = parts.files.iter().zip(&parts.results).filter(|(f, r)| changed.contains(f.rel.as_str()) && r.skipped.is_none()).map(|(f, _)| f).collect();
            run.diagnostics.retain(|d| !changed.contains(d.file.as_str()));
            if !again.is_empty() {
                let more = project.run_adapters(&again);
                run.diagnostics.extend(more.diagnostics);
                for w in more.warnings {
                    if !run.warnings.contains(&w) {
                        run.warnings.push(w);
                    }
                }
            }
        }
        // `--changed-since` and `--staged` are asked again: a save changes
        // what git reports as changed.
        self.project.touched = std::sync::OnceLock::new();
        let report = self.project.conclude(self.project.assemble(&self.parts, &self.paths));
        Ok(report)
    }
}


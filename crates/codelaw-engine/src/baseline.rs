//! The baseline stage (SR-ARCHITECTURE decision 4, step 6; SR-BASELINE):
//! the as-of date and the change set as explicit inputs (L5 decision 2),
//! the engine's own `graph/no-deprecated-usage`, and the baseline applied
//! to every finding — plus what `baseline add`, `prune`, `check` and
//! `deprecate` do.

use crate::lint::{meta_diagnostic, meta_severity};
use crate::project::{Source, suppressed, suppressions};
use crate::{CheckReport, EngineError, Project, git};
use codelaw_baseline::{AddOptions, BaselineFile, Baselines, Context, DEPRECATED_RULE, Date, Deprecation, OnTouch, Occurrence, Policy, Registry, STALE_RULE};
use codelaw_core::{Diagnostic, FixClass, LineIndex};
use codelaw_graph::Graph;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The as-of date of a run and where it came from (SR-BASELINE decision 4).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AsOf {
    pub date: Option<Date>,
    /// `HEAD commit date`, `--as-of` or `CODELAW_AS_OF`; when there is no
    /// date, why.
    pub source: String,
}

impl AsOf {
    /// The report header's line: `as-of 2026-10-18 (HEAD commit date)`.
    pub fn line(&self) -> String {
        match self.date {
            Some(d) => format!("as-of {d} ({})", self.source),
            None => format!("as-of: none ({}); deadlines are not evaluated", self.source),
        }
    }
}

/// The deprecations a project declares.
#[derive(Clone, Debug, Default)]
pub(crate) struct Deprecations {
    pub registry: Registry,
    /// `[deprecation]`: the policy of a symbol marked in code only.
    pub defaults: Policy,
}

/// What a run saw completely, so a count below an entry's is a stale
/// entry and not a partial run.
#[derive(Clone, Debug, Default)]
pub(crate) struct Ran {
    /// Project-relative files checked.
    pub files: BTreeSet<String>,
    /// The checked paths, project-relative; empty is the whole root.
    pub scopes: Vec<String>,
    /// The project rules ran.
    pub project: bool,
}

fn config_error(project_root_file: Option<&Path>, e: String) -> EngineError {
    EngineError::Config(codelaw_config::ConfigError::new(project_root_file.map(Path::to_path_buf), e))
}

/// The baselines and deprecations of a project root, read at load.
pub(crate) fn load(config: &codelaw_config::Config) -> Result<(Baselines, Option<Deprecations>), EngineError> {
    let root = &config.root_dir;
    let baselines = Baselines::load(root).map_err(|e| config_error(None, e))?;
    let registry = Registry::load(root).map_err(|e| config_error(None, e))?;
    let table = config.files.first().and_then(|f| f.raw.deprecation.as_ref());
    let defaults = match table {
        Some(d) => codelaw_baseline::registry::defaults(d.until.as_deref(), d.warn_from.as_deref(), d.on_touch.as_deref()),
        None => Ok(Policy::default()),
    }
    .map_err(|e| config_error(config.root_file(), e))?;
    let named = config.named_rule_ids().iter().any(|(id, _)| id == DEPRECATED_RULE);
    // The rule needs the whole graph, so it runs only when the project
    // asks for it: a registry, a `[deprecation]` table or the rule named.
    let on = registry.is_some() || table.is_some() || named;
    Ok((baselines, on.then(|| Deprecations { registry: registry.unwrap_or_default(), defaults })))
}

/// A date flag or variable, checked at load: a bad one is a usage error.
pub(crate) fn check_dates(options: &crate::EngineOptions) -> Result<(), EngineError> {
    if let Some(d) = &options.as_of {
        Date::parse(d).map_err(|e| EngineError::Usage(format!("--as-of: {e}")))?;
    }
    if let Some(d) = &options.as_of_env {
        Date::parse(d).map_err(|e| EngineError::Usage(format!("CODELAW_AS_OF: {e}")))?;
    }
    Ok(())
}

fn short(symbol: &str) -> &str {
    symbol.rsplit(['#', ':']).next().unwrap_or(symbol)
}

impl Project {
    /// The as-of date: `--as-of`, then `CODELAW_AS_OF`, then `HEAD`'s
    /// commit date. Never the clock (L5 decision 2).
    pub fn as_of(&self) -> &AsOf {
        self.as_of.get_or_init(|| {
            if let Some(d) = self.options.as_of.as_deref().and_then(|d| Date::parse(d).ok()) {
                return AsOf { date: Some(d), source: "--as-of".into() };
            }
            if let Some(d) = self.options.as_of_env.as_deref().and_then(|d| Date::parse(d).ok()) {
                return AsOf { date: Some(d), source: "CODELAW_AS_OF".into() };
            }
            match git::head_date(self.root()).and_then(|d| Date::parse(&d)) {
                Ok(d) => AsOf { date: Some(d), source: "HEAD commit date".into() },
                Err(why) => AsOf { date: None, source: format!("{why}; pass --as-of YYYY-MM-DD or set CODELAW_AS_OF") },
            }
        })
    }

    /// The change set, project-relative (SR-BASELINE decision 3), and what
    /// could not be read. Nothing given: nothing is touched.
    pub fn touched(&self) -> &(BTreeSet<String>, Vec<String>) {
        self.touched.get_or_init(|| {
            let mut set = BTreeSet::new();
            let mut warnings = Vec::new();
            let root = self.root();
            if let Some(r) = &self.options.changed_since {
                match git::changed_since(root, r) {
                    Ok(s) => set.extend(s),
                    Err(e) => warnings.push(format!("--changed-since {r}: {e}; it marks no file as touched, and the rest of the run is unaffected")),
                }
            }
            if self.options.staged {
                match git::staged(root) {
                    Ok(s) => set.extend(s),
                    Err(e) => warnings.push(format!("--staged: {e}; it marks no file as touched, and the rest of the run is unaffected")),
                }
            }
            for p in &self.options.touched_paths {
                let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
                set.insert(crate::discover::rel_path(&abs, root));
            }
            (set, warnings)
        })
    }

    /// Whether `graph/no-deprecated-usage` runs in this run (`--rule`
    /// applied).
    pub(crate) fn deprecations_on(&self) -> bool {
        self.deprecations.is_some() && self.options.rules.as_ref().is_none_or(|o| o.iter().any(|r| r == DEPRECATED_RULE))
    }

    /// Every use of a deprecated symbol in the reported sources: the
    /// registry's, then the in-code markers the registry does not list.
    pub(crate) fn deprecated_occurrences(&self, graph: &Graph, sources: &[Source]) -> Vec<Occurrence> {
        let Some(dep) = &self.deprecations else { return Vec::new() };
        let mut found: Vec<(codelaw_graph::DeprecatedUse, Option<&Deprecation>, Policy, Option<String>)> = Vec::new();
        let mut listed: BTreeSet<(String, String)> = BTreeSet::new();
        for d in &dep.registry.deprecated {
            let uses = match codelaw_graph::deprecated::declared(graph, &d.symbol) {
                Some((module, name)) => {
                    let u = codelaw_graph::deprecated::uses_of_decl(graph, &module, &name, &d.symbol);
                    listed.insert((module, name));
                    u
                }
                None => codelaw_graph::deprecated::uses_of_external(graph, &d.symbol),
            };
            found.extend(uses.into_iter().map(|u| (u, Some(d), d.policy.clone(), d.reason.clone())));
        }
        for m in codelaw_graph::deprecated::marked(graph) {
            if listed.contains(&(m.module.clone(), m.name.clone())) {
                continue;
            }
            let reason = Some(m.message.clone()).filter(|r| !r.is_empty());
            found.extend(codelaw_graph::deprecated::uses_of_decl(graph, &m.module, &m.name, &m.symbol).into_iter().map(|u| (u, None, dep.defaults.clone(), reason.clone())));
        }
        let by_rel: BTreeMap<&str, &Source> = sources.iter().map(|s| (s.rel.as_str(), s)).collect();
        let mut per_file: BTreeMap<&str, Vec<(Diagnostic, String, Policy)>> = BTreeMap::new();
        for (u, d, policy, reason) in found {
            let Some(src) = by_rel.get(u.file.as_str()).copied().filter(|s| s.reported) else { continue };
            let layers = self.config.layers_for(&src.abs);
            let Some(severity) = meta_severity(&layers, DEPRECATED_RULE) else { continue };
            let replacement = d.and_then(|d| d.replacement.as_deref());
            let mut message = format!("`{}` is deprecated", short(&u.symbol));
            if let Some(r) = replacement {
                message.push_str(&format!("; use `{}`", short(r)));
            }
            if let Some(until) = policy.until {
                message.push_str(&format!(" (deadline {until})"));
            }
            let mut steps = vec![match replacement {
                Some(r) => format!("Replace `{}` with `{r}`.", u.symbol),
                None => format!("Replace this use of `{}`, which is deprecated.", u.symbol),
            }];
            if let Some(r) = reason {
                steps.push(format!("Why: {r}"));
            }
            let index = LineIndex::new(&src.text);
            let diag = Diagnostic {
                rule: DEPRECATED_RULE.to_owned(),
                severity,
                file: u.file.clone(),
                start_byte: u.span.0,
                end_byte: u.span.1,
                start: index.position(u.span.0),
                end: index.position(u.span.1),
                message,
                fix: FixClass::Manual,
                edits: Vec::new(),
                steps,
                source: "engine".into(),
                note: None,
            };
            per_file.entry(src.rel.as_str()).or_default().push((diag, u.symbol.clone(), policy));
        }
        let mut out = Vec::new();
        for (file, list) in per_file {
            let sups = suppressions(by_rel[file]);
            for (diag, symbol, policy) in list {
                if !suppressed(&sups, &diag) {
                    out.push(Occurrence { diag, symbol: Some(symbol), policy });
                }
            }
        }
        out
    }

    /// Whether a rule's findings in a file are complete in this run.
    fn evaluated(&self, ran: &Ran, rule: &str, file: &str) -> bool {
        let only = self.options.rules.as_ref();
        let ran_rule = if rule == DEPRECATED_RULE {
            ran.project && self.deprecations_on()
        } else {
            match self.compiled.iter().find(|c| c.id() == rule) {
                Some(c) if matches!(c.matcher, crate::Matcher::Graph(_)) => ran.project && self.graph_rules().iter().any(|(_, r)| r.id() == rule),
                Some(_) => only.is_none_or(|o| o.iter().any(|r| r == rule)),
                None => false,
            }
        };
        ran_rule && crate::project::within(file, &ran.scopes) && (ran.files.contains(file) || !self.root().join(file).exists())
    }

    /// Apply the baselines to a report (SR-BASELINE decision 2): the
    /// `deprecated` uses join the report here, and stale entries are
    /// reported under `codelaw/stale-baseline`.
    pub(crate) fn apply_baselines(&self, report: &mut CheckReport, deprecated: Vec<Occurrence>, ran: &Ran) {
        let (_, touch_warnings) = self.touched();
        report.warnings.extend(touch_warnings.iter().cloned());
        let in_play = !self.baselines.is_empty() || (ran.project && self.deprecations_on());
        if !in_play {
            report.diagnostics.extend(deprecated.into_iter().map(|o| o.diag));
            return;
        }
        let as_of = self.as_of().clone();
        report.as_of = Some(as_of.clone());
        let all = std::mem::take(&mut report.diagnostics);
        let (mine, rest): (Vec<Diagnostic>, Vec<Diagnostic>) = all.into_iter().partition(|d| self.baselines.rules.contains_key(&d.rule));
        let mut occurrences: Vec<Occurrence> = mine.into_iter().map(Occurrence::plain).collect();
        occurrences.extend(deprecated);
        let evaluated = |rule: &str, file: &str| self.evaluated(ran, rule, file);
        let ctx = Context { as_of: as_of.date, touched: &self.touched().0, evaluated: &evaluated };
        let outcome = codelaw_baseline::apply(occurrences, &self.baselines, &ctx);
        report.diagnostics = rest;
        report.diagnostics.extend(outcome.diagnostics);
        for s in outcome.stale {
            let abs = self.root().join(&s.file);
            let layers = self.config.layers_for(&abs);
            let what = s.symbol.as_deref().map(|x| format!(" uses of `{x}`")).unwrap_or_default();
            let message = format!(
                "The baseline of `{}` allows {}{what} in this file, which now has {}: run `codelaw baseline prune` (or `codelaw fix`) to lower it",
                s.rule, s.allowed, s.found
            );
            report.diagnostics.extend(meta_diagnostic(&s.file, &layers, STALE_RULE, message));
        }
    }

    /// The raw counts of a run, per rule, file and symbol, for `add` and
    /// `prune`: `check` without the baseline applied.
    pub(crate) fn counts(&mut self, paths: &[PathBuf]) -> Result<(codelaw_baseline::apply::Counts, Ran), EngineError> {
        let (report, deprecated, ran) = self.check_raw(paths)?;
        let mut occ: Vec<Occurrence> = report.diagnostics.into_iter().map(Occurrence::plain).collect();
        occ.extend(deprecated);
        Ok((codelaw_baseline::counts(&occ), ran))
    }

    /// Lower every entry to what the files now hold (SR-BASELINE decision
    /// 5): what `codelaw baseline prune` and `codelaw fix` do. The
    /// baseline files changed, project-relative.
    pub fn baseline_prune(&mut self, paths: &[PathBuf]) -> Result<Vec<String>, EngineError> {
        if self.baselines.is_empty() {
            return Ok(Vec::new());
        }
        let (counts, ran) = self.counts(paths)?;
        self.prune_with(&counts, &ran)
    }

    pub(crate) fn prune_with(&mut self, counts: &codelaw_baseline::apply::Counts, ran: &Ran) -> Result<Vec<String>, EngineError> {
        let evaluated = |rule: &str, file: &str| self.evaluated(ran, rule, file);
        let changed = codelaw_baseline::prune(&self.baselines, counts, &evaluated);
        let mut out = Vec::new();
        for b in changed {
            Baselines::save(self.root(), &b).map_err(|e| EngineError::Usage(format!("{}: {e}", codelaw_baseline::file::rel_path(&b.rule))))?;
            out.push(codelaw_baseline::file::rel_path(&b.rule));
            if b.entries.is_empty() {
                self.baselines.rules.remove(&b.rule);
            } else {
                self.baselines.insert(b);
            }
        }
        Ok(out)
    }

    /// `codelaw baseline add`: record the current findings of `rule` in
    /// `files` (every file when empty). The only command that may raise a
    /// count, add an entry or extend a deadline (decision 5).
    pub fn baseline_add(&mut self, files: &[PathBuf], options: &BaselineAdd) -> Result<BaselineFile, EngineError> {
        let rule = options.rule.as_str();
        let known = rule == DEPRECATED_RULE || self.rules.get(rule).is_some() || codelaw_rules::engine_rule(rule).is_some();
        if !known {
            let near = codelaw_config::did_you_mean(rule, self.rules.rules.iter().map(|r| r.rule.id.as_str())).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
            return Err(EngineError::Usage(format!("unknown rule `{rule}`{near}")));
        }
        if rule == DEPRECATED_RULE && self.deprecations.is_none() {
            return Err(EngineError::Usage(format!("`{DEPRECATED_RULE}` has nothing to baseline: there is no `{}` (`codelaw deprecate` writes one)", codelaw_baseline::REGISTRY_REL)));
        }
        let only = self.options.rules.replace(vec![rule.to_owned()]);
        let counted = self.counts(files);
        self.options.rules = only;
        let (counts, _) = counted?;
        let root = self.root().to_path_buf();
        let wanted: BTreeSet<String> = files
            .iter()
            .filter(|p| p.is_file())
            .map(|p| crate::discover::rel_path(&std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()), &root))
            .collect();
        let b = codelaw_baseline::add(
            self.baselines.rules.get(rule),
            &counts,
            &AddOptions {
                rule,
                files: &wanted,
                symbol: options.symbol.as_deref(),
                until: options.until,
                warn_from: options.warn_from,
                on_touch: options.on_touch,
                reason: options.reason.as_deref(),
            },
        );
        Baselines::save(&root, &b).map_err(|e| EngineError::Usage(format!("{}: {e}", codelaw_baseline::file::rel_path(rule))))?;
        self.baselines.insert(b.clone());
        Ok(b)
    }

    /// `codelaw baseline check --since <ref>`: every way the baseline got
    /// looser since the merge base of `reference` and `HEAD`, and whether
    /// a CODEOWNERS entry covers the baseline folder.
    pub fn baseline_ratchet(&self, reference: &str) -> Result<Ratchet, EngineError> {
        let root = self.root();
        let usage = |e: String| EngineError::Usage(format!("baseline check needs git: {e}"));
        let base = git::merge_base(root, reference).map_err(usage)?;
        let before = Baselines::from_texts(&git::files_at(root, &base, ".codelaw/baseline").map_err(usage)?).map_err(|e| EngineError::Usage(format!("at {reference}: {e}")))?;
        let loosened = codelaw_baseline::loosened(&before, &self.baselines);
        let top = PathBuf::from(git::toplevel(root).map_err(usage)?);
        let top = std::fs::canonicalize(&top).unwrap_or(top);
        let folder = crate::discover::rel_path(&codelaw_baseline::file::dir(root), &top);
        let probe = format!("{folder}/x.toml");
        let owners = codelaw_baseline::CODEOWNERS_PATHS.iter().find_map(|p| std::fs::read_to_string(top.join(p)).ok().map(|t| (p, t)));
        let codeowners = match owners {
            Some((_, text)) if codelaw_baseline::codeowners_covers(&text, &probe) => None,
            Some((p, _)) => Some(format!("{p} assigns no owner to `{folder}/`: a raised count needs the review CODEOWNERS gives (SR-BASELINE decision 5)")),
            None => Some(format!("no CODEOWNERS file: add one with an entry for `{folder}/` so a raised count needs review (SR-BASELINE decision 5)")),
        };
        Ok(Ratchet { base, loosened, codeowners })
    }

    /// `codelaw deprecate`: add the registry entry, then baseline every
    /// current use, in one reviewed change (SR-BASELINE decision 6).
    pub fn deprecate(&mut self, d: &Deprecation, reason: Option<&str>) -> Result<BaselineFile, EngineError> {
        let path = codelaw_baseline::registry::registry_path(self.root());
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let mut registry = if existing.is_empty() { Registry::default() } else { Registry::parse(&existing).map_err(|e| config_error(None, format!("{}: {e}", codelaw_baseline::REGISTRY_REL)))? };
        if registry.find(&d.symbol).is_some() {
            return Err(EngineError::Usage(format!("`{}` is already in {}", d.symbol, codelaw_baseline::REGISTRY_REL)));
        }
        let mut text = existing.clone();
        if text.is_empty() {
            text.push_str("# Deprecated symbols (SR-BASELINE decision 6): `graph/no-deprecated-usage` reports\n# every use; `.codelaw/baseline/` grandfathers the existing ones.\n");
        }
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push('\n');
        text.push_str(&codelaw_baseline::registry::entry_text(d));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| EngineError::Usage(format!("{}: {e}", codelaw_baseline::REGISTRY_REL)))?;
        }
        std::fs::write(&path, &text).map_err(|e| EngineError::Usage(format!("{}: {e}", codelaw_baseline::REGISTRY_REL)))?;
        registry.deprecated.push(d.clone());
        let defaults = self.deprecations.as_ref().map(|x| x.defaults.clone()).unwrap_or_default();
        self.deprecations = Some(Deprecations { registry, defaults });
        self.baseline_add(
            &[],
            &BaselineAdd {
                rule: DEPRECATED_RULE.into(),
                symbol: Some(d.symbol.clone()),
                until: d.policy.until,
                warn_from: d.policy.warn_from,
                on_touch: d.policy.on_touch,
                reason: reason.map(str::to_owned).or_else(|| d.reason.clone()),
            },
        )
    }
}

/// What `baseline add` records.
#[derive(Clone, Debug, Default)]
pub struct BaselineAdd {
    pub rule: String,
    pub symbol: Option<String>,
    pub until: Option<Date>,
    pub warn_from: Option<Date>,
    pub on_touch: Option<OnTouch>,
    pub reason: Option<String>,
}

/// `baseline check`'s verdict.
#[derive(Clone, Debug)]
pub struct Ratchet {
    /// The merge base compared against.
    pub base: String,
    /// Every loosening, sorted; any fails the check.
    pub loosened: Vec<String>,
    /// Why approval is not guaranteed, when it isn't.
    pub codeowners: Option<String>,
}

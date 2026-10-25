//! The merged rule set of one project: packs in `extends` order, then local
//! rules, with SR-RULE-PACKS decision 14's precedence.

use crate::LoadError;
use crate::engine_rules::engine_rule;
use crate::pack::{ExtendsSpec, PackSearch, RulePack, first_party_npm, first_party_pypi, load_pack_from, load_rules_dir};
use crate::rule::{RESERVED_NAMESPACES, RuleFile, Stability};
use codelaw_cache::{Key, KeyBuilder};
use codelaw_config::{Config, Layer, RuleConfig, RuleSetting};
use codelaw_core::Severity;
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

/// Where a rule came from; named in `rules --why` and the JSON output
/// (decision 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleSource {
    Local,
    Pack { name: String, version: String, first_party: bool, dir: PathBuf },
}

impl RuleSource {
    pub fn label(&self) -> String {
        match self {
            RuleSource::Local => "local".into(),
            RuleSource::Pack { name, version, .. } => format!("{name}@{version}"),
        }
    }
}

/// One rule as loaded, with the settings it runs with when no config
/// layer changes them. `default: None` is loaded but not switched on.
#[derive(Clone, Debug)]
pub struct LoadedRule {
    pub rule: Arc<RuleFile>,
    pub source: RuleSource,
    pub default: Option<RuleConfig>,
    /// Which `extends` entry or `rules_dir` switched it on.
    pub enabled_by: Option<String>,
    /// The rules it shadows, lowest precedence first (`rules --why`).
    pub shadows: Vec<String>,
    /// The pack's `data/` tables.
    pub data: Arc<BTreeMap<String, Arc<serde_json::Value>>>,
    /// A preview pack rule without `preview = true` never runs.
    pub preview_blocked: bool,
    /// An `external` fixer may run (decision 12).
    pub external_allowed: bool,
}

pub struct LoadOptions<'a> {
    pub search: PackSearch,
    /// Whether the engine registers a `native:` id (SR-ARCHITECTURE
    /// decision 15b).
    pub known_native: &'a dyn Fn(&str) -> bool,
}

/// The rule set of a project.
#[derive(Debug)]
pub struct RuleSet {
    /// Sorted by id.
    pub rules: Vec<LoadedRule>,
    pub packs: Vec<(String, PathBuf)>,
    pub warnings: Vec<String>,
    pub fingerprint: Key,
}

fn to_config(rule: &RuleFile, setting: Option<&RuleSetting>) -> Option<RuleConfig> {
    let mut options = rule.default_options();
    let severity = match setting.and_then(|s| s.severity()) {
        Some("off") => return None,
        Some(s) => Severity::parse(s).unwrap_or(rule.severity),
        None => rule.severity,
    };
    if let Some(o) = setting.and_then(|s| s.options()) {
        options.extend(o.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    Some(RuleConfig { severity: Some(severity), options })
}

fn check_options(rule: &RuleFile, setting: &RuleSetting, file: &std::path::Path, errors: &mut Vec<LoadError>) {
    let Some(options) = setting.options() else { return };
    for (name, value) in options {
        match rule.options.get(name) {
            None => {
                let near = codelaw_config::did_you_mean(name, rule.options.keys().map(String::as_str))
                    .map(|n| format!("; did you mean `{n}`?"))
                    .unwrap_or_default();
                errors.push(LoadError::new(Some(file.to_owned()), format!("rule `{}` has no option `{name}`{near}", rule.id)));
            }
            Some(spec) if !spec.accepts(value) => errors.push(LoadError::new(
                Some(file.to_owned()),
                format!("rule `{}` option `{name}` must be a `{}` (SR-LINTER decision 5)", rule.id, spec.kind),
            )),
            Some(_) => {}
        }
    }
}

impl RuleSet {
    pub fn load(config: &Config, options: &LoadOptions) -> Result<RuleSet, Vec<LoadError>> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let config_file = config.root_file().map(|p| p.to_owned());
        let mut merged: BTreeMap<String, LoadedRule> = BTreeMap::new();
        let mut loaded_packs: Vec<(PathBuf, Arc<RulePack>)> = Vec::new();
        let mut key = KeyBuilder::new("ruleset");

        // Each entry, with the presets that extended it (outermost first):
        // a preset's own `extends` are resolved before its `[rules]`, so
        // the preset's settings win (SR-RULE-PACKS decision 3).
        let mut queue: VecDeque<(String, Vec<String>, bool)> = config.extends().iter().map(|s| (s.to_string(), Vec::new(), false)).collect();
        while let Some((spec_owned, via, expanded)) = queue.pop_front() {
            let spec_text = &spec_owned;
            let label = match via.last() {
                Some(v) => format!("{spec_text} (extended by {v})"),
                None => spec_text.clone(),
            };
            let spec = match ExtendsSpec::parse(spec_text) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(LoadError::new(config_file.clone(), e));
                    continue;
                }
            };
            let dir = match &spec {
                ExtendsSpec::FirstParty { pack, .. } => options.search.find_first_party(pack),
                ExtendsSpec::Npm(name) => options.search.find_npm(name),
                ExtendsSpec::Pypi(name) => options.search.find_pypi(name),
                ExtendsSpec::Path(p) => Some(config.config_dir().join(p)).filter(|d| d.join("codelaw-pack.toml").is_file()),
            };
            let Some(dir) = dir else {
                let msg = match &spec {
                    ExtendsSpec::FirstParty { pack, .. } => PackSearch::missing_first_party(&label, pack),
                    ExtendsSpec::Npm(n) => format!("`{spec_text}`: no `node_modules/{n}` with a codelaw-pack.toml above {}", codelaw_core::normalize_path(&options.search.config_dir)),
                    ExtendsSpec::Pypi(n) => format!("`{spec_text}`: `{n}` is not installed in the Python environment"),
                    ExtendsSpec::Path(p) => format!("`{spec_text}`: no codelaw-pack.toml in `{p}`"),
                };
                errors.push(LoadError::new(config_file.clone(), msg));
                continue;
            };
            let first_party_origin = match &spec {
                ExtendsSpec::FirstParty { .. } => true,
                ExtendsSpec::Npm(name) => first_party_npm(name).is_some(),
                ExtendsSpec::Pypi(name) => first_party_pypi(name).is_some(),
                ExtendsSpec::Path(_) => false,
            };
            let pack = match loaded_packs.iter().find(|(d, _)| *d == dir) {
                Some((_, p)) => p.clone(),
                None => match load_pack_from(&dir, first_party_origin) {
                    Ok(p) => {
                        let p = Arc::new(p);
                        key.push(p.label());
                        key.push(p.data_hash.0);
                        loaded_packs.push((dir.clone(), p.clone()));
                        p
                    }
                    Err(e) => {
                        errors.extend(e);
                        continue;
                    }
                },
            };
            if let ExtendsSpec::FirstParty { pack: short, .. } = &spec
                && !pack.first_party
            {
                errors.push(LoadError::new(Some(dir.clone()), format!("`{spec_text}` resolved to {}, which is not the first-party pack `{short}`", pack.label())));
                continue;
            }
            let source = RuleSource::Pack {
                name: pack.manifest.name.clone(),
                version: pack.manifest.version.clone(),
                first_party: pack.first_party,
                dir: pack.dir.clone(),
            };
            let external_allowed = config.allow_pack_fixers().iter().any(|n| *n == pack.manifest.name);
            // Every rule of the pack is loaded; a later pack shadows an
            // earlier one for the same id (decision 14).
            for rule in &pack.rules {
                let entry = merged.get(&rule.id);
                let same_pack = entry.is_some_and(|e| e.source == source);
                if same_pack {
                    continue;
                }
                let mut shadows = entry.map(|e| e.shadows.clone()).unwrap_or_default();
                if let Some(e) = entry {
                    shadows.push(e.source.label());
                }
                let preview_blocked = rule.stability == Stability::Preview && !config.preview();
                // A `graph/` rule two first-party packs ship is one rule over
                // both packs' languages (SR-RULE-PACKS decision 2).
                let mut rule = rule.clone();
                if let Some(e) = entry
                    && rule.namespace() == "graph"
                    && pack.first_party
                    && matches!(&e.source, RuleSource::Pack { first_party: true, .. })
                {
                    let mut languages = e.rule.languages.clone();
                    languages.extend(rule.languages.iter().filter(|l| !e.rule.languages.contains(l)));
                    let mut examples = e.rule.examples.clone();
                    examples.extend(rule.examples.iter().filter(|x| !e.rule.examples.iter().any(|y| y.language == x.language)).cloned());
                    let mut cases = e.rule.cases.clone();
                    cases.extend(rule.cases.iter().cloned());
                    rule.languages = languages;
                    rule.examples = examples;
                    rule.cases = cases;
                }
                merged.insert(
                    rule.id.clone(),
                    LoadedRule {
                        rule: Arc::new(rule.clone()),
                        source: source.clone(),
                        default: None,
                        enabled_by: None,
                        shadows,
                        data: pack.data.clone(),
                        preview_blocked,
                        external_allowed,
                    },
                );
            }
            if let ExtendsSpec::FirstParty { preset, .. } = &spec
                && !expanded
                && let Some(p) = pack.presets.get(preset)
                && !p.extends.is_empty()
            {
                let mut chain = via.clone();
                chain.push(spec_text.clone());
                queue.push_front((spec_text.clone(), via.clone(), true));
                for parent in p.extends.iter().rev() {
                    if chain.contains(parent) {
                        errors.push(LoadError::new(
                            Some(pack.dir.clone()),
                            format!("presets extend each other in a cycle: {} → {parent}", chain.join(" → ")),
                        ));
                        continue;
                    }
                    queue.push_front((parent.clone(), chain.clone(), false));
                }
                continue;
            }
            let enabled: Vec<(String, Option<RuleSetting>)> = match &spec {
                ExtendsSpec::FirstParty { preset, .. } => match pack.presets.get(preset) {
                    Some(p) => p.rules.iter().map(|(k, v)| (k.clone(), Some(v.clone()))).collect(),
                    None => {
                        let near = codelaw_config::did_you_mean(preset, pack.presets.keys().map(String::as_str))
                            .map(|n| format!("; did you mean `codelaw:{n}`?"))
                            .unwrap_or_default();
                        let by = via.last().map(|v| format!(" (extended by {v})")).unwrap_or_default();
                        errors.push(LoadError::new(config_file.clone(), format!("{} has no preset `{preset}`{near}{by}", pack.label())));
                        continue;
                    }
                },
                _ => pack.rules.iter().map(|r| (r.id.clone(), None)).collect(),
            };
            for (id, setting) in enabled {
                let Some(entry) = merged.get_mut(&id) else { continue };
                if entry.source != source {
                    continue;
                }
                if let Some(s) = &setting {
                    check_options(&entry.rule, s, &pack.dir, &mut errors);
                }
                entry.default = to_config(&entry.rule, setting.as_ref());
                entry.enabled_by = Some(label.clone());
            }
        }

        // Local rules: highest precedence; an ejected copy shadows the
        // pack rule of the same id (decision 10).
        let rules_dir = config.rules_dir();
        if rules_dir.is_dir() {
            match load_rules_dir(&rules_dir) {
                Ok(local) => {
                    for rule in local {
                        let shadowed = merged.get(&rule.id);
                        let ns = rule.namespace().to_owned();
                        let shadows_first_party = shadowed.is_some_and(|s| matches!(&s.source, RuleSource::Pack { first_party: true, .. }));
                        if RESERVED_NAMESPACES.contains(&ns.as_str()) && !shadows_first_party {
                            errors.push(LoadError::new(
                                Some(rule.path.clone()),
                                format!("namespace `{ns}` is reserved for the first-party packs; a local rule may use it only as an ejected copy of a pack rule (`codelaw rules eject`)"),
                            ));
                            continue;
                        }
                        if let Some(native) = rule.native_id() {
                            let ok = shadows_first_party && shadowed.is_some_and(|s| s.rule.native_id() == Some(native));
                            if !ok {
                                errors.push(LoadError::new(
                                    Some(rule.path.clone()),
                                    "`native:` is accepted only in first-party packs and in an ejected copy of one of their rules (SR-RULE-PACKS decision 10)",
                                ));
                                continue;
                            }
                        }
                        let mut shadows = shadowed.map(|s| s.shadows.clone()).unwrap_or_default();
                        if let Some(s) = shadowed {
                            shadows.push(s.source.label());
                        }
                        let data = shadowed.map(|s| s.data.clone()).unwrap_or_default();
                        let default = to_config(&rule, None);
                        merged.insert(
                            rule.id.clone(),
                            LoadedRule {
                                rule: Arc::new(rule),
                                source: RuleSource::Local,
                                default,
                                enabled_by: Some(codelaw_core::normalize_path(rules_dir.strip_prefix(&config.root_dir).unwrap_or(&rules_dir))),
                                shadows,
                                data,
                                preview_blocked: false,
                                external_allowed: true,
                            },
                        );
                    }
                }
                Err(e) => errors.extend(e),
            }
        }

        // An unknown `native:` id refuses the rule at load, naming the
        // pack's `min_codelaw` (SR-ARCHITECTURE decision 15b).
        for r in merged.values() {
            if let Some(native) = r.rule.native_id()
                && !(options.known_native)(native)
            {
                let min = loaded_packs
                    .iter()
                    .find(|(_, p)| r.source == RuleSource::Pack { name: p.manifest.name.clone(), version: p.manifest.version.clone(), first_party: p.first_party, dir: p.dir.clone() })
                    .map(|(_, p)| format!("; its pack needs codelaw {} or newer, so upgrade codelaw", p.manifest.min_codelaw))
                    .unwrap_or_default();
                errors.push(LoadError::new(
                    Some(r.rule.path.clone()),
                    format!("`native: {native}` is not an engine rule in codelaw {}{min}", codelaw_core::VERSION),
                ));
            }
        }

        // Every rule id the config names exists (SR-CONFIG decision 5).
        for f in &config.files {
            let mut check = |id: &str, setting: &RuleSetting| {
                if engine_rule(id).is_some() {
                    return;
                }
                match merged.get(id) {
                    Some(r) => {
                        check_options(&r.rule, setting, &f.path, &mut errors);
                        if r.preview_blocked && setting.severity() != Some("off") {
                            warnings.push(format!("`{id}` is a preview rule and runs only with `preview = true`"));
                        }
                    }
                    None => {
                        let near = codelaw_config::did_you_mean(id, merged.keys().map(String::as_str))
                            .map(|n| format!("; did you mean `{n}`?"))
                            .unwrap_or_default();
                        errors.push(LoadError::new(Some(f.path.clone()), format!("unknown rule `{id}`{near}")));
                    }
                }
            };
            for (id, s) in &f.raw.rules {
                check(id, s);
            }
            for o in &f.raw.overrides {
                for (id, s) in &o.rules {
                    check(id, s);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        for r in merged.values() {
            key.push(&r.rule.id);
            key.push(r.rule.hash.0);
            key.push(r.source.label());
            key.push(format!("{:?}{}", r.default, r.preview_blocked));
        }
        Ok(RuleSet {
            rules: merged.into_values().collect(),
            packs: loaded_packs.iter().map(|(d, p)| (p.label(), d.clone())).collect(),
            warnings,
            fingerprint: key.key(),
        })
    }

    pub fn get(&self, id: &str) -> Option<&LoadedRule> {
        self.rules.binary_search_by(|r| r.rule.id.as_str().cmp(id)).ok().map(|i| &self.rules[i])
    }

    /// Whether any rule could run: the `codelaw/no-rules` condition.
    pub fn has_rules(&self) -> bool {
        self.rules.iter().any(|r| !r.preview_blocked)
    }

    /// The settings `rule` runs with for one file, or `None` when it is off
    /// there.
    pub fn effective(&self, rule: &LoadedRule, layers: &[Layer]) -> Option<RuleConfig> {
        if rule.preview_blocked {
            return None;
        }
        let mut current = rule.default.clone();
        for layer in layers {
            let Some(setting) = layer.rules.get(&rule.rule.id) else { continue };
            match setting.severity() {
                Some("off") => current = None,
                Some(s) => {
                    let sev = Severity::parse(s).unwrap_or(rule.rule.severity);
                    let mut cfg = current.take().unwrap_or_else(|| RuleConfig { severity: None, options: rule.rule.default_options() });
                    cfg.severity = Some(sev);
                    current = Some(cfg);
                }
                None => {}
            }
            if let (Some(cfg), Some(opts)) = (current.as_mut(), setting.options()) {
                cfg.options.extend(opts.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }
        current
    }
}

//! Feature-flag expiry in the project stage (SR-PROJECT-GRAPH decision 11,
//! W-130): the `graph: { flags: … }` rules run here, because their inputs
//! are the engine's — `[flags]`, the registry `.codelaw/flags.toml`, every
//! JS, TS and Python file's flag facts (cached by content) and the as-of
//! date. The verdicts are `codelaw_graph::flags`.

use crate::compile::{CompiledRule, Matcher};
use crate::project::Source;
use crate::{EngineError, Project};
use codelaw_cache::KeyBuilder;
use codelaw_config::RuleConfig;
use codelaw_core::{Language, Severity};
use codelaw_graph::flags::{self, FlagCheck, FlagFacts, FlagOptions, FlagReport};
use codelaw_graph::{Graph, GraphFinding, Query};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::Path;

/// A `[flags]` that cannot be used is a config error at load: an unknown
/// preset, a pattern that does not parse, a bad `warn_days`.
pub(crate) fn check_config(config: &codelaw_config::Config) -> Result<(), EngineError> {
    if !config.flags_on() {
        return Ok(());
    }
    FlagOptions::from_options(&config.section_options("flags"))
        .and_then(|o| o.patterns().map(|_| ()))
        .map_err(|e| EngineError::Config(codelaw_config::ConfigError::new(config.root_file().map(Path::to_path_buf), e)))
}

/// Whether a compiled rule is one of the flag rules.
pub(crate) fn is_flag_rule(rule: &CompiledRule) -> bool {
    matches!(&rule.matcher, Matcher::Graph(g) if matches!(g.query, Query::Flags(_)))
}

impl Project {
    /// A flag rule's config: as any rule's, and on with its own severity
    /// when the root config has a `[flags]` table (decision 11: the table
    /// turns the rules on); `off` in `[rules]` still wins.
    pub(crate) fn flag_rule_config(&self, rule: &CompiledRule, layers: &[codelaw_config::Layer]) -> Option<RuleConfig> {
        if let Some(c) = self.rules.effective(&rule.loaded, layers) {
            return Some(c);
        }
        if !self.config.flags_on() || rule.loaded.preview_blocked {
            return None;
        }
        let mut loaded = rule.loaded.clone();
        loaded.default = Some(RuleConfig { severity: Some(loaded.rule.severity), options: loaded.rule.default_options() });
        self.rules.effective(&loaded, layers)
    }

    /// Whether the flag rules are on in this run: then the graph's sources
    /// carry the registry's text.
    pub(crate) fn reads_flags(&self) -> bool {
        self.graph_rules().iter().any(|(_, r)| is_flag_rule(r))
    }

    /// The options a flag rule judges by: `[flags]`, then its own.
    fn flag_options(&self, rule: &CompiledRule) -> BTreeMap<String, serde_json::Value> {
        let mut o = self.config.section_options("flags");
        if let Some(c) = self.flag_rule_config(rule, &[]) {
            o.extend(c.options);
        }
        o
    }

    /// One source's flag facts, from the cache when it has them.
    fn source_flag_facts(&self, s: &Source) -> Option<FlagFacts> {
        let language = s.language.filter(|l| l.is_js_family() || *l == Language::Python)?;
        let key = KeyBuilder::new("flag-facts").part(flags::FLAGS_VERSION).part(language.name()).part(&s.text).part(codelaw_lang::GRAMMARS_FINGERPRINT).key();
        if let Some(f) = self.cache.get(&key) {
            return Some(f);
        }
        let f = std::panic::catch_unwind(|| flags::extract_flags(language, &s.text)).ok().flatten()?;
        self.cache.put(&key, &f);
        Some(f)
    }

    /// The flag report of a project for one set of options: the accessors
    /// in force and every finding. `Err` when the options are unusable.
    pub(crate) fn flag_report(&self, graph: &Graph, sources: &[Source], options: &BTreeMap<String, serde_json::Value>) -> Result<FlagReport, String> {
        let options = FlagOptions::from_options(options)?;
        if !options.is_on() {
            return Ok(FlagReport { accessors: Vec::new(), findings: Vec::new() });
        }
        let facts: Vec<Option<FlagFacts>> = self.pool.install(|| sources.par_iter().map(|s| self.source_flag_facts(s)).collect());
        let files: BTreeMap<String, FlagFacts> = sources.iter().zip(facts).filter_map(|(s, f)| Some((s.rel.clone(), f?))).collect();
        let registry = match sources.iter().find(|s| s.rel == flags::REGISTRY_REL && !s.text.is_empty()) {
            Some(s) => Some(s.text.clone()),
            None => std::fs::read_to_string(self.root().join(flags::REGISTRY_REL)).ok(),
        };
        let as_of = self.as_of().date.map(|d| d.to_string());
        flags::check(graph, &files, registry.as_deref(), &options, as_of.as_deref())
    }

    /// A flag rule's findings, with the severity each one is graded at.
    pub(crate) fn flag_findings(
        &self,
        rule: &CompiledRule,
        check: FlagCheck,
        graph: &Graph,
        sources: &[Source],
        reports: &mut BTreeMap<String, Option<FlagReport>>,
    ) -> Vec<(GraphFinding, Option<Severity>)> {
        let options = self.flag_options(rule);
        let key = serde_json::to_string(&options).unwrap_or_default();
        let report = reports.entry(key).or_insert_with(|| self.flag_report(graph, sources, &options).ok());
        let Some(report) = report else { return Vec::new() };
        let languages = &rule.loaded.rule.languages;
        let language_of: BTreeMap<&str, Option<Language>> = sources.iter().map(|s| (s.rel.as_str(), s.language)).collect();
        report
            .findings
            .iter()
            .filter(|f| f.check == check)
            .filter(|f| check == FlagCheck::Registry || language_of.get(f.file.as_str()).copied().flatten().is_some_and(|l| languages.contains(&l)))
            .map(|f| {
                let finding = GraphFinding { file: f.file.clone(), span: f.span, captures: Vec::new(), edits: f.edits.clone(), uncertain: f.uncertain.clone(), message: Some(f.message.clone()) };
                (finding, Some(f.severity))
            })
            .collect()
    }

    /// `codelaw flags accessors`: every accessor in force — the patterns of
    /// `[flags]` and its presets, then the wrappers found through the graph.
    pub fn flag_accessors(&mut self) -> Result<Vec<flags::Accessor>, EngineError> {
        let config_error = |e: String| EngineError::Config(codelaw_config::ConfigError::new(self.config.root_file().map(Path::to_path_buf), e));
        let options = self.config.section_options("flags");
        let parsed = FlagOptions::from_options(&options).map_err(config_error)?;
        if !parsed.is_on() {
            return Ok(Vec::new());
        }
        let config = self.graph_config()?;
        let sources = self.graph_sources()?;
        let graph = self.build_graph(&sources, config);
        let report = self.flag_report(&graph, &sources, &options).map_err(|e| EngineError::Config(codelaw_config::ConfigError::new(self.config.root_file().map(Path::to_path_buf), e)))?;
        Ok(report.accessors)
    }
}
